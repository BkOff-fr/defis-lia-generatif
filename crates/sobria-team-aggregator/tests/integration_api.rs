//! Flow d'intégration C28.2 : init → code create → enroll → estimate
//! → me/usage → admin login → refresh (rotation).
//!
//! Démarre un serveur axum HTTPS réel (rustls + cert auto-signé) sur un
//! port aléatoire, puis attaque les routes via `reqwest`. Le client accepte
//! les certs invalides — équivalent programmatique du clic « j'accepte le
//! risque » que feront extension + app en C28.6.

use std::net::TcpListener;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use serde_json::json;
use sobria_team_aggregator::{
    commands::{code, init, serve},
    config::DataPaths,
};
use tempfile::tempdir;

struct ServerFixture {
    _tempdir: tempfile::TempDir,
    base_url: String,
    client: reqwest::Client,
    /// Code clair généré pour les tests d'enrollment (12 chiffres).
    enrollment_code: String,
    handle: tokio::task::JoinHandle<()>,
}

impl ServerFixture {
    async fn start() -> Self {
        let dir = tempdir().expect("tempdir");
        let paths = DataPaths::new(dir.path());
        init::run(&paths, "admin", "admin-pw-not-secret", false).expect("init");

        let codes = code::create_batch(&paths, 3, 7, "admin").expect("create codes");
        let enrollment_code = codes[0].code.clone();

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let base_url = format!("https://{addr}");
        let paths_clone = paths.clone();

        let handle = tokio::spawn(async move {
            let _ = serve::run_with_listener(listener, &paths_clone).await;
        });

        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("client");

        // Attend que le handshake TLS soit prêt.
        wait_for_health(&client, &base_url).await;

        Self {
            _tempdir: dir,
            base_url,
            client,
            enrollment_code,
            handle,
        }
    }
}

impl Drop for ServerFixture {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn wait_for_health(client: &reqwest::Client, base_url: &str) {
    for _ in 0..30 {
        if let Ok(r) = client.get(format!("{base_url}/health")).send().await {
            if r.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("serveur jamais en ligne");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_user_flow_enroll_estimate_me_usage() {
    let fx = ServerFixture::start().await;

    // 1. Enroll
    let enroll = fx
        .client
        .post(format!("{}/api/v1/enroll", fx.base_url))
        .json(&json!({
            "code": fx.enrollment_code,
            "password": "user-password-strong",
            "fingerprint": "chrome-mac-abc123",
            "display_name": "Alice"
        }))
        .send()
        .await
        .expect("enroll send");
    assert_eq!(enroll.status(), 200, "enroll status");
    let enroll_json: serde_json::Value = enroll.json().await.expect("enroll json");
    let access = enroll_json["access_token"].as_str().unwrap().to_string();
    let refresh = enroll_json["refresh_token"].as_str().unwrap().to_string();
    assert!(!access.is_empty());
    assert!(refresh.contains('.'));

    // 1bis. Re-enroller le même fingerprint → 409 conflict
    let dup = fx
        .client
        .post(format!("{}/api/v1/enroll", fx.base_url))
        .json(&json!({
            "code": fx.enrollment_code,  // déjà consommé
            "password": "x12345678",
            "fingerprint": "chrome-mac-other"
        }))
        .send()
        .await
        .expect("dup enroll send");
    assert_eq!(
        dup.status(),
        401,
        "code déjà consommé → 401 invalid_enrollment_code"
    );

    // 2. POST estimation (sans auth → 401)
    let unauth = fx
        .client
        .post(format!("{}/api/v1/estimations", fx.base_url))
        .json(&estimate_body(0.42, 120, 800))
        .send()
        .await
        .expect("unauth send");
    assert_eq!(unauth.status(), 401);

    // 3. POST estimation (avec auth user)
    for (gco2eq, t_in, t_out) in [(0.42, 120u32, 800u32), (0.30, 50, 400)] {
        let r = fx
            .client
            .post(format!("{}/api/v1/estimations", fx.base_url))
            .bearer_auth(&access)
            .json(&estimate_body(gco2eq, t_in, t_out))
            .send()
            .await
            .expect("est send");
        assert_eq!(r.status(), 200, "estimation status");
        let body: serde_json::Value = r.json().await.expect("est json");
        assert_eq!(body["ack"], true);
        assert!(body["id"].as_str().unwrap().len() >= 20);
    }

    // 4. GET me/usage
    let usage = fx
        .client
        .get(format!("{}/api/v1/me/usage", fx.base_url))
        .bearer_auth(&access)
        .send()
        .await
        .expect("me send");
    assert_eq!(usage.status(), 200);
    let usage_json: serde_json::Value = usage.json().await.expect("me json");
    assert_eq!(usage_json["totals"]["count"], 2);
    assert_eq!(usage_json["totals"]["tokens_in"], 170);
    assert_eq!(usage_json["totals"]["tokens_out"], 1200);

    // 5. /me/usage requiert role=user → un token admin doit échouer (cf. /admin login)
    let admin_login = fx
        .client
        .post(format!("{}/api/v1/login", fx.base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin-pw-not-secret",
            "role": "admin"
        }))
        .send()
        .await
        .expect("admin login");
    assert_eq!(admin_login.status(), 200);
    let admin_tokens: serde_json::Value = admin_login.json().await.expect("admin login json");
    let admin_access = admin_tokens["access_token"].as_str().unwrap();

    let forbidden = fx
        .client
        .get(format!("{}/api/v1/me/usage", fx.base_url))
        .bearer_auth(admin_access)
        .send()
        .await
        .expect("me forbidden");
    assert_eq!(forbidden.status(), 403);

    // 6. Refresh → rotation. L'ancien refresh ne doit PLUS marcher.
    let refresh_resp = fx
        .client
        .post(format!("{}/api/v1/refresh", fx.base_url))
        .json(&json!({ "refresh_token": refresh }))
        .send()
        .await
        .expect("refresh send");
    assert_eq!(refresh_resp.status(), 200);
    let refreshed: serde_json::Value = refresh_resp.json().await.expect("refresh json");
    let new_access = refreshed["access_token"].as_str().unwrap();
    let new_refresh = refreshed["refresh_token"].as_str().unwrap();
    assert_ne!(new_access, &access, "access token DOIT changer");
    assert_ne!(
        new_refresh, &refresh,
        "refresh token DOIT changer (rotation)"
    );
    assert_eq!(refreshed["role"], "user");

    // L'ancien refresh est révoqué → 401.
    let replay = fx
        .client
        .post(format!("{}/api/v1/refresh", fx.base_url))
        .json(&json!({ "refresh_token": refresh }))
        .send()
        .await
        .expect("replay refresh");
    assert_eq!(replay.status(), 401, "ancien refresh ne doit plus passer");

    // Le nouvel access fonctionne sur /me/usage.
    let ok = fx
        .client
        .get(format!("{}/api/v1/me/usage", fx.base_url))
        .bearer_auth(new_access)
        .send()
        .await
        .expect("me with new token");
    assert_eq!(ok.status(), 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_with_bad_password_returns_401() {
    let fx = ServerFixture::start().await;
    let bad = fx
        .client
        .post(format!("{}/api/v1/login", fx.base_url))
        .json(&json!({
            "username": "admin",
            "password": "wrong",
            "role": "admin"
        }))
        .send()
        .await
        .expect("login send");
    assert_eq!(bad.status(), 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn estimations_rejects_unknown_method() {
    let fx = ServerFixture::start().await;
    let enroll: serde_json::Value = fx
        .client
        .post(format!("{}/api/v1/enroll", fx.base_url))
        .json(&json!({
            "code": fx.enrollment_code,
            "password": "user-password-strong",
            "fingerprint": "chrome-fp-x"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let access = enroll["access_token"].as_str().unwrap();

    let bad = fx
        .client
        .post(format!("{}/api/v1/estimations", fx.base_url))
        .header(AUTHORIZATION, format!("Bearer {access}"))
        .json(&json!({
            "estimate": {
                "method": "unknown_method",
                "modelId": "m",
                "tokensIn": 1,
                "tokensOut": 1,
                "gco2eq": 0.1,
                "waterMl": 0.0,
                "energyWh": 0.0
            },
            "ts": "2026-05-16T12:00:00Z"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
}

fn estimate_body(gco2eq: f64, tokens_in: u32, tokens_out: u32) -> serde_json::Value {
    json!({
        "estimate": {
            "method": "afnor_sobria",
            "modelId": "llama-3-1-70b",
            "tokensIn": tokens_in,
            "tokensOut": tokens_out,
            "gco2eq": gco2eq,
            "waterMl": 1.5,
            "energyWh": 0.42
        },
        "host": "chatgpt",
        "modelDisplayName": "GPT-4o",
        "ts": "2026-05-16T12:34:56Z"
    })
}

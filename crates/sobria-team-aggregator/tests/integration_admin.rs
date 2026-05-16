//! Flow d'intégration C28.3 : admin login → /admin/codes (POST + DELETE)
//! → enroll users avec un code admin-créé → estimations → /admin/users
//! → /admin/analytics (séries + top_models + top_users + method_breakdown).
//!
//! Reprend le pattern du `ServerFixture` de `integration_api.rs` mais isolé
//! pour pouvoir tester les routes admin sans pré-créer des codes via la CLI.

use std::net::TcpListener;
use std::time::Duration;

use serde_json::json;
use sobria_team_aggregator::{
    commands::{init, serve},
    config::DataPaths,
};
use tempfile::tempdir;

struct AdminFixture {
    _tempdir: tempfile::TempDir,
    base_url: String,
    client: reqwest::Client,
    admin_access: String,
    handle: tokio::task::JoinHandle<()>,
}

impl AdminFixture {
    async fn start() -> Self {
        let dir = tempdir().expect("tempdir");
        let paths = DataPaths::new(dir.path());
        init::run(&paths, "admin", "admin-pw-not-secret", false).expect("init");

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

        for _ in 0..30 {
            if let Ok(r) = client.get(format!("{base_url}/health")).send().await {
                if r.status().is_success() {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let resp = client
            .post(format!("{base_url}/api/v1/login"))
            .json(&json!({
                "username": "admin",
                "password": "admin-pw-not-secret",
                "role": "admin"
            }))
            .send()
            .await
            .expect("admin login");
        assert_eq!(resp.status(), 200);
        let tokens: serde_json::Value = resp.json().await.expect("tokens");
        let admin_access = tokens["access_token"].as_str().unwrap().to_string();

        Self {
            _tempdir: dir,
            base_url,
            client,
            admin_access,
            handle,
        }
    }
}

impl Drop for AdminFixture {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_users_codes_and_analytics_end_to_end() {
    let fx = AdminFixture::start().await;

    // 0. /admin/users requiert un admin → 401 sans Bearer
    let unauth = fx
        .client
        .get(format!("{}/api/v1/admin/users", fx.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(unauth.status(), 401);

    // 0bis. Un token user ne suffit pas (RequireAdmin → 403)
    //       Pour vérifier ça il nous faut un user enrôlé → on attend la suite.

    // 1. POST /admin/codes — crée 3 codes 7 jours.
    let create = fx
        .client
        .post(format!("{}/api/v1/admin/codes", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({ "count": 3, "ttl_days": 7 }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 200);
    let created: serde_json::Value = create.json().await.unwrap();
    let codes = created["codes"].as_array().unwrap();
    assert_eq!(codes.len(), 3);
    for c in codes {
        let code_str = c["code"].as_str().unwrap();
        assert_eq!(code_str.len(), 12);
        assert!(code_str.chars().all(|ch| ch.is_ascii_digit()));
    }
    let code_to_use = codes[0]["code"].as_str().unwrap().to_string();
    let code_to_revoke_id = codes[1]["id"].as_str().unwrap().to_string();

    // 2. DELETE /admin/codes/:id sur le 2e code.
    let revoke = fx
        .client
        .delete(format!(
            "{}/api/v1/admin/codes/{}",
            fx.base_url, code_to_revoke_id
        ))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(revoke.status(), 200);
    let revoke_body: serde_json::Value = revoke.json().await.unwrap();
    assert_eq!(revoke_body["revoked"], true);

    // 3. Enroll 2 users avec le code restant + un autre code valide.
    let enroll_resp = |code: &str, fp: &str, name: &str| {
        let code = code.to_string();
        let fp = fp.to_string();
        let name = name.to_string();
        let base = fx.base_url.clone();
        let client = fx.client.clone();
        async move {
            let r = client
                .post(format!("{base}/api/v1/enroll"))
                .json(&json!({
                    "code": code,
                    "password": "user-pw-strong",
                    "fingerprint": fp,
                    "display_name": name
                }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 200, "enroll fp={fp}");
            let body: serde_json::Value = r.json().await.unwrap();
            body["access_token"].as_str().unwrap().to_string()
        }
    };
    let alice_access = enroll_resp(&code_to_use, "chrome-mac-alice", "Alice").await;

    // 4. Le code révoqué ne doit pas pouvoir enroller.
    let bad_code = codes[1]["code"].as_str().unwrap();
    let bad_enroll = fx
        .client
        .post(format!("{}/api/v1/enroll", fx.base_url))
        .json(&json!({
            "code": bad_code,
            "password": "x12345678",
            "fingerprint": "chrome-mac-bob"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad_enroll.status(), 401, "code révoqué doit être rejeté");

    // 5. Un token USER ne doit pas pouvoir accéder à /admin/users (403).
    let user_to_admin = fx
        .client
        .get(format!("{}/api/v1/admin/users", fx.base_url))
        .bearer_auth(&alice_access)
        .send()
        .await
        .unwrap();
    assert_eq!(user_to_admin.status(), 403);

    // 6. Alice envoie 3 estimations (2 llama, 1 gpt).
    for (model, gco2eq) in [
        ("llama-3-1-70b", 0.4),
        ("llama-3-1-70b", 0.3),
        ("gpt-4o", 0.5),
    ] {
        let r = fx
            .client
            .post(format!("{}/api/v1/estimations", fx.base_url))
            .bearer_auth(&alice_access)
            .json(&json!({
                "estimate": {
                    "method": "afnor_sobria",
                    "modelId": model,
                    "tokensIn": 100,
                    "tokensOut": 200,
                    "gco2eq": gco2eq,
                    "waterMl": 0.5,
                    "energyWh": 0.2
                },
                "ts": "2026-05-16T10:00:00Z"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
    }

    // 7. /admin/users → Alice présente avec 3 estimations.
    let users_resp = fx
        .client
        .get(format!("{}/api/v1/admin/users", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(users_resp.status(), 200);
    let users_json: serde_json::Value = users_resp.json().await.unwrap();
    let users_list = users_json["users"].as_array().unwrap();
    assert_eq!(users_list.len(), 1);
    let alice = &users_list[0];
    assert_eq!(alice["display_name"], "Alice");
    assert_eq!(alice["fingerprint"], "chrome-mac-alice");
    assert_eq!(alice["totals"]["count"], 3);

    // 8. /admin/analytics — séries + top_models + top_users + breakdown.
    let analytics = fx
        .client
        .get(format!(
            "{}/api/v1/admin/analytics?from=2026-05-15T00:00:00Z&to=2026-05-17T00:00:00Z&group_by=day",
            fx.base_url
        ))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(analytics.status(), 200);
    let a: serde_json::Value = analytics.json().await.unwrap();
    let series = a["series"].as_array().unwrap();
    assert_eq!(series.len(), 1, "1 jour d'activité (16 mai)");
    assert_eq!(series[0]["bucket"], "2026-05-16");
    assert_eq!(series[0]["count"], 3);

    let top_models = a["top_models"].as_array().unwrap();
    assert_eq!(top_models.len(), 2);
    // llama : 0.4+0.3 = 0.7 > gpt-4o : 0.5 → llama 1er.
    assert_eq!(top_models[0]["model_id"], "llama-3-1-70b");
    assert_eq!(top_models[1]["model_id"], "gpt-4o");

    let top_users = a["top_users"].as_array().unwrap();
    assert_eq!(top_users.len(), 1);
    assert_eq!(top_users[0]["user_id"], alice["id"]);
    assert_eq!(top_users[0]["display_name"], "Alice");

    let method_breakdown = a["method_breakdown"].as_array().unwrap();
    assert_eq!(method_breakdown.len(), 1);
    assert_eq!(method_breakdown[0]["method"], "afnor_sobria");
    assert_eq!(method_breakdown[0]["count"], 3);

    // 9. Bad query: group_by inconnu → 400.
    let bad = fx
        .client
        .get(format!(
            "{}/api/v1/admin/analytics?group_by=hourly",
            fx.base_url
        ))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
}

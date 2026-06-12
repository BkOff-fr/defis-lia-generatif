//! Flow d'intégration C28.3 + C38 (ADR-0015) : admin login → /admin/codes
//! (POST + DELETE) → enroll users → estimations → /admin/users (totaux
//! masqués par défaut, visibles après opt-in) → /admin/analytics (garde
//! k-anonymat, classement opt-in + agrégat anonyme).
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

    // 7. /admin/users → Alice présente, totaux MASQUÉS par défaut
    //    (ADR-0015 §3 : pas d'opt-in → pas de consommation visible).
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
    let alice = users_list[0].clone();
    assert_eq!(alice["display_name"], "Alice");
    assert_eq!(alice["fingerprint"], "chrome-mac-alice");
    assert_eq!(alice["share_identified"], false);
    assert!(
        alice["totals"].is_null(),
        "totaux masqués sans opt-in, reçu: {}",
        alice["totals"]
    );

    // 7bis. Alice consent au partage identifié (PUT /me/sharing, son token).
    let sharing_before: serde_json::Value = fx
        .client
        .get(format!("{}/api/v1/me/sharing", fx.base_url))
        .bearer_auth(&alice_access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(sharing_before["share_identified"], false);

    let opt_in = fx
        .client
        .put(format!("{}/api/v1/me/sharing", fx.base_url))
        .bearer_auth(&alice_access)
        .json(&json!({ "share_identified": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(opt_in.status(), 200);

    // Un token ADMIN ne peut pas écrire le consentement (route user-only).
    let admin_cannot = fx
        .client
        .put(format!("{}/api/v1/me/sharing", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({ "share_identified": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(admin_cannot.status(), 403);

    let users_json: serde_json::Value = fx
        .client
        .get(format!("{}/api/v1/admin/users", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let alice = users_json["users"].as_array().unwrap()[0].clone();
    assert_eq!(alice["share_identified"], true);
    assert_eq!(alice["totals"]["count"], 3);

    // 8. /admin/analytics avec 1 seul user actif → BLOQUÉ par le k-anonymat
    //    (ADR-0015 §2, k défaut = 5) : sections vides, raison explicite.
    let analytics_url = format!(
        "{}/api/v1/admin/analytics?from=2026-05-15T00:00:00Z&to=2026-05-17T00:00:00Z&group_by=day",
        fx.base_url
    );
    let a: serde_json::Value = fx
        .client
        .get(&analytics_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(a["k_anonymity"]["required"], 5);
    assert_eq!(a["k_anonymity"]["active_users"], 1);
    assert_eq!(a["k_anonymity"]["blocked"], true);
    assert!(a["series"].as_array().unwrap().is_empty());
    assert!(a["top_models"].as_array().unwrap().is_empty());
    assert!(a["top_users"]["identified"].as_array().unwrap().is_empty());
    assert_eq!(a["top_users"]["anonymous_users"], 0);

    // 8bis. L'org abaisse k à 3 (config), bob + charlie rejoignent et
    //       poussent chacun une estimation → 3 actifs ≥ k → débloqué.
    {
        use sobria_team_aggregator::storage::Storage;
        let paths = DataPaths::new(fx._tempdir.path());
        let st = Storage::open(&paths.db()).expect("open sqlite (WAL)");
        st.set_config("k_anonymity_min", "3").unwrap();
    }

    let more_codes: serde_json::Value = fx
        .client
        .post(format!("{}/api/v1/admin/codes", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({ "count": 1, "ttl_days": 7 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let extra_code = more_codes["codes"][0]["code"].as_str().unwrap().to_string();
    let free_code = codes[2]["code"].as_str().unwrap();

    let bob_access = enroll_resp(free_code, "chrome-mac-bob", "Bob").await;
    let charlie_access = enroll_resp(&extra_code, "firefox-linux-charlie", "Charlie").await;

    for (token, model, gco2eq) in [
        (&bob_access, "llama-3-1-70b", 0.2),
        (&charlie_access, "gpt-4o", 0.1),
    ] {
        let r = fx
            .client
            .post(format!("{}/api/v1/estimations", fx.base_url))
            .bearer_auth(token)
            .json(&json!({
                "estimate": {
                    "method": "afnor_sobria",
                    "modelId": model,
                    "tokensIn": 50,
                    "tokensOut": 100,
                    "gco2eq": gco2eq,
                    "waterMl": 0.2,
                    "energyWh": 0.1
                },
                "ts": "2026-05-16T11:00:00Z"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
    }

    let a: serde_json::Value = fx
        .client
        .get(&analytics_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(a["k_anonymity"]["required"], 3);
    assert_eq!(a["k_anonymity"]["active_users"], 3);
    assert_eq!(a["k_anonymity"]["blocked"], false);

    let series = a["series"].as_array().unwrap();
    assert_eq!(series.len(), 1, "1 jour d'activité (16 mai)");
    assert_eq!(series[0]["bucket"], "2026-05-16");
    assert_eq!(series[0]["count"], 5);

    let top_models = a["top_models"].as_array().unwrap();
    assert_eq!(top_models.len(), 2);
    // llama : 0.4+0.3+0.2 = 0.9 > gpt-4o : 0.5+0.1 = 0.6 → llama 1er.
    assert_eq!(top_models[0]["model_id"], "llama-3-1-70b");
    assert_eq!(top_models[1]["model_id"], "gpt-4o");

    // Seule Alice (opt-in) apparaît nommément ; bob + charlie agrégés.
    let identified = a["top_users"]["identified"].as_array().unwrap();
    assert_eq!(identified.len(), 1);
    assert_eq!(identified[0]["user_id"], alice["id"]);
    assert_eq!(identified[0]["display_name"], "Alice");
    assert_eq!(a["top_users"]["anonymous_users"], 2);
    assert_eq!(a["top_users"]["anonymous_count"], 2);

    let method_breakdown = a["method_breakdown"].as_array().unwrap();
    assert_eq!(method_breakdown.len(), 1);
    assert_eq!(method_breakdown[0]["method"], "afnor_sobria");
    assert_eq!(method_breakdown[0]["count"], 5);

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

/// C44 — GET /admin/users/:id/analytics sous les 3 politiques ADR-0016.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_user_detail_route_respects_visibility_policy() {
    use sobria_team_aggregator::policy::POLICY_KEY;
    use sobria_team_aggregator::storage::Storage;

    let fx = AdminFixture::start().await;
    let set_policy = |value: &str| {
        let paths = DataPaths::new(fx._tempdir.path());
        let st = Storage::open(&paths.db()).expect("open sqlite (WAL)");
        st.set_config(POLICY_KEY, value).unwrap();
    };

    // Enroll Alice + 1 estimation.
    let created: serde_json::Value = fx
        .client
        .post(format!("{}/api/v1/admin/codes", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({ "count": 1, "ttl_days": 7 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let code = created["codes"][0]["code"].as_str().unwrap();
    let enroll: serde_json::Value = fx
        .client
        .post(format!("{}/api/v1/enroll", fx.base_url))
        .json(&json!({
            "code": code,
            "password": "user-pw-strong",
            "fingerprint": "chrome-mac-alice",
            "display_name": "Alice"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let alice_access = enroll["access_token"].as_str().unwrap().to_string();

    let r = fx
        .client
        .post(format!("{}/api/v1/estimations", fx.base_url))
        .bearer_auth(&alice_access)
        .json(&json!({
            "estimate": {
                "method": "afnor_sobria",
                "modelId": "llama-3-1-70b",
                "tokensIn": 100,
                "tokensOut": 200,
                "gco2eq": 0.4,
                "waterMl": 0.5,
                "energyWh": 0.2
            },
            "ts": "2026-05-16T10:00:00Z"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);

    let users: serde_json::Value = fx
        .client
        .get(format!("{}/api/v1/admin/users", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let alice_id = users["users"][0]["id"].as_str().unwrap().to_string();
    let detail_url = format!("{}/api/v1/admin/users/{}/analytics", fx.base_url, alice_id);

    // opt_in (défaut) sans consentement → 403.
    let denied = fx
        .client
        .get(&detail_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403, "opt_in sans consentement → 403");

    // Un token USER n'accède jamais à la route admin → 403.
    let user_denied = fx
        .client
        .get(&detail_url)
        .bearer_auth(&alice_access)
        .send()
        .await
        .unwrap();
    assert_eq!(user_denied.status(), 403);

    // Alice consent → 200 avec totaux et la route matche bien `:id`
    // (régression C44 : la syntaxe `{id}` d'axum 0.8 donnait un 404).
    let opt_in = fx
        .client
        .put(format!("{}/api/v1/me/sharing", fx.base_url))
        .bearer_auth(&alice_access)
        .json(&json!({ "share_identified": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(opt_in.status(), 200);

    let ok = fx
        .client
        .get(&detail_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200, "opt_in + consentement → 200");
    let body: serde_json::Value = ok.json().await.unwrap();
    assert_eq!(body["policy"], "opt_in");
    assert_eq!(body["user"]["id"], alice_id.as_str());
    assert_eq!(body["user"]["share_identified"], true);
    assert_eq!(body["totals"]["count"], 1);
    assert_eq!(body["top_models"][0]["model_id"], "llama-3-1-70b");

    // anonymous → 403 même avec consentement individuel.
    set_policy("anonymous");
    let anon = fx
        .client
        .get(&detail_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(anon.status(), 403, "anonymous → toujours 403");

    // identified → 200 même sans consentement individuel.
    set_policy("identified");
    let revoke = fx
        .client
        .put(format!("{}/api/v1/me/sharing", fx.base_url))
        .bearer_auth(&alice_access)
        .json(&json!({ "share_identified": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(revoke.status(), 200);
    let identified = fx
        .client
        .get(&detail_url)
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(identified.status(), 200, "identified → 200 sans opt-in");

    // Employé inconnu → 400 explicite (pas un 404 de routing).
    let unknown = fx
        .client
        .get(format!(
            "{}/api/v1/admin/users/01JUNKNOWNULID0000000000/analytics",
            fx.base_url
        ))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 400);
}

/// C43/C46 — le service worker MV3 de l'extension appelle l'API depuis une
/// origine `chrome-extension://…` : le serveur doit répondre au preflight
/// CORS, sinon Chrome bloque la réponse et l'envoi échoue en silence.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cors_allows_extension_origins_only() {
    let fx = AdminFixture::start().await;

    // Preflight depuis l'extension → autorisé, origine reflétée.
    let preflight = fx
        .client
        .request(
            reqwest::Method::OPTIONS,
            format!("{}/api/v1/estimations", fx.base_url),
        )
        .header("origin", "chrome-extension://abcdefghijklmnop")
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "authorization,content-type",
        )
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), 200);
    assert_eq!(
        preflight
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap()),
        Some("chrome-extension://abcdefghijklmnop")
    );
    let allow_headers = preflight
        .headers()
        .get("access-control-allow-headers")
        .map(|v| v.to_str().unwrap().to_ascii_lowercase())
        .unwrap_or_default();
    assert!(allow_headers.contains("authorization"));
    assert!(allow_headers.contains("content-type"));

    // Firefox aussi.
    let firefox = fx
        .client
        .request(
            reqwest::Method::OPTIONS,
            format!("{}/api/v1/estimations", fx.base_url),
        )
        .header("origin", "moz-extension://uuid-here")
        .header("access-control-request-method", "POST")
        .send()
        .await
        .unwrap();
    assert_eq!(
        firefox
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap()),
        Some("moz-extension://uuid-here")
    );

    // Un site web quelconque n'est PAS reflété (API à jeton, pas de raison
    // d'ouvrir aux origines https).
    let web = fx
        .client
        .get(format!("{}/health", fx.base_url))
        .header("origin", "https://evil.example")
        .send()
        .await
        .unwrap();
    assert!(
        web.headers().get("access-control-allow-origin").is_none(),
        "origine web non-extension ne doit pas être autorisée"
    );
}

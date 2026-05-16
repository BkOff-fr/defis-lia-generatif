//! Flow d'intégration C29.4 : admin crée un seuil → user envoie des
//! estimations qui dépassent → vérifie qu'un trigger est enregistré +
//! notifié via un webhook mock (wiremock).
//!
//! Pattern aligné sur `integration_admin.rs` (start fixture, login admin,
//! enroll user, exercise routes).

use std::net::TcpListener;
use std::time::Duration;

use serde_json::json;
use sobria_team_aggregator::{
    commands::{init, serve},
    config::DataPaths,
};
use tempfile::tempdir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct AlertsFixture {
    _tempdir: tempfile::TempDir,
    base_url: String,
    client: reqwest::Client,
    admin_access: String,
    user_access: String,
    handle: tokio::task::JoinHandle<()>,
}

impl AlertsFixture {
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

        // Admin login
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
        let tokens: serde_json::Value = resp.json().await.expect("admin tokens");
        let admin_access = tokens["access_token"].as_str().unwrap().to_string();

        // Crée un code et enroll un user (Alice)
        let codes = client
            .post(format!("{base_url}/api/v1/admin/codes"))
            .bearer_auth(&admin_access)
            .json(&json!({ "count": 1, "ttl_days": 7 }))
            .send()
            .await
            .unwrap();
        let codes_body: serde_json::Value = codes.json().await.unwrap();
        let code = codes_body["codes"][0]["code"].as_str().unwrap().to_string();

        let enroll = client
            .post(format!("{base_url}/api/v1/enroll"))
            .json(&json!({
                "code": code,
                "password": "user-pw-strong",
                "fingerprint": "fp-alice",
                "display_name": "Alice"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(enroll.status(), 200);
        let enroll_body: serde_json::Value = enroll.json().await.unwrap();
        let user_access = enroll_body["access_token"].as_str().unwrap().to_string();

        Self {
            _tempdir: dir,
            base_url,
            client,
            admin_access,
            user_access,
            handle,
        }
    }
}

impl Drop for AlertsFixture {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn alerts_create_list_delete_end_to_end() {
    let fx = AlertsFixture::start().await;

    // 0. Liste vide initialement
    let list = fx
        .client
        .get(format!("{}/api/v1/admin/alerts", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let list_body: serde_json::Value = list.json().await.unwrap();
    assert!(list_body["thresholds"].as_array().unwrap().is_empty());

    // 1. POST scope=user sans target_id → 400
    let bad = fx
        .client
        .post(format!("{}/api/v1/admin/alerts", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "scope": "user",
            "period": "daily",
            "gco2eq_max": 10.0,
            "notify_kind": "log_only"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);

    // 2. POST scope=team OK
    let created = fx
        .client
        .post(format!("{}/api/v1/admin/alerts", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "scope": "team",
            "period": "daily",
            "gco2eq_max": 100.0,
            "notify_kind": "log_only"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 200);
    let created_body: serde_json::Value = created.json().await.unwrap();
    let id = created_body["id"].as_str().unwrap().to_string();
    assert_eq!(created_body["threshold"]["scope"], "team");

    // 3. /admin/alerts liste le seuil
    let list = fx
        .client
        .get(format!("{}/api/v1/admin/alerts", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    let lb: serde_json::Value = list.json().await.unwrap();
    assert_eq!(lb["thresholds"].as_array().unwrap().len(), 1);

    // 4. DELETE soft → disabled=true
    let del = fx
        .client
        .delete(format!("{}/api/v1/admin/alerts/{id}", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    assert_eq!(del.status(), 200);
    let db: serde_json::Value = del.json().await.unwrap();
    assert_eq!(db["disabled"], true);

    // 5. DELETE idempotent → disabled=false (déjà désactivé)
    let del2 = fx
        .client
        .delete(format!("{}/api/v1/admin/alerts/{id}", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .send()
        .await
        .unwrap();
    let db2: serde_json::Value = del2.json().await.unwrap();
    assert_eq!(db2["disabled"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn alerts_fires_webhook_when_threshold_exceeded() {
    let fx = AlertsFixture::start().await;
    let webhook = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sobria-webhook"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1..)
        .mount(&webhook)
        .await;

    // 1. Admin crée un seuil team-wide, daily, max 5g, webhook → mock URL
    let webhook_url = format!("{}/sobria-webhook", webhook.uri());
    let create = fx
        .client
        .post(format!("{}/api/v1/admin/alerts", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "scope": "team",
            "period": "daily",
            "gco2eq_max": 5.0,
            "notify_kind": "webhook",
            "notify_target": webhook_url,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 200);

    // 2. User pousse 10 estimations à 2g (total = 20g > 5g) — UTC now()
    let now = chrono::Utc::now().to_rfc3339();
    for i in 0..10 {
        let r = fx
            .client
            .post(format!("{}/api/v1/estimations", fx.base_url))
            .bearer_auth(&fx.user_access)
            .json(&json!({
                "estimate": {
                    "method": "afnor_sobria",
                    "modelId": "llama-3-1-70b",
                    "tokensIn": 100,
                    "tokensOut": 200,
                    "gco2eq": 2.0,
                    "waterMl": 0.5,
                    "energyWh": 0.2
                },
                "ts": now
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200, "estimation {i} doit être ACKée");
    }

    // 3. Attend que le tokio::spawn ait eu le temps d'envoyer (le webhook est
    // fire-and-forget après ack).
    for _ in 0..50 {
        let triggers = fx
            .client
            .get(format!("{}/api/v1/admin/alerts/triggers", fx.base_url))
            .bearer_auth(&fx.admin_access)
            .send()
            .await
            .unwrap();
        let tb: serde_json::Value = triggers.json().await.unwrap();
        if !tb["triggers"].as_array().unwrap().is_empty() {
            let tr = &tb["triggers"][0];
            assert!(tr["observed_gco2eq"].as_f64().unwrap() > 5.0);
            // Un seul trigger malgré 10 estimations dépassant le seuil
            assert_eq!(tb["triggers"].as_array().unwrap().len(), 1);
            // Webhook reçu par le mock server
            webhook.verify().await;
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("aucun trigger inséré après 5s — alertes::checker non câblé ?");
}

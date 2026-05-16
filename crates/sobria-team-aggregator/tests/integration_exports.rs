//! Flow d'intégration C28.5 : admin login → enroll user → estimations
//! → POST /admin/exports/{csrd|prov-o|csv} → vérifie contenu+headers.

use std::net::TcpListener;
use std::time::Duration;

use serde_json::json;
use sobria_team_aggregator::{
    commands::{init, serve},
    config::DataPaths,
};
use tempfile::tempdir;

struct ExportsFixture {
    _tempdir: tempfile::TempDir,
    base_url: String,
    client: reqwest::Client,
    admin_access: String,
    handle: tokio::task::JoinHandle<()>,
}

impl ExportsFixture {
    async fn start_with_estimations() -> Self {
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

        // 1. Admin login
        let login: serde_json::Value = client
            .post(format!("{base_url}/api/v1/login"))
            .json(
                &json!({ "username": "admin", "password": "admin-pw-not-secret", "role": "admin" }),
            )
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let admin_access = login["access_token"].as_str().unwrap().to_string();

        // 2. POST /admin/codes → 2 codes
        let codes_resp: serde_json::Value = client
            .post(format!("{base_url}/api/v1/admin/codes"))
            .bearer_auth(&admin_access)
            .json(&json!({ "count": 2, "ttl_days": 7 }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let codes = codes_resp["codes"].as_array().unwrap();
        let code_alice = codes[0]["code"].as_str().unwrap().to_string();
        let code_bob = codes[1]["code"].as_str().unwrap().to_string();

        // 3. Enroll 2 users + soumets estimations
        for (code, fp, name, datapoints) in [
            (code_alice, "chrome-mac-alice", "Alice", vec![0.4, 0.3]),
            (code_bob, "firefox-linux-bob", "Bob", vec![0.5]),
        ] {
            let enroll: serde_json::Value = client
                .post(format!("{base_url}/api/v1/enroll"))
                .json(&json!({
                    "code": code,
                    "password": "user-pw-strong",
                    "fingerprint": fp,
                    "display_name": name
                }))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            let access = enroll["access_token"].as_str().unwrap();
            for g in datapoints {
                let r = client
                    .post(format!("{base_url}/api/v1/estimations"))
                    .bearer_auth(access)
                    .json(&json!({
                        "estimate": {
                            "method": "afnor_sobria",
                            "modelId": "llama-3-1-70b",
                            "tokensIn": 100,
                            "tokensOut": 500,
                            "gco2eq": g,
                            "waterMl": 1.5,
                            "energyWh": 0.2
                        },
                        "ts": "2026-05-16T10:00:00Z"
                    }))
                    .send()
                    .await
                    .unwrap();
                assert_eq!(r.status(), 200);
            }
        }

        Self {
            _tempdir: dir,
            base_url,
            client,
            admin_access,
            handle,
        }
    }
}

impl Drop for ExportsFixture {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_csrd_export_returns_pdf() {
    let fx = ExportsFixture::start_with_estimations().await;

    let resp = fx
        .client
        .post(format!("{}/api/v1/admin/exports/csrd", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "from": "2026-05-15T00:00:00Z",
            "to":   "2026-05-17T00:00:00Z",
            "entity_name": "Acme Corp"
        }))
        .send()
        .await
        .expect("csrd send");
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(ct.starts_with("application/pdf"), "content-type = {ct}");
    let disp = resp
        .headers()
        .get("content-disposition")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("")
        .to_string();
    assert!(disp.contains("attachment"));
    assert!(disp.contains("sobria-csrd"));

    let bytes = resp.bytes().await.unwrap();
    assert!(bytes.starts_with(b"%PDF-"), "magic %PDF-");
    assert!(bytes.len() > 1000, "PDF non-vide");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_provo_export_returns_jsonld_with_users() {
    let fx = ExportsFixture::start_with_estimations().await;

    let resp = fx
        .client
        .post(format!("{}/api/v1/admin/exports/prov-o", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "from": "2026-05-15T00:00:00Z",
            "to":   "2026-05-17T00:00:00Z",
            "entity_name": "Acme Corp",
            "anonymize": false
        }))
        .send()
        .await
        .expect("provo send");
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(ct.starts_with("application/ld+json"));

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["@context"].is_object());
    let graph = body["@graph"].as_array().unwrap();
    // bundle + aggregator + 2 users + 3 activities = 7
    assert_eq!(graph.len(), 7);
    let agents: Vec<_> = graph
        .iter()
        .filter(|n| n["@type"] == "prov:Agent")
        .collect();
    assert_eq!(agents.len(), 2, "1 agent par user");
    assert!(agents.iter().any(|a| a["sobria:displayName"] == "Alice"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_provo_export_can_anonymize() {
    let fx = ExportsFixture::start_with_estimations().await;

    let resp = fx
        .client
        .post(format!("{}/api/v1/admin/exports/prov-o", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "from": "2026-05-15T00:00:00Z",
            "to":   "2026-05-17T00:00:00Z",
            "anonymize": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let graph = body["@graph"].as_array().unwrap();
    for n in graph {
        if n["@type"] == "prov:Agent" {
            assert!(n["sobria:fingerprint"].is_null());
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_csv_export_returns_rows() {
    let fx = ExportsFixture::start_with_estimations().await;

    let resp = fx
        .client
        .post(format!("{}/api/v1/admin/exports/csv", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "from": "2026-05-15T00:00:00Z",
            "to":   "2026-05-17T00:00:00Z"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(ct.starts_with("text/csv"));

    let text = resp.text().await.unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4, "header + 3 estimations");
    assert!(lines[0].starts_with("ts,"));
    // Au moins une ligne mentionne Alice (non anonymisé).
    assert!(text.contains("Alice"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_exports_require_admin_role() {
    let fx = ExportsFixture::start_with_estimations().await;
    let nope = fx
        .client
        .post(format!("{}/api/v1/admin/exports/csv", fx.base_url))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(nope.status(), 401, "pas de Bearer → 401");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_exports_reject_inverted_window() {
    let fx = ExportsFixture::start_with_estimations().await;
    let bad = fx
        .client
        .post(format!("{}/api/v1/admin/exports/csv", fx.base_url))
        .bearer_auth(&fx.admin_access)
        .json(&json!({
            "from": "2026-05-20T00:00:00Z",
            "to":   "2026-05-15T00:00:00Z"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
}

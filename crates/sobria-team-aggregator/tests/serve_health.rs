//! Smoke test d'intégration : `init` → `serve` → `GET /health` en HTTPS.
//!
//! Le cert est auto-signé, donc `reqwest` est configuré pour accepter les
//! certs invalides (`danger_accept_invalid_certs`). C'est l'équivalent
//! programmatique du clic "Accepter le risque" qu'un user fera en C28.6.

use std::net::TcpListener;
use std::time::Duration;

use sobria_team_aggregator::{
    commands::{init, serve},
    config::DataPaths,
};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serve_responds_to_health() {
    let dir = tempdir().expect("tempdir");
    let paths = DataPaths::new(dir.path());

    init::run(&paths, "admin", "init-password-not-secret", false).expect("init");

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0");
    let addr = listener.local_addr().expect("local_addr");
    let paths_clone = paths.clone();

    let server = tokio::spawn(async move {
        let _ = serve::run_with_listener(listener, &paths_clone).await;
    });

    // Laisse au serveur le temps de démarrer la handshake TLS.
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("reqwest client");

    let url = format!("https://{addr}/health");
    let resp = retry_get(&client, &url).await;

    assert!(resp.status().is_success(), "status = {}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert_eq!(body["ok"], serde_json::Value::Bool(true));
    let version = body["version"].as_str().expect("version is string");
    assert!(!version.is_empty(), "version doit être non vide");

    server.abort();
}

/// Attend jusqu'à 2 secondes l'apparition du serveur (handshake TLS prêt).
async fn retry_get(client: &reqwest::Client, url: &str) -> reqwest::Response {
    let mut last_err = None;
    for _ in 0..20 {
        match client.get(url).send().await {
            Ok(r) => return r,
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            },
        }
    }
    panic!("serveur jamais joignable : {last_err:?}");
}

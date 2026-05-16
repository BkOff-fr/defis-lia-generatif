//! Smoke test C28.4 : le binaire sert le dashboard Svelte embedded.
//!
//! Vérifie :
//! - GET / → 200 text/html (index.html embedded).
//! - GET /admin/dashboard → 200 text/html (SPA fallback : route client-side
//!   inconnue côté serveur, on doit renvoyer index.html).
//! - GET /_app/version.json → 200 application/json (asset SvelteKit embedded).
//! - GET /health → 200 application/json (API toujours OK).
//! - GET /style.css.does-not-exist → 404 (path avec extension non trouvé).

use std::net::TcpListener;
use std::time::Duration;

use sobria_team_aggregator::{
    commands::{init, serve},
    config::DataPaths,
};
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_dashboard_is_served() {
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
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client");

    // Attend le démarrage du serveur.
    for _ in 0..30 {
        if let Ok(r) = client.get(format!("{base_url}/health")).send().await {
            if r.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 1. GET / → index.html
    let root = client.get(&base_url).send().await.expect("root send");
    assert_eq!(root.status(), 200);
    let ct = root
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(ct.starts_with("text/html"), "content-type = {ct}");
    let body = root.text().await.expect("root text");
    assert!(body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>"));
    assert!(body.contains("Sobr.ia"));

    // 2. GET /admin/dashboard → SPA fallback (same index.html shell)
    let dash = client
        .get(format!("{base_url}/admin/dashboard"))
        .send()
        .await
        .expect("dash send");
    assert_eq!(dash.status(), 200);
    let dash_body = dash.text().await.expect("dash text");
    assert!(dash_body.contains("Sobr.ia"));

    // 3. GET /_app/version.json (asset embedded)
    let v = client
        .get(format!("{base_url}/_app/version.json"))
        .send()
        .await
        .expect("version send");
    assert_eq!(v.status(), 200);
    let v_ct = v
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(
        v_ct.starts_with("application/json"),
        "content-type = {v_ct}"
    );

    // 4. /health reste joignable
    let h = client
        .get(format!("{base_url}/health"))
        .send()
        .await
        .expect("health send");
    assert_eq!(h.status(), 200);

    // 5. Path avec extension qui n'existe pas → 404 (pas de fallback agressif).
    let nf = client
        .get(format!("{base_url}/style.does-not-exist.css"))
        .send()
        .await
        .expect("nf send");
    assert_eq!(nf.status(), 404);

    handle.abort();
}

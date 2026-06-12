//! Commande `sobria-team-aggregator serve` — lance le serveur HTTPS.
//!
//! Deux entrées :
//!
//! - [`run`] : bind sur `bind:port`, attend indéfiniment. Utilisé par la CLI.
//! - [`run_with_listener`] : prend un `std::net::TcpListener` déjà bindé.
//!   Utilisé par les tests d'intégration pour récupérer un port aléatoire.

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;

use crate::config::DataPaths;
use crate::server::{build_router, tls as server_tls, ServerState};
use crate::storage::Storage;

/// Vérifie que l'init a été lancée + construit l'état partagé + le TLS.
fn prepare(paths: &DataPaths) -> Result<(ServerState, RustlsConfig)> {
    let storage = Storage::open(&paths.db()).context("open team.sqlite")?;
    let jwt_signing_key = storage
        .get_config(crate::commands::init::CFG_JWT_SIGNING_KEY)
        .context("lire jwt_signing_key")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "data dir {} non initialisé — exécuter `init` d'abord",
                paths.as_path().display()
            )
        })?;

    let server_config = server_tls::load_server_config_arc(&paths.cert(), &paths.key())
        .context("charger cert TLS")?;
    let tls = RustlsConfig::from_config(Arc::clone(&server_config));

    let state = ServerState::new(storage, jwt_signing_key);
    Ok((state, tls))
}

/// Lance le serveur en bindant `bind:port` (bloquant jusqu'à interruption).
pub async fn run(paths: &DataPaths, bind: &str, port: u16) -> Result<()> {
    let (state, tls) = prepare(paths)?;
    spawn_retention_task(state.clone());
    let app = build_router(state);
    let addr: SocketAddr = format!("{bind}:{port}")
        .parse()
        .with_context(|| format!("addr invalide: {bind}:{port}"))?;

    tracing::info!(%addr, "sobria-team-aggregator HTTPS en écoute");
    axum_server::bind_rustls(addr, tls)
        .serve(app.into_make_service())
        .await
        .context("axum_server serve")?;
    Ok(())
}

/// Variante test : utilise un listener déjà bindé (port arbitraire).
pub async fn run_with_listener(listener: TcpListener, paths: &DataPaths) -> Result<()> {
    let (state, tls) = prepare(paths)?;
    spawn_retention_task(state.clone());
    let app = build_router(state);

    // axum_server attend un `std::net::TcpListener` en non-blocking.
    listener
        .set_nonblocking(true)
        .context("set listener nonblocking")?;
    let addr = listener.local_addr().context("local_addr")?;
    tracing::info!(%addr, "sobria-team-aggregator HTTPS en écoute (listener externe)");

    axum_server::from_tcp_rustls(listener, tls)
        .serve(app.into_make_service())
        .await
        .context("axum_server serve from_tcp_rustls")?;
    Ok(())
}

// ─── Rétention des estimations (ADR-0015 « Conséquences ») ─────────────────

/// Purge au boot puis toutes les 24 h les estimations plus vieilles que
/// `config.retention_days` (défaut 730 j, plancher 30 — cf.
/// `commands::config::RUNTIME_KEYS`). Best-effort : un échec est loggé,
/// jamais fatal pour le serveur.
fn spawn_retention_task(state: crate::server::ServerState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
        loop {
            tick.tick().await;
            match run_retention_purge(&state) {
                Ok((days, 0)) => tracing::debug!(retention_days = days, "rétention: rien à purger"),
                Ok((days, n)) => {
                    tracing::info!(
                        retention_days = days,
                        purged = n,
                        "rétention: purge effectuée"
                    );
                },
                Err(e) => tracing::warn!(error = %e, "rétention: purge échouée (non-fatal)"),
            }
        }
    });
}

/// Lit `retention_days` (défaut/plancher via l'allow-list CLI) et purge.
fn run_retention_purge(state: &crate::server::ServerState) -> anyhow::Result<(i64, u64)> {
    use crate::commands::config::RUNTIME_KEYS;

    let spec = RUNTIME_KEYS
        .iter()
        .find(|k| k.key == "retention_days")
        .expect("retention_days présent dans RUNTIME_KEYS");
    let storage = state
        .storage
        .lock()
        .map_err(|_| anyhow::anyhow!("storage mutex poisoned"))?;
    let days = storage
        .get_config("retention_days")?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(spec.default)
        .max(spec.floor);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let purged = crate::storage::estimations::purge_older_than(storage.connection(), cutoff)?;
    Ok((days, purged))
}

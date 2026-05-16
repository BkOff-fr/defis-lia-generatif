//! Couche serveur HTTP du `sobria-team-aggregator`.
//!
//! C28.2 : routeur axum avec `/health` + `/api/v1/{enroll,login,refresh,
//! estimations,me/usage}`. Les routes admin (`/api/v1/admin/*`) arrivent
//! en C28.3, le dashboard statique en C28.4.

pub mod api;
pub mod auth;
pub mod embedded_web;
pub mod error;
pub mod routes;
pub mod tls;

use std::sync::{Arc, Mutex};

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::storage::Storage;

/// État partagé entre handlers. La `Storage` est encapsulée dans un
/// `Mutex` parce que `rusqlite::Connection` n'est pas `Sync`. Pour les
/// charges C28.2 (faible concurrence) ce n'est pas un goulot — on passera
/// à un pool en C28.4 si les benchs le demandent.
#[derive(Clone)]
pub struct ServerState {
    pub storage: Arc<Mutex<Storage>>,
    /// Clé HS256 (hex, lue de `config.jwt_signing_key` à `serve` startup).
    pub jwt_signing_key: Arc<String>,
}

impl ServerState {
    pub fn new(storage: Storage, jwt_signing_key: String) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
            jwt_signing_key: Arc::new(jwt_signing_key),
        }
    }
}

/// Construit l'application axum (routes + middleware tracing).
///
/// Ordre de matching axum :
/// 1. `/health` (routes statiques)
/// 2. `/api/v1/*` (API REST)
/// 3. `/` → index.html
/// 4. catch-all → static asset OU SPA fallback vers index.html
pub fn build_router(state: ServerState) -> Router {
    use axum::routing::get;
    Router::new()
        .merge(routes::router())
        .nest("/api/v1", api::router())
        .route("/", get(embedded_web::index))
        .fallback(get(embedded_web::handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

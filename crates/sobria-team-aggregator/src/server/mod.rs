//! Couche serveur HTTP du `sobria-team-aggregator`.
//!
//! C28.1 : routeur axum minimal avec uniquement `/health`. Les API REST
//! d'auth, d'admin, d'estimations et de dashboard seront branchées dans
//! les sous-chantiers C28.2 → C28.4.

pub mod routes;
pub mod tls;

use std::sync::{Arc, Mutex};

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::storage::Storage;

/// État partagé entre handlers. La `Storage` est encapsulée dans un
/// `Mutex` parce que `rusqlite::Connection` n'est pas `Sync`. Pour les
/// charges C28.1 (`/health`) ce n'est pas un goulot — on raffinera en
/// pool de connexions en C28.4 si les benchs le demandent.
#[derive(Clone)]
pub struct ServerState {
    pub storage: Arc<Mutex<Storage>>,
}

impl ServerState {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }
}

/// Construit l'application axum (routes + middleware tracing).
pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .merge(routes::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

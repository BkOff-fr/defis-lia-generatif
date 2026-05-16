//! Routes HTTP — pour C28.1 on n'expose que `/health`. Les routes auth /
//! API REST arrivent en C28.2.

use axum::{routing::get, Json, Router};
use serde::Serialize;

use super::ServerState;

/// Réponse du `/health`.
#[derive(Debug, Serialize)]
pub struct Health {
    pub ok: bool,
    pub version: &'static str,
}

/// Sous-routeur des routes publiques (santé). Agrégé dans
/// [`super::build_router`].
pub fn router() -> Router<ServerState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}

//! Routes `/api/v1/*` agrégées dans un sous-routeur.

pub mod enroll;
pub mod estimations;
pub mod login;
pub mod me;
pub mod refresh;

use axum::{
    routing::{get, post},
    Router,
};

use crate::server::ServerState;

/// Sous-routeur monté sous `/api/v1` par [`crate::server::build_router`].
pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/enroll", post(enroll::handle))
        .route("/login", post(login::handle))
        .route("/refresh", post(refresh::handle))
        .route("/estimations", post(estimations::handle))
        .route("/me/usage", get(me::handle))
}

//! Routes admin `/api/v1/admin/*`. Toutes exigent `AuthenticatedAdmin`.

pub mod analytics;
pub mod codes;
pub mod users;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::server::ServerState;

/// Sous-routeur monté sous `/admin` par [`crate::server::api::router`].
pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/users", get(users::handle))
        .route("/codes", post(codes::handle_create))
        .route("/codes/:id", delete(codes::handle_revoke))
        .route("/analytics", get(analytics::handle))
}

//! GET /api/v1/admin/users — liste des employés enrôlés + totaux.

use axum::{extract::State, Json};
use serde::Serialize;

use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::users::{self, UserWithTotals};

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserWithTotals>,
}

pub async fn handle(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
) -> ApiResult<Json<UsersResponse>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let users = users::list_all_with_totals(storage.connection())?;
    Ok(Json(UsersResponse { users }))
}

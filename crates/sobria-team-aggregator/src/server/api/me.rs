//! GET /api/v1/me/usage — usage agrégé de l'utilisateur courant.
//!
//! Pour C28.2 on retourne uniquement les totaux. Le breakdown
//! journalier / mensuel + comparaison équipe arrivent avec C28.3 / C28.4.

use axum::{extract::State, Json};
use serde::Serialize;

use crate::server::auth::middleware::AuthenticatedUser;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::estimations::{self, UsageTotals};

#[derive(Debug, Serialize)]
pub struct MyUsage {
    pub user_id: String,
    pub totals: UsageTotals,
}

pub async fn handle(
    State(state): State<ServerState>,
    user: AuthenticatedUser,
) -> ApiResult<Json<MyUsage>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let totals = estimations::totals_for_user(storage.connection(), &user.claims.sub)?;
    Ok(Json(MyUsage {
        user_id: user.claims.sub.clone(),
        totals,
    }))
}

//! GET /api/v1/me/usage — usage agrégé de l'utilisateur courant.
//! GET/PUT /api/v1/me/sharing — consentement de partage identifié (ADR-0015).
//!
//! Pour C28.2 on retourne uniquement les totaux. Le breakdown
//! journalier / mensuel + comparaison équipe arrivent avec C28.3 / C28.4.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::server::auth::middleware::AuthenticatedUser;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::estimations::{self, UsageTotals};
use crate::storage::users;

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

/// Consentement de partage identifié (ADR-0015 §3). Le salarié est seul
/// maître de ce flag : aucune route admin ne peut l'écrire.
///
/// `policy` (réponse uniquement, C44 — ADR-0016) : la politique de
/// visibilité de l'organisation, pour que chaque salarié sache sous quel
/// régime il travaille. Ignorée en écriture (PUT).
#[derive(Debug, Serialize, Deserialize)]
pub struct SharingState {
    pub share_identified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
}

/// GET /api/v1/me/sharing — lit le consentement courant + la politique.
pub async fn sharing_get(
    State(state): State<ServerState>,
    user: AuthenticatedUser,
) -> ApiResult<Json<SharingState>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let share_identified = users::share_identified(storage.connection(), &user.claims.sub)?;
    let pol = crate::policy::load(&storage);
    Ok(Json(SharingState {
        share_identified,
        policy: Some(pol.as_str().to_string()),
    }))
}

/// PUT /api/v1/me/sharing — écrit le consentement (idempotent).
pub async fn sharing_put(
    State(state): State<ServerState>,
    user: AuthenticatedUser,
    Json(body): Json<SharingState>,
) -> ApiResult<Json<SharingState>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    users::set_share_identified(
        storage.connection(),
        &user.claims.sub,
        body.share_identified,
    )?;
    let pol = crate::policy::load(&storage);
    Ok(Json(SharingState {
        share_identified: body.share_identified,
        policy: Some(pol.as_str().to_string()),
    }))
}

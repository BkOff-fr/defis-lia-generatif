//! Routes admin `/api/v1/admin/alerts` (C29.4).
//!
//! - `POST   /alerts`          : créer un seuil
//! - `GET    /alerts`          : lister les seuils (actifs et désactivés)
//! - `DELETE /alerts/:id`      : soft delete (pose `disabled_at`)
//! - `GET    /alerts/triggers` : historique des N derniers déclenchements

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::alerts::{
    delete_threshold, insert_threshold, list_thresholds_admin, list_triggers_admin, AlertPeriod,
    AlertScope, NewThreshold, NotifyKind, Threshold, TriggerRow,
};
use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub scope: AlertScope,
    #[serde(default)]
    pub target_id: Option<String>,
    pub period: AlertPeriod,
    pub gco2eq_max: f64,
    pub notify_kind: NotifyKind,
    #[serde(default)]
    pub notify_target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatedAlertResponse {
    pub id: String,
    pub threshold: Threshold,
}

pub async fn handle_create(
    State(state): State<ServerState>,
    admin: AuthenticatedAdmin,
    Json(req): Json<CreateAlertRequest>,
) -> ApiResult<Json<CreatedAlertResponse>> {
    // Validation côté handler (les checks de cohérence finale sont aussi dans
    // `insert_threshold` pour défense en profondeur).
    if !req.gco2eq_max.is_finite() || req.gco2eq_max <= 0.0 {
        return Err(ApiError::BadRequest("gco2eq_max doit être > 0".into()));
    }
    match (req.scope, req.target_id.as_deref()) {
        (AlertScope::User, None) => {
            return Err(ApiError::BadRequest("scope=user requiert target_id".into()));
        },
        (AlertScope::Team, Some(_)) => {
            return Err(ApiError::BadRequest("scope=team interdit target_id".into()));
        },
        _ => {},
    }
    if matches!(req.notify_kind, NotifyKind::Webhook | NotifyKind::Email)
        && req.notify_target.is_none()
    {
        return Err(ApiError::BadRequest(
            "notify_kind=webhook|email exige notify_target".into(),
        ));
    }

    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let id = Ulid::new().to_string();
    let now = Utc::now();
    insert_threshold(
        storage.connection(),
        &NewThreshold {
            id: &id,
            scope: req.scope,
            target_id: req.target_id.as_deref(),
            period: req.period,
            gco2eq_max: req.gco2eq_max,
            notify_kind: req.notify_kind,
            notify_target: req.notify_target.as_deref(),
            created_by_admin_id: &admin.claims.sub,
            created_at: now,
        },
    )?;
    let threshold = list_thresholds_admin(storage.connection())?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| ApiError::InternalMsg("seuil créé introuvable".into()))?;
    Ok(Json(CreatedAlertResponse { id, threshold }))
}

#[derive(Debug, Serialize)]
pub struct ListAlertsResponse {
    pub thresholds: Vec<Threshold>,
}

pub async fn handle_list(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
) -> ApiResult<Json<ListAlertsResponse>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let thresholds = list_thresholds_admin(storage.connection())?;
    Ok(Json(ListAlertsResponse { thresholds }))
}

#[derive(Debug, Serialize)]
pub struct DeleteAlertResponse {
    pub disabled: bool,
}

pub async fn handle_delete(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Path(id): Path<String>,
) -> ApiResult<Json<DeleteAlertResponse>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let disabled = delete_threshold(storage.connection(), &id, Utc::now())?;
    Ok(Json(DeleteAlertResponse { disabled }))
}

#[derive(Debug, Deserialize)]
pub struct TriggersQuery {
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    #[serde(default = "default_triggers_limit")]
    pub limit: u32,
}

fn default_triggers_limit() -> u32 {
    50
}

#[derive(Debug, Serialize)]
pub struct TriggersResponse {
    pub triggers: Vec<TriggerRow>,
}

pub async fn handle_triggers(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Query(q): Query<TriggersQuery>,
) -> ApiResult<Json<TriggersResponse>> {
    let limit = q.limit.clamp(1, 500);
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let triggers = list_triggers_admin(storage.connection(), q.from, q.to, limit)?;
    Ok(Json(TriggersResponse { triggers }))
}

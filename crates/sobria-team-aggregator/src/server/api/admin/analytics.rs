//! GET /api/v1/admin/analytics?from&to&group_by=day|week|month
//!
//! Retourne 4 sections agrégées sur la fenêtre `[from, to]` :
//! - `series`           : bucketing temporel selon `group_by` (défaut `day`).
//! - `top_models`       : top 10 modèles par gCO₂eq décroissant.
//! - `top_users`        : top 10 users (admin scope global).
//! - `method_breakdown` : afnor_sobria vs ecologits.
//!
//! Le paramètre `dim` (`user|model|method`) du brief reste accepté mais
//! ignoré en C28.3 — on renvoie tout ce qui peut servir au dashboard.
//! Le filtrage par `dim` peut être réintroduit en C28.4 si la viz le demande.

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::analytics::{self, GroupBy, MethodBreakdown, ModelTop, TimeBucket, UserTop};

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    /// RFC3339 ; par défaut : `now - 30 jours`.
    #[serde(default)]
    pub from: Option<String>,
    /// RFC3339 ; par défaut : `now`.
    #[serde(default)]
    pub to: Option<String>,
    /// `day` (défaut) | `week` | `month`.
    #[serde(default)]
    pub group_by: Option<String>,
    /// `user | model | method` — accepté mais ignoré en C28.3.
    #[allow(dead_code)]
    #[serde(default)]
    pub dim: Option<String>,
    /// Top N (défaut : 10, max 100).
    #[serde(default)]
    pub top: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub from: String,
    pub to: String,
    pub group_by: String,
    pub series: Vec<TimeBucket>,
    pub top_models: Vec<ModelTop>,
    pub top_users: Vec<UserTop>,
    pub method_breakdown: Vec<MethodBreakdown>,
}

pub async fn handle(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Query(q): Query<AnalyticsQuery>,
) -> ApiResult<Json<AnalyticsResponse>> {
    let now = Utc::now();
    let from = parse_or_default(q.from.as_deref(), || now - Duration::days(30))?;
    let to = parse_or_default(q.to.as_deref(), || now)?;
    if from >= to {
        return Err(ApiError::BadRequest("from doit être < to".into()));
    }
    let group_by = match q.group_by.as_deref().unwrap_or("day") {
        "day" => GroupBy::Day,
        "week" => GroupBy::Week,
        "month" => GroupBy::Month,
        other => {
            return Err(ApiError::BadRequest(format!(
                "group_by inconnu: {other} (day|week|month)"
            )))
        },
    };
    let top_n = q.top.unwrap_or(10).clamp(1, 100);

    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let conn = storage.connection();

    let series = analytics::time_buckets(conn, None, from, to, group_by)?;
    let top_models = analytics::top_models(conn, None, from, to, top_n)?;
    let top_users = analytics::top_users(conn, from, to, top_n)?;
    let method_breakdown = analytics::method_breakdown(conn, None, from, to)?;

    Ok(Json(AnalyticsResponse {
        from: from.to_rfc3339(),
        to: to.to_rfc3339(),
        group_by: q.group_by.unwrap_or_else(|| "day".to_string()),
        series,
        top_models,
        top_users,
        method_breakdown,
    }))
}

fn parse_or_default<F: FnOnce() -> DateTime<Utc>>(
    raw: Option<&str>,
    default_fn: F,
) -> Result<DateTime<Utc>, ApiError> {
    match raw {
        None | Some("") => Ok(default_fn()),
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| ApiError::BadRequest(format!("date RFC3339 invalide ({s}): {e}"))),
    }
}

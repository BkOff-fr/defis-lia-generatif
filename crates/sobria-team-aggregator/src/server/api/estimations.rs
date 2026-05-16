//! POST /api/v1/estimations — un user authentifié pousse une estimation.
//!
//! Le body est l'`EstimatePayload v1` de l'extension v0.6.0 (camelCase) :
//!
//! ```json
//! {
//!   "estimate": { "method": "afnor_sobria", "modelId": "llama-3-1-70b",
//!                 "tokensIn": 120, "tokensOut": 800, "gco2eq": 0.42,
//!                 "waterMl": 1.5, "energyWh": 0.42, ... },
//!   "host": "chatgpt",
//!   "modelDisplayName": "GPT-4o",
//!   "ts": "2026-05-16T12:34:56Z"
//! }
//! ```
//!
//! On extrait les champs typés pour `estimations` (colonnes dédiées
//! pour analytics) et on conserve le payload complet dans `raw_payload_json`
//! pour audit / reproductibilité.

use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::server::auth::middleware::AuthenticatedUser;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::{estimations, users};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimatePayload {
    pub estimate: Estimate,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub model_display_name: Option<String>,
    pub ts: DateTime<Utc>,
    /// Région optionnelle (l'extension actuelle n'envoie pas ce champ ;
    /// on l'accepte pour compat future).
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub method: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
}

#[derive(Debug, Serialize)]
pub struct EstimateAck {
    pub id: String,
    pub ack: bool,
}

pub async fn handle(
    State(state): State<ServerState>,
    user: AuthenticatedUser,
    Json(payload): Json<EstimatePayload>,
) -> ApiResult<Json<EstimateAck>> {
    // Garde-fous métiers : pas de méthode arbitraire, pas de tokens négatifs
    // (le type u32 protège déjà), pas de gCO₂eq aberrant.
    if !matches!(
        payload.estimate.method.as_str(),
        "afnor_sobria" | "ecologits"
    ) {
        return Err(ApiError::BadRequest(format!(
            "methode inconnue: {}",
            payload.estimate.method
        )));
    }
    if !payload.estimate.gco2eq.is_finite() || payload.estimate.gco2eq < 0.0 {
        return Err(ApiError::BadRequest("gco2eq invalide".into()));
    }

    let now = Utc::now();
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;

    // Préserve le payload complet pour audit + reproductibilité.
    let raw = serde_json::to_string(&serde_json::json!({
        "estimate": {
            "method": payload.estimate.method,
            "modelId": payload.estimate.model_id,
            "tokensIn": payload.estimate.tokens_in,
            "tokensOut": payload.estimate.tokens_out,
            "gco2eq": payload.estimate.gco2eq,
            "waterMl": payload.estimate.water_ml,
            "energyWh": payload.estimate.energy_wh,
        },
        "host": payload.host,
        "modelDisplayName": payload.model_display_name,
        "ts": payload.ts,
        "region": payload.region,
    }))
    .unwrap_or_else(|_| "{}".to_string());

    let id = Ulid::new().to_string();
    let new_estimation = estimations::NewEstimation {
        id: &id,
        user_id: &user.claims.sub,
        ts: payload.ts,
        method: &payload.estimate.method,
        model_id: &payload.estimate.model_id,
        tokens_in: payload.estimate.tokens_in,
        tokens_out: payload.estimate.tokens_out,
        gco2eq_p50: payload.estimate.gco2eq,
        gco2eq_p5: None,
        gco2eq_p95: None,
        water_ml: payload.estimate.water_ml,
        energy_wh: payload.estimate.energy_wh,
        region: payload.region.as_deref(),
        raw_payload_json: &raw,
        received_at: now,
    };
    estimations::insert(storage.connection(), &new_estimation)?;
    let _ = users::touch_last_seen(storage.connection(), &user.claims.sub, now);

    Ok(Json(EstimateAck { id, ack: true }))
}

//! POST /api/v1/admin/codes — crée N enrollment codes.
//! DELETE /api/v1/admin/codes/:id — révoque un code par id.
//!
//! Les codes ne sont affichés en clair QUE dans la réponse de POST.
//! Argon2id PHC en base, plus rien de réversible ensuite.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::enrollment_codes;

/// Limites raisonnables pour éviter abus.
const MAX_CODES_PER_BATCH: u32 = 500;
const MAX_TTL_DAYS: i64 = 365;

#[derive(Debug, Deserialize)]
pub struct CreateCodesRequest {
    pub count: u32,
    #[serde(default = "default_ttl_days")]
    pub ttl_days: i64,
}

fn default_ttl_days() -> i64 {
    7
}

#[derive(Debug, Serialize)]
pub struct CreatedCode {
    pub id: String,
    pub code: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCodesResponse {
    pub codes: Vec<CreatedCode>,
}

pub async fn handle_create(
    State(state): State<ServerState>,
    admin: AuthenticatedAdmin,
    Json(req): Json<CreateCodesRequest>,
) -> ApiResult<Json<CreateCodesResponse>> {
    if req.count == 0 || req.count > MAX_CODES_PER_BATCH {
        return Err(ApiError::BadRequest(format!(
            "count doit être ∈ [1, {MAX_CODES_PER_BATCH}]"
        )));
    }
    if req.ttl_days <= 0 || req.ttl_days > MAX_TTL_DAYS {
        return Err(ApiError::BadRequest(format!(
            "ttl_days doit être ∈ [1, {MAX_TTL_DAYS}]"
        )));
    }

    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let now = Utc::now();
    let expires_at = now + Duration::days(req.ttl_days);

    let mut out = Vec::with_capacity(req.count as usize);
    for _ in 0..req.count {
        let id = Ulid::new().to_string();
        let code = random_12_digit_code();
        enrollment_codes::insert(
            storage.connection(),
            &id,
            &code,
            &admin.claims.sub,
            now,
            expires_at,
        )?;
        out.push(CreatedCode {
            id,
            code,
            expires_at: expires_at.to_rfc3339(),
        });
    }
    Ok(Json(CreateCodesResponse { codes: out }))
}

#[derive(Debug, Serialize)]
pub struct RevokeResponse {
    pub revoked: bool,
}

pub async fn handle_revoke(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Path(id): Path<String>,
) -> ApiResult<Json<RevokeResponse>> {
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let revoked = enrollment_codes::revoke(storage.connection(), &id, Utc::now())?;
    Ok(Json(RevokeResponse { revoked }))
}

/// Cf. `commands::code::random_12_digit_code` — version dupliquée pour
/// pas exporter trop large. Les deux deviennent un helper partagé si on
/// les fait évoluer.
fn random_12_digit_code() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let mut acc: u128 = 0;
    for b in &bytes {
        acc = acc.wrapping_mul(31).wrapping_add(u128::from(*b));
    }
    let n: u64 = (acc % 1_000_000_000_000u128) as u64;
    format!("{n:012}")
}

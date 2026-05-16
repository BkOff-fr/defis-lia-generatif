//! POST /api/v1/enroll — un employé consomme un enrollment code pour
//! créer son compte user et obtenir un couple access/refresh tokens.

use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::crypto::password;
use crate::server::auth::{
    jwt::{self, Role},
    refresh,
};
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::{enrollment_codes, users};

#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    pub code: String,
    pub password: String,
    pub fingerprint: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnrollResponse {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: String,
    pub refresh_expires_at: String,
}

pub async fn handle(
    State(state): State<ServerState>,
    Json(req): Json<EnrollRequest>,
) -> ApiResult<Json<EnrollResponse>> {
    if req.code.len() != 12 || !req.code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest(
            "code doit être 12 chiffres ASCII".into(),
        ));
    }
    if req.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password ≥ 8 caractères requis".into(),
        ));
    }
    if req.fingerprint.trim().is_empty() {
        return Err(ApiError::BadRequest("fingerprint manquant".into()));
    }

    let now = Utc::now();
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;

    // 1. fingerprint déjà enrôlé ? (single-device en v0.7.0)
    if users::find_by_fingerprint(storage.connection(), &req.fingerprint)?.is_some() {
        return Err(ApiError::FingerprintAlreadyEnrolled);
    }

    // 2. code valide ?
    let code_row = enrollment_codes::verify_active_code(storage.connection(), &req.code, now)?
        .ok_or(ApiError::InvalidEnrollmentCode)?;

    // 3. crée le user
    let user_id = Ulid::new().to_string();
    let password_hash = password::hash_password(&req.password)?;
    users::insert(
        storage.connection(),
        &user_id,
        Some(&code_row.id),
        &req.fingerprint,
        &password_hash,
        req.display_name.as_deref(),
        now,
    )?;

    // 4. consomme le code (single-use)
    if !enrollment_codes::mark_used(storage.connection(), &code_row.id, &user_id, now)? {
        // Course concurrente : un autre client a consommé le code entre la
        // verif et le mark_used. On rejette ce nouvel utilisateur.
        return Err(ApiError::InvalidEnrollmentCode);
    }

    // 5. émet access + refresh tokens
    let access_token = jwt::issue(&state.jwt_signing_key, &user_id, Role::User, now)?;
    let access_expires_at = (now + jwt::ACCESS_TTL).to_rfc3339();
    let issued = refresh::issue(&storage, Some(&user_id), None, now)?;

    Ok(Json(EnrollResponse {
        user_id,
        access_token,
        refresh_token: issued.combined,
        access_expires_at,
        refresh_expires_at: issued.expires_at.to_rfc3339(),
    }))
}

//! POST /api/v1/login — authentification admin OU user.
//!
//! Pour user : `username = fingerprint` + `password` saisi à l'enrollment.
//! Pour admin : `username` admin + password. Le rôle est explicitement passé
//! pour éviter une confusion (un admin et un user ne peuvent pas partager
//! un nom d'utilisateur en pratique car les fingerprints sont préfixés
//! `chrome-*` / `firefox-*`, mais c'est plus propre de demander le rôle).

use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::crypto::password::verify_password;
use crate::server::auth::{
    jwt::{self, Role},
    refresh,
};
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::{admins, users};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<RoleHint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleHint {
    User,
    Admin,
}

impl From<RoleHint> for Role {
    fn from(h: RoleHint) -> Self {
        match h {
            RoleHint::User => Role::User,
            RoleHint::Admin => Role::Admin,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub role: &'static str,
    pub subject_id: String,
    pub access_expires_at: String,
    pub refresh_expires_at: String,
}

pub async fn handle(
    State(state): State<ServerState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    if req.password.is_empty() || req.username.trim().is_empty() {
        return Err(ApiError::BadRequest("username + password requis".into()));
    }

    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let now = Utc::now();
    let role = req.role.map(Role::from).unwrap_or(Role::User);

    match role {
        Role::Admin => {
            let admin = admins::find_by_username(storage.connection(), &req.username)?
                .ok_or(ApiError::InvalidCredentials)?;
            if !verify_password(&admin.password_hash, &req.password) {
                return Err(ApiError::InvalidCredentials);
            }
            admins::touch_last_login(storage.connection(), &admin.id, now)?;
            let access_token = jwt::issue(&state.jwt_signing_key, &admin.id, Role::Admin, now)?;
            let issued = refresh::issue(&storage, None, Some(&admin.id), now)?;
            Ok(Json(LoginResponse {
                access_token,
                refresh_token: issued.combined,
                role: "admin",
                subject_id: admin.id,
                access_expires_at: (now + jwt::ACCESS_TTL).to_rfc3339(),
                refresh_expires_at: issued.expires_at.to_rfc3339(),
            }))
        },
        Role::User => {
            // Pour les users on auth par fingerprint (= username) + password.
            let user = users::find_by_fingerprint(storage.connection(), &req.username)?
                .ok_or(ApiError::InvalidCredentials)?;
            if !verify_password(&user.password_hash, &req.password) {
                return Err(ApiError::InvalidCredentials);
            }
            users::touch_last_seen(storage.connection(), &user.id, now)?;
            let access_token = jwt::issue(&state.jwt_signing_key, &user.id, Role::User, now)?;
            let issued = refresh::issue(&storage, Some(&user.id), None, now)?;
            Ok(Json(LoginResponse {
                access_token,
                refresh_token: issued.combined,
                role: "user",
                subject_id: user.id,
                access_expires_at: (now + jwt::ACCESS_TTL).to_rfc3339(),
                refresh_expires_at: issued.expires_at.to_rfc3339(),
            }))
        },
    }
}

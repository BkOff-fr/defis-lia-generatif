//! Extracteurs axum d'authentification.
//!
//! - [`AuthenticatedUser`] : le token Bearer doit décoder en `role=user`.
//! - [`AuthenticatedAdmin`] : `role=admin`.
//!
//! Les deux lisent `Authorization: Bearer <jwt>`, vérifient la signature
//! avec la `jwt_signing_key` du `ServerState`, puis renvoient les claims.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::server::auth::jwt::{verify, Claims, Role};
use crate::server::error::ApiError;
use crate::server::ServerState;

/// Caller authentifié en `user`.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

/// Caller authentifié en `admin`.
#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub claims: Claims,
}

#[async_trait]
impl FromRequestParts<ServerState> for AuthenticatedUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let claims = extract_claims(parts, state).await?;
        if claims.role != Role::User {
            return Err(ApiError::Forbidden);
        }
        Ok(Self { claims })
    }
}

#[async_trait]
impl FromRequestParts<ServerState> for AuthenticatedAdmin {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let claims = extract_claims(parts, state).await?;
        if claims.role != Role::Admin {
            return Err(ApiError::Forbidden);
        }
        Ok(Self { claims })
    }
}

async fn extract_claims(parts: &Parts, state: &ServerState) -> Result<Claims, ApiError> {
    let header = parts
        .headers
        .get("authorization")
        .ok_or(ApiError::MissingAuth)?
        .to_str()
        .map_err(|_| ApiError::MissingAuth)?;
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or(ApiError::MissingAuth)?
        .trim();
    if token.is_empty() {
        return Err(ApiError::MissingAuth);
    }
    let signing_key = state.jwt_signing_key.as_str();
    verify(signing_key, token)
}

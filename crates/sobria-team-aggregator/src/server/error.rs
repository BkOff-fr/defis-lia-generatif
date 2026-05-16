//! Erreurs API typées → réponse HTTP JSON.
//!
//! Pour C28.2 on garde une enum simple ; on raffinera (codes machine,
//! détails par champ) en C28.3 quand le dashboard admin consommera les
//! erreurs.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use crate::error::AggregatorError;

/// Erreur publique d'une route API.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("identifiants invalides")]
    InvalidCredentials,

    #[error("code d'enrôlement invalide ou expiré")]
    InvalidEnrollmentCode,

    #[error("ce fingerprint est déjà enrôlé")]
    FingerprintAlreadyEnrolled,

    #[error("token invalide ou révoqué")]
    InvalidToken,

    #[error("authentification requise")]
    MissingAuth,

    #[error("rôle insuffisant pour cette opération")]
    Forbidden,

    #[error("payload invalide : {0}")]
    BadRequest(String),

    #[error(transparent)]
    Internal(#[from] AggregatorError),

    #[error("erreur interne : {0}")]
    InternalMsg(String),
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            ApiError::InvalidCredentials | ApiError::InvalidEnrollmentCode => {
                StatusCode::UNAUTHORIZED
            },
            ApiError::InvalidToken | ApiError::MissingAuth => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::FingerprintAlreadyEnrolled => StatusCode::CONFLICT,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal(_) | ApiError::InternalMsg(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            ApiError::InvalidCredentials => "invalid_credentials",
            ApiError::InvalidEnrollmentCode => "invalid_enrollment_code",
            ApiError::FingerprintAlreadyEnrolled => "fingerprint_already_enrolled",
            ApiError::InvalidToken => "invalid_token",
            ApiError::MissingAuth => "missing_auth",
            ApiError::Forbidden => "forbidden",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Internal(_) | ApiError::InternalMsg(_) => "internal_error",
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    code: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let msg = self.to_string();
        if matches!(self, ApiError::Internal(_) | ApiError::InternalMsg(_)) {
            tracing::error!(error = %msg, code, "API internal error");
        } else {
            tracing::debug!(error = %msg, code, "API client error");
        }
        let body = ErrorBody { error: &msg, code };
        (status, Json(body)).into_response()
    }
}

/// Alias pour les handlers axum.
pub type ApiResult<T> = Result<T, ApiError>;

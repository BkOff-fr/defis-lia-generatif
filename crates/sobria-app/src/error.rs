//! Erreurs IPC publiques.
//!
//! Toutes les commandes Tauri retournent `Result<T, IpcError>`. Côté
//! frontend, on récupère `T` ou une promesse rejetée avec un objet
//! `{ code, message, details? }`.

use serde::Serialize;
use thiserror::Error;

/// Alias `Result` pour les commandes IPC.
pub type IpcResult<T> = Result<T, IpcError>;

/// Erreur structurée envoyée vers le frontend via Tauri IPC.
///
/// Sérialisée en JSON :
/// ```json
/// { "code": "unknown_model",
///   "message": "model_id 'foo' inconnu",
///   "details": null }
/// ```
#[derive(Debug, Error, Serialize)]
#[error("{code}: {message}")]
pub struct IpcError {
    /// Code stable (machine-readable).
    pub code: &'static str,
    /// Message humain (déjà localisable côté front).
    pub message: String,
    /// Détails optionnels (chemin, ID fautif, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IpcError {
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Erreurs internes au crate `sobria-app`. Converties en `IpcError`
/// à la frontière de chaque commande.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("modèle inconnu : {0}")]
    UnknownModel(String),

    #[error("requête invalide : {0}")]
    InvalidRequest(String),

    #[error("estimator : {0}")]
    Estimator(#[from] sobria_estimator::EstimatorError),

    #[error("audit : {0}")]
    Audit(#[from] sobria_audit::AuditError),

    #[error("core : {0}")]
    Core(#[from] sobria_core::SobriaError),

    #[error("sqlite : {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io : {0}")]
    Io(#[from] std::io::Error),

    #[error("json : {0}")]
    Json(#[from] serde_json::Error),

    #[error("verrou empoisonné : {0}")]
    Poisoned(String),

    #[error("interne : {0}")]
    Internal(String),
}

impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self {
        let (code, message) = match &e {
            AppError::UnknownModel(id) => ("unknown_model", format!("model_id '{id}' inconnu")),
            AppError::InvalidRequest(m) => ("invalid_request", m.clone()),
            AppError::Estimator(_) => ("estimator_error", e.to_string()),
            AppError::Audit(_) => ("audit_error", e.to_string()),
            AppError::Core(_) => ("core_error", e.to_string()),
            AppError::Sqlite(_) => ("sqlite_error", e.to_string()),
            AppError::Io(_) => ("io_error", e.to_string()),
            AppError::Json(_) => ("json_error", e.to_string()),
            AppError::Poisoned(_) | AppError::Internal(_) => ("internal", e.to_string()),
        };
        IpcError::new(code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_error_serializes_with_optional_details() {
        let err = IpcError::new("unknown_model", "model 'foo' inconnu");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "unknown_model");
        assert_eq!(json["message"], "model 'foo' inconnu");
        assert!(json.get("details").is_none());
    }

    #[test]
    fn ipc_error_with_details() {
        let err = IpcError::new("invalid_request", "tokens_in trop grand")
            .with_details(serde_json::json!({ "tokens_in": 99_999_999 }));
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["details"]["tokens_in"], 99_999_999);
    }

    #[test]
    fn app_error_unknown_model_maps_correctly() {
        let app: AppError = AppError::UnknownModel("foo".into());
        let ipc: IpcError = app.into();
        assert_eq!(ipc.code, "unknown_model");
        assert!(ipc.message.contains("foo"));
    }
}

//! Erreurs publiques de génération de rapport.

use thiserror::Error;

pub type ExportResult<T> = Result<T, ExportError>;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("aucune entrée d'audit dans la période demandée")]
    EmptyPeriod,

    #[error("payload audit invalide : {0}")]
    InvalidAuditPayload(String),

    #[error("io : {0}")]
    Io(#[from] std::io::Error),

    #[error("json : {0}")]
    Json(#[from] serde_json::Error),

    #[error("pdf : {0}")]
    Pdf(String),

    #[error("interne : {0}")]
    Internal(String),
}

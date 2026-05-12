//! Erreurs publiques de `sobria-audit`.

use thiserror::Error;

/// Alias `Result` standard.
pub type AuditResult<T> = Result<T, AuditError>;

/// Catégories d'erreurs publiques.
#[derive(Debug, Error)]
pub enum AuditError {
    /// I/O système.
    #[error("io : {0}")]
    Io(#[from] std::io::Error),

    /// SQLite.
    #[error("sqlite : {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Sérialisation JSON.
    #[error("json : {0}")]
    Json(#[from] serde_json::Error),

    /// Signature invalide (entrée altérée).
    #[error("signature invalide pour l'entrée {id}")]
    InvalidSignature {
        /// ID de l'entrée fautive.
        id: i64,
    },

    /// Continuité de chaîne rompue.
    #[error("chaîne rompue : entrée {id} ne référence pas correctement la précédente")]
    BrokenChain {
        /// ID de l'entrée fautive.
        id: i64,
    },

    /// Erreur arbitraire.
    #[error("erreur : {0}")]
    Other(String),
}

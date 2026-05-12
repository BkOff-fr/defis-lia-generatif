//! Erreurs publiques de `sobria-ingest`.

use thiserror::Error;

/// Alias `Result` standard pour la crate.
pub type IngestResult<T> = Result<T, IngestError>;

/// Catégories d'erreurs publiques.
#[derive(Debug, Error)]
pub enum IngestError {
    /// I/O système.
    #[error("io : {0}")]
    Io(#[from] std::io::Error),

    /// (Dé)sérialisation JSON.
    #[error("json : {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP / réseau.
    #[error("http : {0}")]
    Http(#[from] reqwest::Error),

    /// Hash calculé ≠ hash attendu.
    #[error("hash mismatch (attendu {expected}, obtenu {actual})")]
    HashMismatch {
        /// Hash hexadécimal attendu.
        expected: String,
        /// Hash hexadécimal réellement calculé.
        actual: String,
    },

    /// Donnée non conforme au schéma.
    #[error("schéma : {0}")]
    Schema(String),

    /// Téléchargement interrompu sans pouvoir reprendre.
    #[error("téléchargement interrompu après {bytes} octets : {reason}")]
    DownloadInterrupted {
        /// Octets reçus avant interruption.
        bytes: u64,
        /// Cause humainement lisible.
        reason: String,
    },

    /// Source inconnue (id non enregistré dans le registry).
    #[error("source inconnue : {0}")]
    UnknownSource(String),

    /// Lineage rompu (un hash Copper référencé n'existe pas).
    #[error("lineage rompu : {0}")]
    BrokenLineage(String),

    /// Erreur arbitraire (à éviter — préférer une variante typée).
    #[error("erreur : {0}")]
    Other(String),
}

impl IngestError {
    /// Helper : crée une erreur de schéma.
    #[must_use]
    pub fn schema(msg: impl Into<String>) -> Self {
        Self::Schema(msg.into())
    }
}

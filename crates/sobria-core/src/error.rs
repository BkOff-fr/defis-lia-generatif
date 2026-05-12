//! Erreurs publiques de Sobr.ia.

use thiserror::Error;

/// Alias `Result` standard pour Sobr.ia.
pub type SobriaResult<T> = Result<T, SobriaError>;

/// Catégories d'erreur publiques.
#[derive(Debug, Error)]
pub enum SobriaError {
    /// Erreur d'entrée / sortie sur le disque.
    #[error("erreur io : {0}")]
    Io(#[from] std::io::Error),

    /// Erreur de sérialisation JSON.
    #[error("erreur json : {0}")]
    Json(#[from] serde_json::Error),

    /// Donnée manquante dans le référentiel.
    #[error("élément introuvable : {0}")]
    NotFound(String),

    /// Validation de schéma échouée.
    #[error("validation échouée : {0}")]
    SchemaValidation(String),

    /// Lineage cassé (Copper → Silver → Gold).
    #[error("lineage rompu : {0}")]
    BrokenLineage(String),

    /// Erreur méthodologique (hors plage admissible).
    #[error("erreur méthodologique : {0}")]
    Methodology(String),

    /// Autre erreur non catégorisée.
    #[error("erreur : {0}")]
    Other(String),
}

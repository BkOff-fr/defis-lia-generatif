//! Erreurs publiques de `sobria-estimator`.

use thiserror::Error;

/// Alias `Result` pour la crate.
pub type EstimatorResult<T> = Result<T, EstimatorError>;

/// Catégories d'erreurs publiques.
#[derive(Debug, Error)]
pub enum EstimatorError {
    /// Donnée d'entrée non conforme.
    #[error("schéma : {0}")]
    Schema(String),

    /// Échec de validation interne.
    #[error("validation : {0}")]
    Validation(String),

    /// Erreur arbitraire.
    #[error("erreur : {0}")]
    Other(String),
}

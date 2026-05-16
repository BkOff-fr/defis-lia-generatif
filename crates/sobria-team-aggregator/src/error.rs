//! Erreurs typées du binaire `sobria-team-aggregator`.
//!
//! Convention CLAUDE.md §4.1 : `thiserror` pour les erreurs publiques d'API,
//! `anyhow` pour les chemins binaires (CLI). Les variantes ci-dessous couvrent
//! le périmètre C28.1 — autres variantes (auth, JWT, exports) viendront avec
//! les sous-chantiers suivants.

use thiserror::Error;

/// Erreur du daemon `sobria-team-aggregator`.
#[derive(Debug, Error)]
pub enum AggregatorError {
    /// Échec I/O (lecture cert, écriture data dir, etc.).
    #[error("erreur I/O : {0}")]
    Io(#[from] std::io::Error),

    /// Échec de migration ou de requête SQLite.
    #[error("erreur SQLite : {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Échec de génération / parsing de certificat TLS auto-signé.
    #[error("erreur TLS : {0}")]
    Tls(String),

    /// Hash / vérification Argon2id en échec.
    #[error("erreur cryptographique : {0}")]
    Crypto(String),

    /// Erreur de configuration (data dir invalide, certif manquant, etc.).
    #[error("configuration invalide : {0}")]
    Config(String),

    /// Le data dir existe déjà et la commande `init` a été appelée sans `--force`.
    #[error("data dir déjà initialisé : {0} (utiliser `--force` pour réinitialiser)")]
    AlreadyInitialized(String),

    /// Erreur interne (validations métier, format inattendu, etc.).
    #[error("erreur interne : {0}")]
    Internal(String),
}

/// Alias pratique pour les chemins fallibles internes.
pub type AggregatorResult<T> = Result<T, AggregatorError>;

//! Chemins et constantes du binaire `sobria-team-aggregator`.
//!
//! Le data dir contient `team.sqlite`, `cert.pem` et `key.pem`. Par défaut
//! `./team-data/` (relatif au CWD). L'admin peut le remplacer via
//! `--data-dir`.

use std::path::{Path, PathBuf};

/// Nom du fichier SQLite principal.
pub const DB_FILENAME: &str = "team.sqlite";

/// Nom du fichier de certificat TLS (PEM).
pub const CERT_FILENAME: &str = "cert.pem";

/// Nom du fichier de clé privée TLS (PEM).
pub const KEY_FILENAME: &str = "key.pem";

/// Port HTTPS par défaut.
pub const DEFAULT_PORT: u16 = 8443;

/// Bind address par défaut (toutes les interfaces).
pub const DEFAULT_BIND: &str = "0.0.0.0";

/// Data dir par défaut (relatif au CWD au lancement).
pub const DEFAULT_DATA_DIR: &str = "./team-data";

/// Bundle de chemins dérivés du data dir.
#[derive(Debug, Clone)]
pub struct DataPaths {
    pub data_dir: PathBuf,
}

impl DataPaths {
    #[must_use]
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    #[must_use]
    pub fn db(&self) -> PathBuf {
        self.data_dir.join(DB_FILENAME)
    }

    #[must_use]
    pub fn cert(&self) -> PathBuf {
        self.data_dir.join(CERT_FILENAME)
    }

    #[must_use]
    pub fn key(&self) -> PathBuf {
        self.data_dir.join(KEY_FILENAME)
    }

    /// `true` si au moins un artefact critique existe déjà.
    #[must_use]
    pub fn already_initialized(&self) -> bool {
        self.db().exists() || self.cert().exists() || self.key().exists()
    }

    /// Crée le dossier s'il n'existe pas.
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.data_dir.as_path()
    }
}

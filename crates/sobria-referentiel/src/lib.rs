//! # sobria-referentiel
//!
//! Lecture du référentiel **Gold** produit par le pipeline médaillon
//! (`sobria-ingest` + ADR-0009 + chantier C26).
//!
//! Cette crate **lit** uniquement — elle n'écrit jamais. Les modifications
//! du référentiel passent toutes par la régénération du Gold via
//! `cargo run -p sobria-ingest -- pipeline run` ou `dvc repro`.
//!
//! ## Variables d'environnement
//!
//! - `SOBRIA_REFERENTIEL_PATH` — chemin du SQLite Gold à lire.
//!   Défaut : `data/gold/referentiel.sqlite`.
//!
//! ## Exemple
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! let r = sobria_referentiel::load()?;
//! let status = r.status()?;
//! println!("Référentiel v{} — {} sources, {} modèles",
//!     status.version, status.source_count, status.model_count);
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)] // « SQLite » est utilisé partout sans backticks dans les autres crates Sobr.ia

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Version de la crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Variable d'environnement honorée pour surcharger le chemin du SQLite.
pub const REFERENTIEL_PATH_ENV: &str = "SOBRIA_REFERENTIEL_PATH";

/// Chemin par défaut quand `SOBRIA_REFERENTIEL_PATH` n'est pas défini.
pub const DEFAULT_REFERENTIEL_PATH: &str = "data/gold/referentiel.sqlite";

/// Erreurs publiques.
#[derive(Debug, Error)]
pub enum ReferentielError {
    /// Fichier SQLite introuvable.
    #[error("référentiel introuvable : {0} (lancez `cargo run -p sobria-ingest -- pipeline run` ou `dvc pull`)")]
    NotFound(PathBuf),
    /// Erreur SQLite.
    #[error("sqlite : {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Erreur d'I/O.
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
}

/// Alias `Result` standard pour la crate.
pub type ReferentielResult<T> = Result<T, ReferentielError>;

/// Statut compact du référentiel — utilisé par l'IPC `get_referentiel_status`
/// et l'écran Paramètres pour afficher version, date, hash, comptages.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReferentielStatus {
    /// Version sémantique de la crate qui a produit le référentiel
    /// (héritée de `sobria-ingest`).
    pub version: String,
    /// Date de snapshot (RFC 3339).
    pub snapshot_date: DateTime<Utc>,
    /// SHA-256 du fichier `referentiel.sqlite` (intégrité).
    pub sha256: String,
    /// Nombre de sources contributrices (lignes table `sources`).
    pub source_count: u64,
    /// Nombre de modèles distincts (lignes table `model_overview`).
    pub model_count: u64,
    /// Chemin absolu du SQLite lu.
    pub path: String,
}

/// Handle de lecture vers le référentiel Gold.
pub struct Referentiel {
    path: PathBuf,
    conn: Connection,
}

impl Referentiel {
    /// Ouvre le référentiel en lecture seule.
    ///
    /// # Erreurs
    ///
    /// - [`ReferentielError::NotFound`] si le fichier n'existe pas.
    /// - [`ReferentielError::Sqlite`] si SQLite refuse l'ouverture.
    pub fn open(path: &Path) -> ReferentielResult<Self> {
        if !path.exists() {
            return Err(ReferentielError::NotFound(path.to_path_buf()));
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Ok(Self {
            path: path.to_path_buf(),
            conn,
        })
    }

    /// Statut compact (utilisé par l'IPC).
    pub fn status(&self) -> ReferentielResult<ReferentielStatus> {
        let source_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM sources", [], |r| r.get(0))?;
        // `model_overview` peut ne pas exister sur les vieux Gold (avant C26.3).
        let model_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='model_overview'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let model_count = if model_count > 0 {
            self.conn
                .query_row("SELECT COUNT(*) FROM model_overview", [], |r| r.get(0))
                .unwrap_or(0_i64)
        } else {
            0
        };

        let metadata = std::fs::metadata(&self.path)?;
        let snapshot_date: DateTime<Utc> = metadata
            .modified()
            .map_or_else(|_| Utc::now(), DateTime::<Utc>::from);

        let sha256 = sha256_file(&self.path)?;

        Ok(ReferentielStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            snapshot_date,
            sha256,
            source_count: u64::try_from(source_count).unwrap_or(0),
            model_count: u64::try_from(model_count).unwrap_or(0),
            path: self.path.to_string_lossy().to_string(),
        })
    }

    /// Accès brut à la connexion SQLite (lecture seule).
    /// Réservé aux modules qui ont besoin de requêter directement
    /// (`model_overview_fts` notamment).
    #[must_use]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Chemin du fichier SQLite ouvert.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Ouvre le référentiel en honorant `SOBRIA_REFERENTIEL_PATH` puis
/// [`DEFAULT_REFERENTIEL_PATH`].
pub fn load() -> ReferentielResult<Referentiel> {
    let path = std::env::var(REFERENTIEL_PATH_ENV)
        .map_or_else(|_| PathBuf::from(DEFAULT_REFERENTIEL_PATH), PathBuf::from);
    Referentiel::open(&path)
}

/// Calcule le SHA-256 d'un fichier en streaming (peu de RAM).
fn sha256_file(path: &Path) -> ReferentielResult<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    // Buffer 64 ko alloué sur le tas (clippy::large_stack_arrays au-delà de
    // 16 ko sur la pile).
    let mut buf = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
#[allow(unsafe_code)] // tests : `std::env::set_var` est unsafe en Rust 2024
mod tests {
    use super::*;

    /// Crée une mini SQLite de la forme attendue (sources + model_overview)
    /// dans un tempdir. Utilisée par les tests de `status()`.
    fn fake_referentiel(dir: &Path, n_sources: usize, n_models: usize) -> PathBuf {
        let path = dir.join("referentiel.sqlite");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            r"
            CREATE TABLE sources (id TEXT PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE model_overview (id TEXT PRIMARY KEY, name TEXT, family TEXT, vendor TEXT);
            ",
        )
        .unwrap();
        for i in 0..n_sources {
            conn.execute(
                "INSERT INTO sources(id, name) VALUES (?1, ?2)",
                rusqlite::params![format!("src_{i}"), format!("Source {i}")],
            )
            .unwrap();
        }
        for i in 0..n_models {
            conn.execute(
                "INSERT INTO model_overview(id, name, family, vendor) VALUES (?1, ?2, 'gpt', 'OpenAI')",
                rusqlite::params![format!("m_{i}"), format!("model {i}")],
            )
            .unwrap();
        }
        path
    }

    #[test]
    fn open_reports_not_found_for_missing_file() {
        // `Referentiel` n'implémente pas `Debug` (Connection sqlite ne le fait
        // pas), donc on extrait l'erreur via let…else.
        let Err(err) = Referentiel::open(Path::new("/inexistant/x.sqlite")) else {
            panic!("doit échouer pour fichier absent");
        };
        assert!(matches!(err, ReferentielError::NotFound(_)));
        let msg = format!("{err}");
        assert!(msg.contains("introuvable"), "msg : {msg}");
        assert!(msg.contains("dvc pull") || msg.contains("pipeline run"));
    }

    #[test]
    fn status_returns_counts_and_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let path = fake_referentiel(tmp.path(), 2, 5);
        let r = Referentiel::open(&path).unwrap();
        let s = r.status().unwrap();
        assert_eq!(s.source_count, 2);
        assert_eq!(s.model_count, 5);
        assert_eq!(s.sha256.len(), 64);
        assert!(s.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(s.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(s.path, path.to_string_lossy().to_string());
    }

    #[test]
    fn status_reports_zero_models_when_table_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("legacy.sqlite");
        // SQLite avec uniquement `sources` → pas de model_overview.
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE sources (id TEXT PRIMARY KEY);")
            .unwrap();
        conn.execute("INSERT INTO sources(id) VALUES ('x')", [])
            .unwrap();
        drop(conn);

        let r = Referentiel::open(&path).unwrap();
        let s = r.status().unwrap();
        assert_eq!(s.source_count, 1);
        assert_eq!(s.model_count, 0, "absence de model_overview → 0 (graceful)");
    }

    #[test]
    fn load_honors_env_var() {
        let tmp = tempfile::tempdir().unwrap();
        let path = fake_referentiel(tmp.path(), 1, 1);
        let prev = std::env::var(REFERENTIEL_PATH_ENV).ok();
        // SAFETY: tests sériels par défaut sur cette variable.
        unsafe {
            std::env::set_var(REFERENTIEL_PATH_ENV, &path);
        }
        let r = load().unwrap();
        assert_eq!(r.path(), path.as_path());
        // Restore.
        unsafe {
            match prev {
                Some(v) => std::env::set_var(REFERENTIEL_PATH_ENV, v),
                None => std::env::remove_var(REFERENTIEL_PATH_ENV),
            }
        }
    }

    #[test]
    fn sha256_file_is_stable() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("x.bin");
        std::fs::write(&p, b"hello world").unwrap();
        let h1 = sha256_file(&p).unwrap();
        let h2 = sha256_file(&p).unwrap();
        assert_eq!(h1, h2);
        // SHA-256("hello world") connu :
        assert_eq!(
            h1,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}

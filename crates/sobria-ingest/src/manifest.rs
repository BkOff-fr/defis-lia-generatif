//! Manifest Copper — fichier `manifest.json` associé à chaque snapshot.
//!
//! Voir ADR-0009 §"Couche Copper". Un manifest est :
//! - **immuable** : une fois écrit, il ne change plus.
//! - **complet** : il décrit tous les fichiers du snapshot.
//! - **vérifiable** : chaque entrée a son hash SHA-256.

use std::{collections::BTreeMap, path::Path};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{IngestError, IngestResult};

/// Version courante du schéma de manifest.
pub const MANIFEST_SCHEMA_VERSION: &str = "1";

/// Manifest Copper complet pour un snapshot d'une source.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CopperManifest {
    /// Version du schéma de manifest (toujours `"1"` pour l'instant).
    pub schema_version: String,
    /// Identifiant de la source (ex: `"comparia"`).
    pub source_id: String,
    /// Date de récupération UTC.
    pub fetched_at: DateTime<Utc>,
    /// Fichiers du snapshot (au moins un).
    pub files: Vec<ManifestFileEntry>,
    /// Libellé de licence (ex: `"Etalab 2.0"`).
    pub license: String,
    /// URL canonique de la licence (optionnelle).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_url: Option<String>,
}

/// Une entrée du manifest pour un fichier individuel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ManifestFileEntry {
    /// Nom relatif du fichier dans le snapshot (ex: `"conversations.parquet"`).
    pub name: String,
    /// URL d'origine du fichier (doit être HTTPS).
    pub url: String,
    /// SHA-256 hexadécimal sur 64 caractères minuscules.
    pub sha256: String,
    /// Taille en octets.
    pub size_bytes: u64,
    /// Headers HTTP utiles à la traçabilité (ETag, Last-Modified, …).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub http_headers: BTreeMap<String, String>,
}

impl CopperManifest {
    /// Crée un nouveau manifest pour la source donnée (vide, à compléter).
    #[must_use]
    pub fn new(source_id: impl Into<String>, license: impl Into<String>) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION.into(),
            source_id: source_id.into(),
            fetched_at: Utc::now(),
            files: Vec::new(),
            license: license.into(),
            license_url: None,
        }
    }

    /// Ajoute une entrée au manifest. Aucune validation à ce stade — appeler
    /// [`Self::validate`] avant `save`.
    pub fn add_file(&mut self, entry: ManifestFileEntry) {
        self.files.push(entry);
    }

    /// Sauvegarde le manifest en JSON pretty-printed (deterministe).
    pub async fn save(&self, path: &Path) -> IngestResult<()> {
        self.validate()?;
        let json = serde_json::to_vec_pretty(self)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Charge un manifest depuis un chemin et le valide.
    pub async fn load(path: &Path) -> IngestResult<Self> {
        let bytes = tokio::fs::read(path).await?;
        let manifest: Self = serde_json::from_slice(&bytes)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Recalcule le SHA-256 de chaque fichier du snapshot et compare au
    /// hash enregistré dans le manifest. Échoue à la première divergence avec
    /// un message identifiant le fichier fautif.
    ///
    /// `snapshot_dir` est le dossier qui contient le manifest et les fichiers
    /// référencés (tous les `entry.name` sont résolus relativement à ce
    /// chemin).
    pub async fn verify_files(&self, snapshot_dir: &Path) -> IngestResult<()> {
        use crate::hash;

        for entry in &self.files {
            let path = snapshot_dir.join(&entry.name);
            if !path.exists() {
                return Err(IngestError::schema(format!(
                    "fichier manquant : {} (attendu sous {})",
                    entry.name,
                    snapshot_dir.display()
                )));
            }
            let actual = hash::sha256_file(&path).await?;
            if !actual.eq_ignore_ascii_case(&entry.sha256) {
                return Err(IngestError::schema(format!(
                    "hash divergent pour {} : attendu {}, observé {}",
                    entry.name, entry.sha256, actual
                )));
            }
        }
        Ok(())
    }

    /// Valide tous les invariants du manifest.
    pub fn validate(&self) -> IngestResult<()> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(IngestError::schema(format!(
                "schema_version inattendue : {} (attendu {MANIFEST_SCHEMA_VERSION})",
                self.schema_version
            )));
        }
        if self.source_id.trim().is_empty() {
            return Err(IngestError::schema("source_id vide"));
        }
        if self.files.is_empty() {
            return Err(IngestError::schema(
                "files vide — un manifest doit contenir au moins une entrée",
            ));
        }
        if self.license.trim().is_empty() {
            return Err(IngestError::schema("license vide"));
        }
        for f in &self.files {
            f.validate()?;
        }
        Ok(())
    }
}

impl ManifestFileEntry {
    /// Valide une entrée individuelle.
    pub fn validate(&self) -> IngestResult<()> {
        if self.name.trim().is_empty() {
            return Err(IngestError::schema("entry.name vide"));
        }
        if !is_acceptable_url(&self.url) {
            return Err(IngestError::schema(format!(
                "entry.url doit être HTTPS (ou loopback http pour les tests) : {}",
                self.url
            )));
        }
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(IngestError::schema(format!(
                "entry.sha256 invalide : {:?} (attendu 64 hex chars)",
                self.sha256
            )));
        }
        Ok(())
    }
}

/// Vérifie qu'une URL est acceptable pour un manifest :
/// HTTPS partout, ou HTTP loopback (utile en tests avec wiremock).
fn is_acceptable_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://127.0.0.1")
        || url.starts_with("http://localhost")
        || url.starts_with("http://[::1]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_entry() -> ManifestFileEntry {
        let mut headers = BTreeMap::new();
        headers.insert("etag".into(), "\"abc123\"".into());
        ManifestFileEntry {
            name: "conversations.parquet".into(),
            url: "https://www.data.gouv.fr/api/1/datasets/r/abc".into(),
            sha256: "a".repeat(64),
            size_bytes: 715_456_000,
            http_headers: headers,
        }
    }

    fn sample_manifest() -> CopperManifest {
        let mut m = CopperManifest::new("comparia", "Etalab 2.0");
        m.license_url = Some("https://www.etalab.gouv.fr/licence-ouverte-open-licence".into());
        m.add_file(sample_entry());
        m
    }

    #[test]
    fn entry_validates_https() {
        let mut e = sample_entry();
        e.url = "http://insecure.example/".into();
        assert!(e.validate().is_err());
    }
    #[test]
    fn entry_accepts_loopback_http_for_tests() {
        for url in [
            "http://127.0.0.1:12345/file",
            "http://localhost:8080/x",
            "http://[::1]:9000/y",
        ] {
            let mut e = sample_entry();
            e.url = url.into();
            assert!(e.validate().is_ok(), "loopback {url} doit être accepté");
        }
    }

    #[test]
    fn entry_validates_hash_length() {
        let mut e = sample_entry();
        e.sha256 = "abc".into();
        assert!(e.validate().is_err());
    }

    #[test]
    fn entry_validates_hash_chars() {
        let mut e = sample_entry();
        e.sha256 = "z".repeat(64);
        assert!(e.validate().is_err());
    }

    #[test]
    fn entry_validates_name() {
        let mut e = sample_entry();
        e.name = "  ".into();
        assert!(e.validate().is_err());
    }

    #[test]
    fn manifest_validates_ok() {
        assert!(sample_manifest().validate().is_ok());
    }

    #[test]
    fn manifest_rejects_empty_files() {
        let mut m = CopperManifest::new("x", "MIT");
        assert!(m.validate().is_err());
        m.add_file(sample_entry());
        assert!(m.validate().is_ok());
    }

    #[test]
    fn manifest_rejects_bad_schema_version() {
        let mut m = sample_manifest();
        m.schema_version = "2".into();
        assert!(m.validate().is_err());
    }

    #[test]
    fn manifest_serializes_round_trip() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).unwrap();
        let back: CopperManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[tokio::test]
    async fn manifest_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        let m = sample_manifest();
        m.save(&path).await.unwrap();
        let loaded = CopperManifest::load(&path).await.unwrap();
        assert_eq!(loaded, m);
    }

    #[tokio::test]
    async fn manifest_save_rejects_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        let m = CopperManifest::new("comparia", "Etalab 2.0"); // pas de files
        assert!(m.save(&path).await.is_err());
    }

    #[tokio::test]
    async fn manifest_load_rejects_invalid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        tokio::fs::write(&path, br#"{"not_a_manifest": true}"#)
            .await
            .unwrap();
        assert!(CopperManifest::load(&path).await.is_err());
    }

    #[tokio::test]
    async fn verify_files_detects_missing() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = sample_manifest();
        // Fichier référencé absent
        assert!(manifest.verify_files(dir.path()).await.is_err());
    }

    #[tokio::test]
    async fn verify_files_ok_on_matching_hash() {
        let dir = tempfile::tempdir().unwrap();
        // Construit un fichier dont on connaît le hash SHA-256
        let content = b"hello world";
        tokio::fs::write(dir.path().join("conversations.parquet"), content)
            .await
            .unwrap();
        let mut entry = sample_entry();
        entry.sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".into();
        entry.size_bytes = content.len() as u64;
        let mut manifest = CopperManifest::new("comparia", "Etalab 2.0");
        manifest.add_file(entry);
        manifest.verify_files(dir.path()).await.unwrap();
    }

    #[tokio::test]
    async fn verify_files_detects_corruption() {
        let dir = tempfile::tempdir().unwrap();
        // Le manifest annonce un fichier de hash X, mais le contenu sur disque
        // est Y → divergence.
        tokio::fs::write(dir.path().join("conversations.parquet"), b"wrong content")
            .await
            .unwrap();
        let mut entry = sample_entry();
        entry.sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".into();
        let mut manifest = CopperManifest::new("comparia", "Etalab 2.0");
        manifest.add_file(entry);
        let err = manifest.verify_files(dir.path()).await.unwrap_err();
        assert!(
            format!("{err}").contains("hash divergent"),
            "msg attendu : « hash divergent » ; reçu : {err}"
        );
    }
}

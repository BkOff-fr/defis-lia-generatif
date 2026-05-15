//! Trait fondateur du pipeline médaillon — voir ADR-0009.

use std::path::Path;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{IngestError, IngestResult},
    lineage::CopperRef,
    manifest::CopperManifest,
};

/// Métadonnées descriptives d'une source.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SourceMeta {
    /// Identifiant stable et unique.
    pub id: String,
    /// Nom humainement lisible.
    pub name: String,
    /// URL principale documentaire.
    pub url: String,
    /// Licence des données.
    pub license: String,
    /// Fréquence amont de mise à jour.
    pub update_frequency: String,
    /// Tier (1, 2 ou 3).
    pub tier: u8,
}

/// Référence à un snapshot Copper figé.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CopperSnapshot {
    /// Source d'origine.
    pub source_id: String,
    /// Horodatage UTC de la récupération.
    pub fetched_at: DateTime<Utc>,
    /// Chemin du snapshot.
    pub path: std::path::PathBuf,
    /// Références fichier (un par fichier du snapshot, avec hash).
    pub files: Vec<CopperRef>,
    /// Licence détectée à l'ingestion.
    pub license: String,
}

impl CopperSnapshot {
    /// Reconstruit un [`CopperSnapshot`] depuis un dossier Copper persistant
    /// contenant un `manifest.json`. Permet à la sous-commande `silver` de
    /// repartir d'un snapshot existant **sans ré-ingérer la source**.
    ///
    /// L'intégrité des fichiers est vérifiée (`CopperManifest::verify_files`) :
    /// la fonction échoue si un fichier listé est absent ou si son SHA-256 ne
    /// correspond plus au manifest.
    ///
    /// # Erreurs
    ///
    /// - [`IngestError::Io`] si `manifest.json` est introuvable.
    /// - [`IngestError::Schema`] si le manifest est invalide ou si l'intégrité
    ///   échoue.
    pub async fn from_manifest(snapshot_dir: &Path) -> IngestResult<Self> {
        let manifest_path = snapshot_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(IngestError::schema(format!(
                "manifest.json introuvable sous {} — le snapshot Copper est incomplet",
                snapshot_dir.display()
            )));
        }
        let manifest = CopperManifest::load(&manifest_path).await?;
        manifest.verify_files(snapshot_dir).await?;
        let files = manifest
            .files
            .iter()
            .map(|f| CopperRef {
                source_id: manifest.source_id.clone(),
                manifest_path: manifest_path.clone(),
                file_name: f.name.clone(),
                file_sha256: f.sha256.clone(),
            })
            .collect();
        Ok(Self {
            source_id: manifest.source_id,
            fetched_at: manifest.fetched_at,
            path: snapshot_dir.to_path_buf(),
            files,
            license: manifest.license,
        })
    }
}

/// Une entité Silver écrite en Parquet.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SilverEntity {
    /// Nom de l'entité.
    pub name: String,
    /// Chemin du Parquet écrit.
    pub path: std::path::PathBuf,
    /// Version du schéma utilisé.
    pub schema_version: String,
    /// Références Copper d'origine (lineage).
    pub copper_refs: Vec<CopperRef>,
    /// Nombre de lignes écrites.
    pub row_count: u64,
}

/// Contribution d'une source au Gold final.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GoldContribution {
    /// Source contributrice.
    pub source_id: String,
    /// Tables Gold enrichies par cette source.
    pub tables_touched: Vec<String>,
    /// Notes méthodologiques.
    pub notes: Vec<String>,
}

/// Rapport de santé d'une source.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct HealthReport {
    /// `true` si la source est disponible.
    pub ok: bool,
    /// Message humain.
    pub message: String,
    /// Horodatage de la vérification.
    pub last_check: DateTime<Utc>,
}

impl HealthReport {
    /// Construit un rapport "OK".
    #[must_use]
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            last_check: Utc::now(),
        }
    }

    /// Construit un rapport "KO".
    #[must_use]
    pub fn ko(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            last_check: Utc::now(),
        }
    }
}

/// Trait unique du pipeline médaillon — voir ADR-0009.
#[async_trait]
pub trait DataLayer: Send + Sync {
    /// Identifiant stable et unique de la source.
    fn id(&self) -> &'static str;

    /// Métadonnées de la source.
    fn meta(&self) -> SourceMeta;

    /// Sources dont celle-ci dépend.
    fn depends_on(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Vérifie la disponibilité de la source.
    async fn health_check(&self, _ctx: &Context) -> IngestResult<HealthReport> {
        Ok(HealthReport::ok(format!(
            "{} : health check par défaut (non surchargé)",
            self.id()
        )))
    }

    /// Étape 1 — récupération brute → Copper.
    async fn ingest_copper(&self, ctx: &Context) -> IngestResult<CopperSnapshot>;

    /// Étape 2 — promotion vers Silver.
    async fn promote_silver(
        &self,
        snapshot: &CopperSnapshot,
        ctx: &Context,
    ) -> IngestResult<Vec<SilverEntity>>;

    /// Étape 3 — contribution à Gold.
    async fn contribute_gold(
        &self,
        silver: &[SilverEntity],
        ctx: &Context,
    ) -> IngestResult<GoldContribution>;
}

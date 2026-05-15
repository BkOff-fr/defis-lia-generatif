//! Source Compar:IA (Ministère de la Culture / Beta.gouv).
//!
//! Premier dataset du défi data.gouv.fr — voir CDC §0 et catalogue S01.
//!
//! Trois fichiers Parquet sont téléchargés depuis data.gouv.fr :
//! - `conversations.parquet` (~682 MB)
//! - `votes.parquet` (~733 MB)
//! - `reactions.parquet` (~3,4 GB)
//!
//! Licence : Licence Ouverte Etalab 2.0. Authentification : aucune.
//!
//! ## Stratégie Silver (v1)
//!
//! Le schéma Silver est volontairement laxe : on conserve toutes les colonnes
//! ComparIA d'origine et on ajoute deux colonnes systématiques de
//! traçabilité :
//! - `_copper_sha256` : SHA-256 du fichier Copper d'origine (lineage).
//! - `_ingested_at` : horodatage UTC d'écriture en Silver.
//!
//! Le mapping métier précis (extraction de `co2_eq`, `tokens`, `model_id`
//! typés) interviendra dans une v2 du schéma Silver une fois la spec
//! ComparIA validée. Voir `briefs/chantiers/C02-source-comparia.md`.
//!
//! ## Polars en contexte async
//!
//! Polars est synchrone bloquant. Chaque appel à `LazyFrame::scan_parquet`
//! et `ParquetWriter::finish` est encapsulé dans `tokio::task::spawn_blocking`
//! pour ne pas bloquer le runtime tokio.

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use tracing::{debug, info};

use crate::{
    context::Context,
    download::Downloader,
    error::{IngestError, IngestResult},
    layer::{CopperSnapshot, DataLayer, GoldContribution, HealthReport, SilverEntity, SourceMeta},
    lineage::CopperRef,
    manifest::{CopperManifest, ManifestFileEntry},
};

/// Identifiant stable de la source.
const SOURCE_ID: &str = "comparia";

/// Licence des données.
const LICENSE: &str = "Etalab 2.0";

/// URL canonique de la licence.
const LICENSE_URL: &str = "https://www.etalab.gouv.fr/licence-ouverte-open-licence";

/// Fichiers à télécharger : `(nom local, URL data.gouv.fr, nom de l'entité Silver)`.
const COMPARIA_FILES: &[(&str, &str, &str)] = &[
    (
        "conversations.parquet",
        "https://www.data.gouv.fr/api/1/datasets/r/7651fd0b-f222-43b3-8db8-ed6ae660d313",
        "comparia_conversations",
    ),
    (
        "votes.parquet",
        "https://www.data.gouv.fr/api/1/datasets/r/4ffc86e1-84a4-4fdc-9726-66408e596fef",
        "comparia_votes",
    ),
    (
        "reactions.parquet",
        "https://www.data.gouv.fr/api/1/datasets/r/9dd3d51f-4299-4193-ab46-81ae039fe1be",
        "comparia_reactions",
    ),
];

/// Source ComparIA — Tier 1 du défi data.gouv.fr.
pub struct ComparIASource {
    downloader: Downloader,
    /// URLs surchargées (injection en tests).
    override_urls: Option<Vec<(String, String, String)>>,
}

impl Default for ComparIASource {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparIASource {
    /// Construit la source avec ses URLs réelles data.gouv.fr.
    #[must_use]
    pub fn new() -> Self {
        Self {
            downloader: Downloader::new(),
            override_urls: None,
        }
    }

    /// Construit la source avec un downloader et des URLs personnalisés.
    /// Utile pour les tests (wiremock + URLs injectées).
    #[must_use]
    pub fn for_test(downloader: Downloader, urls: Vec<(String, String, String)>) -> Self {
        Self {
            downloader,
            override_urls: Some(urls),
        }
    }

    /// Retourne les triplets `(filename, url, silver_name)` à utiliser.
    fn files(&self) -> Vec<(String, String, String)> {
        if let Some(ref overrides) = self.override_urls {
            overrides.clone()
        } else {
            COMPARIA_FILES
                .iter()
                .map(|(f, u, s)| ((*f).to_string(), (*u).to_string(), (*s).to_string()))
                .collect()
        }
    }
}

#[async_trait]
impl DataLayer for ComparIASource {
    fn id(&self) -> &'static str {
        SOURCE_ID
    }

    fn meta(&self) -> SourceMeta {
        SourceMeta {
            id: SOURCE_ID.into(),
            name: "Compar:IA — Ministère de la Culture / Beta.gouv".into(),
            url: "https://www.data.gouv.fr/datasets/compar-ia".into(),
            license: LICENSE.into(),
            update_frequency: "trimestrielle".into(),
            tier: 1,
        }
    }

    async fn health_check(&self, _ctx: &Context) -> IngestResult<HealthReport> {
        // Health check léger : HEAD sur le premier fichier.
        // (En cas d'échec on retourne KO sans propager pour ne pas bloquer
        // l'ensemble du registry.)
        let url = &self.files()[0].1;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(IngestError::Http)?;
        match client.head(url).send().await {
            Ok(r) if r.status().is_success() => Ok(HealthReport::ok(format!(
                "data.gouv.fr joignable ({}).",
                r.status()
            ))),
            Ok(r) => Ok(HealthReport::ko(format!("statut HTTP {}", r.status()))),
            Err(e) => Ok(HealthReport::ko(format!("réseau : {e}"))),
        }
    }

    async fn ingest_copper(&self, ctx: &Context) -> IngestResult<CopperSnapshot> {
        let date_tag = Utc::now().format("%Y-%m-%d").to_string();
        let snapshot_dir = ctx.copper_root(SOURCE_ID).join(&date_tag);
        tokio::fs::create_dir_all(&snapshot_dir).await?;

        let mut manifest = CopperManifest::new(SOURCE_ID, LICENSE);
        manifest.license_url = Some(LICENSE_URL.into());
        let mut copper_refs = Vec::new();
        let manifest_path = snapshot_dir.join("manifest.json");

        for (filename, url, _silver_name) in self.files() {
            let dest = snapshot_dir.join(&filename);
            info!(source = SOURCE_ID, file = %filename, "copper: téléchargement");
            let outcome = self.downloader.fetch_to_file(&url, &dest, None).await?;

            manifest.add_file(ManifestFileEntry {
                name: filename.clone(),
                url: url.clone(),
                sha256: outcome.sha256.clone(),
                size_bytes: outcome.bytes,
                http_headers: outcome.headers,
            });
            copper_refs.push(CopperRef {
                source_id: SOURCE_ID.into(),
                manifest_path: manifest_path.clone(),
                file_name: filename,
                file_sha256: outcome.sha256,
            });
        }

        manifest.save(&manifest_path).await?;
        info!(
            source = SOURCE_ID,
            files = copper_refs.len(),
            "copper: snapshot écrit"
        );

        Ok(CopperSnapshot {
            source_id: SOURCE_ID.into(),
            fetched_at: Utc::now(),
            path: snapshot_dir,
            files: copper_refs,
            license: LICENSE.into(),
        })
    }

    async fn promote_silver(
        &self,
        snapshot: &CopperSnapshot,
        ctx: &Context,
    ) -> IngestResult<Vec<SilverEntity>> {
        let silver_root = ctx.silver_root(SOURCE_ID);
        tokio::fs::create_dir_all(&silver_root).await?;

        let file_to_silver: std::collections::HashMap<String, String> =
            self.files().into_iter().map(|(f, _u, s)| (f, s)).collect();

        let mut entities = Vec::new();
        for copper_ref in &snapshot.files {
            let silver_name = file_to_silver.get(&copper_ref.file_name).ok_or_else(|| {
                IngestError::schema(format!(
                    "fichier ComparIA inattendu : {}",
                    copper_ref.file_name
                ))
            })?;

            let copper_path = snapshot.path.join(&copper_ref.file_name);
            let silver_path = silver_root.join(format!("{silver_name}-v1.parquet"));

            info!(source = SOURCE_ID, entity = %silver_name, "silver: transformation");
            let row_count = transform_parquet_enrich(
                copper_path,
                silver_path.clone(),
                copper_ref.file_sha256.clone(),
            )
            .await?;
            debug!(rows = row_count, "silver: parquet écrit");

            let entity = SilverEntity {
                name: silver_name.clone(),
                path: silver_path,
                schema_version: "v1".into(),
                copper_refs: vec![copper_ref.clone()],
                row_count,
            };
            // Validation structurelle ADR-0009 : refuse de promouvoir une entité
            // dont le Parquet ne respecte pas son schéma versionné.
            crate::silver_validate::validate_silver(&entity).await?;
            entities.push(entity);
        }
        Ok(entities)
    }

    async fn contribute_gold(
        &self,
        silver: &[SilverEntity],
        _ctx: &Context,
    ) -> IngestResult<GoldContribution> {
        let tables_touched = silver.iter().map(|e| e.name.clone()).collect();
        Ok(GoldContribution {
            source_id: SOURCE_ID.into(),
            tables_touched,
            notes: vec![
                "ComparIA — Ministère de la Culture (Beta.gouv).".into(),
                "Méthodologie environnementale : EcoLogits (ISO 14044), intégrée en amont par la plateforme.".into(),
                format!(
                    "Snapshot Silver v1 = passthrough Parquet + colonnes _copper_sha256 et _ingested_at. \
                     Mapping métier typé en v2."
                ),
            ],
        })
    }
}

/// Transforme un Parquet Copper en Parquet Silver enrichi de deux colonnes
/// systématiques de traçabilité : `_copper_sha256` et `_ingested_at`.
///
/// Polars étant synchrone bloquant, on délègue à `spawn_blocking`.
async fn transform_parquet_enrich(
    input: PathBuf,
    output: PathBuf,
    copper_sha256: String,
) -> IngestResult<u64> {
    let join = tokio::task::spawn_blocking(move || -> IngestResult<u64> {
        use polars::prelude::*;

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let lf = LazyFrame::scan_parquet(&input, ScanArgsParquet::default())
            .map_err(|e| IngestError::Other(format!("scan_parquet({}): {e}", input.display())))?;

        let now_iso = Utc::now().to_rfc3339();
        let enriched = lf
            .with_column(lit(copper_sha256).alias("_copper_sha256"))
            .with_column(lit(now_iso).alias("_ingested_at"));

        let mut df = enriched
            .collect()
            .map_err(|e| IngestError::Other(format!("collect: {e}")))?;
        let row_count = u64::try_from(df.height()).unwrap_or(u64::MAX);

        let file = std::fs::File::create(&output)?;
        ParquetWriter::new(file)
            .finish(&mut df)
            .map_err(|e| IngestError::Other(format!("parquet_writer: {e}")))?;

        Ok(row_count)
    })
    .await
    .map_err(|e| IngestError::Other(format!("spawn_blocking join: {e}")))?;
    join
}

/// Détecte si un chemin pointe vers un fichier reconnu par cette source.
/// Helper public utile pour les tests d'intégration.
#[must_use]
pub fn is_comparia_file(name: &str) -> bool {
    COMPARIA_FILES.iter().any(|(f, _, _)| *f == name)
}

/// Retourne le nom Silver attendu pour un fichier ComparIA reconnu.
#[must_use]
pub fn silver_name_for(file: &str) -> Option<&'static str> {
    COMPARIA_FILES
        .iter()
        .find(|(f, _, _)| *f == file)
        .map(|(_, _, s)| *s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_files() {
        assert!(is_comparia_file("conversations.parquet"));
        assert!(is_comparia_file("votes.parquet"));
        assert!(is_comparia_file("reactions.parquet"));
        assert!(!is_comparia_file("random.parquet"));
    }

    #[test]
    fn silver_name_mapping() {
        assert_eq!(
            silver_name_for("conversations.parquet"),
            Some("comparia_conversations")
        );
        assert_eq!(silver_name_for("votes.parquet"), Some("comparia_votes"));
        assert_eq!(
            silver_name_for("reactions.parquet"),
            Some("comparia_reactions")
        );
        assert_eq!(silver_name_for("inconnu.parquet"), None);
    }

    #[test]
    fn meta_exposes_etalab_license() {
        let s = ComparIASource::new();
        let m = s.meta();
        assert_eq!(m.id, "comparia");
        assert_eq!(m.license, "Etalab 2.0");
        assert_eq!(m.tier, 1);
    }
}

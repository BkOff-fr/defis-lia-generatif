//! Source RTE / NaTran / Teréga IRIS (Open Data Réseaux Énergies).
//!
//! Deuxième dataset officiel du défi data.gouv.fr — voir CDC §0 et catalogue S02.
//!
//! Deux fichiers téléchargés depuis data.gouv.fr :
//! - `consommation_iris.csv` (~90 MB) — table tabulaire principale, promue en Silver.
//! - `iris_geometries.geojson` (~91 MB) — ressource cartographique, conservée
//!   uniquement en Copper (consommée par le module M12 plus tard).
//!
//! Licence : Licence Ouverte Etalab 2.0. Authentification : aucune.
//!
//! ## Stratégie Silver (v1)
//!
//! Identique à ComparIA : passthrough du CSV avec ajout de deux colonnes
//! systématiques `_copper_sha256` et `_ingested_at`. Une seule entité Silver
//! produite : `rte_iris_consommation`.
//!
//! Voir `briefs/chantiers/C03-source-rte-iris.md` pour la rationale complète.

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use tracing::{debug, info};

use crate::{
    context::Context,
    download::Downloader,
    error::{IngestError, IngestResult},
    layer::{
        CopperSnapshot, DataLayer, GoldContribution, HealthReport, SilverEntity, SourceMeta,
    },
    lineage::CopperRef,
    manifest::{CopperManifest, ManifestFileEntry},
};

/// Identifiant stable de la source.
const SOURCE_ID: &str = "rte-iris";

/// Licence des données.
const LICENSE: &str = "Etalab 2.0";

/// URL canonique de la licence.
const LICENSE_URL: &str = "https://www.etalab.gouv.fr/licence-ouverte-open-licence";

/// Nom de l'entité Silver produite à partir du CSV.
const SILVER_ENTITY: &str = "rte_iris_consommation";

/// Nom du fichier CSV de consommation (Copper).
const CSV_FILE: &str = "consommation_iris.csv";

/// Nom du fichier GeoJSON des géométries IRIS (Copper uniquement).
const GEOJSON_FILE: &str = "iris_geometries.geojson";

/// URL du CSV sur data.gouv.fr.
const CSV_URL: &str =
    "https://www.data.gouv.fr/api/1/datasets/r/631d6ec4-74c5-4f0f-9187-442cf9d1f0bc";

/// URL du GeoJSON sur data.gouv.fr.
const GEOJSON_URL: &str =
    "https://www.data.gouv.fr/api/1/datasets/r/2b584d52-f4e0-4232-87df-c456e715f334";

/// Source RTE/NaTran/Teréga IRIS — Tier 1 du défi data.gouv.fr.
pub struct RteIrisSource {
    downloader: Downloader,
    /// URLs surchargées (injection en tests).
    /// Format : `(csv_url, geojson_url)`.
    override_urls: Option<(String, String)>,
}

impl Default for RteIrisSource {
    fn default() -> Self {
        Self::new()
    }
}

impl RteIrisSource {
    /// Construit la source avec ses URLs réelles data.gouv.fr.
    #[must_use]
    pub fn new() -> Self {
        Self { downloader: Downloader::new(), override_urls: None }
    }

    /// Construit la source avec un downloader et des URLs personnalisés.
    /// Utile pour les tests.
    #[must_use]
    pub fn for_test(downloader: Downloader, csv_url: String, geojson_url: String) -> Self {
        Self {
            downloader,
            override_urls: Some((csv_url, geojson_url)),
        }
    }

    fn csv_url(&self) -> String {
        self.override_urls
            .as_ref()
            .map_or_else(|| CSV_URL.to_string(), |(c, _)| c.clone())
    }

    fn geojson_url(&self) -> String {
        self.override_urls
            .as_ref()
            .map_or_else(|| GEOJSON_URL.to_string(), |(_, g)| g.clone())
    }
}

#[async_trait]
impl DataLayer for RteIrisSource {
    fn id(&self) -> &'static str {
        SOURCE_ID
    }

    fn meta(&self) -> SourceMeta {
        SourceMeta {
            id: SOURCE_ID.into(),
            name: "RTE / NaTran / Teréga — Consommation industrielle IRIS (ODRÉ)".into(),
            url: "https://www.data.gouv.fr/datasets/consommation-annuelle-definitive-delectricite-et-de-gaz-par-iris-des-sites-industriels-raccordes-aux-reseaux-de-transport".into(),
            license: LICENSE.into(),
            update_frequency: "annuelle".into(),
            tier: 1,
        }
    }

    async fn health_check(&self, _ctx: &Context) -> IngestResult<HealthReport> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(IngestError::Http)?;
        match client.head(self.csv_url()).send().await {
            Ok(r) if r.status().is_success() => {
                Ok(HealthReport::ok(format!("ODRÉ joignable ({}).", r.status())))
            },
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

        // Fichier 1 : CSV de consommation
        info!(source = SOURCE_ID, file = %CSV_FILE, "copper: téléchargement");
        let csv_dest = snapshot_dir.join(CSV_FILE);
        let csv_url = self.csv_url();
        let csv_outcome = self.downloader.fetch_to_file(&csv_url, &csv_dest, None).await?;
        manifest.add_file(ManifestFileEntry {
            name: CSV_FILE.into(),
            url: csv_url,
            sha256: csv_outcome.sha256.clone(),
            size_bytes: csv_outcome.bytes,
            http_headers: csv_outcome.headers,
        });
        copper_refs.push(CopperRef {
            source_id: SOURCE_ID.into(),
            manifest_path: manifest_path.clone(),
            file_name: CSV_FILE.into(),
            file_sha256: csv_outcome.sha256,
        });

        // Fichier 2 : GeoJSON des géométries IRIS (Copper uniquement)
        info!(source = SOURCE_ID, file = %GEOJSON_FILE, "copper: téléchargement");
        let geojson_dest = snapshot_dir.join(GEOJSON_FILE);
        let geojson_url = self.geojson_url();
        let geojson_outcome = self
            .downloader
            .fetch_to_file(&geojson_url, &geojson_dest, None)
            .await?;
        manifest.add_file(ManifestFileEntry {
            name: GEOJSON_FILE.into(),
            url: geojson_url,
            sha256: geojson_outcome.sha256.clone(),
            size_bytes: geojson_outcome.bytes,
            http_headers: geojson_outcome.headers,
        });
        copper_refs.push(CopperRef {
            source_id: SOURCE_ID.into(),
            manifest_path: manifest_path.clone(),
            file_name: GEOJSON_FILE.into(),
            file_sha256: geojson_outcome.sha256,
        });

        manifest.save(&manifest_path).await?;
        info!(source = SOURCE_ID, files = copper_refs.len(), "copper: snapshot écrit");

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

        // Trouve la référence Copper du CSV (le GeoJSON n'est pas promu en Silver).
        let csv_ref = snapshot
            .files
            .iter()
            .find(|r| r.file_name == CSV_FILE)
            .ok_or_else(|| {
                IngestError::schema(format!(
                    "snapshot RTE IRIS sans {CSV_FILE} — promotion Silver impossible"
                ))
            })?;

        let copper_path = snapshot.path.join(&csv_ref.file_name);
        let silver_path = silver_root.join(format!("{SILVER_ENTITY}-v1.parquet"));

        info!(source = SOURCE_ID, entity = %SILVER_ENTITY, "silver: transformation CSV→Parquet");
        let row_count = transform_csv_enrich(
            copper_path,
            silver_path.clone(),
            csv_ref.file_sha256.clone(),
        )
        .await?;
        debug!(rows = row_count, "silver: parquet écrit");

        Ok(vec![SilverEntity {
            name: SILVER_ENTITY.into(),
            path: silver_path,
            schema_version: "v1".into(),
            copper_refs: vec![csv_ref.clone()],
            row_count,
        }])
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
                "RTE / NaTran / Teréga via ODRÉ (Open Data Réseaux Énergies).".into(),
                "Consommation industrielle annuelle élec + gaz par maille IRIS (référentiel INSEE 2023).".into(),
                "GeoJSON des géométries IRIS conservé en Copper, consommé directement par le module M12 (Territoire français).".into(),
            ],
        })
    }
}

/// Lit le CSV ODRÉ, enrichit avec `_copper_sha256` + `_ingested_at`,
/// écrit un Parquet Silver. Polars synchrone — délégué à `spawn_blocking`.
async fn transform_csv_enrich(
    input: PathBuf,
    output: PathBuf,
    copper_sha256: String,
) -> IngestResult<u64> {
    let join = tokio::task::spawn_blocking(move || -> IngestResult<u64> {
        use polars::prelude::*;

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Le CSV ODRÉ est en UTF-8 avec virgule comme séparateur dans la dernière
        // édition observée. On reste sur la config par défaut de LazyCsvReader.
        let lf = LazyCsvReader::new(&input)
            .with_has_header(true)
            .finish()
            .map_err(|e| IngestError::Other(format!("csv_reader({}): {e}", input.display())))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_exposes_etalab_license_and_tier_1() {
        let s = RteIrisSource::new();
        let m = s.meta();
        assert_eq!(m.id, "rte-iris");
        assert_eq!(m.license, "Etalab 2.0");
        assert_eq!(m.tier, 1);
        assert_eq!(m.update_frequency, "annuelle");
    }

    #[test]
    fn override_urls_used_in_test_mode() {
        let s = RteIrisSource::for_test(
            Downloader::new(),
            "http://localhost/csv".into(),
            "http://localhost/geo".into(),
        );
        assert_eq!(s.csv_url(), "http://localhost/csv");
        assert_eq!(s.geojson_url(), "http://localhost/geo");
    }

    #[test]
    fn default_urls_when_no_override() {
        let s = RteIrisSource::new();
        assert!(s.csv_url().starts_with("https://www.data.gouv.fr/"));
        assert!(s.geojson_url().starts_with("https://www.data.gouv.fr/"));
    }
}

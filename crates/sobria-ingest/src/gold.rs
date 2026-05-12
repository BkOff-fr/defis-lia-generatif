//! Assemblage de la couche Gold du pipeline médaillon.
//!
//! Produit 4 artefacts dans `data/gold/` :
//! - `referentiel.sqlite` — base SQLite WAL (sources, silver_entities, lineage).
//! - `analytics.parquet` — catalogue Parquet des entités Silver (DuckDB-friendly).
//! - `datasheet.jsonld` — Datasheet for Datasets PROV-O + schema.org.
//! - `MANIFEST.sha256` — hashes SHA-256 des artefacts (format `sha256sum`).
//!
//! Voir ADR-0009 §"Gold Layer" et `briefs/chantiers/C04-gold-assembly.md`.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::{
    context::Context,
    error::{IngestError, IngestResult},
    hash,
    layer::SourceMeta,
    lineage::GoldLineage,
    registry::StepResult,
};

/// Chemins des 4 artefacts Gold produits.
#[derive(Debug, Clone)]
pub struct GoldArtifacts {
    /// Base SQLite (référentiel transactionnel).
    pub referentiel_sqlite: PathBuf,
    /// Parquet catalogue (lecture DuckDB).
    pub analytics_parquet: PathBuf,
    /// Datasheet PROV-O / schema.org.
    pub datasheet_jsonld: PathBuf,
    /// Manifest SHA-256 des artefacts.
    pub manifest_sha256: PathBuf,
}

/// Assemble la couche Gold à partir des entités Silver et du lineage.
///
/// Orchestrateur principal du module — appelé par
/// [`crate::LayerRegistry::run_full_pipeline`].
pub async fn assemble_gold(
    ctx: &Context,
    silver_results: &[StepResult<Vec<crate::SilverEntity>>],
    sources_meta: &[SourceMeta],
    lineage: &GoldLineage,
) -> IngestResult<GoldArtifacts> {
    let gold_root = ctx.gold_root();
    tokio::fs::create_dir_all(&gold_root).await?;

    let sqlite = gold_root.join("referentiel.sqlite");
    let parquet = gold_root.join("analytics.parquet");
    let datasheet = gold_root.join("datasheet.jsonld");
    let manifest = gold_root.join("MANIFEST.sha256");

    // 1. SQLite — exécuté en spawn_blocking car rusqlite est sync.
    build_referentiel_sqlite(
        sqlite.clone(),
        silver_results_clone(silver_results),
        sources_meta.to_vec(),
    )
    .await?;

    // 2. Parquet catalogue.
    build_analytics_parquet(parquet.clone(), silver_results_clone(silver_results)).await?;

    // 3. JSON-LD à partir de la lignée.
    let jsonld = lineage.to_jsonld();
    let pretty = serde_json::to_vec_pretty(&jsonld)?;
    tokio::fs::write(&datasheet, &pretty).await?;

    // 4. Manifest SHA-256 (hash des 3 premiers fichiers).
    write_manifest_sha256(&manifest, &[&sqlite, &parquet, &datasheet]).await?;

    Ok(GoldArtifacts {
        referentiel_sqlite: sqlite,
        analytics_parquet: parquet,
        datasheet_jsonld: datasheet,
        manifest_sha256: manifest,
    })
}

/// Clone propre des StepResult pour les passer dans spawn_blocking.
fn silver_results_clone(
    results: &[StepResult<Vec<crate::SilverEntity>>],
) -> Vec<(String, Vec<crate::SilverEntity>)> {
    results
        .iter()
        .filter_map(|r| match &r.result {
            Ok(entities) => Some((r.source_id.clone(), entities.clone())),
            Err(_) => None,
        })
        .collect()
}

/// Crée le fichier `referentiel.sqlite` avec ses 3 tables peuplées.
///
/// Mode WAL activé pour permettre lectures concurrentes.
async fn build_referentiel_sqlite(
    path: PathBuf,
    silver: Vec<(String, Vec<crate::SilverEntity>)>,
    sources: Vec<SourceMeta>,
) -> IngestResult<()> {
    tokio::task::spawn_blocking(move || -> IngestResult<()> {
        // S'il y a un fichier antérieur, on l'écrase pour un build déterministe.
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let mut conn = Connection::open(&path)
            .map_err(|e| IngestError::Other(format!("open sqlite: {e}")))?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| IngestError::Other(format!("wal: {e}")))?;
        conn.execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                license TEXT NOT NULL,
                update_frequency TEXT NOT NULL,
                tier INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS silver_entities (
                entity_name TEXT NOT NULL,
                source_id TEXT NOT NULL,
                schema_version TEXT NOT NULL,
                parquet_path TEXT NOT NULL,
                row_count INTEGER NOT NULL,
                PRIMARY KEY (entity_name, source_id),
                FOREIGN KEY (source_id) REFERENCES sources(id)
            );
            CREATE TABLE IF NOT EXISTS lineage (
                copper_sha256 TEXT NOT NULL,
                silver_entity TEXT NOT NULL,
                source_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                PRIMARY KEY (copper_sha256, silver_entity)
            );
            ",
        )
        .map_err(|e| IngestError::Other(format!("schema: {e}")))?;

        let tx = conn.transaction()
            .map_err(|e| IngestError::Other(format!("tx: {e}")))?;

        for src in &sources {
            tx.execute(
                "INSERT OR REPLACE INTO sources \
                 (id, name, url, license, update_frequency, tier) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![src.id, src.name, src.url, src.license, src.update_frequency, src.tier],
            )
            .map_err(|e| IngestError::Other(format!("insert sources: {e}")))?;
        }

        for (source_id, entities) in &silver {
            for e in entities {
                tx.execute(
                    "INSERT OR REPLACE INTO silver_entities \
                     (entity_name, source_id, schema_version, parquet_path, row_count) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        e.name,
                        source_id,
                        e.schema_version,
                        e.path.to_string_lossy(),
                        // SQLite n'a pas d'unsigned int : cast en i64 (safe pour les volumes attendus).
                        i64::try_from(e.row_count).unwrap_or(i64::MAX)
                    ],
                )
                .map_err(|err| IngestError::Other(format!("insert silver_entities: {err}")))?;

                for r in &e.copper_refs {
                    tx.execute(
                        "INSERT OR REPLACE INTO lineage \
                         (copper_sha256, silver_entity, source_id, file_name) \
                         VALUES (?1, ?2, ?3, ?4)",
                        params![r.file_sha256, e.name, r.source_id, r.file_name],
                    )
                    .map_err(|err| IngestError::Other(format!("insert lineage: {err}")))?;
                }
            }
        }

        tx.commit()
            .map_err(|e| IngestError::Other(format!("commit: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|e| IngestError::Other(format!("spawn_blocking sqlite: {e}")))?
}

/// Crée le fichier `analytics.parquet` (catalogue tabulaire des entités Silver).
async fn build_analytics_parquet(
    path: PathBuf,
    silver: Vec<(String, Vec<crate::SilverEntity>)>,
) -> IngestResult<()> {
    tokio::task::spawn_blocking(move || -> IngestResult<()> {
        use polars::prelude::*;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Collecte des lignes.
        let mut entity_name = Vec::new();
        let mut source_id = Vec::new();
        let mut schema_version = Vec::new();
        let mut row_count = Vec::new();
        let mut copper_sha256_list = Vec::new();

        for (src, entities) in &silver {
            for e in entities {
                entity_name.push(e.name.clone());
                source_id.push(src.clone());
                schema_version.push(e.schema_version.clone());
                row_count.push(i64::try_from(e.row_count).unwrap_or(i64::MAX));
                let hashes: Vec<String> =
                    e.copper_refs.iter().map(|r| r.file_sha256.clone()).collect();
                copper_sha256_list.push(hashes.join(","));
            }
        }

        // Si aucune entité, on écrit un Parquet vide mais valide (schéma explicite).
        let mut df = if entity_name.is_empty() {
            DataFrame::new(vec![
                Series::new_empty("entity_name".into(), &DataType::String).into(),
                Series::new_empty("source_id".into(), &DataType::String).into(),
                Series::new_empty("schema_version".into(), &DataType::String).into(),
                Series::new_empty("row_count".into(), &DataType::Int64).into(),
                Series::new_empty("copper_sha256_list".into(), &DataType::String).into(),
            ])
            .map_err(|e| IngestError::Other(format!("empty df: {e}")))?
        } else {
            df![
                "entity_name" => entity_name,
                "source_id" => source_id,
                "schema_version" => schema_version,
                "row_count" => row_count,
                "copper_sha256_list" => copper_sha256_list,
            ]
            .map_err(|e| IngestError::Other(format!("df: {e}")))?
        };

        let file = std::fs::File::create(&path)?;
        ParquetWriter::new(file)
            .finish(&mut df)
            .map_err(|e| IngestError::Other(format!("parquet: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|e| IngestError::Other(format!("spawn_blocking parquet: {e}")))?
}

/// Écrit le `MANIFEST.sha256` au format `sha256sum` standard.
async fn write_manifest_sha256(
    manifest_path: &Path,
    files: &[&Path],
) -> IngestResult<()> {
    let mut lines = Vec::new();
    for p in files {
        let digest = hash::sha256_file(p).await?;
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        // Format standard sha256sum : "<hash><deux espaces><nom>"
        lines.push(format!("{digest}  {name}"));
    }
    let mut content = lines.join("\n");
    content.push('\n');
    tokio::fs::write(manifest_path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lineage::CopperRef, SilverEntity};
    use chrono::Utc;

    fn sample_silver(source: &str, entity: &str, hash_byte: char) -> SilverEntity {
        SilverEntity {
            name: entity.into(),
            path: PathBuf::from(format!("silver/{source}/{entity}.parquet")),
            schema_version: "v1".into(),
            copper_refs: vec![CopperRef {
                source_id: source.into(),
                manifest_path: PathBuf::from("manifest.json"),
                file_name: format!("{entity}.parquet"),
                file_sha256: std::iter::repeat_n(hash_byte, 64).collect(),
            }],
            row_count: 100,
        }
    }

    fn sample_meta(id: &str, tier: u8) -> SourceMeta {
        SourceMeta {
            id: id.into(),
            name: format!("Test source {id}"),
            url: "https://example.test".into(),
            license: "Etalab 2.0".into(),
            update_frequency: "annuelle".into(),
            tier,
        }
    }

    #[tokio::test]
    async fn assembles_gold_with_two_sources() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };

        let silver: Vec<StepResult<Vec<SilverEntity>>> = vec![
            StepResult::ok("comparia", vec![sample_silver("comparia", "comparia_conversations", 'a')]),
            StepResult::ok("rte-iris", vec![sample_silver("rte-iris", "rte_iris_consommation", 'b')]),
        ];
        let metas = vec![sample_meta("comparia", 1), sample_meta("rte-iris", 1)];

        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("referentiel.sqlite");
        lineage.add_artifact("analytics.parquet");

        let artifacts = assemble_gold(&ctx, &silver, &metas, &lineage)
            .await
            .expect("assemble_gold ok");

        // Les 4 fichiers existent
        assert!(artifacts.referentiel_sqlite.exists());
        assert!(artifacts.analytics_parquet.exists());
        assert!(artifacts.datasheet_jsonld.exists());
        assert!(artifacts.manifest_sha256.exists());

        // Le manifest contient 3 lignes (une par artefact, sauf lui-même)
        let manifest_content = tokio::fs::read_to_string(&artifacts.manifest_sha256)
            .await
            .unwrap();
        let lines: Vec<&str> = manifest_content.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            // Format "<hash 64 hex>  <name>"
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].len(), 64);
        }

        // Datasheet est un JSON-LD valide avec @context
        let ds_bytes = tokio::fs::read(&artifacts.datasheet_jsonld).await.unwrap();
        let ds: serde_json::Value = serde_json::from_slice(&ds_bytes).unwrap();
        assert!(ds.get("@context").is_some());
    }

    #[tokio::test]
    async fn referentiel_sqlite_contains_expected_tables_and_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };

        let silver = vec![StepResult::ok(
            "comparia",
            vec![sample_silver("comparia", "comparia_votes", 'c')],
        )];
        let metas = vec![sample_meta("comparia", 1)];
        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("referentiel.sqlite");

        let artifacts = assemble_gold(&ctx, &silver, &metas, &lineage).await.unwrap();

        // Re-ouvrir la SQLite et vérifier les contenus.
        let sqlite_path = artifacts.referentiel_sqlite.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&sqlite_path).unwrap();

            // sources
            let nb_sources: i64 = conn
                .query_row("SELECT COUNT(*) FROM sources", [], |r| r.get(0))
                .unwrap();
            assert_eq!(nb_sources, 1);

            // silver_entities
            let nb_entities: i64 = conn
                .query_row("SELECT COUNT(*) FROM silver_entities", [], |r| r.get(0))
                .unwrap();
            assert_eq!(nb_entities, 1);

            // lineage
            let nb_lineage: i64 = conn
                .query_row("SELECT COUNT(*) FROM lineage", [], |r| r.get(0))
                .unwrap();
            assert_eq!(nb_lineage, 1);

            // Spécifique
            let entity: String = conn
                .query_row(
                    "SELECT entity_name FROM silver_entities WHERE source_id = 'comparia'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(entity, "comparia_votes");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn analytics_parquet_readable() {
        use polars::prelude::*;

        let tmp = tempfile::tempdir().unwrap();
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };

        let silver = vec![
            StepResult::ok("s1", vec![sample_silver("s1", "e1", 'a')]),
            StepResult::ok("s2", vec![sample_silver("s2", "e2", 'b')]),
        ];
        let metas = vec![sample_meta("s1", 1), sample_meta("s2", 2)];
        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("analytics.parquet");

        let artifacts = assemble_gold(&ctx, &silver, &metas, &lineage).await.unwrap();

        let path = artifacts.analytics_parquet.clone();
        tokio::task::spawn_blocking(move || {
            let df = LazyFrame::scan_parquet(&path, ScanArgsParquet::default())
                .unwrap()
                .collect()
                .unwrap();
            assert_eq!(df.height(), 2);
            let cols: Vec<&str> = df.get_column_names().iter().map(|c| c.as_str()).collect();
            for needed in [
                "entity_name",
                "source_id",
                "schema_version",
                "row_count",
                "copper_sha256_list",
            ] {
                assert!(cols.contains(&needed), "colonne manquante : {needed}");
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn empty_silver_still_writes_valid_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };

        let silver: Vec<StepResult<Vec<SilverEntity>>> = Vec::new();
        let metas: Vec<SourceMeta> = Vec::new();
        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("referentiel.sqlite");

        let artifacts = assemble_gold(&ctx, &silver, &metas, &lineage).await.unwrap();
        assert!(artifacts.referentiel_sqlite.exists());
        assert!(artifacts.analytics_parquet.exists());
        assert!(artifacts.datasheet_jsonld.exists());
        assert!(artifacts.manifest_sha256.exists());
    }
}

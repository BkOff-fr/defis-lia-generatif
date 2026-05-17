//! Assemblage de la couche Gold du pipeline médaillon (ADR-0009 + C26.3).
//!
//! Produit 4 (ou 5 si GPG actif) artefacts dans `data/gold/` :
//! - `referentiel.sqlite` — base SQLite WAL avec :
//!   * `sources` : producteurs (id, name, url, license, tier, …).
//!   * `silver_entities` : datasets typés produits par la promotion Silver.
//!   * `lineage` : lien Silver entity ↔ hash Copper.
//!   * `model_overview` : un modèle = une ligne (extraction distincte
//!     depuis `comparia_conversations`) + index FTS5 pour la recherche M9.
//!   * `scenario_inputs` : table dénormalisée prête pour le simulateur M13.
//!   * `time_series_mix` : mix horaire RTE par région NUTS-2 (placeholder
//!     v1, peuplé en v2 quand RTE eco2mix sera ingéré).
//!   * `comparison_matrix` : modèles × méthodologies (vide à l'init,
//!     remplie au runtime par l'app).
//!   * `datacenter_iris_link` : datacenter européen → maille IRIS la plus
//!     proche (haversine sur centroïdes IRIS depuis Copper RTE).
//! - `analytics.parquet` — catalogue Parquet des entités Silver
//!   (DuckDB-friendly).
//! - `datasheet.jsonld` — Datasheet for Datasets (Gebru et al. 2018) +
//!   schema.org/Dataset + DCAT 3 + PROV-O. Validée à l'écriture contre
//!   `schemas/gold/datasheet-v1.json`.
//! - `MANIFEST.sha256` — hashes SHA-256 des 3 artefacts (format
//!   `sha256sum`). Si la variable d'environnement `SOBRIA_GPG_KEY_ID` est
//!   présente, on génère aussi `MANIFEST.sha256.asc` via `gpg --sign`.
//!
//! Voir aussi `briefs/chantiers/C26-pipeline-medaillon-activation.md`.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, Transaction};

use crate::{
    context::Context,
    datasheet::{build_gebru_datasheet, validate_datasheet, ArtifactMeta},
    error::{IngestError, IngestResult},
    hash,
    iris_link::{build_links_from_snapshot, DatacenterIrisLink},
    layer::SourceMeta,
    lineage::GoldLineage,
    registry::StepResult,
};

/// Chemins des artefacts Gold produits.
#[derive(Debug, Clone)]
pub struct GoldArtifacts {
    /// Base SQLite (référentiel transactionnel).
    pub referentiel_sqlite: PathBuf,
    /// Parquet catalogue (lecture DuckDB).
    pub analytics_parquet: PathBuf,
    /// Datasheet PROV-O / schema.org / Gebru.
    pub datasheet_jsonld: PathBuf,
    /// Manifest SHA-256 des artefacts.
    pub manifest_sha256: PathBuf,
    /// Signature GPG du manifest, si `SOBRIA_GPG_KEY_ID` était défini.
    pub manifest_signature: Option<PathBuf>,
}

/// Assemble la couche Gold à partir des entités Silver et du lineage.
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

    // 0. Pré-calcul async : `datacenter_iris_link` depuis Copper RTE.
    //    Vec vide si RTE n'a pas encore été ingéré ou si le GeoJSON est
    //    absent — la table reste vide, le pipeline ne casse pas.
    let iris_links = compute_iris_links(ctx).await?;
    tracing::info!(
        n_links = iris_links.len(),
        "gold: jointure datacenter_iris_link"
    );

    // 1. SQLite — exécuté en spawn_blocking car rusqlite + polars sont sync.
    build_referentiel_sqlite(
        sqlite.clone(),
        silver_results_clone(silver_results),
        sources_meta.to_vec(),
        iris_links,
    )
    .await?;

    // 2. Parquet catalogue.
    build_analytics_parquet(parquet.clone(), silver_results_clone(silver_results)).await?;

    // 3. Datasheet Gebru — on a besoin des hashes/tailles des artefacts
    //    SQLite + Parquet pour la section `schema:distribution`.
    let artifacts_meta = collect_artifact_meta(&[
        (&sqlite, "application/x-sqlite3"),
        (&parquet, "application/vnd.apache.parquet"),
    ])
    .await?;
    let datasheet_value = build_gebru_datasheet(
        sources_meta,
        lineage,
        &artifacts_meta,
        env!("CARGO_PKG_VERSION"),
    );
    validate_datasheet(&datasheet_value)?;
    let pretty = serde_json::to_vec_pretty(&datasheet_value)?;
    tokio::fs::write(&datasheet, &pretty).await?;

    // 4. Manifest SHA-256 (inclut sqlite + parquet + datasheet).
    write_manifest_sha256(&manifest, &[&sqlite, &parquet, &datasheet]).await?;

    // 5. Signature GPG optionnelle — skip silencieusement si l'env n'est pas
    //    configuré ou si gpg n'est pas installé.
    let manifest_signature = maybe_sign_manifest(&manifest).await;

    Ok(GoldArtifacts {
        referentiel_sqlite: sqlite,
        analytics_parquet: parquet,
        datasheet_jsonld: datasheet,
        manifest_sha256: manifest,
        manifest_signature,
    })
}

/// Trouve le snapshot Copper RTE IRIS le plus récent et calcule la jointure
/// datacenter ↔ IRIS. Vec vide si snapshot absent.
async fn compute_iris_links(ctx: &Context) -> IngestResult<Vec<DatacenterIrisLink>> {
    let copper_root = ctx.copper_root("rte-iris");
    if !copper_root.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&copper_root)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("manifest.json").exists())
        .collect();
    entries.sort();
    let Some(snapshot_dir) = entries.pop() else {
        return Ok(Vec::new());
    };
    let datacenters = sobria_geoloc::all_datacenters().to_vec();
    build_links_from_snapshot(&snapshot_dir, &datacenters).await
}

/// Clone propre des `StepResult` pour les passer dans `spawn_blocking`.
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

/// Collecte (taille, hash) pour chaque artefact à inclure dans la datasheet.
async fn collect_artifact_meta(files: &[(&Path, &'static str)]) -> IngestResult<Vec<ArtifactMeta>> {
    let mut out = Vec::with_capacity(files.len());
    for (path, encoding) in files {
        let metadata = tokio::fs::metadata(path).await?;
        let sha256 = hash::sha256_file(path).await?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        out.push(ArtifactMeta {
            name,
            encoding_format: encoding,
            sha256,
            size_bytes: metadata.len(),
        });
    }
    Ok(out)
}

/// Crée le fichier `referentiel.sqlite` avec ses tables, vues matérialisées
/// et index FTS5.
#[allow(clippy::too_many_lines)] // pipeline SQL séquentiel lisible in-place ; sub-fns introduisent du bruit sans bénéfice
async fn build_referentiel_sqlite(
    path: PathBuf,
    silver: Vec<(String, Vec<crate::SilverEntity>)>,
    sources: Vec<SourceMeta>,
    iris_links: Vec<DatacenterIrisLink>,
) -> IngestResult<()> {
    tokio::task::spawn_blocking(move || -> IngestResult<()> {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let mut conn =
            Connection::open(&path).map_err(|e| IngestError::Other(format!("open sqlite: {e}")))?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| IngestError::Other(format!("wal: {e}")))?;

        // ─── Tables de base (lineage / catalogue) ────────────────────────
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

            -- Vues matérialisées C26.3 (ADR-0009 §Gold) ─────────────────
            CREATE TABLE IF NOT EXISTS model_overview (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                family TEXT NOT NULL,
                vendor TEXT NOT NULL,
                n_conversations INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS scenario_inputs (
                model_id TEXT NOT NULL,
                country_iso TEXT NOT NULL,
                pue REAL,
                if_g_per_kwh REAL,
                wue_l_per_kwh REAL,
                PRIMARY KEY (model_id, country_iso)
            );
            CREATE TABLE IF NOT EXISTS time_series_mix (
                region_iso TEXT NOT NULL,
                hour_utc TEXT NOT NULL,
                production_mw REAL NOT NULL,
                PRIMARY KEY (region_iso, hour_utc)
            );
            CREATE TABLE IF NOT EXISTS comparison_matrix (
                model_id TEXT NOT NULL,
                method TEXT NOT NULL,
                co2_g_per_request REAL,
                computed_at TEXT,
                PRIMARY KEY (model_id, method)
            );
            CREATE TABLE IF NOT EXISTS datacenter_iris_link (
                datacenter_id TEXT PRIMARY KEY,
                code_iris TEXT NOT NULL,
                distance_km REAL NOT NULL,
                iris_centroid_lat REAL NOT NULL,
                iris_centroid_lon REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_datacenter_iris_link_code
                ON datacenter_iris_link(code_iris);

            -- Index FTS5 sur model_overview pour la recherche full-text M9.
            CREATE VIRTUAL TABLE IF NOT EXISTS model_overview_fts
                USING fts5(name, family, vendor);

            -- ─── C32.4 — Vendor disclosures (migration v3) ──────────────
            -- Chiffres officiels publiés par les fabricants (Mistral × ADEME,
            -- Google Gemini, Meta Llama, etc.). Source de vérité runtime :
            -- sobria_estimator::MODEL_REGISTRY[*].vendor_disclosures.
            -- Cette table matérialise les disclosures dans le Gold pour
            -- lineage + analytics DuckDB futurs. Schéma idempotent
            -- (CREATE IF NOT EXISTS) — la migration v3 ne casse pas les
            -- Gold v2 ré-assemblés.
            CREATE TABLE IF NOT EXISTS vendor_disclosures (
                id TEXT PRIMARY KEY,
                model_id TEXT NOT NULL,
                vendor TEXT NOT NULL,
                scope TEXT NOT NULL CHECK (scope IN ('training', 'inference_per_prompt')),
                value REAL NOT NULL,
                unit TEXT NOT NULL,
                source_url TEXT NOT NULL,
                published_at TEXT NOT NULL,
                methodology_note TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_vendor_disclosures_model
                ON vendor_disclosures(model_id);
            CREATE INDEX IF NOT EXISTS idx_vendor_disclosures_vendor
                ON vendor_disclosures(vendor);
            ",
        )
        .map_err(|e| IngestError::Other(format!("schema: {e}")))?;

        let tx = conn
            .transaction()
            .map_err(|e| IngestError::Other(format!("tx: {e}")))?;

        // ─── sources ────────────────────────────────────────────────────
        for src in &sources {
            tx.execute(
                "INSERT OR REPLACE INTO sources \
                 (id, name, url, license, update_frequency, tier) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    src.id,
                    src.name,
                    src.url,
                    src.license,
                    src.update_frequency,
                    src.tier
                ],
            )
            .map_err(|e| IngestError::Other(format!("insert sources: {e}")))?;
        }

        // ─── silver_entities + lineage ─────────────────────────────────
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
                        // SQLite n'a pas d'unsigned int : cast en i64 (safe pour
                        // les volumes attendus côté Tier 1).
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

        // ─── model_overview + FTS5 (depuis comparia_conversations) ──────
        let n_models = populate_model_overview(&tx, &silver)?;
        tracing::info!(n_models, "gold: model_overview peuplé");

        // ─── datacenter_iris_link (depuis iris_links précalculés) ───────
        for link in &iris_links {
            tx.execute(
                "INSERT OR REPLACE INTO datacenter_iris_link \
                 (datacenter_id, code_iris, distance_km, iris_centroid_lat, iris_centroid_lon) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    link.datacenter_id,
                    link.code_iris,
                    link.distance_km,
                    link.iris_centroid_lat,
                    link.iris_centroid_lon
                ],
            )
            .map_err(|e| IngestError::Other(format!("insert iris link: {e}")))?;
        }

        tx.commit()
            .map_err(|e| IngestError::Other(format!("commit: {e}")))?;
        Ok(())
    })
    .await
    .map_err(|e| IngestError::Other(format!("spawn_blocking sqlite: {e}")))?
}

/// Lit le Parquet `comparia_conversations` (s'il existe), extrait les
/// modèles distincts, peuple `model_overview` et son index FTS5.
///
/// Retourne le nombre de modèles insérés. Tolérant : 0 si le parquet est
/// absent, si la colonne modèle n'existe pas, ou si la lecture échoue.
fn populate_model_overview(
    tx: &Transaction,
    silver: &[(String, Vec<crate::SilverEntity>)],
) -> IngestResult<usize> {
    use polars::prelude::*;

    let parquet_path = silver
        .iter()
        .flat_map(|(_, entities)| entities.iter())
        .find(|e| e.name == "comparia_conversations")
        .map(|e| e.path.clone());
    let Some(parquet_path) = parquet_path else {
        return Ok(0);
    };
    if !parquet_path.exists() {
        return Ok(0);
    }

    let mut lf = match LazyFrame::scan_parquet(&parquet_path, ScanArgsParquet::default()) {
        Ok(lf) => lf,
        Err(e) => {
            tracing::warn!(
                path = %parquet_path.display(),
                error = %e,
                "model_overview: scan_parquet a échoué — table laissée vide"
            );
            return Ok(0);
        },
    };
    let schema = match lf.collect_schema() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "model_overview: collect_schema a échoué");
            return Ok(0);
        },
    };
    let cols: Vec<String> = schema.iter().map(|(n, _)| n.to_string()).collect();
    // Cherche la colonne « modèle » par ordre de préférence.
    let model_col = ["model_id", "model", "model_name"]
        .iter()
        .find(|c| cols.iter().any(|x| x == *c))
        .map(|s| (*s).to_string());
    let Some(model_col) = model_col else {
        tracing::info!(
            cols = ?cols,
            "model_overview: aucune colonne `model_id`/`model`/`model_name` — table laissée vide"
        );
        return Ok(0);
    };

    let distinct = lf
        .select([col(model_col.as_str())])
        .unique(None, UniqueKeepStrategy::Any)
        .collect()
        .map_err(|e| IngestError::Other(format!("collect distinct: {e}")))?;
    let series = distinct
        .column(model_col.as_str())
        .map_err(|e| IngestError::Other(format!("get column: {e}")))?;

    let mut count = 0;
    let str_chunked = series
        .cast(&DataType::String)
        .map_err(|e| IngestError::Other(format!("cast model col → string: {e}")))?;
    let str_view = str_chunked
        .str()
        .map_err(|e| IngestError::Other(format!("str view: {e}")))?;
    for opt_val in str_view.into_iter().flatten() {
        let id = opt_val.trim();
        if id.is_empty() {
            continue;
        }
        let family = infer_family(id);
        let vendor = infer_vendor(id);
        tx.execute(
            "INSERT OR REPLACE INTO model_overview (id, name, family, vendor, n_conversations) \
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![id, id, family, vendor],
        )
        .map_err(|e| IngestError::Other(format!("insert model_overview: {e}")))?;
        tx.execute(
            "INSERT INTO model_overview_fts (name, family, vendor) VALUES (?1, ?2, ?3)",
            params![id, family, vendor],
        )
        .map_err(|e| IngestError::Other(format!("insert fts5: {e}")))?;
        count += 1;
    }
    Ok(count)
}

/// Heuristique conservative : déduit la famille d'un modèle depuis son id.
fn infer_family(model_id: &str) -> &'static str {
    let lower = model_id.to_lowercase();
    if lower.contains("gpt") || lower.contains("o1") || lower.contains("o3") {
        "gpt"
    } else if lower.contains("claude") {
        "claude"
    } else if lower.contains("gemini") || lower.contains("palm") {
        "gemini"
    } else if lower.contains("llama") {
        "llama"
    } else if lower.contains("mistral") || lower.contains("mixtral") {
        "mistral"
    } else if lower.contains("qwen") {
        "qwen"
    } else if lower.contains("deepseek") {
        "deepseek"
    } else {
        "other"
    }
}

/// Heuristique conservative : déduit le vendeur d'un modèle depuis son id.
fn infer_vendor(model_id: &str) -> &'static str {
    let lower = model_id.to_lowercase();
    if lower.contains("gpt") || lower.contains("o1") || lower.contains("o3") {
        "OpenAI"
    } else if lower.contains("claude") {
        "Anthropic"
    } else if lower.contains("gemini") || lower.contains("palm") {
        "Google"
    } else if lower.contains("llama") {
        "Meta"
    } else if lower.contains("mistral") || lower.contains("mixtral") {
        "Mistral AI"
    } else if lower.contains("qwen") {
        "Alibaba"
    } else if lower.contains("deepseek") {
        "DeepSeek"
    } else {
        "unknown"
    }
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
                let hashes: Vec<String> = e
                    .copper_refs
                    .iter()
                    .map(|r| r.file_sha256.clone())
                    .collect();
                copper_sha256_list.push(hashes.join(","));
            }
        }

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
async fn write_manifest_sha256(manifest_path: &Path, files: &[&Path]) -> IngestResult<()> {
    let mut lines = Vec::new();
    for p in files {
        let digest = hash::sha256_file(p).await?;
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        lines.push(format!("{digest}  {name}"));
    }
    let mut content = lines.join("\n");
    content.push('\n');
    tokio::fs::write(manifest_path, content).await?;
    Ok(())
}

/// Si `SOBRIA_GPG_KEY_ID` est défini, signe `manifest_path` via `gpg
/// --armor --detach-sign --local-user $KEY` et renvoie le chemin du
/// fichier `.asc` produit. Sinon (ou si gpg indisponible) renvoie `None`
/// — le pipeline reste fonctionnel sans signature.
async fn maybe_sign_manifest(manifest_path: &Path) -> Option<PathBuf> {
    let key_id = std::env::var("SOBRIA_GPG_KEY_ID").ok()?;
    let asc_path = manifest_path.with_extension("sha256.asc");
    let manifest_arg = manifest_path.to_path_buf();
    let asc_arg = asc_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::process::Command::new("gpg")
            .args([
                "--batch",
                "--yes",
                "--armor",
                "--detach-sign",
                "--local-user",
            ])
            .arg(&key_id)
            .arg("--output")
            .arg(&asc_arg)
            .arg(&manifest_arg)
            .output()
    })
    .await;
    match result {
        Ok(Ok(out)) if out.status.success() => {
            tracing::info!(asc = %asc_path.display(), "gold: MANIFEST signé GPG");
            Some(asc_path)
        },
        Ok(Ok(out)) => {
            tracing::warn!(
                stderr = %String::from_utf8_lossy(&out.stderr),
                "gold: gpg --detach-sign a échoué — signature ignorée"
            );
            None
        },
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "gold: gpg indisponible — signature ignorée");
            None
        },
        Err(e) => {
            tracing::warn!(error = %e, "gold: spawn_blocking gpg — signature ignorée");
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lineage::CopperRef, SilverEntity};

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
            StepResult::ok(
                "comparia",
                vec![sample_silver("comparia", "comparia_conversations", 'a')],
            ),
            StepResult::ok(
                "rte-iris",
                vec![sample_silver("rte-iris", "rte_iris_consommation", 'b')],
            ),
        ];
        let metas = vec![sample_meta("comparia", 1), sample_meta("rte-iris", 1)];

        let mut lineage = GoldLineage::empty();
        lineage.add_silver(crate::lineage::SilverLineage {
            entity: "comparia_conversations".into(),
            schema_version: "v1".into(),
            silver_path: silver[0].result.as_ref().unwrap()[0].path.clone(),
            copper_refs: silver[0].result.as_ref().unwrap()[0].copper_refs.clone(),
            row_count: 100,
            written_at: chrono::Utc::now(),
        });
        lineage.add_artifact("referentiel.sqlite");
        lineage.add_artifact("analytics.parquet");

        let artifacts = assemble_gold(&ctx, &silver, &metas, &lineage)
            .await
            .expect("assemble_gold ok");

        assert!(artifacts.referentiel_sqlite.exists());
        assert!(artifacts.analytics_parquet.exists());
        assert!(artifacts.datasheet_jsonld.exists());
        assert!(artifacts.manifest_sha256.exists());

        // Manifest = 3 lignes (sqlite + parquet + datasheet)
        let manifest_content = tokio::fs::read_to_string(&artifacts.manifest_sha256)
            .await
            .unwrap();
        let lines: Vec<&str> = manifest_content.lines().collect();
        assert_eq!(lines.len(), 3);

        // Datasheet = JSON-LD au format Gebru valide vs son schéma
        let ds_bytes = tokio::fs::read(&artifacts.datasheet_jsonld).await.unwrap();
        let ds: serde_json::Value = serde_json::from_slice(&ds_bytes).unwrap();
        assert!(ds.get("@context").is_some());
        for section in crate::datasheet::gebru_sections() {
            assert!(ds.get(*section).is_some(), "section {section} manquante");
        }
    }

    #[tokio::test]
    async fn referentiel_sqlite_creates_all_expected_tables() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };

        let artifacts = assemble_gold(&ctx, &[], &[], &{
            let mut l = GoldLineage::empty();
            l.add_artifact("referentiel.sqlite");
            l
        })
        .await
        .expect("assemble_gold ok");

        let sqlite = artifacts.referentiel_sqlite.clone();
        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&sqlite).unwrap();
            for table in [
                "sources",
                "silver_entities",
                "lineage",
                "model_overview",
                "scenario_inputs",
                "time_series_mix",
                "comparison_matrix",
                "datacenter_iris_link",
            ] {
                let n: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?1",
                        params![table],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert_eq!(n, 1, "table {table} attendue");
            }
            // FTS5 = virtual table, type 'table' aussi.
            let n_fts: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE name = 'model_overview_fts'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(n_fts >= 1, "FTS5 model_overview_fts attendu");
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
        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("referentiel.sqlite");

        let artifacts = assemble_gold(&ctx, &[], &[], &lineage).await.unwrap();
        assert!(artifacts.referentiel_sqlite.exists());
        assert!(artifacts.analytics_parquet.exists());
        assert!(artifacts.datasheet_jsonld.exists());
        assert!(artifacts.manifest_sha256.exists());
        assert!(artifacts.manifest_signature.is_none(), "pas de GPG en test");
    }

    #[test]
    fn infer_family_recognizes_common_models() {
        assert_eq!(infer_family("gpt-4o-mini"), "gpt");
        assert_eq!(infer_family("claude-3-5-sonnet"), "claude");
        assert_eq!(infer_family("gemini-1.5-pro"), "gemini");
        assert_eq!(infer_family("llama-3-70b"), "llama");
        assert_eq!(infer_family("Mistral-Large-2"), "mistral");
        assert_eq!(infer_family("foo-bar-9000"), "other");
    }

    #[test]
    fn infer_vendor_recognizes_common_models() {
        assert_eq!(infer_vendor("gpt-4o"), "OpenAI");
        assert_eq!(infer_vendor("claude-3-5"), "Anthropic");
        assert_eq!(infer_vendor("Llama-3"), "Meta");
        assert_eq!(infer_vendor("inconnu-9000"), "unknown");
    }
}

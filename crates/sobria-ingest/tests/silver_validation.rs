//! Tests d'intégration pour `silver_validate` (chantier C26.2).
//!
//! Trois familles de tests :
//!
//! 1. **Round-trip schémas embarqués** — vérifie que les 4 schémas JSON
//!    Schema 2020-12 versionnés sous `schemas/silver/` sont bien lisibles,
//!    parsables, et exposent les invariants minimaux (`required`, `$schema`).
//! 2. **Golden snapshots `insta`** — fige le contenu de chaque schéma pour
//!    qu'une modification accidentelle d'un fichier `schemas/silver/*.json`
//!    soit immédiatement détectée par `cargo test`.
//! 3. **Property tests `proptest`** — génère des Parquet avec/sans colonnes
//!    de lineage, vérifie que [`validate_silver`] accepte les Parquet
//!    conformes et rejette ceux à qui il manque les colonnes requises.

use std::path::Path;

use chrono::Utc;
use polars::prelude::*;
use proptest::prelude::*;
use sobria_ingest::{
    layer::SilverEntity,
    silver_validate::{embedded_schema, known_entities, validate_silver},
};

/// Construit un Parquet synthétique avec des colonnes paramétrables.
/// Si `with_lineage` est `true`, ajoute `_copper_sha256` et `_ingested_at`.
fn write_synthetic_parquet(
    path: &Path,
    n_rows: usize,
    n_extra_cols: usize,
    with_lineage: bool,
    with_code_iris: bool,
) {
    let mut series_vec: Vec<Column> = Vec::new();

    // Colonnes ComparIA-like de base
    let ids: Vec<u32> = (0..n_rows).map(|i| u32::try_from(i).unwrap_or(0)).collect();
    series_vec.push(Series::new("id".into(), ids).into());

    let models: Vec<&str> = (0..n_rows).map(|_| "gpt-4o-mini").collect();
    series_vec.push(Series::new("model".into(), models).into());

    // Colonnes supplémentaires aléatoires
    for k in 0..n_extra_cols {
        let col_name = format!("extra_{k}");
        let vals: Vec<i64> = (0..n_rows).map(|i| (i * (k + 1)) as i64).collect();
        series_vec.push(Series::new(col_name.into(), vals).into());
    }

    if with_code_iris {
        let codes: Vec<&str> = (0..n_rows).map(|_| "751010101").collect();
        series_vec.push(Series::new("code_iris".into(), codes).into());
    }

    if with_lineage {
        let hash: String = "a".repeat(64);
        let now = Utc::now().to_rfc3339();
        let hashes: Vec<String> = (0..n_rows).map(|_| hash.clone()).collect();
        let timestamps: Vec<String> = (0..n_rows).map(|_| now.clone()).collect();
        series_vec.push(Series::new("_copper_sha256".into(), hashes).into());
        series_vec.push(Series::new("_ingested_at".into(), timestamps).into());
    }

    let mut df = DataFrame::new(series_vec).expect("dataframe creation");
    let file = std::fs::File::create(path).expect("create parquet");
    ParquetWriter::new(file)
        .finish(&mut df)
        .expect("write parquet");
}

fn fake_silver_entity(name: &str, path: std::path::PathBuf) -> SilverEntity {
    use sobria_ingest::lineage::CopperRef;
    SilverEntity {
        name: name.into(),
        path,
        schema_version: "v1".into(),
        copper_refs: vec![CopperRef {
            source_id: "test".into(),
            manifest_path: std::path::PathBuf::from("manifest.json"),
            file_name: "x.parquet".into(),
            file_sha256: "a".repeat(64),
        }],
        row_count: 0,
    }
}

#[test]
fn embedded_schemas_present_and_well_formed() {
    let entities = known_entities();
    assert_eq!(
        entities.len(),
        4,
        "4 entités Silver attendues (3 ComparIA + 1 RTE)"
    );
    for entity in entities {
        let schema = embedded_schema(entity).expect("schéma embarqué présent");
        let parsed: serde_json::Value = serde_json::from_str(schema).expect("schéma JSON valide");
        assert_eq!(
            parsed["$schema"].as_str(),
            Some("https://json-schema.org/draft/2020-12/schema"),
            "$schema = JSON Schema 2020-12 attendu pour {entity}"
        );
        let required = parsed["required"]
            .as_array()
            .expect("required array présent");
        let required_names: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            required_names.contains(&"_copper_sha256"),
            "{entity} doit exiger _copper_sha256"
        );
        assert!(
            required_names.contains(&"_ingested_at"),
            "{entity} doit exiger _ingested_at"
        );
    }
}

#[test]
fn rte_schema_requires_code_iris_and_lineage() {
    let schema = embedded_schema("rte_iris_consommation").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(schema).unwrap();
    let required: Vec<&str> = parsed["required"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        required.contains(&"code_iris"),
        "RTE IRIS Silver doit exiger code_iris : {required:?}"
    );
}

// ─── Golden snapshots insta ───────────────────────────────────────────────

#[test]
fn snapshot_schema_comparia_conversations() {
    let s = embedded_schema("comparia_conversations").unwrap();
    insta::assert_snapshot!("schema_comparia_conversations", s);
}

#[test]
fn snapshot_schema_comparia_votes() {
    let s = embedded_schema("comparia_votes").unwrap();
    insta::assert_snapshot!("schema_comparia_votes", s);
}

#[test]
fn snapshot_schema_comparia_reactions() {
    let s = embedded_schema("comparia_reactions").unwrap();
    insta::assert_snapshot!("schema_comparia_reactions", s);
}

#[test]
fn snapshot_schema_rte_iris_consommation() {
    let s = embedded_schema("rte_iris_consommation").unwrap();
    insta::assert_snapshot!("schema_rte_iris_consommation", s);
}

// ─── Property tests proptest ─────────────────────────────────────────────

proptest! {
    /// Un Parquet ComparIA contenant les colonnes lineage est valide.
    #[test]
    fn comparia_parquet_with_lineage_passes_validation(
        n_rows in 1usize..=20,
        n_extra in 0usize..=4,
    ) {
        let tmp = tempfile::tempdir().expect("tmp");
        let parquet = tmp.path().join("conversations-v1.parquet");
        write_synthetic_parquet(&parquet, n_rows, n_extra, true, false);
        let entity = fake_silver_entity("comparia_conversations", parquet);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let result = rt.block_on(validate_silver(&entity));
        prop_assert!(
            result.is_ok(),
            "comparia_conversations valide doit passer : {:?}",
            result.err()
        );
    }

    /// Un Parquet ComparIA sans colonnes lineage est rejeté.
    #[test]
    fn comparia_parquet_without_lineage_rejected(
        n_rows in 1usize..=20,
        n_extra in 0usize..=4,
    ) {
        let tmp = tempfile::tempdir().expect("tmp");
        let parquet = tmp.path().join("conversations-v1.parquet");
        write_synthetic_parquet(&parquet, n_rows, n_extra, false, false);
        let entity = fake_silver_entity("comparia_conversations", parquet);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let result = rt.block_on(validate_silver(&entity));
        prop_assert!(
            result.is_err(),
            "Parquet sans _copper_sha256 doit être rejeté"
        );
        let err_msg = format!("{:?}", result.unwrap_err());
        prop_assert!(
            err_msg.contains("_copper_sha256") || err_msg.contains("requise"),
            "le message d'erreur doit identifier la colonne manquante : {err_msg}"
        );
    }

    /// Un Parquet RTE IRIS doit avoir code_iris en plus du lineage.
    #[test]
    fn rte_iris_parquet_with_code_iris_passes_validation(
        n_rows in 1usize..=20,
    ) {
        let tmp = tempfile::tempdir().expect("tmp");
        let parquet = tmp.path().join("rte_iris_consommation-v1.parquet");
        write_synthetic_parquet(&parquet, n_rows, 0, true, true);
        let entity = fake_silver_entity("rte_iris_consommation", parquet);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let result = rt.block_on(validate_silver(&entity));
        prop_assert!(
            result.is_ok(),
            "RTE IRIS avec code_iris + lineage doit passer : {:?}",
            result.err()
        );
    }

    /// Un Parquet RTE IRIS SANS code_iris est rejeté même s'il a le lineage.
    #[test]
    fn rte_iris_parquet_without_code_iris_rejected(
        n_rows in 1usize..=20,
    ) {
        let tmp = tempfile::tempdir().expect("tmp");
        let parquet = tmp.path().join("rte_iris_consommation-v1.parquet");
        // with_lineage=true, with_code_iris=false → lineage OK mais pas de code_iris
        write_synthetic_parquet(&parquet, n_rows, 0, true, false);
        let entity = fake_silver_entity("rte_iris_consommation", parquet);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let result = rt.block_on(validate_silver(&entity));
        prop_assert!(
            result.is_err(),
            "RTE IRIS sans code_iris doit être rejeté"
        );
        let err_msg = format!("{:?}", result.unwrap_err());
        prop_assert!(
            err_msg.contains("code_iris"),
            "le message d'erreur doit mentionner code_iris : {err_msg}"
        );
    }
}

#[tokio::test]
async fn unknown_entity_rejected_explicitly() {
    let tmp = tempfile::tempdir().expect("tmp");
    let parquet = tmp.path().join("anything.parquet");
    write_synthetic_parquet(&parquet, 1, 0, true, false);
    let entity = fake_silver_entity("entity_qui_nexiste_pas", parquet);
    let result = validate_silver(&entity).await;
    let err = result.expect_err("entité inconnue doit être rejetée");
    let msg = format!("{err}");
    assert!(msg.contains("aucun schéma"), "msg : {msg}");
    assert!(msg.contains("entités connues"), "msg : {msg}");
}

#[tokio::test]
async fn missing_parquet_file_rejected_clearly() {
    let entity = fake_silver_entity(
        "comparia_conversations",
        std::path::PathBuf::from("/this/path/does/not/exist.parquet"),
    );
    let result = validate_silver(&entity).await;
    let err = result.expect_err("Parquet absent doit être rejeté");
    let msg = format!("{err}");
    assert!(msg.contains("introuvable"), "msg : {msg}");
}

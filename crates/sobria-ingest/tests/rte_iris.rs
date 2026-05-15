//! Tests d'intégration pour la source RTE / NaTran / Teréga IRIS.
//!
//! Vérifie :
//! 1. `ingest_copper` télécharge le CSV ET le GeoJSON via wiremock,
//!    et écrit un manifest contenant les deux.
//! 2. `promote_silver` lit le CSV via polars, ajoute les colonnes
//!    `_copper_sha256` et `_ingested_at`, écrit le Silver Parquet.
//!    Le GeoJSON n'est PAS promu en Silver (conservé en Copper).
//! 3. `contribute_gold` retourne les tables touchées + notes.
//! 4. `RteIrisSource` s'enregistre dans `LayerRegistry`.

use std::{path::Path, sync::Arc};

use polars::prelude::*;
use sobria_ingest::{sources::RteIrisSource, Context, DataLayer, Downloader, LayerRegistry};
use wiremock::{
    matchers::{method, path as wm_path},
    Mock, MockServer, ResponseTemplate,
};

/// CSV synthétique minimal au format ODRÉ : code_iris + conso + nb_sites.
const SYNTHETIC_CSV: &str = "code_iris,conso_elec_mwh,conso_gaz_mwh,nb_sites,annee
751010101,12500.5,4200.0,3,2023
751020202,8700.0,1500.0,2,2023
920400404,33000.0,0.0,5,2023
";

/// GeoJSON synthétique minimal avec un seul Feature IRIS.
const SYNTHETIC_GEOJSON: &str = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": { "code_iris": "751010101" },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[2.34, 48.86],[2.35, 48.86],[2.35, 48.87],[2.34, 48.87],[2.34, 48.86]]]
      }
    }
  ]
}"#;

/// Vérifie qu'un Parquet contient une colonne donnée.
fn parquet_has_column(path: &Path, expected: &str) -> bool {
    let df = LazyFrame::scan_parquet(path, ScanArgsParquet::default())
        .expect("scan_parquet")
        .collect()
        .expect("collect");
    df.get_column_names().iter().any(|c| c.as_str() == expected)
}

#[tokio::test]
async fn rte_iris_full_pipeline_with_synthetic_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(wm_path("/csv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/csv; charset=utf-8")
                .set_body_bytes(SYNTHETIC_CSV.as_bytes().to_vec()),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/geojson"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/geo+json")
                .set_body_bytes(SYNTHETIC_GEOJSON.as_bytes().to_vec()),
        )
        .mount(&server)
        .await;

    let source = RteIrisSource::for_test(
        Downloader::new(),
        format!("{}/csv", server.uri()),
        format!("{}/geojson", server.uri()),
    );

    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    // 1. ingest_copper : doit produire 2 CopperRef (CSV + GeoJSON)
    let snapshot = source.ingest_copper(&ctx).await.expect("ingest_copper ok");
    assert_eq!(snapshot.source_id, "rte-iris");
    assert_eq!(snapshot.files.len(), 2, "CSV + GeoJSON dans le manifest");
    let names: Vec<&str> = snapshot
        .files
        .iter()
        .map(|f| f.file_name.as_str())
        .collect();
    assert!(names.contains(&"consommation_iris.csv"));
    assert!(names.contains(&"iris_geometries.geojson"));
    for f in &snapshot.files {
        assert_eq!(f.file_sha256.len(), 64);
    }
    assert!(snapshot.path.join("manifest.json").exists());

    // 2. promote_silver : doit produire UNE SEULE entité (CSV → Parquet),
    //    le GeoJSON reste uniquement en Copper.
    let silver = source
        .promote_silver(&snapshot, &ctx)
        .await
        .expect("promote_silver ok");
    assert_eq!(silver.len(), 1, "1 entité Silver (le GeoJSON ne l'est pas)");

    let entity = &silver[0];
    assert_eq!(entity.name, "rte_iris_consommation");
    assert_eq!(entity.schema_version, "v1");
    assert!(entity.path.exists(), "Parquet Silver écrit");
    assert_eq!(entity.copper_refs.len(), 1, "1 ref Copper (le CSV)");
    assert_eq!(entity.copper_refs[0].file_name, "consommation_iris.csv");
    assert_eq!(entity.row_count, 3, "3 lignes synthétiques");

    // Colonnes systématiques présentes + colonnes ODRÉ originales préservées
    assert!(parquet_has_column(&entity.path, "_copper_sha256"));
    assert!(parquet_has_column(&entity.path, "_ingested_at"));
    assert!(parquet_has_column(&entity.path, "code_iris"));
    assert!(parquet_has_column(&entity.path, "conso_elec_mwh"));
    assert!(parquet_has_column(&entity.path, "conso_gaz_mwh"));

    // 3. contribute_gold
    let gold = source
        .contribute_gold(&silver, &ctx)
        .await
        .expect("contribute_gold ok");
    assert_eq!(gold.source_id, "rte-iris");
    assert_eq!(
        gold.tables_touched,
        vec!["rte_iris_consommation".to_string()]
    );
    assert!(
        gold.notes
            .iter()
            .any(|n| n.to_lowercase().contains("odré") || n.to_lowercase().contains("iris")),
        "note ODRÉ / IRIS attendue"
    );
}

#[tokio::test]
async fn rte_iris_registers_in_layer_registry() {
    let mut reg = LayerRegistry::new();
    reg.register(Arc::new(RteIrisSource::new()));
    assert_eq!(reg.len(), 1);
    let s = reg.sources().next().expect("au moins une source");
    assert_eq!(s.id(), "rte-iris");
    let meta = s.meta();
    assert_eq!(meta.tier, 1);
    assert_eq!(meta.license, "Etalab 2.0");
}

#[tokio::test]
async fn rte_iris_geojson_preserved_in_copper() {
    // Vérifie que le GeoJSON est bien téléchargé et accessible dans Copper,
    // même s'il n'apparaît pas dans le Silver.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(wm_path("/csv"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(SYNTHETIC_CSV.as_bytes().to_vec()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/geojson"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(SYNTHETIC_GEOJSON.as_bytes().to_vec()),
        )
        .mount(&server)
        .await;

    let source = RteIrisSource::for_test(
        Downloader::new(),
        format!("{}/csv", server.uri()),
        format!("{}/geojson", server.uri()),
    );
    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    let snapshot = source.ingest_copper(&ctx).await.expect("copper");
    let geojson_path = snapshot.path.join("iris_geometries.geojson");
    assert!(geojson_path.exists(), "GeoJSON présent en Copper");

    let content = tokio::fs::read_to_string(&geojson_path)
        .await
        .expect("lecture geojson");
    assert!(content.contains("FeatureCollection"));
    assert!(content.contains("751010101"));
}

//! Test d'intégration : pipeline complet ComparIA + RTE IRIS → Gold final.
//!
//! Vérifie qu'après `run_full_pipeline`, le rapport contient bien les
//! `gold_artifacts` (4 fichiers Gold sur disque), que `referentiel.sqlite`
//! liste les 2 sources et leurs entités, et que `MANIFEST.sha256` est cohérent.

use std::sync::Arc;

use polars::prelude::*;
use rusqlite::Connection;
use sobria_ingest::{
    sources::{ComparIASource, RteIrisSource},
    Context, Downloader, LayerRegistry,
};
use wiremock::{
    matchers::{method, path as wm_path},
    Mock, MockServer, ResponseTemplate,
};

/// Crée 3 Parquet ComparIA synthétiques et retourne leurs bytes.
fn make_comparia_parquets() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    fn one() -> Vec<u8> {
        let mut df = df![
            "id" => [1u32, 2],
            "model" => ["gpt-4o-mini", "claude-3-5"],
        ]
        .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        ParquetWriter::new(std::io::Cursor::new(&mut buf))
            .finish(&mut df)
            .unwrap();
        buf
    }
    (one(), one(), one())
}

const CSV_RTE: &str = "code_iris,conso_elec_mwh,conso_gaz_mwh,nb_sites,annee
751010101,12500.5,4200.0,3,2023
920400404,33000.0,0.0,5,2023
";

const GEOJSON_RTE: &str = r#"{"type":"FeatureCollection","features":[]}"#;

#[tokio::test]
async fn full_pipeline_assembles_gold_for_two_sources() {
    // === Setup wiremock pour ComparIA (3 fichiers) ===
    let server = MockServer::start().await;
    let (conv, vot, react) = make_comparia_parquets();
    Mock::given(method("GET"))
        .and(wm_path("/comparia/conversations"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(conv))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/comparia/votes"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vot))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/comparia/reactions"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(react))
        .mount(&server)
        .await;
    // === Setup wiremock pour RTE IRIS (CSV + GeoJSON) ===
    Mock::given(method("GET"))
        .and(wm_path("/rte/csv"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(CSV_RTE.as_bytes().to_vec()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/rte/geojson"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(GEOJSON_RTE.as_bytes().to_vec()))
        .mount(&server)
        .await;

    let comparia_urls = vec![
        (
            "conversations.parquet".to_string(),
            format!("{}/comparia/conversations", server.uri()),
            "comparia_conversations".to_string(),
        ),
        (
            "votes.parquet".to_string(),
            format!("{}/comparia/votes", server.uri()),
            "comparia_votes".to_string(),
        ),
        (
            "reactions.parquet".to_string(),
            format!("{}/comparia/reactions", server.uri()),
            "comparia_reactions".to_string(),
        ),
    ];

    // === Registry ===
    let mut reg = LayerRegistry::new();
    reg.register(Arc::new(ComparIASource::for_test(
        Downloader::new(),
        comparia_urls,
    )));
    reg.register(Arc::new(RteIrisSource::for_test(
        Downloader::new(),
        format!("{}/rte/csv", server.uri()),
        format!("{}/rte/geojson", server.uri()),
    )));

    // === Context tempdir ===
    let tmp = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: tmp.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    // === Pipeline complet ===
    let report = reg.run_full_pipeline(&ctx).await.expect("pipeline ok");
    assert_eq!(report.fully_successful_count(), 2, "2 sources OK");
    assert!(report.failed_sources().is_empty());

    // === Gold artefacts présents ===
    let arts = report
        .gold_artifacts
        .as_ref()
        .expect("gold artifacts présents");
    assert!(arts.referentiel_sqlite.exists());
    assert!(arts.analytics_parquet.exists());
    assert!(arts.datasheet_jsonld.exists());
    assert!(arts.manifest_sha256.exists());

    // === Vérifier referentiel.sqlite ===
    let sqlite_path = arts.referentiel_sqlite.clone();
    tokio::task::spawn_blocking(move || {
        let conn = Connection::open(&sqlite_path).unwrap();
        let nb_sources: i64 = conn
            .query_row("SELECT COUNT(*) FROM sources", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            nb_sources, 2,
            "2 sources enregistrées (comparia + rte-iris)"
        );

        // ComparIA produit 3 entités Silver, RTE IRIS en produit 1 → 4 au total
        let nb_entities: i64 = conn
            .query_row("SELECT COUNT(*) FROM silver_entities", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nb_entities, 4, "3 ComparIA + 1 RTE IRIS");

        // Lineage : un hash Copper par fichier copper (3 + 1 CSV = 4
        // ; le GeoJSON RTE n'a pas d'entité Silver donc pas de ligne lineage)
        let nb_lineage: i64 = conn
            .query_row("SELECT COUNT(*) FROM lineage", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nb_lineage, 4, "4 liens Silver→Copper");

        // Cohérence des FK : tous les source_id de silver_entities existent dans sources
        let orphans: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM silver_entities se \
                 LEFT JOIN sources s ON s.id = se.source_id \
                 WHERE s.id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orphans, 0, "pas de silver orphelin");
    })
    .await
    .unwrap();

    // === Vérifier analytics.parquet ===
    let parquet_path = arts.analytics_parquet.clone();
    tokio::task::spawn_blocking(move || {
        let df = LazyFrame::scan_parquet(&parquet_path, ScanArgsParquet::default())
            .unwrap()
            .collect()
            .unwrap();
        assert_eq!(df.height(), 4, "4 entités Silver dans le catalogue");
        let cols: Vec<&str> = df.get_column_names().iter().map(|c| c.as_str()).collect();
        for c in [
            "entity_name",
            "source_id",
            "schema_version",
            "row_count",
            "copper_sha256_list",
        ] {
            assert!(cols.contains(&c), "colonne manquante : {c}");
        }
    })
    .await
    .unwrap();

    // === Vérifier datasheet.jsonld ===
    let ds: serde_json::Value =
        serde_json::from_slice(&tokio::fs::read(&arts.datasheet_jsonld).await.unwrap()).unwrap();
    assert!(ds.get("@context").is_some());
    assert!(ds["schema:distribution"].is_array());

    // === Vérifier MANIFEST.sha256 ===
    let manifest_content = tokio::fs::read_to_string(&arts.manifest_sha256)
        .await
        .unwrap();
    let lines: Vec<&str> = manifest_content.lines().collect();
    assert_eq!(lines.len(), 3, "manifest = 3 artefacts hashés");
    for line in &lines {
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 64, "SHA-256 hex 64 chars");
    }
}

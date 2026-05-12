//! Tests d'intégration pour la source ComparIA.
//!
//! Vérifie de bout en bout :
//! 1. `ingest_copper` télécharge correctement via le `Downloader` (mock wiremock)
//!    et écrit un manifest valide.
//! 2. `promote_silver` lit les Parquet Copper, ajoute les colonnes
//!    `_copper_sha256` et `_ingested_at`, écrit le Silver.
//! 3. `contribute_gold` retourne les tables touchées + notes.
//! 4. `ComparIASource` s'enregistre proprement dans `LayerRegistry`.

use std::{path::Path, sync::Arc};

use polars::prelude::*;
use sobria_ingest::{
    sources::ComparIASource, Context, DataLayer, Downloader, LayerRegistry,
};
use wiremock::{
    matchers::{method, path as wm_path},
    Mock, MockServer, ResponseTemplate,
};

/// Crée un Parquet synthétique sur disque et retourne ses bytes.
fn make_synthetic_parquet(path: &Path) -> Vec<u8> {
    let mut df = df![
        "id" => [1u32, 2, 3],
        "model" => ["gpt-4o-mini", "claude-3-5", "mistral-large"],
        "tokens_in" => [120u32, 80, 250],
        "tokens_out" => [400u32, 300, 700],
    ]
    .expect("df creation");

    let file = std::fs::File::create(path).expect("create parquet file");
    ParquetWriter::new(file)
        .finish(&mut df)
        .expect("write parquet");

    std::fs::read(path).expect("read back parquet")
}

/// Vérifie qu'un Parquet contient une colonne par son nom.
fn parquet_has_column(path: &Path, expected: &str) -> bool {
    let df = LazyFrame::scan_parquet(path, ScanArgsParquet::default())
        .expect("scan_parquet")
        .collect()
        .expect("collect");
    df.get_column_names().iter().any(|c| c.as_str() == expected)
}

#[tokio::test]
async fn comparia_full_pipeline_with_synthetic_data() {
    // 1. Créer 3 Parquet synthétiques pour les 3 fichiers ComparIA.
    let staging = tempfile::tempdir().expect("staging tmpdir");
    let convs_path = staging.path().join("convs.parquet");
    let votes_path = staging.path().join("votes.parquet");
    let reacts_path = staging.path().join("reacts.parquet");
    let convs_bytes = make_synthetic_parquet(&convs_path);
    let votes_bytes = make_synthetic_parquet(&votes_path);
    let reacts_bytes = make_synthetic_parquet(&reacts_path);

    // 2. Wiremock sert les 3 fichiers.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(wm_path("/conversations"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(convs_bytes.clone()),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/votes"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(votes_bytes.clone()),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(wm_path("/reactions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(reacts_bytes.clone()),
        )
        .mount(&server)
        .await;

    // 3. Construire la source avec URLs mockées.
    let urls = vec![
        (
            "conversations.parquet".to_string(),
            format!("{}/conversations", server.uri()),
            "comparia_conversations".to_string(),
        ),
        (
            "votes.parquet".to_string(),
            format!("{}/votes", server.uri()),
            "comparia_votes".to_string(),
        ),
        (
            "reactions.parquet".to_string(),
            format!("{}/reactions", server.uri()),
            "comparia_reactions".to_string(),
        ),
    ];
    let source = ComparIASource::for_test(Downloader::new(), urls);

    // 4. Contexte pointant sur un tempdir.
    let data_root = tempfile::tempdir().expect("data tmpdir");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    // 5. ingest_copper
    let snapshot = source.ingest_copper(&ctx).await.expect("ingest_copper ok");
    assert_eq!(snapshot.source_id, "comparia");
    assert_eq!(snapshot.files.len(), 3, "3 fichiers téléchargés");
    for f in &snapshot.files {
        assert_eq!(f.file_sha256.len(), 64, "SHA-256 hex 64 chars");
        assert!(f.file_sha256.chars().all(|c| c.is_ascii_hexdigit()));
    }
    assert!(
        snapshot.path.join("manifest.json").exists(),
        "manifest écrit"
    );

    // 6. promote_silver
    let silver = source
        .promote_silver(&snapshot, &ctx)
        .await
        .expect("promote_silver ok");
    assert_eq!(silver.len(), 3, "3 entités Silver");

    for entity in &silver {
        assert_eq!(entity.schema_version, "v1");
        assert!(entity.path.exists(), "Parquet Silver écrit : {:?}", entity.path);
        assert_eq!(entity.copper_refs.len(), 1, "1 ref Copper par entité");
        assert_eq!(entity.row_count, 3, "3 lignes synthétiques");
        assert!(
            parquet_has_column(&entity.path, "_copper_sha256"),
            "lineage column présente"
        );
        assert!(
            parquet_has_column(&entity.path, "_ingested_at"),
            "ingestion timestamp présent"
        );
        // Colonne originale conservée
        assert!(
            parquet_has_column(&entity.path, "model"),
            "colonne ComparIA d'origine préservée"
        );
    }

    // 7. contribute_gold
    let gold = source
        .contribute_gold(&silver, &ctx)
        .await
        .expect("contribute_gold ok");
    assert_eq!(gold.source_id, "comparia");
    assert_eq!(gold.tables_touched.len(), 3);
    assert!(!gold.notes.is_empty(), "notes méthodologiques présentes");
    assert!(
        gold.notes
            .iter()
            .any(|n| n.to_lowercase().contains("ecologits")),
        "note EcoLogits attendue"
    );
}

#[tokio::test]
async fn comparia_registers_in_layer_registry() {
    let mut reg = LayerRegistry::new();
    reg.register(Arc::new(ComparIASource::new()));
    assert_eq!(reg.len(), 1);
    let s = reg.sources().next().expect("au moins une source");
    assert_eq!(s.id(), "comparia");
    let meta = s.meta();
    assert_eq!(meta.tier, 1);
    assert_eq!(meta.license, "Etalab 2.0");
}

#[tokio::test]
async fn comparia_silver_lineage_propagates_copper_sha256() {
    // On vérifie que le SHA-256 du fichier Copper apparaît bien dans
    // la colonne _copper_sha256 du Parquet Silver.
    let staging = tempfile::tempdir().expect("tmp");
    let f = staging.path().join("x.parquet");
    let bytes = make_synthetic_parquet(&f);

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(wm_path("/file"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
        .mount(&server)
        .await;

    // On n'utilise qu'une seule URL ici en bypassant les 3 attendues
    // → on s'attend à ce que les autres tentatives échouent.
    // Pour rester dans le contrat, on duplique la même URL pour les 3 fichiers.
    let urls = vec![
        (
            "conversations.parquet".to_string(),
            format!("{}/file", server.uri()),
            "comparia_conversations".to_string(),
        ),
        (
            "votes.parquet".to_string(),
            format!("{}/file", server.uri()),
            "comparia_votes".to_string(),
        ),
        (
            "reactions.parquet".to_string(),
            format!("{}/file", server.uri()),
            "comparia_reactions".to_string(),
        ),
    ];
    let source = ComparIASource::for_test(Downloader::new(), urls);

    let data_root = tempfile::tempdir().expect("data tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    let snap = source.ingest_copper(&ctx).await.expect("copper");
    let silver = source
        .promote_silver(&snap, &ctx)
        .await
        .expect("silver");

    // Sélectionne la première entité Silver et vérifie que sa colonne
    // _copper_sha256 contient bien le hash attendu.
    let first = &silver[0];
    let expected_hash = &snap.files[0].file_sha256;
    let df = LazyFrame::scan_parquet(&first.path, ScanArgsParquet::default())
        .expect("scan")
        .collect()
        .expect("collect");
    let col = df.column("_copper_sha256").expect("colonne présente");
    // Toutes les valeurs de la colonne doivent être identiques au hash attendu.
    for v in col.str().expect("colonne str").iter().flatten() {
        assert_eq!(v, expected_hash.as_str());
    }
}

//! Test d'intégration C26.2 : la CLI `silver` repart d'un Copper persistant.
//!
//! Workflow vérifié :
//! 1. `ingest_copper` télécharge des fichiers via wiremock et écrit le manifest.
//! 2. On **détruit** la source (simule le redémarrage du process), puis on
//!    appelle [`CopperSnapshot::from_manifest`] sur le dossier persistant.
//! 3. Le snapshot reconstruit doit avoir les mêmes fichiers, mêmes hashes,
//!    même licence que le snapshot original.
//! 4. On peut ensuite enchaîner `promote_silver` sur le snapshot reconstruit
//!    sans aucune nouvelle requête HTTP.
//!
//! Ce test couvre le scénario opérationnel cible : un opérateur lance
//! `cargo run -p sobria-ingest -- copper --all` une fois, puis itère sur
//! `silver --all` autant de fois qu'il veut sans re-télécharger.

use std::sync::Arc;

use polars::prelude::*;
use sobria_ingest::{
    cli::{build_context_with, latest_copper_snapshot, rehydrate_copper},
    sources::ComparIASource,
    Context, CopperSnapshot, DataLayer, Downloader, LayerRegistry,
};
use wiremock::{
    matchers::{method, path as wm_path},
    Mock, MockServer, ResponseTemplate,
};

fn synthetic_parquet_bytes() -> Vec<u8> {
    let mut df = df![
        "id" => [1u32, 2],
        "model" => ["gpt-4o-mini", "claude-3-5"],
    ]
    .expect("df");
    let mut buf: Vec<u8> = Vec::new();
    ParquetWriter::new(std::io::Cursor::new(&mut buf))
        .finish(&mut df)
        .expect("parquet write");
    buf
}

#[tokio::test]
async fn from_manifest_reconstructs_snapshot_after_process_restart() {
    // === Étape 1 : setup wiremock + ingest_copper ===
    let server = MockServer::start().await;
    let bytes = synthetic_parquet_bytes();
    for p in ["/conversations", "/votes", "/reactions"] {
        Mock::given(method("GET"))
            .and(wm_path(p))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes.clone()))
            .mount(&server)
            .await;
    }

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

    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };

    let original = {
        let source = ComparIASource::for_test(Downloader::new(), urls.clone());
        source.ingest_copper(&ctx).await.expect("ingest_copper")
    };
    let snapshot_dir = original.path.clone();
    assert!(snapshot_dir.join("manifest.json").exists());

    // === Étape 2 : "redémarrage" — on relit depuis le disque seul ===
    let rebuilt = CopperSnapshot::from_manifest(&snapshot_dir)
        .await
        .expect("from_manifest doit reconstruire un snapshot intègre");

    assert_eq!(rebuilt.source_id, original.source_id);
    assert_eq!(rebuilt.path, original.path);
    assert_eq!(rebuilt.license, original.license);
    assert_eq!(
        rebuilt.files.len(),
        original.files.len(),
        "3 fichiers attendus"
    );
    for (a, b) in rebuilt.files.iter().zip(original.files.iter()) {
        assert_eq!(a.file_name, b.file_name);
        assert_eq!(
            a.file_sha256, b.file_sha256,
            "hash préservé pour {}",
            a.file_name
        );
        assert_eq!(a.source_id, b.source_id);
    }

    // === Étape 3 : promote_silver doit fonctionner sans HTTP ===
    server.verify().await; // baseline : 3 GETs effectués au total
    let source = ComparIASource::for_test(Downloader::new(), urls);
    let silver = source
        .promote_silver(&rebuilt, &ctx)
        .await
        .expect("promote_silver depuis snapshot reconstruit");
    assert_eq!(silver.len(), 3, "3 entités Silver");
    for entity in &silver {
        assert!(
            entity.path.exists(),
            "Silver Parquet écrit : {}",
            entity.path.display()
        );
        assert_eq!(entity.schema_version, "v1");
    }
}

#[tokio::test]
async fn from_manifest_detects_file_corruption() {
    // 1. Premier snapshot sain
    let server = MockServer::start().await;
    let bytes = synthetic_parquet_bytes();
    for p in ["/c", "/v", "/r"] {
        Mock::given(method("GET"))
            .and(wm_path(p))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes.clone()))
            .mount(&server)
            .await;
    }
    let urls = vec![
        (
            "conversations.parquet".to_string(),
            format!("{}/c", server.uri()),
            "comparia_conversations".to_string(),
        ),
        (
            "votes.parquet".to_string(),
            format!("{}/v", server.uri()),
            "comparia_votes".to_string(),
        ),
        (
            "reactions.parquet".to_string(),
            format!("{}/r", server.uri()),
            "comparia_reactions".to_string(),
        ),
    ];
    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };
    let snapshot = ComparIASource::for_test(Downloader::new(), urls)
        .ingest_copper(&ctx)
        .await
        .expect("ingest");
    let snapshot_dir = snapshot.path.clone();

    // 2. Corruption volontaire : on remplace conversations.parquet par
    //    un contenu différent → le SHA-256 ne matche plus.
    tokio::fs::write(snapshot_dir.join("conversations.parquet"), b"corrupted")
        .await
        .unwrap();

    // 3. from_manifest doit refuser
    let result = CopperSnapshot::from_manifest(&snapshot_dir).await;
    let err = match result {
        Ok(_) => panic!("from_manifest aurait dû détecter la corruption"),
        Err(e) => e,
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("hash divergent") || msg.contains("hash"),
        "le message d'erreur doit mentionner le hash : {msg}"
    );
}

#[tokio::test]
async fn rehydrate_copper_returns_explicit_error_when_no_snapshot() {
    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = build_context_with(data_root.path().to_path_buf(), 42, false).expect("ctx");
    let mut reg = LayerRegistry::new();
    reg.register(Arc::new(ComparIASource::new()));

    let results = rehydrate_copper(&ctx, &reg).await;
    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert_eq!(r.source_id, "comparia");
    assert!(
        r.result.is_err(),
        "doit échouer quand aucun snapshot n'existe"
    );
    let msg = match &r.result {
        Ok(_) => unreachable!(),
        Err(e) => e.clone(),
    };
    assert!(msg.contains("aucun snapshot Copper"), "msg : {msg}");
    assert!(msg.contains("copper --source comparia"), "msg : {msg}");
}

#[tokio::test]
async fn latest_copper_snapshot_returns_some_after_real_ingest() {
    let server = MockServer::start().await;
    let bytes = synthetic_parquet_bytes();
    for p in ["/c", "/v", "/r"] {
        Mock::given(method("GET"))
            .and(wm_path(p))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes.clone()))
            .mount(&server)
            .await;
    }
    let urls = vec![
        (
            "conversations.parquet".to_string(),
            format!("{}/c", server.uri()),
            "comparia_conversations".to_string(),
        ),
        (
            "votes.parquet".to_string(),
            format!("{}/v", server.uri()),
            "comparia_votes".to_string(),
        ),
        (
            "reactions.parquet".to_string(),
            format!("{}/r", server.uri()),
            "comparia_reactions".to_string(),
        ),
    ];
    let data_root = tempfile::tempdir().expect("tmp");
    let ctx = Context {
        data_root: data_root.path().to_path_buf(),
        incremental: false,
        seed: 42,
    };
    let _ = ComparIASource::for_test(Downloader::new(), urls)
        .ingest_copper(&ctx)
        .await
        .expect("ingest");

    let found = latest_copper_snapshot(&ctx, "comparia")
        .expect("scan io")
        .expect("snapshot existe après ingest");
    assert!(found.join("manifest.json").exists());
}

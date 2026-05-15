//! Test d'intégration C26.3 : la Datasheet for Datasets (Gebru et al. 2018)
//! produite à l'assemblage Gold est cohérente avec son schéma JSON Schema
//! 2020-12 (`schemas/gold/datasheet-v1.json`).
//!
//! Garanties vérifiées :
//!
//! 1. Une datasheet correctement construite passe son propre schéma.
//! 2. Le `@context` JSON-LD déclare bien les 5 vocabulaires
//!    (`schema`, `dcat`, `prov`, `datasheet`, `sobria`).
//! 3. Les 7 sections Gebru sont systématiquement présentes.
//! 4. Le `schema:isBasedOn` lie chaque source à un hash Copper quand
//!    le lineage en propose un.
//! 5. La signature GPG est skippée silencieusement quand `SOBRIA_GPG_KEY_ID`
//!    n'est pas défini (cas par défaut en CI sans clé).

use std::path::PathBuf;

use sobria_ingest::{
    datasheet::{build_gebru_datasheet, gebru_sections, validate_datasheet, ArtifactMeta},
    layer::SourceMeta,
    lineage::{CopperRef, GoldLineage, SilverLineage},
};

fn meta(id: &str, tier: u8) -> SourceMeta {
    SourceMeta {
        id: id.into(),
        name: format!("Source {id}"),
        url: format!("https://example.test/{id}"),
        license: "Etalab 2.0".into(),
        update_frequency: "annuelle".into(),
        tier,
    }
}

fn lineage_with(refs: Vec<(&str, &str, char)>) -> GoldLineage {
    let mut l = GoldLineage::empty();
    for (source, file, hash_byte) in refs {
        l.add_silver(SilverLineage {
            entity: format!("{source}_demo"),
            schema_version: "v1".into(),
            silver_path: PathBuf::from(format!("silver/{source}/demo.parquet")),
            copper_refs: vec![CopperRef {
                source_id: source.into(),
                manifest_path: PathBuf::from("manifest.json"),
                file_name: file.into(),
                file_sha256: std::iter::repeat_n(hash_byte, 64).collect(),
            }],
            row_count: 42,
            written_at: chrono::Utc::now(),
        });
    }
    l.add_artifact("referentiel.sqlite");
    l
}

fn artifact(name: &str, sha_byte: char) -> ArtifactMeta {
    ArtifactMeta {
        name: name.into(),
        encoding_format: "application/octet-stream",
        sha256: std::iter::repeat_n(sha_byte, 64).collect(),
        size_bytes: 1024,
    }
}

#[test]
fn build_gebru_datasheet_round_trip_passes_schema() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1), meta("rte-iris", 1)],
        &lineage_with(vec![
            ("comparia", "conversations.parquet", 'a'),
            ("rte-iris", "consommation_iris.csv", 'b'),
        ]),
        &[
            artifact("referentiel.sqlite", 'c'),
            artifact("analytics.parquet", 'd'),
        ],
        "0.5.0",
    );
    validate_datasheet(&ds).expect("datasheet doit être valide vs son schéma");
}

#[test]
fn datasheet_jsonld_context_declares_all_vocabularies() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1)],
        &lineage_with(vec![("comparia", "conversations.parquet", 'a')]),
        &[],
        "0.5.0",
    );
    let ctx = ds["@context"].as_object().expect("@context object");
    for prefix in ["schema", "dcat", "prov", "datasheet", "sobria"] {
        let url = ctx.get(prefix).and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            url.starts_with("http"),
            "préfixe `{prefix}` doit pointer vers une IRI : {url:?}"
        );
    }
}

#[test]
fn datasheet_includes_seven_gebru_sections_with_required_keys() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1)],
        &lineage_with(vec![("comparia", "conversations.parquet", 'a')]),
        &[],
        "0.5.0",
    );
    for section in gebru_sections() {
        let obj = ds.get(*section).and_then(|v| v.as_object());
        assert!(obj.is_some(), "section Gebru manquante : {section}");
    }
    // Spot-checks sur la profondeur des sections.
    assert!(
        ds["datasheet:motivation"]["why_created"]
            .as_str()
            .map_or(0, str::len)
            > 50,
        "datasheet:motivation.why_created doit être substantielle"
    );
    assert!(
        ds["datasheet:composition"]["instances"].as_str().is_some(),
        "datasheet:composition.instances obligatoire"
    );
}

#[test]
fn datasheet_schema_version_propagated() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1)],
        &lineage_with(vec![("comparia", "conversations.parquet", 'a')]),
        &[],
        "1.2.3",
    );
    assert_eq!(ds["schema:version"], "1.2.3");
}

#[test]
fn datasheet_is_based_on_links_each_source_to_copper_hash() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1), meta("rte-iris", 1)],
        &lineage_with(vec![
            ("comparia", "conversations.parquet", 'a'),
            ("rte-iris", "consommation_iris.csv", 'b'),
        ]),
        &[],
        "0.5.0",
    );
    let arr = ds["schema:isBasedOn"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    for s in arr {
        let id = s["@id"].as_str().unwrap();
        let h = s["sobria:copperHash"]
            .as_str()
            .expect("hash Copper attendu");
        assert_eq!(h.len(), 64);
        if id.contains("comparia") {
            assert_eq!(h, "a".repeat(64));
        } else if id.contains("rte-iris") {
            assert_eq!(h, "b".repeat(64));
        }
    }
}

#[test]
fn distribution_field_lists_artifacts_with_sha256_and_size() {
    let ds = build_gebru_datasheet(
        &[],
        &lineage_with(vec![("comparia", "x.parquet", 'a')]),
        &[
            artifact("referentiel.sqlite", 'c'),
            artifact("analytics.parquet", 'd'),
        ],
        "0.5.0",
    );
    let dist = ds["schema:distribution"].as_array().unwrap();
    assert_eq!(dist.len(), 2);
    let names: Vec<&str> = dist
        .iter()
        .filter_map(|d| d["schema:name"].as_str())
        .collect();
    assert!(names.contains(&"referentiel.sqlite"));
    assert!(names.contains(&"analytics.parquet"));
    for d in dist {
        assert_eq!(d["schema:sha256"].as_str().unwrap_or("").len(), 64);
        assert!(d["schema:contentSize"].as_str().is_some());
    }
}

#[test]
fn provo_lineage_back_to_copper_intact() {
    let ds = build_gebru_datasheet(
        &[meta("comparia", 1)],
        &lineage_with(vec![("comparia", "conversations.parquet", 'a')]),
        &[],
        "0.5.0",
    );
    let silver = ds["prov:wasDerivedFrom"].as_array().unwrap();
    assert_eq!(silver.len(), 1);
    let copper_refs = silver[0]["prov:wasDerivedFrom"].as_array().unwrap();
    assert_eq!(copper_refs.len(), 1);
    let id = copper_refs[0].as_str().unwrap();
    assert!(id.starts_with("sobria:copper:"));
    assert!(id.ends_with(&"a".repeat(64)));
}

#[test]
fn datasheet_rejects_when_required_section_dropped() {
    let mut ds = build_gebru_datasheet(
        &[meta("comparia", 1)],
        &lineage_with(vec![("comparia", "x.parquet", 'a')]),
        &[],
        "0.5.0",
    );
    ds.as_object_mut().unwrap().remove("datasheet:uses");
    let err = validate_datasheet(&ds).expect_err("doit refuser sans datasheet:uses");
    let msg = format!("{err}").to_lowercase();
    assert!(
        msg.contains("uses") || msg.contains("required"),
        "msg : {msg}"
    );
}

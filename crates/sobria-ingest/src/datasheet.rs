//! Datasheet for Datasets (Gebru et al. 2018) — embarquée dans la couche Gold.
//!
//! Voir ADR-0009 §"Gold Layer". Le format combine :
//!
//! - **Gebru et al. 2018** (`doi:10.48550/arXiv.1803.09010`) : 7 sections
//!   (Motivation, Composition, Collection, Preprocessing, Uses, Distribution,
//!   Maintenance) qui documentent un dataset pour usage scientifique.
//! - **schema.org/Dataset** : compatibilité moteurs de recherche +
//!   data.gouv.fr.
//! - **DCAT 3** (W3C) : standard catalogues de données.
//! - **PROV-O** : traçabilité de chaque ligne Gold vers son hash Copper
//!   (déjà géré par [`crate::lineage::GoldLineage::to_jsonld`] qu'on étend
//!   ici).
//!
//! La datasheet est validée à chaque assemblage Gold contre
//! `schemas/gold/datasheet-v1.json`. Toute évolution du schéma → bump de
//! version + ADR.

use serde_json::{json, Value};

use crate::{
    error::{IngestError, IngestResult},
    layer::SourceMeta,
    lineage::GoldLineage,
};

/// Schéma JSON Schema 2020-12 embarqué pour validation.
const DATASHEET_SCHEMA: &str = include_str!("../../../schemas/gold/datasheet-v1.json");

/// Métadonnée d'un artefact Gold distribué (fichier final).
#[derive(Debug, Clone)]
pub struct ArtifactMeta {
    /// Nom du fichier (ex: `"referentiel.sqlite"`).
    pub name: String,
    /// MIME / format d'encodage (ex: `"application/x-sqlite3"`).
    pub encoding_format: &'static str,
    /// SHA-256 du fichier.
    pub sha256: String,
    /// Taille en octets.
    pub size_bytes: u64,
}

/// Construit la Datasheet for Datasets v1 pour l'assemblage Gold courant.
///
/// La sortie est un `serde_json::Value` JSON-LD validable contre
/// `schemas/gold/datasheet-v1.json` via [`validate_datasheet`].
#[must_use]
#[allow(clippy::too_many_lines)] // les 7 sections Gebru + JSON-LD context sont délibérément lisibles in-place ; éclater en sub-fns nuirait à la lecture
pub fn build_gebru_datasheet(
    sources: &[SourceMeta],
    lineage: &GoldLineage,
    artifacts: &[ArtifactMeta],
    schema_version: &str,
) -> Value {
    let context = json!({
        "schema": "https://schema.org/",
        "dcat": "http://www.w3.org/ns/dcat#",
        "prov": "http://www.w3.org/ns/prov#",
        "datasheet": "https://sobr.ia/vocab/datasheet#",
        "sobria": "https://sobr.ia/vocab#"
    });

    let sources_jsonld: Vec<Value> = sources
        .iter()
        .map(|s| {
            // Hash Copper le plus récent observé pour cette source dans le lineage,
            // s'il existe (utile pour citer la version exacte du snapshot).
            let copper_hash = lineage
                .silver_inputs
                .iter()
                .flat_map(|si| si.copper_refs.iter())
                .find(|r| r.source_id == s.id)
                .map(|r| r.file_sha256.clone());
            let mut entry = json!({
                "@type": ["dcat:Dataset", "schema:Dataset"],
                "@id": format!("sobria:source:{}", s.id),
                "schema:name": s.name,
                "schema:url": s.url,
                "schema:license": s.license,
                "sobria:tier": s.tier,
                "sobria:updateFrequency": s.update_frequency,
            });
            if let Some(h) = copper_hash {
                entry
                    .as_object_mut()
                    .expect("object")
                    .insert("sobria:copperHash".into(), Value::String(h));
            }
            entry
        })
        .collect();

    let distribution_jsonld: Vec<Value> = artifacts
        .iter()
        .map(|a| {
            json!({
                "@type": "dcat:Distribution",
                "schema:name": a.name,
                "schema:encodingFormat": a.encoding_format,
                "schema:contentSize": format!("{}", a.size_bytes),
                "schema:sha256": a.sha256,
            })
        })
        .collect();

    let copper_files: Vec<Value> = lineage
        .silver_inputs
        .iter()
        .flat_map(|s| s.copper_refs.iter())
        .map(|r| {
            json!({
                "@type": "prov:Entity",
                "@id": format!("sobria:copper:{}", r.file_sha256),
                "schema:identifier": r.file_sha256,
                "schema:name": r.file_name,
                "prov:wasAttributedTo": format!("sobria:source:{}", r.source_id),
            })
        })
        .collect();

    let silver_entities: Vec<Value> = lineage
        .silver_inputs
        .iter()
        .map(|s| {
            let copper_ids: Vec<String> = s
                .copper_refs
                .iter()
                .map(|r| format!("sobria:copper:{}", r.file_sha256))
                .collect();
            json!({
                "@type": "prov:Entity",
                "@id": format!("sobria:silver:{}", s.entity),
                "schema:name": s.entity,
                "schema:version": s.schema_version,
                "prov:wasDerivedFrom": copper_ids,
                "sobria:rowCount": s.row_count,
            })
        })
        .collect();

    let n_sources = sources.len();
    let n_silver = lineage.silver_inputs.len();
    let n_copper: usize = lineage.copper_hashes().count();

    json!({
        "@context": context,
        "@type": ["schema:Dataset", "dcat:Dataset", "prov:Entity"],
        "@id": "sobria:gold",
        "schema:name": "Sobr.ia — Référentiel Gold (pipeline médaillon)",
        "schema:description": "Référentiel transactionnel + Parquet analytique \
            assemblés à partir des sources officielles (ComparIA, RTE IRIS, …) \
            pour mesurer l'impact environnemental de l'usage des LLMs. \
            Conforme à AFNOR SPEC 2314 et à la méthodologie EcoLogits.",
        "schema:dateCreated": lineage.assembled_at.to_rfc3339(),
        "schema:version": schema_version,
        "schema:license": "MIT (code) — Etalab 2.0 (données publiques agrégées)",
        "schema:creator": {
            "@type": "schema:Organization",
            "schema:name": "Sobr.ia contributors",
            "schema:url": "https://sobr.ia"
        },
        "schema:keywords": [
            "LLM", "empreinte carbone", "énergie", "EcoLogits",
            "AFNOR SPEC 2314", "frugal AI", "data.gouv.fr"
        ],
        "schema:isBasedOn": sources_jsonld,
        "schema:distribution": distribution_jsonld,

        // ─── Sections Gebru et al. 2018 ──────────────────────────────────
        "datasheet:motivation": {
            "why_created": "Mesurer et visualiser l'impact environnemental \
                (CO₂eq, énergie, eau) de l'usage des LLMs en France et en \
                Europe à partir de sources officielles, pour le défi \
                data.gouv.fr 2026 « L'impact environnemental de l'IA \
                générative ».",
            "creators": "Sobr.ia contributors (équipe candidate au défi data.gouv.fr).",
            "funding": "Projet open source — pas de financement direct."
        },
        "datasheet:composition": {
            "instances": format!(
                "Le Gold est un agrégat de {n_silver} entité(s) Silver issues \
                de {n_sources} source(s) amont, traçables jusqu'à {n_copper} \
                fichier(s) Copper unique(s). La table `sources` recense les \
                producteurs ; `silver_entities` les datasets typés ; \
                `lineage` relie chaque entité à son hash Copper d'origine."
            ),
            "instance_count": format!(
                "{n_sources} sources, {n_silver} entités Silver, {n_copper} fichiers Copper."
            ),
            "sampling": "Pas d'échantillonnage : toutes les sources Tier 1 \
                du défi sont incluses intégralement.",
            "data_split": "N/A (pas de split train/test — usage analytique).",
            "noise_or_redundancy": "Aucun nettoyage destructif côté Silver \
                (passthrough enrichi de colonnes lineage). Les déduplications \
                inter-sources se font côté Gold avec règles documentées.",
            "self_contained": true,
            "confidential_data": false,
            "personal_data": false
        },
        "datasheet:collection_process": {
            "how_collected": "Téléchargement HTTPS depuis data.gouv.fr et \
                Open Data Réseaux Énergies (ODRÉ). Hash SHA-256 calculé à \
                la réception ; manifest immuable conservé dans Copper. \
                Voir `crates/sobria-ingest/src/sources/`.",
            "instruments": "`reqwest` + `polars` + `rusqlite` (Rust stable). \
                Chaque téléchargement passe par le trait `Downloader` qui \
                normalise les en-têtes HTTP et journalise les durées.",
            "time_frame": format!("Snapshot Gold du {}", lineage.assembled_at.to_rfc3339()),
            "ethical_review": "N/A (données publiques agrégées, licence \
                ouverte Etalab 2.0).",
            "consent": "N/A — sources ouvertes."
        },
        "datasheet:preprocessing": {
            "steps": "Pipeline médaillon ADR-0009 en 3 étapes : Copper \
                (brut immutable + manifest hashé) → Silver (Parquet validé \
                par JSON Schema 2020-12 versionné, lineage `_copper_sha256` \
                + `_ingested_at`) → Gold (jointures, vues matérialisées, \
                FTS5).",
            "raw_data_saved": true,
            "raw_data_location": "data/copper/<source>/<YYYY-MM-DD>/ — \
                conservés 30 jours pleins, mensuels 2 ans, annuels indéfiniment \
                (cf. ADR-0009 §Politique de rétention).",
            "software_used": "sobria-ingest (Rust 2021 / tokio / polars 0.46), \
                versionné via DVC."
        },
        "datasheet:uses": {
            "recommended_uses": "Calcul d'empreinte LLM (CO₂eq, énergie, eau) \
                à l'échelle France ; comparaison de modèles ; analyse \
                territoriale par maille IRIS ; reproductibilité scientifique \
                (notebook Quarto qui cite les hashes Copper).",
            "out_of_scope_uses": "Pas conçu pour décider individuellement \
                de l'usage d'un modèle (les valeurs sont des estimations \
                d'incertitude P5-P95). Ne pas utiliser pour benchmarks \
                marketing : c'est un outil de réflexion, pas une note de \
                produit.",
            "tasks_used_for": "Module M1 (Atelier estimation), M9 (fiche \
                modèle), M12 (territoire IRIS), M13 (simulateur), M16 \
                (forecaster), M17 (datasheet)."
        },
        "datasheet:distribution": {
            "distributed_to": "Public — application Tauri Sobr.ia + remote \
                DVC (snapshot tagué par version SemVer).",
            "license": "Référentiel : MIT (code) + Etalab 2.0 (données). \
                Snapshot Gold lui-même : Etalab 2.0 (compatible avec les \
                sources amont).",
            "ip_restrictions": "Aucune.",
            "export_controls": "Aucun."
        },
        "datasheet:maintenance": {
            "maintainer": "Sobr.ia contributors.",
            "contact": "https://github.com/<TBD>/sobria/issues",
            "update_policy": "Régénération automatique nocturne via \
                `dvc repro` ; release manuelle à chaque snapshot Tier 1 \
                amont. Versionnage SemVer du Gold (cf. CHANGELOG.md).",
            "retention_policy": "Voir ADR-0009 §Politique de rétention.",
            "errata_url": "https://github.com/<TBD>/sobria/issues?q=label%3Adata-correction"
        },

        // ─── PROV-O / lineage technique ──────────────────────────────────
        "prov:wasDerivedFrom": silver_entities,
        "sobria:copperReferences": copper_files,
        "sobria:lineageVersion": "1"
    })
}

/// Valide une datasheet contre le JSON Schema embarqué.
///
/// Échoue avec `IngestError::Schema` listant le premier chemin invalide
/// (utile pour comprendre rapidement quel champ manque ou est mal formé).
pub fn validate_datasheet(datasheet: &Value) -> IngestResult<()> {
    let schema: Value = serde_json::from_str(DATASHEET_SCHEMA)?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| IngestError::Other(format!("compilation du schéma datasheet : {e}")))?;
    if let Err(err) = validator.validate(datasheet) {
        return Err(IngestError::schema(format!("datasheet invalide : {err}")));
    }
    Ok(())
}

/// Liste figée des sections Gebru exigées par le schéma — utile pour
/// l'introspection / la documentation.
#[must_use]
pub fn gebru_sections() -> &'static [&'static str] {
    &[
        "datasheet:motivation",
        "datasheet:composition",
        "datasheet:collection_process",
        "datasheet:preprocessing",
        "datasheet:uses",
        "datasheet:distribution",
        "datasheet:maintenance",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineage::{CopperRef, SilverLineage};
    use std::path::PathBuf;

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

    fn sample_lineage() -> GoldLineage {
        let mut l = GoldLineage::empty();
        l.add_silver(SilverLineage {
            entity: "comparia_conversations".into(),
            schema_version: "v1".into(),
            silver_path: PathBuf::from("silver/comparia/conversations.parquet"),
            copper_refs: vec![CopperRef {
                source_id: "comparia".into(),
                manifest_path: PathBuf::from("manifest.json"),
                file_name: "conversations.parquet".into(),
                file_sha256: "a".repeat(64),
            }],
            row_count: 1000,
            written_at: chrono::Utc::now(),
        });
        l.add_artifact("referentiel.sqlite");
        l
    }

    fn sample_artifact() -> ArtifactMeta {
        ArtifactMeta {
            name: "referentiel.sqlite".into(),
            encoding_format: "application/x-sqlite3",
            sha256: "b".repeat(64),
            size_bytes: 12_345,
        }
    }

    #[test]
    fn datasheet_passes_its_own_schema() {
        let metas = vec![sample_meta("comparia", 1), sample_meta("rte-iris", 1)];
        let lineage = sample_lineage();
        let artifacts = vec![sample_artifact()];
        let ds = build_gebru_datasheet(&metas, &lineage, &artifacts, "0.5.0");
        validate_datasheet(&ds).expect("datasheet valide vs son propre schéma");
    }

    #[test]
    fn datasheet_includes_all_seven_gebru_sections() {
        let metas = vec![sample_meta("comparia", 1)];
        let lineage = sample_lineage();
        let ds = build_gebru_datasheet(&metas, &lineage, &[], "0.5.0");
        for section in gebru_sections() {
            assert!(
                ds.get(*section).is_some(),
                "section Gebru manquante : {section}"
            );
        }
    }

    #[test]
    fn datasheet_links_sources_to_copper_hashes() {
        let metas = vec![sample_meta("comparia", 1)];
        let lineage = sample_lineage();
        let ds = build_gebru_datasheet(&metas, &lineage, &[], "0.5.0");
        let sources = ds["schema:isBasedOn"].as_array().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0]["sobria:copperHash"].as_str(),
            Some(&"a".repeat(64) as &str),
            "le hash Copper de la première source doit apparaître"
        );
    }

    #[test]
    fn datasheet_distribution_includes_artifacts() {
        let metas = vec![];
        let lineage = sample_lineage();
        let ds = build_gebru_datasheet(&metas, &lineage, &[sample_artifact()], "0.5.0");
        let dist = ds["schema:distribution"].as_array().unwrap();
        assert_eq!(dist.len(), 1);
        assert_eq!(dist[0]["schema:name"], "referentiel.sqlite");
        assert_eq!(dist[0]["schema:sha256"], "b".repeat(64));
    }

    #[test]
    fn validate_datasheet_rejects_missing_section() {
        // Construit un JSON-LD volontairement amputé d'une section Gebru.
        let mut ds = build_gebru_datasheet(
            &[sample_meta("comparia", 1)],
            &sample_lineage(),
            &[],
            "0.5.0",
        );
        ds.as_object_mut().unwrap().remove("datasheet:motivation");
        let err = validate_datasheet(&ds).expect_err("doit refuser sans motivation");
        let msg = format!("{err}");
        assert!(
            msg.to_lowercase().contains("motivation") || msg.to_lowercase().contains("required"),
            "msg : {msg}"
        );
    }
}

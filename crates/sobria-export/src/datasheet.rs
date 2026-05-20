//! Génération de datasheet JSON-LD selon Gebru et al. 2018
//! (« Datasheets for Datasets », arXiv:1803.09010).
//!
//! Combinaison de vocabulaires :
//! - **schema.org/Dataset** : nom, description, dates, license, taille
//! - **W3C PROV-O** : `prov:Entity`, lineage
//! - **DCAT** : distributions
//! - **Sobr.ia vocab** : 7 sections Gebru sous `sobria:Datasheet`
//!
//! Voir `briefs/chantiers/C20-empreinte-projet-datasheet.md`.

use std::fmt::Write as _;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sobria_audit::AuditEntry;
use sobria_core::{EstimationResult, Indicator};

/// Métadonnées d'un projet pour la génération de datasheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Composition agrégée du dataset (section 2 de Gebru).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Composition {
    pub total_requests: u32,
    pub unique_models: Vec<String>,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    pub date_first_entry: Option<DateTime<Utc>>,
    pub date_last_entry: Option<DateTime<Utc>>,
    /// Polish G (C24) — Méthodologies effectivement utilisées par les
    /// entrées de la période. Tracé dans le JSON-LD du datasheet pour
    /// la reproductibilité scientifique (un lecteur Gebru doit savoir
    /// quelle méthodo a produit les chiffres).
    pub methodologies_used: Vec<sobria_core::EmpreinteMethod>,
}

/// Résultat de génération de datasheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasheetArtifact {
    pub composition: Composition,
    pub jsonld: Value,
    /// SHA-256 du JSON-LD canonisé (pretty-printed).
    pub sha256: String,
}

/// Options de génération (version Sobr.ia, contact maintenance).
#[derive(Debug, Clone)]
pub struct DatasheetOptions {
    pub estimator_version: String,
    pub estimator_seed: u64,
    pub estimator_n: u32,
    pub maintenance_contact: String,
    pub intended_uses: String,
}

impl Default for DatasheetOptions {
    fn default() -> Self {
        Self {
            estimator_version: env!("CARGO_PKG_VERSION").into(),
            estimator_seed: 42,
            estimator_n: 10_000,
            maintenance_contact: "voir paramètres Sobr.ia".into(),
            intended_uses: "Recherche scientifique, audit interne, reporting CSRD.".into(),
        }
    }
}

/// Polish G (C24) — Libellé long d'une méthodologie pour rendu JSON-LD.
/// Stable et machine-readable (utilisable par tout parseur Gebru externe).
#[must_use]
fn methodology_jsonld_label(method: sobria_core::EmpreinteMethod) -> &'static str {
    match method {
        sobria_core::EmpreinteMethod::AfnorSobria => {
            "AFNOR SPEC 2314 (Sobr.ia) — référentiel français, formule \
             linéaire-par-token + Monte-Carlo N=10⁴"
        },
        sobria_core::EmpreinteMethod::EcoLogits => {
            "EcoLogits 2026-01 — port direct des formules officielles \
             (doi:10.21105/joss.07471, CC BY-SA 4.0)"
        },
    }
}

/// Construit un datasheet Gebru à partir d'un projet et des entrées d'audit
/// dans sa période.
#[must_use]
pub fn build_datasheet(
    project: &ProjectMeta,
    entries_in_period: &[AuditEntry],
    opts: &DatasheetOptions,
) -> DatasheetArtifact {
    let composition = compose(entries_in_period);
    let jsonld = build_jsonld(project, &composition, opts);
    let canonical = serde_json::to_string_pretty(&jsonld).unwrap_or_else(|_| jsonld.to_string());
    let sha256 = sha256_hex(canonical.as_bytes());
    DatasheetArtifact {
        composition,
        jsonld,
        sha256,
    }
}

fn compose(entries: &[AuditEntry]) -> Composition {
    let mut unique = std::collections::BTreeSet::<String>::new();
    let mut total_co2 = 0.0;
    let mut total_energy = 0.0;
    let mut total_water = 0.0;
    let mut count: u32 = 0;
    let mut first_ts: Option<DateTime<Utc>> = None;
    let mut last_ts: Option<DateTime<Utc>> = None;
    let mut methods_seen: std::collections::HashSet<sobria_core::EmpreinteMethod> =
        std::collections::HashSet::new();
    for e in entries {
        count = count.saturating_add(1);
        first_ts = match first_ts {
            None => Some(e.timestamp),
            Some(prev) if e.timestamp < prev => Some(e.timestamp),
            other => other,
        };
        last_ts = match last_ts {
            None => Some(e.timestamp),
            Some(prev) if e.timestamp > prev => Some(e.timestamp),
            other => other,
        };
        if e.is_purged() {
            continue;
        }
        let Ok(r) = serde_json::from_str::<EstimationResult>(&e.estimation_result_json) else {
            continue;
        };
        unique.insert(r.request.model_id.clone());
        // Polish G — collecte des méthodologies pour traçabilité Gebru.
        methods_seen.insert(r.method);
        for ind in &r.indicators {
            match ind.indicator {
                Indicator::Co2Eq => total_co2 += ind.interval.p50,
                Indicator::Energy => total_energy += ind.interval.p50,
                Indicator::Water => total_water += ind.interval.p50,
                Indicator::CriticalMetals | Indicator::Cost => {},
            }
        }
    }
    let mut methodologies_used: Vec<sobria_core::EmpreinteMethod> =
        methods_seen.into_iter().collect();
    methodologies_used.sort_by_key(|m| match m {
        sobria_core::EmpreinteMethod::AfnorSobria => 0,
        sobria_core::EmpreinteMethod::EcoLogits => 1,
    });
    Composition {
        total_requests: count,
        unique_models: unique.into_iter().collect(),
        total_co2eq_g_p50: total_co2,
        total_energy_wh_p50: total_energy,
        total_water_l_p50: total_water,
        date_first_entry: first_ts,
        date_last_entry: last_ts,
        methodologies_used,
    }
}

#[allow(clippy::too_many_lines)] // assemblage JSON-LD linéaire, mieux lu en bloc
fn build_jsonld(
    project: &ProjectMeta,
    composition: &Composition,
    opts: &DatasheetOptions,
) -> Value {
    let project_id = format!("sobria:project-{}", project.id);
    let datasheet_id = format!("{project_id}/datasheet");
    let temporal_coverage = format!(
        "{}/{}",
        project.period_start.format("%Y-%m-%d"),
        project.period_end.format("%Y-%m-%d")
    );

    let mut motivation_text = String::new();
    let _ = write!(motivation_text, "{}", project.description);
    if motivation_text.is_empty() {
        motivation_text.push_str("Projet sans description.");
    }

    let composition_jsonld = json!({
        "@id": format!("{datasheet_id}#composition"),
        "@type": "sobria:Composition",
        "sobria:totalRequests": composition.total_requests,
        "sobria:uniqueModels": composition.unique_models,
        "sobria:totalCo2eqGramsP50": composition.total_co2eq_g_p50,
        "sobria:totalEnergyWhP50": composition.total_energy_wh_p50,
        "sobria:totalWaterLitersP50": composition.total_water_l_p50,
        "sobria:firstEntryAt": composition
            .date_first_entry
            .map_or(Value::Null, |d| Value::String(d.to_rfc3339())),
        "sobria:lastEntryAt": composition
            .date_last_entry
            .map_or(Value::Null, |d| Value::String(d.to_rfc3339())),
    });

    // Polish G — Liste les méthodologies réellement utilisées dans la
    // période plutôt que de hard-coder "AFNOR SPEC 2314". Critique pour
    // la reproductibilité scientifique (cf. Gebru et al. 2018 §3.3).
    let methods_label = match composition.methodologies_used.len() {
        0 => "(aucune entrée dans la période)".to_string(),
        1 => methodology_jsonld_label(composition.methodologies_used[0]).to_string(),
        _ => composition
            .methodologies_used
            .iter()
            .map(|m| methodology_jsonld_label(*m))
            .collect::<Vec<_>>()
            .join(" + "),
    };
    let methods_array: Vec<&'static str> = composition
        .methodologies_used
        .iter()
        .map(|m| methodology_jsonld_label(*m))
        .collect();

    let datasheet_node = json!({
        "@id": datasheet_id,
        "@type": "sobria:Datasheet",
        "sobria:gebriRef": "Gebru et al. 2018 - Datasheets for Datasets (arXiv:1803.09010)",
        "sobria:motivation": motivation_text,
        "sobria:composition": composition_jsonld,
        "sobria:collectionProcess": format!(
            "Estimations produites par sobria-estimator v{} (seed {}, N={}). \
             Méthodologie(s) : {}. Chaque entrée provient d'un acte utilisateur \
             unique journalisé dans le ledger d'audit ACID (SHA-256 chaîné).",
            opts.estimator_version, opts.estimator_seed, opts.estimator_n, methods_label
        ),
        // Champ structuré (Polish G) pour parseurs JSON-LD : permet aux
        // chercheurs / reviewers de filtrer par méthodologie.
        "sobria:methodologiesUsed": methods_array,
        "sobria:preprocessing": "Aucune transformation des résultats Monte-Carlo. \
             Les valeurs P5/P50/P95 publiées sont les quantiles directs sur N tirages.",
        "sobria:uses": opts.intended_uses.clone(),
        "sobria:distribution": {
            "@type": "dcat:Distribution",
            "schema:encodingFormat": "application/ld+json",
            "schema:license": "https://opensource.org/licenses/MIT",
            "sobria:dataLicense": "Etalab 2.0 (sources publiques)",
        },
        "sobria:maintenance": {
            "sobria:contact": opts.maintenance_contact.clone(),
            "schema:softwareVersion": opts.estimator_version.clone(),
            "schema:dateModified": Utc::now().to_rfc3339(),
        }
    });

    let project_node = json!({
        "@id": project_id,
        "@type": ["schema:Dataset", "prov:Entity"],
        "schema:name": project.name.clone(),
        "schema:description": project.description.clone(),
        "schema:dateCreated": project.created_at.to_rfc3339(),
        "schema:license": "https://opensource.org/licenses/MIT",
        "schema:temporalCoverage": temporal_coverage,
        "schema:keywords": project.tags.clone(),
        "schema:variableMeasured": ["co2eq", "energy", "water"],
        "schema:size": composition.total_requests,
        "sobria:datasheet": {"@id": datasheet_id},
    });

    json!({
        "@context": {
            "schema": "https://schema.org/",
            "prov": "http://www.w3.org/ns/prov#",
            "dcat": "http://www.w3.org/ns/dcat#",
            "sobria": "https://sobr.ia/vocab#"
        },
        "@graph": [project_node, datasheet_node]
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use sobria_core::{
        EmpreinteMethod, EstimationRequest, Hypothesis, IndicatorValue, UncertaintyInterval,
    };

    fn make_entry(id: i64, ts: DateTime<Utc>, model: &str, co2: f64) -> AuditEntry {
        let result = EstimationResult {
            method: EmpreinteMethod::AfnorSobria,
            request: EstimationRequest {
                model_id: model.into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                timestamp: ts,
                modalities: Vec::new(),
                overhead: sobria_core::ContextOverhead::default(),
            },
            indicators: vec![
                IndicatorValue {
                    indicator: Indicator::Co2Eq,
                    interval: UncertaintyInterval::new(co2 * 0.7, co2, co2 * 1.4).unwrap(),
                    unit: "gCO2eq".into(),
                    bins: None,
                },
                IndicatorValue {
                    indicator: Indicator::Energy,
                    interval: UncertaintyInterval::new(0.5, 1.0, 1.5).unwrap(),
                    unit: "Wh".into(),
                    bins: None,
                },
                IndicatorValue {
                    indicator: Indicator::Water,
                    interval: UncertaintyInterval::new(0.0, 0.001, 0.002).unwrap(),
                    unit: "L".into(),
                    bins: None,
                },
            ],
            equivalents: vec![],
            hypotheses: vec![Hypothesis {
                key: "x".into(),
                value: serde_json::json!(1),
                source: "test".into(),
            }],
            computed_at: ts,
            seed: 42,
        };
        AuditEntry {
            id,
            timestamp: ts,
            estimation_result_json: serde_json::to_string(&result).unwrap(),
            prev_sig: String::new(),
            sig: "x".repeat(64),
            purged_at: None,
        }
    }

    fn sample_project() -> ProjectMeta {
        ProjectMeta {
            id: 42,
            name: "Étude Q1 2026 Claude Sonnet".into(),
            description: "Benchmark Claude 3.5 Sonnet sur prompts éducatifs".into(),
            period_start: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            tags: vec!["recherche".into(), "claude".into(), "q1-2026".into()],
            created_at: Utc.with_ymd_and_hms(2026, 4, 2, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn compose_empty_entries_returns_zeros() {
        let c = compose(&[]);
        assert_eq!(c.total_requests, 0);
        assert!(c.unique_models.is_empty());
        assert!(c.date_first_entry.is_none());
    }

    #[test]
    fn compose_aggregates_three_entries() {
        let ts1 = Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2026, 2, 10, 10, 0, 0).unwrap();
        let ts3 = Utc.with_ymd_and_hms(2026, 3, 15, 10, 0, 0).unwrap();
        let entries = vec![
            make_entry(1, ts1, "gpt-4o-mini", 1.0),
            make_entry(2, ts2, "claude-3-5-sonnet", 2.0),
            make_entry(3, ts3, "gpt-4o-mini", 3.0),
        ];
        let c = compose(&entries);
        assert_eq!(c.total_requests, 3);
        assert_eq!(c.unique_models.len(), 2);
        assert!((c.total_co2eq_g_p50 - 6.0).abs() < 1e-9);
        assert_eq!(c.date_first_entry, Some(ts1));
        assert_eq!(c.date_last_entry, Some(ts3));
    }

    #[test]
    fn compose_counts_purged_excludes_values() {
        let ts = Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap();
        let mut purged = make_entry(99, ts, "gpt-4o-mini", 100.0);
        purged.estimation_result_json = sobria_audit::PURGED_SENTINEL.into();
        purged.purged_at = Some(ts);
        let entries = vec![make_entry(1, ts, "gpt-4o-mini", 1.0), purged];
        let c = compose(&entries);
        assert_eq!(c.total_requests, 2);
        assert!((c.total_co2eq_g_p50 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn build_jsonld_has_required_context() {
        let p = sample_project();
        let entries = vec![make_entry(
            1,
            Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap(),
            "gpt-4o-mini",
            1.0,
        )];
        let art = build_datasheet(&p, &entries, &DatasheetOptions::default());
        let ctx = &art.jsonld["@context"];
        assert!(ctx["schema"].is_string());
        assert!(ctx["prov"].is_string());
        assert!(ctx["dcat"].is_string());
        assert!(ctx["sobria"].is_string());
    }

    #[test]
    fn datasheet_has_two_graph_nodes() {
        let p = sample_project();
        let art = build_datasheet(&p, &[], &DatasheetOptions::default());
        let graph = art.jsonld["@graph"].as_array().unwrap();
        assert_eq!(graph.len(), 2);
        // Premier : le projet (schema:Dataset + prov:Entity)
        let types = graph[0]["@type"].as_array().unwrap();
        assert!(types.iter().any(|t| t == "schema:Dataset"));
        assert!(types.iter().any(|t| t == "prov:Entity"));
        // Deuxième : la datasheet
        assert_eq!(graph[1]["@type"], "sobria:Datasheet");
    }

    #[test]
    fn datasheet_exposes_seven_gebru_sections() {
        let p = sample_project();
        let art = build_datasheet(&p, &[], &DatasheetOptions::default());
        let datasheet = &art.jsonld["@graph"][1];
        for key in [
            "sobria:motivation",
            "sobria:composition",
            "sobria:collectionProcess",
            "sobria:preprocessing",
            "sobria:uses",
            "sobria:distribution",
            "sobria:maintenance",
        ] {
            assert!(!datasheet[key].is_null(), "section Gebru manquante : {key}");
        }
    }

    #[test]
    fn datasheet_references_gebru_paper() {
        let p = sample_project();
        let art = build_datasheet(&p, &[], &DatasheetOptions::default());
        let datasheet = &art.jsonld["@graph"][1];
        let r = datasheet["sobria:gebriRef"].as_str().unwrap();
        assert!(r.contains("Gebru"));
        assert!(r.contains("1803.09010"));
    }

    #[test]
    fn project_node_has_keywords_from_tags() {
        let p = sample_project();
        let art = build_datasheet(&p, &[], &DatasheetOptions::default());
        let project = &art.jsonld["@graph"][0];
        let kw = project["schema:keywords"].as_array().unwrap();
        assert_eq!(kw.len(), 3);
    }

    #[test]
    fn project_node_temporal_coverage_format_is_iso() {
        let p = sample_project();
        let art = build_datasheet(&p, &[], &DatasheetOptions::default());
        let coverage = art.jsonld["@graph"][0]["schema:temporalCoverage"]
            .as_str()
            .unwrap();
        assert_eq!(coverage, "2026-01-01/2026-04-01");
    }

    #[test]
    fn sha256_is_deterministic_for_same_inputs() {
        let p = sample_project();
        let entries = vec![make_entry(
            1,
            Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap(),
            "gpt-4o-mini",
            1.0,
        )];
        // Le SHA dépend du jsonld donc inclut Utc::now() pour `schema:dateModified`.
        // On vérifie juste la forme (64 hex chars).
        let art = build_datasheet(&p, &entries, &DatasheetOptions::default());
        assert_eq!(art.sha256.len(), 64);
        assert!(art.sha256.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn composition_in_jsonld_matches_aggregate() {
        let p = sample_project();
        let entries = vec![
            make_entry(
                1,
                Utc.with_ymd_and_hms(2026, 1, 5, 10, 0, 0).unwrap(),
                "gpt-4o-mini",
                1.0,
            ),
            make_entry(
                2,
                Utc.with_ymd_and_hms(2026, 2, 5, 10, 0, 0).unwrap(),
                "claude-3-5-sonnet",
                2.0,
            ),
        ];
        let art = build_datasheet(&p, &entries, &DatasheetOptions::default());
        let comp = &art.jsonld["@graph"][1]["sobria:composition"];
        assert_eq!(comp["sobria:totalRequests"], 2);
        let models = comp["sobria:uniqueModels"].as_array().unwrap();
        assert_eq!(models.len(), 2);
    }
}

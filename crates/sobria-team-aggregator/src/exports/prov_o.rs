//! Export PROV-O JSON-LD team-spécifique (différent du sidecar produit par
//! `sobria-export` qui est centré audit-ledger).
//!
//! Le document expose :
//!
//! - un `prov:Bundle` racine pour le rapport équipe (org + période) ;
//! - un `prov:Agent` par employé enrôlé (anonymisable via flag) ;
//! - un `prov:Activity` par estimation, avec :
//!   * `prov:wasAssociatedWith` → l'agent user
//!   * `prov:generatedAtTime`   → `ts` de l'estimation
//!   * `sobria:methodology`, `sobria:modelId`, `sobria:gco2eqGramsP50`...
//!
//! Voir <https://www.w3.org/TR/prov-o/>.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::storage::estimations::EstimationRow;

/// Construit le JSON-LD PROV-O à partir des rows et de l'option `anonymize`.
pub fn build_team_provo(
    rows: &[EstimationRow],
    organization_name: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    anonymize: bool,
) -> Value {
    let bundle_id = format!(
        "sobria-team:report-{}-to-{}",
        period_start.format("%Y%m%d"),
        period_end.format("%Y%m%d")
    );

    // Regroupe par user_id → ordre stable pour rendu déterministe.
    let mut users: BTreeMap<&str, (&str, Option<&str>)> = BTreeMap::new();
    for r in rows {
        users
            .entry(&r.user_id)
            .or_insert((&r.user_fingerprint, r.user_display_name.as_deref()));
    }

    let agents: Vec<Value> = users
        .iter()
        .enumerate()
        .map(|(i, (uid, (fp, name)))| {
            let display = if anonymize {
                format!("Employé #{}", i + 1)
            } else {
                name.unwrap_or(*fp).to_string()
            };
            let label = if anonymize {
                format!("Employé #{}", i + 1)
            } else {
                format!("Employé {fp}")
            };
            json!({
                "@id": format!("sobria-team:user-{uid}"),
                "@type": "prov:Agent",
                "prov:label": label,
                "sobria:displayName": display,
                "sobria:fingerprint": if anonymize { Value::Null } else { json!(fp) }
            })
        })
        .collect();

    let activities: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "@id": format!("sobria-team:estimation-{}", r.id),
                "@type": "prov:Activity",
                "prov:generatedAtTime": {
                    "@value": r.ts.to_rfc3339(),
                    "@type": "xsd:dateTime"
                },
                "prov:wasAssociatedWith": {
                    "@id": format!("sobria-team:user-{}", r.user_id)
                },
                "sobria:methodology": r.method,
                "sobria:modelId": r.model_id,
                "sobria:tokensIn": r.tokens_in,
                "sobria:tokensOut": r.tokens_out,
                "sobria:gco2eqGramsP50": r.gco2eq_p50,
                "sobria:waterMl": r.water_ml,
                "sobria:energyWh": r.energy_wh,
                "sobria:region": r.region
            })
        })
        .collect();

    // Totaux agrégés sur la fenêtre (utile pour les consommateurs PROV-O
    // qui ne veulent qu'un résumé).
    let total_count = rows.len() as u64;
    let total_gco2eq: f64 = rows.iter().map(|r| r.gco2eq_p50).sum();
    let total_water_ml: f64 = rows.iter().map(|r| r.water_ml).sum();
    let total_energy_wh: f64 = rows.iter().map(|r| r.energy_wh).sum();

    let bundle = json!({
        "@id": bundle_id,
        "@type": "prov:Bundle",
        "schema:about": {
            "schema:startDate": period_start.format("%Y-%m-%d").to_string(),
            "schema:endDate": period_end.format("%Y-%m-%d").to_string()
        },
        "sobria:organizationName": organization_name,
        "sobria:totalEstimations": total_count,
        "sobria:totalCo2eqGramsP50": total_gco2eq,
        "sobria:totalWaterMl": total_water_ml,
        "sobria:totalEnergyWh": total_energy_wh,
        "sobria:anonymized": anonymize,
        "prov:wasAttributedTo": { "@id": "sobria-team:agent-aggregator" }
    });

    let aggregator_agent = json!({
        "@id": "sobria-team:agent-aggregator",
        "@type": "prov:SoftwareAgent",
        "schema:name": "sobria-team-aggregator",
        "schema:version": env!("CARGO_PKG_VERSION")
    });

    let mut graph = Vec::with_capacity(2 + agents.len() + activities.len());
    graph.push(bundle);
    graph.push(aggregator_agent);
    graph.extend(agents);
    graph.extend(activities);

    json!({
        "@context": {
            "prov": "http://www.w3.org/ns/prov#",
            "sobria": "https://sobr.ia/vocab#",
            "schema": "https://schema.org/",
            "xsd": "http://www.w3.org/2001/XMLSchema#"
        },
        "@graph": graph
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn rows() -> Vec<EstimationRow> {
        vec![
            EstimationRow {
                id: "e-1".into(),
                user_id: "u-1".into(),
                user_fingerprint: "chrome-mac-abc".into(),
                user_display_name: Some("Alice".into()),
                ts: Utc.with_ymd_and_hms(2026, 5, 10, 12, 0, 0).unwrap(),
                method: "afnor_sobria".into(),
                model_id: "llama-3-1-70b".into(),
                tokens_in: 100,
                tokens_out: 500,
                gco2eq_p50: 0.4,
                gco2eq_p5: None,
                gco2eq_p95: None,
                water_ml: 1.5,
                energy_wh: 0.2,
                region: Some("FR".into()),
            },
            EstimationRow {
                id: "e-2".into(),
                user_id: "u-2".into(),
                user_fingerprint: "firefox-linux-def".into(),
                user_display_name: None,
                ts: Utc.with_ymd_and_hms(2026, 5, 11, 14, 0, 0).unwrap(),
                method: "ecologits".into(),
                model_id: "gpt-4o".into(),
                tokens_in: 80,
                tokens_out: 400,
                gco2eq_p50: 0.6,
                gco2eq_p5: None,
                gco2eq_p95: None,
                water_ml: 2.0,
                energy_wh: 0.3,
                region: None,
            },
        ]
    }

    fn window() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 5, 31, 23, 59, 59).unwrap(),
        )
    }

    #[test]
    fn provo_has_required_context_and_graph_size() {
        let (from, to) = window();
        let v = build_team_provo(&rows(), "Acme", from, to, false);
        let ctx = &v["@context"];
        assert!(ctx["prov"].is_string());
        assert!(ctx["sobria"].is_string());
        let graph = v["@graph"].as_array().unwrap();
        // bundle + aggregator agent + 2 users + 2 activities = 6
        assert_eq!(graph.len(), 6);
    }

    #[test]
    fn provo_exposes_fingerprint_when_not_anonymized() {
        let (from, to) = window();
        let v = build_team_provo(&rows(), "Acme", from, to, false);
        let graph = v["@graph"].as_array().unwrap();
        let agent = graph
            .iter()
            .find(|n| n["@type"] == "prov:Agent")
            .expect("user agent");
        let fp = agent["sobria:fingerprint"].as_str().expect("fingerprint");
        assert!(fp.starts_with("chrome-") || fp.starts_with("firefox-"));
    }

    #[test]
    fn provo_anonymizes_fingerprints_when_flag_set() {
        let (from, to) = window();
        let v = build_team_provo(&rows(), "Acme", from, to, true);
        let graph = v["@graph"].as_array().unwrap();
        for n in graph {
            if n["@type"] == "prov:Agent" {
                assert!(
                    n["sobria:fingerprint"].is_null(),
                    "fingerprint doit être null"
                );
                assert!(
                    n["sobria:displayName"]
                        .as_str()
                        .unwrap()
                        .starts_with("Employé"),
                    "displayName doit être anonymisé"
                );
            }
        }
        // Le bundle doit signaler l'anonymisation.
        assert_eq!(v["@graph"][0]["sobria:anonymized"], true);
    }

    #[test]
    fn provo_activity_links_to_user_agent() {
        let (from, to) = window();
        let v = build_team_provo(&rows(), "Acme", from, to, false);
        let graph = v["@graph"].as_array().unwrap();
        let activity = graph
            .iter()
            .find(|n| n["@type"] == "prov:Activity")
            .expect("activity");
        let assoc = activity["prov:wasAssociatedWith"]["@id"].as_str().unwrap();
        assert!(assoc.starts_with("sobria-team:user-"));
    }

    #[test]
    fn provo_totals_match_rows() {
        let (from, to) = window();
        let v = build_team_provo(&rows(), "Acme", from, to, false);
        let bundle = &v["@graph"][0];
        assert_eq!(bundle["sobria:totalEstimations"], 2);
        let total_co2 = bundle["sobria:totalCo2eqGramsP50"].as_f64().unwrap();
        assert!((total_co2 - 1.0).abs() < 1e-9);
    }
}

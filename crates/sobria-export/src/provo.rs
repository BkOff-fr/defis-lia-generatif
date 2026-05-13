//! Sérialisation JSON-LD W3C PROV-O à partir d'un résumé de rapport.
//!
//! Voir : <https://www.w3.org/TR/prov-o/>

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::summary::ReportSummary;

/// Options de génération PROV-O (URIs et version de l'estimateur).
#[derive(Debug, Clone)]
pub struct ProvOOptions {
    pub report_id: String,
    pub organization_name: String,
    pub estimator_version: String,
    pub estimator_seed: u64,
    pub estimator_n: u32,
    pub generated_at: DateTime<Utc>,
}

impl Default for ProvOOptions {
    fn default() -> Self {
        Self {
            report_id: "sobria:report-unknown".into(),
            organization_name: String::new(),
            estimator_version: env!("CARGO_PKG_VERSION").into(),
            estimator_seed: 42,
            estimator_n: 10_000,
            generated_at: Utc::now(),
        }
    }
}

/// Construit le JSON-LD PROV-O à partir du résumé du rapport et des
/// options de provenance. Toutes les valeurs proviennent du résumé
/// (lui-même issu de l'audit ledger) ou des options explicites — pas
/// de génération synthétique.
#[must_use]
pub fn build_provo_jsonld(
    summary: &ReportSummary,
    pdf_sha256: &str,
    opts: &ProvOOptions,
) -> Value {
    let activity_id = format!("{}-activity", opts.report_id);
    let audit_id = format!(
        "sobria:audit-entries-{}-{}",
        summary.first_audit_id, summary.last_audit_id
    );
    let estimator_id = format!("sobria:estimator-engine-v{}", opts.estimator_version);

    json!({
        "@context": {
            "prov": "http://www.w3.org/ns/prov#",
            "sobria": "https://sobr.ia/vocab#",
            "schema": "https://schema.org/",
            "xsd": "http://www.w3.org/2001/XMLSchema#"
        },
        "@graph": [
            {
                "@id": opts.report_id,
                "@type": "prov:Entity",
                "prov:generatedAtTime": {
                    "@value": opts.generated_at.to_rfc3339(),
                    "@type": "xsd:dateTime"
                },
                "prov:wasGeneratedBy": {"@id": activity_id},
                "schema:contentSha256": pdf_sha256,
                "schema:datePublished": opts.generated_at.format("%Y-%m-%d").to_string(),
                "schema:about": {
                    "schema:startDate": summary.period_start.format("%Y-%m-%d").to_string(),
                    "schema:endDate": summary.period_end.format("%Y-%m-%d").to_string()
                },
                "sobria:organizationName": opts.organization_name.clone(),
                "sobria:totalRequests": summary.total_requests,
                "sobria:totalCo2eqGramsP50": summary.total_co2eq_g_p50,
                "sobria:totalCo2eqGramsP5": summary.total_co2eq_g_p5,
                "sobria:totalCo2eqGramsP95": summary.total_co2eq_g_p95,
                "sobria:totalEnergyWhP50": summary.total_energy_wh_p50,
                "sobria:totalWaterLitersP50": summary.total_water_l_p50
            },
            {
                "@id": activity_id,
                "@type": "prov:Activity",
                "prov:startedAtTime": {
                    "@value": opts.generated_at.to_rfc3339(),
                    "@type": "xsd:dateTime"
                },
                "prov:endedAtTime": {
                    "@value": opts.generated_at.to_rfc3339(),
                    "@type": "xsd:dateTime"
                },
                "prov:used": [
                    {"@id": audit_id},
                    {"@id": estimator_id}
                ],
                "prov:wasAssociatedWith": {"@id": "sobria:agent-sobria-export"}
            },
            {
                "@id": audit_id,
                "@type": "prov:Entity",
                "sobria:firstAuditId": summary.first_audit_id,
                "sobria:lastAuditId": summary.last_audit_id,
                "sobria:entryCount": summary.total_requests
            },
            {
                "@id": estimator_id,
                "@type": "prov:Entity",
                "schema:version": opts.estimator_version,
                "sobria:seed": opts.estimator_seed,
                "sobria:monteCarloN": opts.estimator_n,
                "sobria:methodology": "AFNOR SPEC 2314 — Monte-Carlo distributional"
            },
            {
                "@id": "sobria:agent-sobria-export",
                "@type": "prov:SoftwareAgent",
                "schema:name": "sobria-export",
                "schema:version": env!("CARGO_PKG_VERSION")
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_summary() -> ReportSummary {
        ReportSummary {
            period_start: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            total_requests: 247,
            total_co2eq_g_p5: 1500.0,
            total_co2eq_g_p50: 2400.0,
            total_co2eq_g_p95: 3800.0,
            total_energy_wh_p50: 400.0,
            total_water_l_p50: 1.2,
            first_audit_id: 1,
            last_audit_id: 247,
        }
    }

    fn sample_opts() -> ProvOOptions {
        ProvOOptions {
            report_id: "sobria:report-2026-q1".into(),
            organization_name: "Acme Corp".into(),
            estimator_version: "0.2.0".into(),
            estimator_seed: 42,
            estimator_n: 10_000,
            generated_at: Utc.with_ymd_and_hms(2026, 4, 2, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn provo_has_required_context() {
        let v = build_provo_jsonld(&sample_summary(), "abc", &sample_opts());
        let ctx = &v["@context"];
        assert!(ctx["prov"].is_string());
        assert!(ctx["sobria"].is_string());
        assert!(ctx["schema"].is_string());
    }

    #[test]
    fn provo_has_5_nodes_in_graph() {
        let v = build_provo_jsonld(&sample_summary(), "abc", &sample_opts());
        let graph = v["@graph"].as_array().unwrap();
        assert_eq!(graph.len(), 5);
    }

    #[test]
    fn provo_report_node_links_to_activity() {
        let v = build_provo_jsonld(&sample_summary(), "abc", &sample_opts());
        let report = &v["@graph"][0];
        assert_eq!(report["@type"], "prov:Entity");
        let act_ref = &report["prov:wasGeneratedBy"]["@id"];
        assert_eq!(act_ref, "sobria:report-2026-q1-activity");
    }

    #[test]
    fn provo_activity_references_audit_and_estimator() {
        let v = build_provo_jsonld(&sample_summary(), "abc", &sample_opts());
        let activity = &v["@graph"][1];
        let used = activity["prov:used"].as_array().unwrap();
        assert_eq!(used.len(), 2);
        assert!(used.iter().any(|u| u["@id"]
            .as_str()
            .unwrap()
            .contains("audit-entries-1-247")));
    }

    #[test]
    fn provo_includes_pdf_sha256() {
        let v = build_provo_jsonld(&sample_summary(), "deadbeef", &sample_opts());
        assert_eq!(v["@graph"][0]["schema:contentSha256"], "deadbeef");
    }
}

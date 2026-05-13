//! Calcul des agrégats sur une période d'entrées d'audit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sobria_audit::AuditEntry;
use sobria_core::{EstimationResult, Indicator};

use crate::error::{ExportError, ExportResult};

/// Synthèse agrégée des indicateurs sur la période du rapport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u32,
    pub total_co2eq_g_p5: f64,
    pub total_co2eq_g_p50: f64,
    pub total_co2eq_g_p95: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    /// Premier et dernier `audit_id` couverts.
    pub first_audit_id: i64,
    pub last_audit_id: i64,
}

/// Agrège les indicateurs sur la période. Les P5/P50/P95 totaux sont les
/// **sommes des P5/P50/P95 par requête** — pas une convolution probabiliste.
/// C'est documenté dans la méthodologie : pour une lecture worst-case
/// (P95), c'est conservateur.
#[allow(clippy::similar_names)] // sum_p5 / sum_p50 / sum_p95 sont intentionnels
pub(crate) fn aggregate(
    entries: &[AuditEntry],
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> ExportResult<ReportSummary> {
    let mut total_requests: u32 = 0;
    let mut sum_p5 = 0.0;
    let mut sum_p50 = 0.0;
    let mut sum_p95 = 0.0;
    let mut sum_energy = 0.0;
    let mut sum_water = 0.0;
    let mut first_id = i64::MAX;
    let mut last_id = i64::MIN;

    for entry in entries {
        if entry.timestamp < period_start || entry.timestamp >= period_end {
            continue;
        }
        if entry.is_purged() {
            // Entrée purgée RGPD : on compte la requête mais on n'agrège pas
            // les valeurs (payload remplacé par sentinel).
            total_requests += 1;
            first_id = first_id.min(entry.id);
            last_id = last_id.max(entry.id);
            continue;
        }
        let result: EstimationResult = serde_json::from_str(&entry.estimation_result_json)
            .map_err(|e| ExportError::InvalidAuditPayload(format!("entry {}: {e}", entry.id)))?;
        total_requests += 1;
        first_id = first_id.min(entry.id);
        last_id = last_id.max(entry.id);

        for ind in &result.indicators {
            match ind.indicator {
                Indicator::Co2Eq => {
                    sum_p5 += ind.interval.p5;
                    sum_p50 += ind.interval.p50;
                    sum_p95 += ind.interval.p95;
                },
                Indicator::Energy => sum_energy += ind.interval.p50,
                Indicator::Water => sum_water += ind.interval.p50,
                Indicator::CriticalMetals | Indicator::Cost => {},
            }
        }
    }

    if total_requests == 0 {
        return Err(ExportError::EmptyPeriod);
    }
    if first_id == i64::MAX {
        first_id = 0;
    }
    if last_id == i64::MIN {
        last_id = 0;
    }

    Ok(ReportSummary {
        period_start,
        period_end,
        total_requests,
        total_co2eq_g_p5: sum_p5,
        total_co2eq_g_p50: sum_p50,
        total_co2eq_g_p95: sum_p95,
        total_energy_wh_p50: sum_energy,
        total_water_l_p50: sum_water,
        first_audit_id: first_id,
        last_audit_id: last_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use sobria_core::{
        EstimationRequest, Hypothesis, IndicatorValue, UncertaintyInterval,
    };

    fn make_result(co2eq_p50: f64) -> EstimationResult {
        EstimationResult {
            request: EstimationRequest {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                timestamp: Utc::now(),
            },
            indicators: vec![
                IndicatorValue {
                    indicator: Indicator::Co2Eq,
                    interval: UncertaintyInterval::new(
                        co2eq_p50 * 0.7,
                        co2eq_p50,
                        co2eq_p50 * 1.4,
                    )
                    .unwrap(),
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
                key: "k".into(),
                value: serde_json::json!(1),
                source: "test".into(),
            }],
            computed_at: Utc::now(),
            seed: 42,
        }
    }

    fn make_entry(id: i64, ts: DateTime<Utc>, co2eq: f64) -> AuditEntry {
        let payload = serde_json::to_string(&make_result(co2eq)).unwrap();
        AuditEntry {
            id,
            timestamp: ts,
            estimation_result_json: payload,
            prev_sig: String::new(),
            sig: "x".repeat(64),
            purged_at: None,
        }
    }

    #[test]
    fn aggregate_empty_period_returns_error() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap();
        let entries: Vec<AuditEntry> = vec![];
        let err = aggregate(&entries, start, end).unwrap_err();
        assert!(matches!(err, ExportError::EmptyPeriod));
    }

    #[test]
    fn aggregate_sums_co2_per_request() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap();
        let entries = vec![
            make_entry(1, start, 1.0),
            make_entry(2, start, 2.0),
            make_entry(3, start, 3.0),
        ];
        let s = aggregate(&entries, start, end).unwrap();
        assert_eq!(s.total_requests, 3);
        assert!((s.total_co2eq_g_p50 - 6.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_excludes_entries_outside_period() {
        let start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        let entries = vec![
            make_entry(1, Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap(), 1.0),
            make_entry(2, Utc.with_ymd_and_hms(2026, 2, 15, 0, 0, 0).unwrap(), 2.0),
            make_entry(3, Utc.with_ymd_and_hms(2026, 3, 15, 0, 0, 0).unwrap(), 3.0),
        ];
        let s = aggregate(&entries, start, end).unwrap();
        assert_eq!(s.total_requests, 1);
        assert!((s.total_co2eq_g_p50 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_counts_purged_but_excludes_values() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap();
        let mut purged = make_entry(99, start, 5.0);
        purged.estimation_result_json = sobria_audit::PURGED_SENTINEL.into();
        purged.purged_at = Some(Utc::now());
        let entries = vec![make_entry(1, start, 1.0), purged];
        let s = aggregate(&entries, start, end).unwrap();
        assert_eq!(s.total_requests, 2, "purgée comptée");
        assert!((s.total_co2eq_g_p50 - 1.0).abs() < 1e-9, "valeur purgée exclue");
    }
}

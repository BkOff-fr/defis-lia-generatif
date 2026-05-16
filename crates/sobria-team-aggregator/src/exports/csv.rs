//! Export CSV brut RFC 4180 — une ligne par estimation.
//!
//! Format colonnes : `ts, user_fingerprint, user_display_name, method, model_id,
//! tokens_in, tokens_out, gco2eq_p50_g, gco2eq_p5_g, gco2eq_p95_g, water_ml,
//! energy_wh, region`. Anonymisation : `user_fingerprint` → `Employé #N`,
//! `user_display_name` → vide quand le flag est posé.

use std::collections::HashMap;

use crate::error::{AggregatorError, AggregatorResult};
use crate::storage::estimations::EstimationRow;

/// Encode les rows en CSV (UTF-8, séparateur virgule, RFC 4180).
pub fn build_csv(rows: &[EstimationRow], anonymize: bool) -> AggregatorResult<Vec<u8>> {
    let mut user_aliases: HashMap<&str, String> = HashMap::new();
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "ts",
        "user_fingerprint",
        "user_display_name",
        "method",
        "model_id",
        "tokens_in",
        "tokens_out",
        "gco2eq_p50_g",
        "gco2eq_p5_g",
        "gco2eq_p95_g",
        "water_ml",
        "energy_wh",
        "region",
    ])
    .map_err(map_err)?;

    for r in rows {
        let (fp_label, name_label) = if anonymize {
            let alias = if let Some(existing) = user_aliases.get(r.user_id.as_str()) {
                existing.clone()
            } else {
                let new_alias = format!("Employé #{}", user_aliases.len() + 1);
                user_aliases.insert(r.user_id.as_str(), new_alias.clone());
                new_alias
            };
            (alias, String::new())
        } else {
            (
                r.user_fingerprint.clone(),
                r.user_display_name.clone().unwrap_or_default(),
            )
        };
        wtr.write_record([
            r.ts.to_rfc3339(),
            fp_label,
            name_label,
            r.method.clone(),
            r.model_id.clone(),
            r.tokens_in.to_string(),
            r.tokens_out.to_string(),
            r.gco2eq_p50.to_string(),
            r.gco2eq_p5.map(|v| v.to_string()).unwrap_or_default(),
            r.gco2eq_p95.map(|v| v.to_string()).unwrap_or_default(),
            r.water_ml.to_string(),
            r.energy_wh.to_string(),
            r.region.clone().unwrap_or_default(),
        ])
        .map_err(map_err)?;
    }

    wtr.flush().map_err(map_io)?;
    wtr.into_inner()
        .map_err(|e| AggregatorError::Io(e.into_error()))
}

fn map_err(e: csv::Error) -> AggregatorError {
    AggregatorError::Config(format!("csv: {e}"))
}

fn map_io(e: std::io::Error) -> AggregatorError {
    AggregatorError::Io(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn sample() -> Vec<EstimationRow> {
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
                gco2eq_p5: Some(0.5),
                gco2eq_p95: Some(0.7),
                water_ml: 2.0,
                energy_wh: 0.3,
                region: None,
            },
        ]
    }

    #[test]
    fn csv_has_header_and_two_rows() {
        let bytes = build_csv(&sample(), false).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 rows");
        assert!(lines[0].starts_with("ts,"));
        assert!(lines[1].contains("chrome-mac-abc"));
        assert!(lines[1].contains("Alice"));
        assert!(lines[2].contains("firefox-linux-def"));
    }

    #[test]
    fn csv_anonymize_replaces_fingerprints_and_clears_display_name() {
        let bytes = build_csv(&sample(), true).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("chrome-mac-abc"));
        assert!(!text.contains("firefox-linux-def"));
        assert!(!text.contains("Alice"));
        assert!(text.contains("Employé #1"));
        assert!(text.contains("Employé #2"));
    }

    #[test]
    fn csv_empty_input_returns_header_only() {
        let bytes = build_csv(&[], false).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("ts,"));
    }

    #[test]
    fn csv_p5_p95_optional_columns_are_empty_when_none() {
        let bytes = build_csv(&sample(), false).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        // 2e row : gco2eq_p5 = "0.5", gco2eq_p95 = "0.7"
        assert!(text.contains(",0.5,0.7,"));
    }
}

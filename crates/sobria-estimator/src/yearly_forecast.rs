//! Forecaster 12 mois — module M16 (chantier C15).
//!
//! Projette l'empreinte cumulée d'une utilisation IA avec **bande
//! d'incertitude P5/P50/P95** propagée mensuellement, et superposition
//! de plusieurs scénarios de croissance d'adoption.
//!
//! Voir `briefs/chantiers/C15-forecaster-12-mois.md`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sobria_core::{EstimationRequest, Indicator};

use crate::{
    error::{EstimatorError, EstimatorResult},
    params::EstimationParams,
};
// `MonteCarloEngine` n'est plus référencé par la signature publique
// (`forecast_yearly` prend désormais `&dyn EmpreinteEngine`, C24) ; il reste
// utilisé par les tests internes.
#[cfg(test)]
use crate::engine::MonteCarloEngine;

/// Nombre maximum de scénarios par requête de forecast.
pub const MAX_FORECAST_SCENARIOS: usize = 10;

/// Horizon maximum d'une projection (mois).
pub const MAX_FORECAST_HORIZON_MONTHS: u32 = 60;

/// Configuration d'un scénario d'adoption mensuelle.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct YearlyScenario {
    /// Libellé affiché par l'UI (ex: « Status quo », « +10% / mois »).
    pub label: String,
    /// Croissance mensuelle en pourcentage (ex: 5.0 = +5%/mois).
    /// Valeurs négatives autorisées (décroissance d'usage).
    pub monthly_growth_pct: f64,
}

/// Requête de forecast.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct YearlyForecastRequest {
    /// Estimation baseline — utilisée pour extraire `co2eq_p5/p50/p95`.
    pub baseline: EstimationRequest,
    /// Scénarios à projeter (1..=`MAX_FORECAST_SCENARIOS`).
    pub scenarios: Vec<YearlyScenario>,
    /// Horizon de projection en mois.
    pub months: u32,
    /// Volume baseline (prompts par jour au mois 0).
    pub base_volume_per_day: f64,
}

/// Résultat d'un scénario : séries mensuelles + cumulatives + annuel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct YearlyScenarioOutcome {
    pub label: String,
    pub monthly_growth_pct: f64,
    pub monthly_p5_g: Vec<f64>,
    pub monthly_p50_g: Vec<f64>,
    pub monthly_p95_g: Vec<f64>,
    pub cumulative_p5_g: Vec<f64>,
    pub cumulative_p50_g: Vec<f64>,
    pub cumulative_p95_g: Vec<f64>,
    pub annual_p5_g: f64,
    pub annual_p50_g: f64,
    pub annual_p95_g: f64,
}

/// Résultat global du forecast.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct YearlyForecastResult {
    /// Quantiles baseline (1 requête unitaire, sans agrégation volume).
    pub baseline_co2eq_p5_g: f64,
    pub baseline_co2eq_p50_g: f64,
    pub baseline_co2eq_p95_g: f64,
    pub scenarios: Vec<YearlyScenarioOutcome>,
}

// ─────────────────────────────────────────────────────────────────────────────
// forecast_yearly()
// ─────────────────────────────────────────────────────────────────────────────

/// Lance le forecast sur 1-60 mois pour 1-10 scénarios de croissance.
///
/// Polish G (C24) — `engine` est devenu `&dyn EmpreinteEngine` pour
/// honorer la méthodologie choisie par l'utilisateur.
///
/// **Errors** :
/// - `Schema` si `scenarios` est vide ou > `MAX_FORECAST_SCENARIOS`.
/// - `Schema` si `months` hors `[1, MAX_FORECAST_HORIZON_MONTHS]`.
/// - `Schema` si `base_volume_per_day` hors `[0, 10⁶]`.
/// - `Schema` si un `monthly_growth_pct` hors `[-50, 50]`.
/// - `Schema` si deux scénarios ont le même `label`.
/// - propage les erreurs de `engine.estimate`.
pub fn forecast_yearly(
    engine: &dyn crate::engine_trait::EmpreinteEngine,
    req: &YearlyForecastRequest,
) -> EstimatorResult<YearlyForecastResult> {
    validate(req)?;

    // 1. Estimation baseline (1 appel Monte-Carlo).
    let params = EstimationParams::for_model(&req.baseline.model_id)?;
    let result = engine.estimate(&req.baseline, &params)?;
    let co2 = result
        .indicators
        .iter()
        .find(|i| matches!(i.indicator, Indicator::Co2Eq))
        .ok_or_else(|| EstimatorError::Schema("CO2eq indicator manquant".into()))?;
    let p5 = co2.interval.p5;
    let p50 = co2.interval.p50;
    let p95 = co2.interval.p95;

    // 2. Pour chaque scénario, projection des 3 quantiles indépendamment.
    let mut outcomes = Vec::with_capacity(req.scenarios.len());
    for s in &req.scenarios {
        let outcome = project_scenario(
            &s.label,
            s.monthly_growth_pct,
            req.months,
            req.base_volume_per_day,
            p5,
            p50,
            p95,
        );
        outcomes.push(outcome);
    }

    Ok(YearlyForecastResult {
        baseline_co2eq_p5_g: p5,
        baseline_co2eq_p50_g: p50,
        baseline_co2eq_p95_g: p95,
        scenarios: outcomes,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

fn validate(req: &YearlyForecastRequest) -> EstimatorResult<()> {
    if req.scenarios.is_empty() {
        return Err(EstimatorError::Schema(
            "au moins un scénario de forecast requis".into(),
        ));
    }
    if req.scenarios.len() > MAX_FORECAST_SCENARIOS {
        return Err(EstimatorError::Schema(format!(
            "trop de scénarios : {} (max {MAX_FORECAST_SCENARIOS})",
            req.scenarios.len()
        )));
    }
    if req.months == 0 || req.months > MAX_FORECAST_HORIZON_MONTHS {
        return Err(EstimatorError::Schema(format!(
            "months {} hors bornes [1, {MAX_FORECAST_HORIZON_MONTHS}]",
            req.months
        )));
    }
    if !req.base_volume_per_day.is_finite()
        || req.base_volume_per_day < 0.0
        || req.base_volume_per_day > 1_000_000.0
    {
        return Err(EstimatorError::Schema(format!(
            "base_volume_per_day {} hors bornes [0, 10⁶]",
            req.base_volume_per_day
        )));
    }
    let mut seen_labels = std::collections::HashSet::new();
    for s in &req.scenarios {
        if !(-50.0..=50.0).contains(&s.monthly_growth_pct) {
            return Err(EstimatorError::Schema(format!(
                "monthly_growth_pct {} hors bornes [-50, 50] pour '{}'",
                s.monthly_growth_pct, s.label
            )));
        }
        if !seen_labels.insert(s.label.clone()) {
            return Err(EstimatorError::Schema(format!(
                "label scénario en doublon : '{}'",
                s.label
            )));
        }
    }
    Ok(())
}

#[allow(clippy::similar_names)] // p5 / p50 / p95 sont intentionnels et lisibles
fn project_scenario(
    label: &str,
    growth_pct: f64,
    months: u32,
    volume_per_day: f64,
    baseline_p5: f64,
    baseline_p50: f64,
    baseline_p95: f64,
) -> YearlyScenarioOutcome {
    let growth = 1.0 + growth_pct / 100.0;
    let base_factor = volume_per_day * 30.0;
    let mut monthly_p5 = Vec::with_capacity(months as usize);
    let mut monthly_p50 = Vec::with_capacity(months as usize);
    let mut monthly_p95 = Vec::with_capacity(months as usize);
    let mut cum_p5 = Vec::with_capacity(months as usize);
    let mut cum_p50 = Vec::with_capacity(months as usize);
    let mut cum_p95 = Vec::with_capacity(months as usize);
    let mut running_p5 = 0.0_f64;
    let mut running_p50 = 0.0_f64;
    let mut running_p95 = 0.0_f64;
    for n in 0..months {
        let exp = i32::try_from(n).unwrap_or(i32::MAX);
        let factor = base_factor * growth.powi(exp);
        let m_p5 = baseline_p5 * factor;
        let m_p50 = baseline_p50 * factor;
        let m_p95 = baseline_p95 * factor;
        running_p5 += m_p5;
        running_p50 += m_p50;
        running_p95 += m_p95;
        monthly_p5.push(m_p5);
        monthly_p50.push(m_p50);
        monthly_p95.push(m_p95);
        cum_p5.push(running_p5);
        cum_p50.push(running_p50);
        cum_p95.push(running_p95);
    }
    YearlyScenarioOutcome {
        label: label.to_string(),
        monthly_growth_pct: growth_pct,
        monthly_p5_g: monthly_p5,
        monthly_p50_g: monthly_p50,
        monthly_p95_g: monthly_p95,
        cumulative_p5_g: cum_p5,
        cumulative_p50_g: cum_p50,
        cumulative_p95_g: cum_p95,
        annual_p5_g: running_p5,
        annual_p50_g: running_p50,
        annual_p95_g: running_p95,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn baseline_request() -> EstimationRequest {
        EstimationRequest {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            timestamp: Utc::now(),
            modalities: Vec::new(),
            overhead: sobria_core::ContextOverhead::default(),
        }
    }

    fn simple_request(scenarios: Vec<YearlyScenario>) -> YearlyForecastRequest {
        YearlyForecastRequest {
            baseline: baseline_request(),
            scenarios,
            months: 12,
            base_volume_per_day: 100.0,
        }
    }

    #[test]
    fn empty_scenarios_rejected() {
        let engine = MonteCarloEngine::default();
        let req = simple_request(vec![]);
        let err = forecast_yearly(&engine, &req).unwrap_err();
        assert!(format!("{err}").contains("au moins un scénario"));
    }

    #[test]
    fn too_many_scenarios_rejected() {
        let engine = MonteCarloEngine::default();
        let scenarios: Vec<YearlyScenario> = (0..=MAX_FORECAST_SCENARIOS)
            .map(|i| YearlyScenario {
                label: format!("sc{i}"),
                monthly_growth_pct: 0.0,
            })
            .collect();
        let req = simple_request(scenarios);
        let err = forecast_yearly(&engine, &req).unwrap_err();
        assert!(format!("{err}").contains("trop de scénarios"));
    }

    #[test]
    fn invalid_months_rejected() {
        let engine = MonteCarloEngine::default();
        let mut req = simple_request(vec![YearlyScenario {
            label: "ok".into(),
            monthly_growth_pct: 0.0,
        }]);
        req.months = 0;
        assert!(forecast_yearly(&engine, &req).is_err());
        req.months = MAX_FORECAST_HORIZON_MONTHS + 1;
        assert!(forecast_yearly(&engine, &req).is_err());
    }

    #[test]
    fn invalid_growth_rejected() {
        let engine = MonteCarloEngine::default();
        let req = simple_request(vec![YearlyScenario {
            label: "fou".into(),
            monthly_growth_pct: 200.0,
        }]);
        assert!(forecast_yearly(&engine, &req).is_err());
    }

    #[test]
    fn duplicate_labels_rejected() {
        let engine = MonteCarloEngine::default();
        let req = simple_request(vec![
            YearlyScenario {
                label: "x".into(),
                monthly_growth_pct: 0.0,
            },
            YearlyScenario {
                label: "x".into(),
                monthly_growth_pct: 5.0,
            },
        ]);
        let err = forecast_yearly(&engine, &req).unwrap_err();
        assert!(format!("{err}").contains("doublon"));
    }

    #[test]
    fn unknown_model_rejected() {
        let engine = MonteCarloEngine::default();
        let mut req = simple_request(vec![YearlyScenario {
            label: "ok".into(),
            monthly_growth_pct: 0.0,
        }]);
        req.baseline.model_id = "does-not-exist".into();
        assert!(forecast_yearly(&engine, &req).is_err());
    }

    #[test]
    fn growth_zero_produces_constant_series() {
        let engine = MonteCarloEngine::new(42);
        let req = simple_request(vec![YearlyScenario {
            label: "status-quo".into(),
            monthly_growth_pct: 0.0,
        }]);
        let res = forecast_yearly(&engine, &req).unwrap();
        let outcome = &res.scenarios[0];
        assert_eq!(outcome.monthly_p50_g.len(), 12);
        let first = outcome.monthly_p50_g[0];
        for v in &outcome.monthly_p50_g {
            assert!((v - first).abs() < 1e-9, "série pas constante");
        }
    }

    #[test]
    fn growth_5pct_produces_geometric_series() {
        let engine = MonteCarloEngine::new(42);
        let req = simple_request(vec![YearlyScenario {
            label: "growth-5".into(),
            monthly_growth_pct: 5.0,
        }]);
        let res = forecast_yearly(&engine, &req).unwrap();
        let outcome = &res.scenarios[0];
        let m0 = outcome.monthly_p50_g[0];
        let m11 = outcome.monthly_p50_g[11];
        let expected = m0 * 1.05_f64.powi(11);
        assert!(
            (m11 - expected).abs() < 1e-6,
            "geom progress: m11={m11}, expected={expected}"
        );
    }

    #[test]
    fn cumulative_is_strictly_increasing_with_positive_growth() {
        let engine = MonteCarloEngine::new(42);
        let req = simple_request(vec![YearlyScenario {
            label: "growth-5".into(),
            monthly_growth_pct: 5.0,
        }]);
        let res = forecast_yearly(&engine, &req).unwrap();
        let cum = &res.scenarios[0].cumulative_p50_g;
        for i in 1..cum.len() {
            assert!(
                cum[i] > cum[i - 1],
                "cumul doit croître strictement (i={i}, prev={}, cur={})",
                cum[i - 1],
                cum[i]
            );
        }
        // annual_p50_g == dernière valeur cumulée
        assert!((res.scenarios[0].annual_p50_g - cum[cum.len() - 1]).abs() < 1e-9);
    }

    #[test]
    fn p5_p50_p95_ordering_preserved_each_month() {
        let engine = MonteCarloEngine::new(42);
        let req = simple_request(vec![YearlyScenario {
            label: "g".into(),
            monthly_growth_pct: 5.0,
        }]);
        let res = forecast_yearly(&engine, &req).unwrap();
        let o = &res.scenarios[0];
        for n in 0..(o.monthly_p5_g.len()) {
            assert!(
                o.monthly_p5_g[n] <= o.monthly_p50_g[n] + 1e-9,
                "P5 ≤ P50 violé au mois {n}"
            );
            assert!(
                o.monthly_p50_g[n] <= o.monthly_p95_g[n] + 1e-9,
                "P50 ≤ P95 violé au mois {n}"
            );
            assert!(
                o.cumulative_p5_g[n] <= o.cumulative_p50_g[n] + 1e-9,
                "cum P5 ≤ P50 violé au mois {n}"
            );
        }
    }

    #[test]
    fn multi_scenarios_overlay_all_present() {
        let engine = MonteCarloEngine::new(42);
        let req = simple_request(vec![
            YearlyScenario {
                label: "status-quo".into(),
                monthly_growth_pct: 0.0,
            },
            YearlyScenario {
                label: "acceleration".into(),
                monthly_growth_pct: 10.0,
            },
            YearlyScenario {
                label: "ralentissement".into(),
                monthly_growth_pct: -5.0,
            },
        ]);
        let res = forecast_yearly(&engine, &req).unwrap();
        assert_eq!(res.scenarios.len(), 3);
        // L'accélération dépasse status quo, ralentissement plus bas.
        let sq = res.scenarios[0].annual_p50_g;
        let acc = res.scenarios[1].annual_p50_g;
        let ralenti = res.scenarios[2].annual_p50_g;
        assert!(acc > sq, "acc {acc} doit dépasser sq {sq}");
        assert!(ralenti < sq, "ralenti {ralenti} doit être < sq {sq}");
    }

    #[test]
    fn zero_volume_produces_zero_series() {
        let engine = MonteCarloEngine::new(42);
        let mut req = simple_request(vec![YearlyScenario {
            label: "ok".into(),
            monthly_growth_pct: 5.0,
        }]);
        req.base_volume_per_day = 0.0;
        let res = forecast_yearly(&engine, &req).unwrap();
        for v in &res.scenarios[0].monthly_p50_g {
            assert!((*v).abs() < 1e-12);
        }
        assert!(res.scenarios[0].annual_p50_g.abs() < 1e-12);
    }
}

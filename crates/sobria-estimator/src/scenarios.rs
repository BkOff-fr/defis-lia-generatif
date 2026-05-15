//! Simulateur « Et si...? » — chantier C11, module M13.
//!
//! Permet de comparer un baseline à N scénarios en faisant varier
//! ponctuellement certains paramètres (modèle, PUE, mix élec, etc.) et
//! de projeter ces variations sur 12 mois avec une hypothèse géométrique
//! de croissance d'adoption.
//!
//! Voir `briefs/chantiers/C11-simulateur-et-si.md` pour la spec.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sobria_core::{EstimationRequest, EstimationResult, Indicator};

use crate::{
    distributions::Distribution,
    error::{EstimatorError, EstimatorResult},
    params::EstimationParams,
};
// `MonteCarloEngine` n'est plus référencé par la signature publique
// (`simulate` prend désormais `&dyn EmpreinteEngine`, C24) ; il reste utilisé
// par les tests internes.
#[cfg(test)]
use crate::engine::MonteCarloEngine;

/// Borne haute du nombre de scénarios autorisés par requête (anti-abus).
pub const MAX_SCENARIOS: usize = 20;

/// Borne haute de l'horizon de projection (mois).
pub const MAX_FORECAST_MONTHS: u32 = 60;

/// Overrides partiels appliqués sur la requête et les paramètres du baseline.
///
/// Tous les champs sont `Option` : `None` = garde la valeur du baseline.
/// Quand un champ est `Some`, il devient une `Distribution::Point` (valeur
/// déterministe) — pour explorer "et si PUE valait exactement 1.1 ?".
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ParamOverrides {
    /// Modèle alternatif (utilise alors `EstimationParams::for_model`).
    pub model_id: Option<String>,
    /// Tokens de sortie alternatifs (impacte le calcul de E_compute).
    pub tokens_out: Option<u32>,
    /// PUE forcé à cette valeur (déterministe).
    pub pue: Option<f64>,
    /// Facteur d'émission électrique forcé (gCO₂eq/kWh).
    pub if_electrical_g_per_kwh: Option<f64>,
    /// Embodied par requête forcé (gCO₂eq).
    pub embodied_g_per_request: Option<f64>,
    /// WUE forcé (L/kWh).
    pub wue_l_per_kwh: Option<f64>,
}

impl ParamOverrides {
    /// `true` si au moins un champ est défini.
    #[must_use]
    pub fn any(&self) -> bool {
        self.model_id.is_some()
            || self.tokens_out.is_some()
            || self.pue.is_some()
            || self.if_electrical_g_per_kwh.is_some()
            || self.embodied_g_per_request.is_some()
            || self.wue_l_per_kwh.is_some()
    }
}

/// Un scénario nommé avec ses overrides.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Scenario {
    /// Libellé affiché dans la UI (ex: « France 100% renouvelable »).
    pub label: String,
    /// Overrides appliqués. Si vide, le scénario est identique au baseline.
    #[serde(default)]
    pub overrides: ParamOverrides,
}

/// Configuration d'une projection 12 mois (ou autre horizon).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ForecastConfig {
    /// Horizon en mois. Borné à `MAX_FORECAST_MONTHS`.
    pub months: u32,
    /// Croissance mensuelle exprimée en pourcentage (ex : `5.0` = +5%/mois).
    /// Valeurs négatives autorisées (décroissance d'usage).
    pub monthly_growth_pct: f64,
    /// Volume baseline : nombre de prompts par jour au mois 0.
    pub base_volume_per_day: f64,
}

/// Requête de simulation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SimulationRequest {
    /// Estimation de référence — utilisée comme point de comparaison.
    pub baseline: EstimationRequest,
    /// Scénarios à évaluer (1..= `MAX_SCENARIOS`).
    pub scenarios: Vec<Scenario>,
    /// Projection optionnelle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forecast: Option<ForecastConfig>,
}

/// Résultat d'un scénario.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScenarioOutcome {
    /// Libellé hérité du scénario d'entrée.
    pub label: String,
    /// Résultat d'estimation complet (avec bins).
    pub result: EstimationResult,
    /// Δ par rapport au baseline P50, en gCO₂eq (peut être négatif).
    pub delta_co2eq_g: f64,
    /// Δ relatif par rapport au baseline P50, en pourcentage.
    pub delta_pct: f64,
}

/// Résultat d'une projection 12 mois.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ForecastResult {
    /// Horizon effectif (peut être ≤ months demandé si bornage).
    pub months: u32,
    /// Volume baseline par jour utilisé pour calculer les masses.
    pub base_volume_per_day: f64,
    /// Croissance mensuelle utilisée (pourcentage).
    pub monthly_growth_pct: f64,
    /// Série mensuelle baseline en gCO₂eq (longueur = months).
    pub baseline_monthly_co2eq_g: Vec<f64>,
    /// Cumul annuel baseline en gCO₂eq.
    pub baseline_annual_co2eq_g: f64,
    /// Cumul annuel par scénario (ordre identique à `SimulationRequest.scenarios`).
    pub scenarios_annual_co2eq_g: Vec<f64>,
}

/// Résultat global d'une simulation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulationResult {
    /// Estimation du baseline (incl. bins distributionnels).
    pub baseline: EstimationResult,
    /// Un outcome par scénario, dans l'ordre de la requête.
    pub scenarios: Vec<ScenarioOutcome>,
    /// Projection optionnelle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forecast: Option<ForecastResult>,
}

// ─────────────────────────────────────────────────────────────────────────────
// simulate()
// ─────────────────────────────────────────────────────────────────────────────

/// Exécute le baseline + tous les scénarios + la projection.
///
/// Polish G (C24) — Le paramètre `engine` est devenu `&dyn EmpreinteEngine`
/// pour que la simulation honore la méthodologie choisie par l'utilisateur
/// (AFNOR Sobr.ia ou EcoLogits 2026-01). Les wrappers existants qui
/// passaient `&MonteCarloEngine` continuent de fonctionner via coercion
/// `&MonteCarloEngine` → `&dyn EmpreinteEngine` (trait impl additif).
///
/// C25-A8 — Le paramètre `baseline_params` est désormais fourni par
/// l'appelant. Cela permet à la couche IPC (`sobria-app::logic`) d'y
/// appliquer un override datacenter (PUE/IF/WUE) avant de déléguer.
/// Les scénarios partent toujours de `baseline_params` (sauf
/// `model_id` override qui reconstruit les params depuis le nouveau
/// preset — comportement préservé dans `apply_overrides`).
///
/// **Erreurs** :
/// - `Schema` si plus de `MAX_SCENARIOS` scénarios,
/// - `Schema` si forecast hors bornes,
/// - `Schema` si deux scénarios partagent le même `label` (anti-confusion UI),
/// - propage les erreurs de `engine.estimate` (model inconnu, etc.).
pub fn simulate(
    engine: &dyn crate::engine_trait::EmpreinteEngine,
    request: &SimulationRequest,
    baseline_params: &EstimationParams,
) -> EstimatorResult<SimulationResult> {
    if request.scenarios.len() > MAX_SCENARIOS {
        return Err(EstimatorError::Schema(format!(
            "trop de scénarios : {} (max {MAX_SCENARIOS})",
            request.scenarios.len()
        )));
    }
    // Vérification unicité des labels.
    let mut seen_labels = std::collections::HashSet::new();
    for s in &request.scenarios {
        if !seen_labels.insert(s.label.clone()) {
            return Err(EstimatorError::Schema(format!(
                "label scénario en doublon : '{}'",
                s.label
            )));
        }
    }
    if let Some(f) = &request.forecast {
        validate_forecast(f)?;
    }

    // 1. Baseline
    let baseline = engine.estimate(&request.baseline, baseline_params)?;
    let baseline_co2_p50 = co2_p50(&baseline)?;

    // 2. Scénarios
    let mut outcomes: Vec<ScenarioOutcome> = Vec::with_capacity(request.scenarios.len());
    for scenario in &request.scenarios {
        let (req_derived, params_derived) =
            apply_overrides(&request.baseline, baseline_params, &scenario.overrides)?;
        let result = engine.estimate(&req_derived, &params_derived)?;
        let p50 = co2_p50(&result)?;
        let delta = p50 - baseline_co2_p50;
        let delta_pct = if baseline_co2_p50.abs() > f64::EPSILON {
            (delta / baseline_co2_p50) * 100.0
        } else {
            0.0
        };
        outcomes.push(ScenarioOutcome {
            label: scenario.label.clone(),
            result,
            delta_co2eq_g: delta,
            delta_pct,
        });
    }

    // 3. Forecast (optionnel)
    let forecast = request.forecast.as_ref().map(|f| {
        let baseline_monthly = monthly_series(baseline_co2_p50, f);
        let baseline_annual: f64 = baseline_monthly.iter().sum();
        let scenarios_annual: Vec<f64> = outcomes
            .iter()
            .map(|o| {
                let p50 = co2_p50(&o.result).unwrap_or(0.0);
                monthly_series(p50, f).iter().sum()
            })
            .collect();
        ForecastResult {
            months: f.months,
            base_volume_per_day: f.base_volume_per_day,
            monthly_growth_pct: f.monthly_growth_pct,
            baseline_monthly_co2eq_g: baseline_monthly,
            baseline_annual_co2eq_g: baseline_annual,
            scenarios_annual_co2eq_g: scenarios_annual,
        }
    });

    Ok(SimulationResult {
        baseline,
        scenarios: outcomes,
        forecast,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

fn validate_forecast(f: &ForecastConfig) -> EstimatorResult<()> {
    if f.months == 0 || f.months > MAX_FORECAST_MONTHS {
        return Err(EstimatorError::Schema(format!(
            "months {} hors bornes [1, {MAX_FORECAST_MONTHS}]",
            f.months
        )));
    }
    if !(-50.0..=50.0).contains(&f.monthly_growth_pct) {
        return Err(EstimatorError::Schema(format!(
            "monthly_growth_pct {} hors bornes [-50, 50]",
            f.monthly_growth_pct
        )));
    }
    if !f.base_volume_per_day.is_finite()
        || f.base_volume_per_day < 0.0
        || f.base_volume_per_day > 1_000_000.0
    {
        return Err(EstimatorError::Schema(format!(
            "base_volume_per_day {} hors bornes [0, 10⁶]",
            f.base_volume_per_day
        )));
    }
    Ok(())
}

fn co2_p50(result: &EstimationResult) -> EstimatorResult<f64> {
    result
        .indicators
        .iter()
        .find(|i| matches!(i.indicator, Indicator::Co2Eq))
        .map(|i| i.interval.p50)
        .ok_or_else(|| EstimatorError::Schema("CO2eq indicator missing".into()))
}

/// Construit une requête et des paramètres dérivés en appliquant les overrides.
fn apply_overrides(
    baseline_req: &EstimationRequest,
    baseline_params: &EstimationParams,
    overrides: &ParamOverrides,
) -> EstimatorResult<(EstimationRequest, EstimationParams)> {
    // Requête : on remplace tokens_out et/ou model_id si demandé.
    let mut req = baseline_req.clone();
    if let Some(t) = overrides.tokens_out {
        req.tokens_out_estimated = t;
    }
    if let Some(model) = &overrides.model_id {
        req.model_id.clone_from(model);
    }

    // Params : on part de ceux du model_id effectif (potentiellement changé),
    // puis on applique les overrides scalaires comme distributions Point.
    let mut params = if let Some(model) = &overrides.model_id {
        EstimationParams::for_model(model)?
    } else {
        *baseline_params
    };
    if let Some(v) = overrides.pue {
        params.pue = Distribution::Point { value: v };
    }
    if let Some(v) = overrides.if_electrical_g_per_kwh {
        params.if_electrical_g_per_kwh = Distribution::Point { value: v };
    }
    if let Some(v) = overrides.embodied_g_per_request {
        params.embodied_g_per_request = Distribution::Point { value: v };
    }
    if let Some(v) = overrides.wue_l_per_kwh {
        params.wue_l_per_kwh = Distribution::Point { value: v };
    }
    params.validate()?;
    Ok((req, params))
}

/// Calcule la série mensuelle de gCO₂eq agrégée à partir d'un P50 unitaire.
///
/// `mois_n = (P50 × volume_jour × 30) × (1 + growth)^n`.
fn monthly_series(p50_per_req_g: f64, cfg: &ForecastConfig) -> Vec<f64> {
    let base_monthly = p50_per_req_g * cfg.base_volume_per_day * 30.0;
    let growth = 1.0 + cfg.monthly_growth_pct / 100.0;
    (0..cfg.months)
        .map(|n| {
            // Borné à MAX_FORECAST_MONTHS=60 par validate_forecast → cast safe.
            let exp = i32::try_from(n).unwrap_or(i32::MAX);
            base_monthly * growth.powi(exp)
        })
        .collect()
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
        }
    }

    fn sim_request_with(scenarios: Vec<Scenario>) -> SimulationRequest {
        SimulationRequest {
            baseline: baseline_request(),
            scenarios,
            forecast: None,
        }
    }

    /// Helper test (C25-A8) : construit les params du baseline du request
    /// et délègue à `simulate`. Permet de garder les tests internes
    /// concis après le changement de signature `simulate(engine, req,
    /// baseline_params)`.
    fn sim(
        engine: &dyn crate::engine_trait::EmpreinteEngine,
        req: &SimulationRequest,
    ) -> EstimatorResult<SimulationResult> {
        let params = EstimationParams::for_model(&req.baseline.model_id)?;
        simulate(engine, req, &params)
    }

    #[test]
    fn baseline_only_runs_without_scenarios() {
        let engine = MonteCarloEngine::default();
        let res = sim(&engine, &sim_request_with(vec![])).unwrap();
        assert_eq!(res.scenarios.len(), 0);
        assert_eq!(res.baseline.indicators.len(), 3);
    }

    #[test]
    fn scenario_with_pue_override_changes_p50() {
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![
            Scenario {
                label: "PUE bas (1.05)".into(),
                overrides: ParamOverrides {
                    pue: Some(1.05),
                    ..Default::default()
                },
            },
            Scenario {
                label: "PUE haut (1.6)".into(),
                overrides: ParamOverrides {
                    pue: Some(1.6),
                    ..Default::default()
                },
            },
        ];
        let res = sim(&engine, &sim_request_with(scenarios)).unwrap();
        assert_eq!(res.scenarios.len(), 2);
        let pue_low = res.scenarios[0].result.indicators[0].interval.p50;
        let pue_high = res.scenarios[1].result.indicators[0].interval.p50;
        assert!(
            pue_low < pue_high,
            "PUE 1.05 ({pue_low}) doit être < PUE 1.6 ({pue_high})"
        );
        // Delta calculé correctement.
        let baseline_p50 = res.baseline.indicators[0].interval.p50;
        assert!((res.scenarios[0].delta_co2eq_g - (pue_low - baseline_p50)).abs() < 1e-9);
    }

    #[test]
    fn scenario_with_if_override_drastically_changes_p50() {
        // France (56 g/kWh) vs USA charbon-lourd (~600 g/kWh).
        //
        // On force `embodied = 0` dans les deux scénarios pour isoler la
        // contribution du mix électrique. Sans ça, pour un petit modèle
        // type gpt-4o-mini sur 100+500 tokens, l'embodied (~0.002 g)
        // dominerait la composante électrique (~0.00001 g) et le ratio
        // IF charbon/FR serait masqué. Ce constat est un enseignement
        // méthodologique surfacé dans l'UI M13.
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![
            Scenario {
                label: "Mix France".into(),
                overrides: ParamOverrides {
                    if_electrical_g_per_kwh: Some(56.0),
                    embodied_g_per_request: Some(0.0),
                    ..Default::default()
                },
            },
            Scenario {
                label: "Mix charbon".into(),
                overrides: ParamOverrides {
                    if_electrical_g_per_kwh: Some(600.0),
                    embodied_g_per_request: Some(0.0),
                    ..Default::default()
                },
            },
        ];
        let res = sim(&engine, &sim_request_with(scenarios)).unwrap();
        let fr = res.scenarios[0].result.indicators[0].interval.p50;
        let coal = res.scenarios[1].result.indicators[0].interval.p50;
        assert!(
            coal > fr * 2.0,
            "charbon ({coal}) doit être >2× France ({fr})"
        );
    }

    #[test]
    fn scenario_with_model_override_uses_new_params() {
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![Scenario {
            label: "Sonnet à la place de mini".into(),
            overrides: ParamOverrides {
                model_id: Some("claude-3-5-sonnet".into()),
                ..Default::default()
            },
        }];
        let res = sim(&engine, &sim_request_with(scenarios)).unwrap();
        // model_id remplacé → impact attendu sur P50 (probable mais pas testé en
        // valeur absolue : on vérifie juste que ça tourne et que le delta est
        // calculé).
        let outcome = &res.scenarios[0];
        assert!(outcome.delta_co2eq_g.is_finite());
        assert!(outcome.delta_pct.is_finite());
    }

    #[test]
    fn unknown_model_in_scenario_returns_error() {
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![Scenario {
            label: "Modèle qui n'existe pas".into(),
            overrides: ParamOverrides {
                model_id: Some("does-not-exist".into()),
                ..Default::default()
            },
        }];
        let err = sim(&engine, &sim_request_with(scenarios)).unwrap_err();
        assert!(
            format!("{err:?}").to_lowercase().contains("inconnu")
                || format!("{err:?}").to_lowercase().contains("unknown")
        );
    }

    #[test]
    fn duplicate_scenario_labels_rejected() {
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![
            Scenario {
                label: "même".into(),
                overrides: ParamOverrides::default(),
            },
            Scenario {
                label: "même".into(),
                overrides: ParamOverrides::default(),
            },
        ];
        let err = sim(&engine, &sim_request_with(scenarios)).unwrap_err();
        assert!(format!("{err}").contains("doublon"));
    }

    #[test]
    fn too_many_scenarios_rejected() {
        let engine = MonteCarloEngine::new(42);
        let scenarios = (0..=MAX_SCENARIOS)
            .map(|i| Scenario {
                label: format!("scenario_{i}"),
                overrides: ParamOverrides::default(),
            })
            .collect();
        let err = sim(&engine, &sim_request_with(scenarios)).unwrap_err();
        assert!(format!("{err}").contains("trop de scénarios"));
    }

    #[test]
    fn forecast_zero_growth_returns_constant_series() {
        let engine = MonteCarloEngine::new(42);
        let mut req = sim_request_with(vec![]);
        req.forecast = Some(ForecastConfig {
            months: 12,
            monthly_growth_pct: 0.0,
            base_volume_per_day: 10.0,
        });
        let res = sim(&engine, &req).unwrap();
        let f = res.forecast.unwrap();
        assert_eq!(f.baseline_monthly_co2eq_g.len(), 12);
        let first = f.baseline_monthly_co2eq_g[0];
        for v in &f.baseline_monthly_co2eq_g {
            assert!(
                (v - first).abs() < 1e-9,
                "série pas constante : {v} ≠ {first}"
            );
        }
        // Cumul annuel = 12 × mois 0.
        assert!((f.baseline_annual_co2eq_g - first * 12.0).abs() < 1e-6);
    }

    #[test]
    fn forecast_positive_growth_produces_geometric_series() {
        let engine = MonteCarloEngine::new(42);
        let mut req = sim_request_with(vec![]);
        req.forecast = Some(ForecastConfig {
            months: 12,
            monthly_growth_pct: 5.0,
            base_volume_per_day: 10.0,
        });
        let res = sim(&engine, &req).unwrap();
        let f = res.forecast.unwrap();
        // mois 11 = mois 0 × 1.05^11
        let expected_last = f.baseline_monthly_co2eq_g[0] * 1.05_f64.powi(11);
        let last = *f.baseline_monthly_co2eq_g.last().unwrap();
        assert!(
            (last - expected_last).abs() < 1e-6,
            "geo series : last={last}, expected={expected_last}"
        );
    }

    #[test]
    fn forecast_invalid_months_rejected() {
        let engine = MonteCarloEngine::new(42);
        let mut req = sim_request_with(vec![]);
        req.forecast = Some(ForecastConfig {
            months: 0,
            monthly_growth_pct: 5.0,
            base_volume_per_day: 10.0,
        });
        assert!(sim(&engine, &req).is_err());
        req.forecast = Some(ForecastConfig {
            months: MAX_FORECAST_MONTHS + 1,
            monthly_growth_pct: 5.0,
            base_volume_per_day: 10.0,
        });
        assert!(sim(&engine, &req).is_err());
    }

    #[test]
    fn forecast_invalid_growth_rejected() {
        let engine = MonteCarloEngine::new(42);
        let mut req = sim_request_with(vec![]);
        req.forecast = Some(ForecastConfig {
            months: 12,
            monthly_growth_pct: 200.0,
            base_volume_per_day: 10.0,
        });
        assert!(sim(&engine, &req).is_err());
    }

    #[test]
    fn param_overrides_any_returns_true_when_any_field_set() {
        let mut o = ParamOverrides::default();
        assert!(!o.any());
        o.pue = Some(1.2);
        assert!(o.any());
    }

    #[test]
    fn empty_scenario_outcome_equals_baseline_within_seed_determinism() {
        // Un scénario avec overrides vide doit reproduire le baseline
        // (même graine, même calcul → même résultat exact).
        let engine = MonteCarloEngine::new(42);
        let scenarios = vec![Scenario {
            label: "no-op".into(),
            overrides: ParamOverrides::default(),
        }];
        let res = sim(&engine, &sim_request_with(scenarios)).unwrap();
        let baseline_p50 = res.baseline.indicators[0].interval.p50;
        let scenario_p50 = res.scenarios[0].result.indicators[0].interval.p50;
        assert!(
            (baseline_p50 - scenario_p50).abs() < 1e-9,
            "no-op scenario doit reproduire baseline (déterministe)"
        );
        assert!(res.scenarios[0].delta_co2eq_g.abs() < 1e-9);
        assert!(res.scenarios[0].delta_pct.abs() < 1e-9);
    }
}

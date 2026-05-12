//! Moteur Monte-Carlo de l'estimateur (ADR-0004).
//!
//! N=10⁴ tirages par défaut, seed déterministe (`SOBRIA_SEED`, défaut 42).
//! Voir CDC §9 et `briefs/chantiers/C05-estimator-monte-carlo.md`.

use chrono::Utc;
use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};
use sobria_core::{
    EstimationRequest, EstimationResult, Hypothesis, Indicator, IndicatorValue,
    UncertaintyInterval, DEFAULT_SEED,
};
use tracing::debug;

use crate::{
    equivalents, error::{EstimatorError, EstimatorResult}, params::EstimationParams,
};

/// Nombre de tirages par défaut. Voir ADR-0004.
pub const DEFAULT_N: u32 = 10_000;

/// Moteur Monte-Carlo réutilisable.
#[derive(Debug, Clone, Copy)]
pub struct MonteCarloEngine {
    n: u32,
    seed: u64,
}

impl Default for MonteCarloEngine {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

impl MonteCarloEngine {
    /// Construit un moteur avec un seed donné et N=10⁴ par défaut.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { n: DEFAULT_N, seed }
    }

    /// Remplace le nombre de tirages.
    #[must_use]
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = n;
        self
    }

    /// Retourne le seed configuré.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Retourne N (nombre de tirages).
    #[must_use]
    pub fn n(&self) -> u32 {
        self.n
    }

    /// Lance l'estimation pour une requête + un set de paramètres.
    pub fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult> {
        request
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("request: {e}")))?;
        params.validate()?;
        if self.n == 0 {
            return Err(EstimatorError::Schema("N=0 — aucun tirage à effectuer".into()));
        }

        let n_usize = self.n as usize;
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(self.seed);

        let mut co2eq_samples = Vec::with_capacity(n_usize);
        let mut energy_samples = Vec::with_capacity(n_usize);
        let mut water_samples = Vec::with_capacity(n_usize);

        let t_in = f64::from(request.tokens_in);
        let t_out = f64::from(request.tokens_out_estimated);

        for _ in 0..self.n {
            let eps_prefill = params.epsilon_prefill_mj_per_token.sample(&mut rng);
            let eps_decode = params.epsilon_decode_mj_per_token.sample(&mut rng);
            let pue = params.pue.sample(&mut rng);
            let if_elec = params.if_electrical_g_per_kwh.sample(&mut rng);
            let embodied = params.embodied_g_per_request.sample(&mut rng);
            let wue = params.wue_l_per_kwh.sample(&mut rng);

            // E_compute en mJ, puis Wh : 1 Wh = 3 600 J = 3 600 000 mJ.
            // Donc E_total_wh = (E_compute_mj × PUE) / 3 600 000.
            let e_compute_mj = t_in * eps_prefill + t_out * eps_decode;
            let e_total_wh = (e_compute_mj * pue) / 3_600_000.0;

            // CO₂eq (g) = (E_total / 1000) * IF (g/kWh) + embodied
            let co2eq_g = (e_total_wh / 1000.0) * if_elec + embodied;

            // Eau (L) = (E_total / 1000) * WUE (L/kWh)
            let water_l = (e_total_wh / 1000.0) * wue;

            co2eq_samples.push(co2eq_g);
            energy_samples.push(e_total_wh);
            water_samples.push(water_l);
        }

        let co2eq_interval = quantile_interval(&mut co2eq_samples)?;
        let energy_interval = quantile_interval(&mut energy_samples)?;
        let water_interval = quantile_interval(&mut water_samples)?;

        let indicators = vec![
            IndicatorValue {
                indicator: Indicator::Co2Eq,
                interval: co2eq_interval,
                unit: Indicator::Co2Eq.default_unit().into(),
            },
            IndicatorValue {
                indicator: Indicator::Energy,
                interval: energy_interval,
                unit: Indicator::Energy.default_unit().into(),
            },
            IndicatorValue {
                indicator: Indicator::Water,
                interval: water_interval,
                unit: Indicator::Water.default_unit().into(),
            },
        ];

        let equivalents = vec![
            equivalents::co2eq_to_car_km(co2eq_interval.p50),
            equivalents::energy_wh_to_shower_seconds(energy_interval.p50),
            equivalents::energy_wh_to_screen_hours(energy_interval.p50),
        ];

        let hypotheses = hypotheses_from_params(params);

        debug!(
            seed = self.seed,
            n = self.n,
            co2eq_p50 = co2eq_interval.p50,
            "monte-carlo terminé"
        );

        Ok(EstimationResult {
            request: request.clone(),
            indicators,
            equivalents,
            hypotheses,
            computed_at: Utc::now(),
            seed: self.seed,
        })
    }
}

/// Convertit une liste de tirages en intervalle P5/P50/P95.
/// La liste est triée en place (mutable) pour calculer les quantiles.
fn quantile_interval(samples: &mut [f64]) -> EstimatorResult<UncertaintyInterval> {
    if samples.is_empty() {
        return Err(EstimatorError::Schema("samples vide".into()));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p5 = percentile(samples, 0.05);
    let p50 = percentile(samples, 0.50);
    let p95 = percentile(samples, 0.95);
    UncertaintyInterval::new(p5, p50, p95)
        .map_err(|e| EstimatorError::Validation(format!("interval: {e}")))
}

/// Renvoie le percentile à partir d'un tableau **trié**. Implémentation
/// linéaire simple, suffisante pour N=10⁴.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx_f = p * (sorted.len() as f64 - 1.0);
    let lo = idx_f.floor() as usize;
    let hi = idx_f.ceil() as usize;
    let frac = idx_f - lo as f64;
    if hi >= sorted.len() {
        sorted[sorted.len() - 1]
    } else {
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

/// Construit la liste des hypothèses à partir des paramètres distributionnels.
/// Une ligne par paramètre, valeur sérialisée en JSON.
fn hypotheses_from_params(params: &EstimationParams) -> Vec<Hypothesis> {
    fn h(key: &str, dist: &crate::distributions::Distribution) -> Hypothesis {
        Hypothesis {
            key: key.into(),
            value: serde_json::to_value(dist).unwrap_or(serde_json::Value::Null),
            source: "Sobr.ia — paramètres distributionnels (voir docs/methodology/)".into(),
        }
    }
    vec![
        h("epsilon_prefill_mj_per_token", &params.epsilon_prefill_mj_per_token),
        h("epsilon_decode_mj_per_token", &params.epsilon_decode_mj_per_token),
        h("pue", &params.pue),
        h("if_electrical_g_per_kwh", &params.if_electrical_g_per_kwh),
        h("embodied_g_per_request", &params.embodied_g_per_request),
        h("wue_l_per_kwh", &params.wue_l_per_kwh),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributions::Distribution;
    use chrono::Utc;
    use proptest::prelude::*;

    fn sample_request(tokens_in: u32, tokens_out: u32) -> EstimationRequest {
        EstimationRequest {
            model_id: "test-model".into(),
            tokens_in,
            tokens_out_estimated: tokens_out,
            datacenter_id: None,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn estimate_basic_case() {
        let engine = MonteCarloEngine::default();
        let params = EstimationParams::conservative_default();
        let req = sample_request(100, 500);
        let res = engine.estimate(&req, &params).unwrap();
        assert_eq!(res.indicators.len(), 3);
        assert_eq!(res.seed, DEFAULT_SEED);
        // Les 3 indicateurs ont des valeurs strictement positives
        for ind in &res.indicators {
            assert!(ind.interval.p50 > 0.0, "indicateur {:?} p50 ≤ 0", ind.indicator);
        }
    }

    #[test]
    fn reproducibility_same_seed_same_result() {
        let engine = MonteCarloEngine::new(123);
        let params = EstimationParams::conservative_default();
        let req = sample_request(50, 200);
        let r1 = engine.estimate(&req, &params).unwrap();
        let r2 = engine.estimate(&req, &params).unwrap();
        for (a, b) in r1.indicators.iter().zip(r2.indicators.iter()) {
            assert_eq!(a.interval, b.interval, "reproductibilité violée pour {:?}", a.indicator);
        }
    }

    #[test]
    fn quantile_order_preserved() {
        let engine = MonteCarloEngine::default();
        let params = EstimationParams::conservative_default();
        let req = sample_request(80, 300);
        let res = engine.estimate(&req, &params).unwrap();
        for ind in &res.indicators {
            assert!(ind.interval.p5 <= ind.interval.p50);
            assert!(ind.interval.p50 <= ind.interval.p95);
        }
    }

    #[test]
    fn doubling_tokens_approximately_doubles_co2eq() {
        // PUE + IF + ε fixés pour ne mesurer que l'effet tokens
        let params = EstimationParams::conservative_default()
            .with_pue(Distribution::Point { value: 1.3 })
            .with_if_electrical(Distribution::Point { value: 56.0 })
            .with_embodied(Distribution::Point { value: 0.0 });
        let engine = MonteCarloEngine::new(42);
        let r1 = engine.estimate(&sample_request(100, 500), &params).unwrap();
        let r2 = engine.estimate(&sample_request(200, 1000), &params).unwrap();
        let p50_1 = r1.indicators[0].interval.p50;
        let p50_2 = r2.indicators[0].interval.p50;
        let ratio = p50_2 / p50_1;
        assert!((ratio - 2.0).abs() < 0.05, "ratio ≠ 2 : {ratio}");
    }

    #[test]
    fn all_point_distributions_yield_degenerate_interval() {
        let params = EstimationParams {
            epsilon_prefill_mj_per_token: Distribution::Point { value: 1.0 },
            epsilon_decode_mj_per_token: Distribution::Point { value: 2.0 },
            pue: Distribution::Point { value: 1.3 },
            if_electrical_g_per_kwh: Distribution::Point { value: 56.0 },
            embodied_g_per_request: Distribution::Point { value: 0.02 },
            wue_l_per_kwh: Distribution::Point { value: 1.5 },
        };
        let engine = MonteCarloEngine::new(42).with_n(1000);
        let res = engine.estimate(&sample_request(100, 500), &params).unwrap();
        for ind in &res.indicators {
            // Avec uniquement des Point, tous les tirages sont identiques.
            assert!(
                (ind.interval.p95 - ind.interval.p5).abs() < 1e-9,
                "indicateur {:?} pas dégénéré (p5={}, p95={})",
                ind.indicator,
                ind.interval.p5,
                ind.interval.p95,
            );
        }
    }

    #[test]
    fn rejects_empty_model_id() {
        let mut req = sample_request(10, 50);
        req.model_id.clear();
        let res = MonteCarloEngine::default()
            .estimate(&req, &EstimationParams::conservative_default());
        assert!(res.is_err());
    }

    #[test]
    fn rejects_n_zero() {
        let engine = MonteCarloEngine::default().with_n(0);
        let res = engine.estimate(
            &sample_request(10, 50),
            &EstimationParams::conservative_default(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn result_contains_hypotheses_and_equivalents() {
        let res = MonteCarloEngine::default()
            .estimate(
                &sample_request(50, 250),
                &EstimationParams::conservative_default(),
            )
            .unwrap();
        assert_eq!(res.hypotheses.len(), 6, "6 hypothèses (1 par paramètre)");
        assert_eq!(res.equivalents.len(), 3);
    }

    proptest! {
        /// Pour tout couple (request valide, params conservateurs), le résultat
        /// est valide (3 indicateurs, intervalles ordonnés, valeurs finies).
        #[test]
        fn prop_result_is_valid(
            tokens_in in 1u32..10_000,
            tokens_out in 1u32..10_000,
            seed in 0u64..1_000,
        ) {
            let engine = MonteCarloEngine::new(seed).with_n(500);
            let params = EstimationParams::conservative_default();
            let req = sample_request(tokens_in, tokens_out);
            let res = engine.estimate(&req, &params).unwrap();
            prop_assert_eq!(res.indicators.len(), 3);
            for ind in &res.indicators {
                prop_assert!(ind.interval.p5.is_finite());
                prop_assert!(ind.interval.p50.is_finite());
                prop_assert!(ind.interval.p95.is_finite());
                prop_assert!(ind.interval.p5 <= ind.interval.p50);
                prop_assert!(ind.interval.p50 <= ind.interval.p95);
            }
        }
    }
}

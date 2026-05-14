//! Moteur **EcoLogits** — port direct des formules publiées dans :
//!
//! - Rincé S., Banse A., *EcoLogits: Evaluating the Environmental Impacts
//!   of Generative AI*, JOSS 10(111):7471, 2025.
//!   [doi:10.21105/joss.07471](https://doi.org/10.21105/joss.07471)
//! - Documentation officielle :
//!   <https://ecologits.ai/latest/methodology/llm_inference/>
//! - Code source de référence (CC BY-SA 4.0) :
//!   <https://github.com/genai-impact/ecologits>
//!
//! ## License & attribution
//!
//! Les **formules** ici portées sont publiées par Genai Impact sous
//! licence **CC BY-SA 4.0** ([texte](https://creativecommons.org/licenses/by-sa/4.0/)).
//! Le code Rust de portage (cette implémentation) est sous licence MIT
//! pour cohérence avec le reste du projet Sobr.ia. L'attribution exigée
//! par CC BY-SA est respectée via les références ci-dessus + dans
//! `docs/methodology/ECOLOGITS-PORT.md` + dans la fiche méthodologie
//! affichée en UI (page `/methodologies`).
//!
//! ## Différences avec l'implémentation Python de référence
//!
//! Cette v1.0 du port :
//! - **Est déterministe** : on n'échantillonne pas les coefficients
//!   internes (`α`, `β`, `γ`) — on prend leurs valeurs ponctuelles
//!   publiées. L'utilisateur qui veut une plage P5/P95 utilise plutôt
//!   `AfnorSobria`. (Pour ECoLogits avec uncertainty, v1.1+.)
//! - **Utilise B=64** (vLLM, batch typique cloud), constante.
//! - **Utilise H100 80GB / serveur p5.48xlarge** comme référence.
//! - **N'utilise** des `EstimationParams` que `pue` et `if_electrical_g_per_kwh`
//!   et `wue_l_per_kwh` (les `epsilon_*` et `embodied_g_per_request` sont
//!   *ignorés* car remplacés par les formules EcoLogits internes).

use chrono::Utc;
use sobria_core::{
    DistributionBins, EstimationRequest, EstimationResult, Hypothesis, Indicator,
    IndicatorValue, UncertaintyInterval,
};

use crate::{
    distributions::Distribution,
    engine_trait::{EmpreinteEngine, EmpreinteMethod},
    equivalents,
    error::{EstimatorError, EstimatorResult},
    model_presets::find_preset,
    params::EstimationParams,
};

// ─────────────────────────────────────────────────────────────────────────────
// Constantes EcoLogits 2026-01
// Source : <https://ecologits.ai/latest/methodology/llm_inference/>
// Accédée et fixée à la version 2026-01 (cf. notebook/validation.qmd)
// ─────────────────────────────────────────────────────────────────────────────

/// `α` du fit f_E (énergie GPU par token de sortie, Wh).
const ALPHA_E: f64 = 1.17e-6;
/// `β` du fit f_E.
const BETA_E: f64 = -1.12e-2;
/// `γ` du fit f_E.
const GAMMA_E: f64 = 4.05e-5;

/// `α` du fit f_L (latence par token, s).
const ALPHA_L: f64 = 6.78e-4;
/// `β` du fit f_L.
const BETA_L: f64 = 3.12e-4;
/// `γ` du fit f_L.
const GAMMA_L: f64 = 1.94e-2;

/// Batch size de référence (vLLM continuous batching).
const BATCH_SIZE: f64 = 64.0;

/// Puissance électrique du serveur de référence (hors GPUs), p5.48xlarge.
const P_SERVER_W: f64 = 1200.0;

/// Nombre de GPUs installés sur le serveur de référence.
const N_GPU_INSTALLED: f64 = 8.0;

/// Mémoire VRAM par GPU (H100 80 Go).
const MEM_GPU_GB: f64 = 80.0;

/// Bits par poids du modèle (FP16).
const Q_BITS: f64 = 16.0;

/// Overhead mémoire d'inférence (KV cache, activations, marges).
const MEM_OVERHEAD: f64 = 1.2;

/// Embodied carbon du serveur hors GPUs (Boavizta amorti sur 3 ans).
const I_SERVER_NO_GPU_KG: f64 = 5700.0;

/// Embodied carbon d'un GPU H100 (Boavizta).
const I_GPU_KG: f64 = 273.0;

/// Durée de vie hardware (3 ans, en secondes).
const HW_LIFETIME_SEC: f64 = 3.0 * 365.25 * 86_400.0;

// ─────────────────────────────────────────────────────────────────────────────
// Formules EcoLogits
// ─────────────────────────────────────────────────────────────────────────────

/// Nombre de GPUs requis pour servir un modèle de `p_billions` milliards
/// de paramètres en FP16, arrondi à la puissance de 2 supérieure.
///
/// Formule : `n_GPU = next_pow2(ceil(1.2 × P × 16/8 / 80))`.
#[must_use]
pub fn n_gpu(p_billions: f64) -> u32 {
    let m_model_gb = MEM_OVERHEAD * p_billions * Q_BITS / 8.0;
    let raw = (m_model_gb / MEM_GPU_GB).ceil() as u32;
    if raw <= 1 {
        1
    } else {
        raw.next_power_of_two()
    }
}

/// Énergie GPU par token de sortie, Wh, à batch B=64.
///
/// `f_E(P, 64) = α × exp(β × 64) × P + γ`
#[must_use]
pub fn f_energy_per_token_wh(p_billions: f64) -> f64 {
    ALPHA_E * (BETA_E * BATCH_SIZE).exp() * p_billions + GAMMA_E
}

/// Latence par token de sortie, secondes, à batch B=64.
///
/// `f_L(P, 64) = α' × P + β' × 64 + γ'`
#[must_use]
pub fn f_latency_per_token_sec(p_billions: f64) -> f64 {
    ALPHA_L * p_billions + BETA_L * BATCH_SIZE + GAMMA_L
}

/// Énergie totale d'une requête en kWh (usage seul, hors embodied).
#[must_use]
pub fn request_energy_kwh(p_billions: f64, tokens_out: u32, pue: f64) -> f64 {
    let n_gpu_f = f64::from(n_gpu(p_billions));
    let t_out = f64::from(tokens_out);
    let dt_sec = t_out * f_latency_per_token_sec(p_billions);
    let e_gpu_wh = n_gpu_f * t_out * f_energy_per_token_wh(p_billions);
    let e_server_no_gpu_wh = dt_sec * P_SERVER_W * (n_gpu_f / N_GPU_INSTALLED) / BATCH_SIZE / 3600.0;
    let e_server_wh = e_gpu_wh + e_server_no_gpu_wh;
    let e_request_wh = pue * e_server_wh;
    e_request_wh / 1000.0
}

/// Embodied carbon amorti par requête (g CO₂eq).
///
/// `I_request_emb = (ΔT / (B × ΔL)) × I_server`
/// avec `I_server = (n_GPU/N_installed) × I_server_noGPU + n_GPU × I_GPU`.
#[must_use]
pub fn request_embodied_co2eq_g(p_billions: f64, tokens_out: u32) -> f64 {
    let n_gpu_f = f64::from(n_gpu(p_billions));
    let t_out = f64::from(tokens_out);
    let dt_sec = t_out * f_latency_per_token_sec(p_billions);
    // kg CO2eq amorti pour la requête, puis × 1000 pour passer en g.
    let i_server_kg = (n_gpu_f / N_GPU_INSTALLED) * I_SERVER_NO_GPU_KG + n_gpu_f * I_GPU_KG;
    let alloc_ratio = dt_sec / (BATCH_SIZE * HW_LIFETIME_SEC);
    alloc_ratio * i_server_kg * 1000.0
}

// ─────────────────────────────────────────────────────────────────────────────
// Moteur
// ─────────────────────────────────────────────────────────────────────────────

/// Moteur d'estimation EcoLogits (déterministe).
#[derive(Debug, Clone, Copy)]
pub struct EcoLogitsEngine {
    /// Seed conservé pour cohérence d'interface avec [`crate::engine::MonteCarloEngine`].
    /// Non utilisé par les formules EcoLogits (déterministes), mais consigné
    /// dans le résultat pour traçabilité.
    seed: u64,
}

impl EcoLogitsEngine {
    /// Construit un moteur EcoLogits avec un seed donné (utilisé seulement
    /// pour traçabilité dans le ledger).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Retourne le seed configuré.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Échantillonne une valeur représentative d'une distribution
    /// (médiane) — utilisée pour PUE, IF, WUE qui peuvent être passés sous
    /// forme distributionnelle dans `EstimationParams`.
    fn point_value(dist: Distribution) -> f64 {
        match dist {
            Distribution::Point { value } => value,
            Distribution::Uniform { low, high } => f64::midpoint(low, high),
            Distribution::Normal { mean, .. } => mean,
            Distribution::LogNormal { mu, .. } => mu.exp(),
        }
    }

    /// Lance l'estimation EcoLogits.
    ///
    /// `params.pue`, `params.if_electrical_g_per_kwh` et `params.wue_l_per_kwh`
    /// sont utilisés (réduits à leur valeur centrale si distribution).
    /// `params.epsilon_*` et `params.embodied_g_per_request` sont **ignorés**
    /// car remplacés par les formules EcoLogits internes.
    // Pipeline linéaire (validation → params → énergie → CO2 → eau →
    // indicateurs/equivalents/hypothèses → résultat). Le gros du volume
    // vient des littéraux `Hypothesis` explicites destinés à l'audit ;
    // fragmenter en helpers ferait perdre la cohésion sans gagner en clarté.
    #[allow(clippy::too_many_lines)]
    pub fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult> {
        request
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("request: {e}")))?;
        params.validate()?;

        // Retrouver le modèle depuis le registry pour récupérer le nombre
        // de paramètres en milliards.
        let preset = find_preset(&request.model_id).ok_or_else(|| {
            EstimatorError::Schema(format!(
                "EcoLogits: modèle inconnu {:?} (registry Sobr.ia)",
                request.model_id
            ))
        })?;
        let p_billions = preset.approx_params_billions;

        let pue = Self::point_value(params.pue);
        let if_elec = Self::point_value(params.if_electrical_g_per_kwh);
        let wue = Self::point_value(params.wue_l_per_kwh);

        // Usage
        let energy_kwh = request_energy_kwh(p_billions, request.tokens_out_estimated, pue);
        let energy_watt_hours = energy_kwh * 1000.0;
        let usage_co2_g = energy_kwh * if_elec;

        // Embodied : par défaut, formule EcoLogits interne. Mais si
        // l'utilisateur a explicitement passé `params.embodied = Point(0)`
        // (override "usage-only"), on respecte ce choix. Cas typique :
        // ReproductionCase usage-only et comparaisons croisées.
        let embodied_co2_g = match params.embodied_g_per_request {
            Distribution::Point { value: 0.0 } => 0.0,
            _ => request_embodied_co2eq_g(p_billions, request.tokens_out_estimated),
        };

        // Total CO2
        let total_co2_g = usage_co2_g + embodied_co2_g;

        // Eau (formule équivalente à Sobr.ia : E_kWh × WUE)
        let water_l = energy_kwh * wue;

        // Sortie : ECoLogits est déterministe en v1 — P5 = P50 = P95.
        let degenerate = |v: f64| {
            UncertaintyInterval::new(v, v, v).map_err(|e| {
                EstimatorError::Validation(format!("interval EcoLogits ({v}): {e}"))
            })
        };
        let co2_interval = degenerate(total_co2_g)?;
        let energy_interval = degenerate(energy_watt_hours)?;
        let water_interval = degenerate(water_l)?;

        // Pas d'histogramme (samples tous identiques par construction).
        let no_bins: Option<DistributionBins> = None;

        let indicators = vec![
            IndicatorValue {
                indicator: Indicator::Co2Eq,
                interval: co2_interval,
                unit: Indicator::Co2Eq.default_unit().into(),
                bins: no_bins.clone(),
            },
            IndicatorValue {
                indicator: Indicator::Energy,
                interval: energy_interval,
                unit: Indicator::Energy.default_unit().into(),
                bins: no_bins.clone(),
            },
            IndicatorValue {
                indicator: Indicator::Water,
                interval: water_interval,
                unit: Indicator::Water.default_unit().into(),
                bins: no_bins,
            },
        ];

        let equivalents = vec![
            equivalents::co2eq_to_car_km(total_co2_g),
            equivalents::energy_wh_to_shower_seconds(energy_watt_hours),
            equivalents::energy_wh_to_screen_hours(energy_watt_hours),
        ];

        // Hypothèses : on documente les coefficients EcoLogits + les
        // paramètres user (PUE / IF / WUE) effectivement utilisés.
        let hypotheses = vec![
            Hypothesis {
                key: "method".into(),
                value: serde_json::Value::String("ecologits-2026-01".into()),
                source: "EcoLogits — doi:10.21105/joss.07471 (CC BY-SA 4.0)".into(),
            },
            Hypothesis {
                key: "model_params_billions".into(),
                value: serde_json::Value::from(p_billions),
                source: format!("Sobr.ia preset {}", preset.id),
            },
            Hypothesis {
                key: "n_gpu".into(),
                value: serde_json::Value::from(n_gpu(p_billions)),
                source: "EcoLogits formula: next_pow2(ceil(1.2×P×16/8 / 80))".into(),
            },
            Hypothesis {
                key: "pue".into(),
                value: serde_json::to_value(params.pue)
                    .unwrap_or(serde_json::Value::Null),
                source: "User param (réduit à la médiane pour EcoLogits déterministe)".into(),
            },
            Hypothesis {
                key: "if_electrical_g_per_kwh".into(),
                value: serde_json::to_value(params.if_electrical_g_per_kwh)
                    .unwrap_or(serde_json::Value::Null),
                source: "User param (réduit à la médiane)".into(),
            },
            Hypothesis {
                key: "wue_l_per_kwh".into(),
                value: serde_json::to_value(params.wue_l_per_kwh)
                    .unwrap_or(serde_json::Value::Null),
                source: "User param (réduit à la médiane)".into(),
            },
        ];

        Ok(EstimationResult {
            method: EmpreinteMethod::EcoLogits,
            request: request.clone(),
            indicators,
            equivalents,
            hypotheses,
            computed_at: Utc::now(),
            seed: self.seed,
        })
    }
}

impl Default for EcoLogitsEngine {
    fn default() -> Self {
        Self::new(sobria_core::DEFAULT_SEED)
    }
}

impl EmpreinteEngine for EcoLogitsEngine {
    fn method(&self) -> EmpreinteMethod {
        EmpreinteMethod::EcoLogits
    }

    fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult> {
        EcoLogitsEngine::estimate(self, request, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(model_id: &str, t_in: u32, t_out: u32) -> EstimationRequest {
        EstimationRequest {
            model_id: model_id.into(),
            tokens_in: t_in,
            tokens_out_estimated: t_out,
            datacenter_id: None,
            timestamp: Utc::now(),
        }
    }

    fn params_point(pue: f64, if_elec: f64, wue: f64) -> EstimationParams {
        EstimationParams::for_model("llama-3-1-70b")
            .unwrap()
            .with_pue(Distribution::Point { value: pue })
            .with_if_electrical(Distribution::Point { value: if_elec })
            .with_wue(Distribution::Point { value: wue })
    }

    #[test]
    fn n_gpu_llama_70b_is_4() {
        // 1.2 × 70 × 16/8 = 168 GB ; ceil(168/80) = 3 ; next_pow2(3) = 4.
        assert_eq!(n_gpu(70.0), 4);
    }

    #[test]
    fn n_gpu_llama_8b_is_1() {
        // 1.2 × 8 × 16/8 = 19.2 GB ; ceil(19.2/80) = 1.
        assert_eq!(n_gpu(8.0), 1);
    }

    #[test]
    fn n_gpu_mistral_123b_is_4() {
        // 1.2 × 123 × 16/8 = 295.2 GB ; ceil(295.2/80) = 4.
        assert_eq!(n_gpu(123.0), 4);
    }

    #[test]
    fn n_gpu_gpt_4o_200b_is_8() {
        // 1.2 × 200 × 16/8 = 480 GB ; ceil(480/80) = 6 ; next_pow2(6) = 8.
        assert_eq!(n_gpu(200.0), 8);
    }

    #[test]
    fn f_energy_at_b64_llama_70b() {
        // f_E(70, 64) = 1.17e-6 × exp(-0.7168) × 70 + 4.05e-5
        //            ≈ 4.00e-5 + 4.05e-5 ≈ 8.05e-5 Wh/token.
        let v = f_energy_per_token_wh(70.0);
        assert!((v - 8.05e-5).abs() < 1.0e-7, "f_E(70) ≈ 8.05e-5, reçu {v}");
    }

    #[test]
    fn f_latency_at_b64_llama_70b() {
        // f_L(70, 64) = 6.78e-4 × 70 + 3.12e-4 × 64 + 1.94e-2
        //            ≈ 0.0475 + 0.0200 + 0.0194 = 0.0869 s/token.
        let v = f_latency_per_token_sec(70.0);
        assert!((v - 0.0869).abs() < 1.0e-3, "f_L(70) ≈ 0.0869, reçu {v}");
    }

    #[test]
    fn llama_70b_fr_500_tokens_matches_reference() {
        // Cible recalculée Python (notebook/validation.qmd) :
        //   E_request = 0.329 Wh, usage_g (FR 56) = 0.01843 g.
        let eng = EcoLogitsEngine::default();
        let res = eng.estimate(&req("llama-3-1-70b", 100, 500), &params_point(1.2, 56.0, 1.5))
            .unwrap();
        let co2 = res.indicators.iter().find(|i| i.indicator == Indicator::Co2Eq).unwrap();
        let total = co2.interval.p50;
        let embodied = request_embodied_co2eq_g(70.0, 500);
        let usage = total - embodied;
        assert!((usage - 0.01843).abs() / 0.01843 < 0.005, "usage {usage} ≠ 0.01843 ±0.5%");
    }

    #[test]
    fn deterministic_intervals_are_degenerate() {
        let eng = EcoLogitsEngine::default();
        let res = eng.estimate(&req("llama-3-1-70b", 100, 500), &params_point(1.2, 56.0, 1.5))
            .unwrap();
        for ind in &res.indicators {
            assert!(
                (ind.interval.p95 - ind.interval.p5).abs() < 1e-12,
                "EcoLogits déterministe → P5 = P95 attendu pour {:?}, reçu p5={} p95={}",
                ind.indicator,
                ind.interval.p5,
                ind.interval.p95,
            );
        }
    }

    #[test]
    fn unknown_model_id_errors() {
        let eng = EcoLogitsEngine::default();
        let err = eng
            .estimate(&req("modele-bidon", 10, 50), &params_point(1.2, 56.0, 1.5))
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("modèle inconnu"), "message inattendu : {msg}");
    }

    #[test]
    fn implements_empreinte_engine_trait() {
        let eng: Box<dyn EmpreinteEngine> = Box::new(EcoLogitsEngine::default());
        assert_eq!(eng.method(), EmpreinteMethod::EcoLogits);
        assert_eq!(eng.methodology_info().method, EmpreinteMethod::EcoLogits);
    }

    #[test]
    fn doubling_tokens_approximately_doubles_co2eq() {
        let eng = EcoLogitsEngine::default();
        let p = params_point(1.2, 56.0, 1.5);
        let r1 = eng.estimate(&req("llama-3-1-70b", 100, 500), &p).unwrap();
        let r2 = eng.estimate(&req("llama-3-1-70b", 100, 1000), &p).unwrap();
        let g1 = r1.indicators[0].interval.p50;
        let g2 = r2.indicators[0].interval.p50;
        let ratio = g2 / g1;
        // Pas exactement 2x à cause de l'overhead γ (terme constant
        // par token, plus prefill-like). Mais entre 1.9 et 2.1.
        assert!(
            (1.85..=2.15).contains(&ratio),
            "ratio attendu ~2.0, reçu {ratio}"
        );
    }

    #[test]
    fn higher_carbon_grid_increases_co2eq() {
        let eng = EcoLogitsEngine::default();
        let r_fr = eng
            .estimate(&req("llama-3-1-70b", 100, 500), &params_point(1.2, 56.0, 1.5))
            .unwrap();
        let r_va = eng
            .estimate(&req("llama-3-1-70b", 100, 500), &params_point(1.2, 412.0, 1.5))
            .unwrap();
        let g_fr = r_fr.indicators[0].interval.p50;
        let g_va = r_va.indicators[0].interval.p50;
        assert!(g_va > g_fr, "US-VA ({g_va}) doit > FR ({g_fr})");
    }
}

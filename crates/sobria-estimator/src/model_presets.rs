//! Registry de paramètres distributionnels par modèle.
//!
//! Voir `briefs/chantiers/C06-model-presets.md` pour la méthodologie
//! d'extrapolation et `docs/methodology/MODEL-PRESETS.md` pour la
//! documentation chiffre par chiffre.
//!
//! **Garde-fou méthodologique** : les valeurs fournies ici sont
//! *indicatives* ou *extrapolées* pour les modèles fermés. Elles
//! seront raffinées en chantier #7 par validation croisée contre
//! Luccioni 2023 et EcoLogits.

use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    distributions::Distribution,
    error::{EstimatorError, EstimatorResult},
    params::EstimationParams,
};

/// Statut de calibration d'un preset.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationStatus {
    /// Validé contre une étude de référence à ±15 %.
    Validated,
    /// Calibré par ordre de grandeur depuis HF AI Energy Score / EcoLogits.
    Indicative,
    /// Extrapolé depuis un modèle ouvert comparable (modèles fermés).
    Extrapolated,
}

/// Ouverture du modèle.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Openness {
    /// Code et poids publics sous licence ouverte.
    Open,
    /// Poids publics, conditions d'usage variables.
    OpenWeights,
    /// Modèle fermé (API uniquement).
    Closed,
}

/// Preset distributionnel pour un modèle.
///
/// `Deserialize` n'est pas dérivé : les presets sont des constantes statiques
/// définies dans le code Rust (avec `&'static str` et `&'static [&'static str]`,
/// qui ne sont pas désérialisables). `Serialize` suffit pour exposer le
/// catalogue côté frontend.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct ModelPreset {
    /// Identifiant stable (ex: `"gpt-4o-mini"`).
    pub id: &'static str,
    /// Nom commercial.
    pub display_name: &'static str,
    /// Provider.
    pub provider: &'static str,
    /// Famille (regroupement intra-provider).
    pub family: &'static str,
    /// Nombre de paramètres en milliards (estimation publique).
    pub approx_params_billions: f64,
    /// Ouverture du modèle.
    pub openness: Openness,
    /// (P5, P50, P95) de ε_prefill en mJ/token.
    pub epsilon_prefill_mj: (f64, f64, f64),
    /// (P5, P50, P95) de ε_decode en mJ/token.
    pub epsilon_decode_mj: (f64, f64, f64),
    /// (P5, P50, P95) de l'embodied amorti en gCO₂eq/req.
    pub embodied_g_per_req: (f64, f64, f64),
    /// Statut de calibration.
    pub calibration: CalibrationStatus,
    /// Sources documentaires.
    pub sources: &'static [&'static str],
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers d'extrapolation
//
// Ces constantes et fonctions publient les **formules** utilisées pour
// calculer les valeurs du registry. Elles ne sont pas appelées au runtime
// (les valeurs sont hardcodées pour rester lisibles côté lecteur du code),
// mais elles servent de "specs" exécutables : les tests de ce module
// vérifient que chaque preset reste cohérent avec ces formules.
// Voir `docs/methodology/MODEL-PRESETS.md` §2.
// ─────────────────────────────────────────────────────────────────────────────

/// Coefficient `mJ/token/B` pour décoder.
///
/// Recalibré 2026-05 (chantier C24) : auparavant `0.025` (sous-évaluait
/// d'un facteur ~1000), désormais `25.0` pour aligner avec la mesure
/// HF AI Energy Score et ML.ENERGY (Llama 3.1 70B ≈ 1.75 J/token decode,
/// ≈ 25 × 70 mJ/token).
///
/// Cf. ADR-0012 et `briefs/chantiers/C24-multi-methodologie-ecologits.md`
/// pour le détail de la calibration.
pub const K_DECODE_MJ_PER_TOKEN_PER_B: f64 = 25.0;

/// Ratio prefill/decode (le prefill est plus efficient, batché sur GPU).
pub const PREFILL_DECODE_RATIO: f64 = 0.4;

/// Coefficient embodied amorti — Gupta et al. 2022, amorti sur 10⁹ req/an.
pub const K_EMBODIED_G_PER_B: f64 = 0.000_25;

/// Largeur d'incertitude : ratio P95/P50 (et symétrique P50/P5).
/// Correspond à σ_log ≈ 0.30 (CV typique des mesures HF).
pub const UNCERTAINTY_RATIO: f64 = 1.65;

/// Construit un triplet `(p5, p50, p95)` autour d'une médiane donnée
/// avec la largeur standard `UNCERTAINTY_RATIO`.
#[must_use]
pub fn around(p50: f64) -> (f64, f64, f64) {
    (p50 / UNCERTAINTY_RATIO, p50, p50 * UNCERTAINTY_RATIO)
}

/// Extrapole ε_decode (P5, P50, P95) en mJ/token pour un modèle de
/// `n_b` milliards de paramètres.
#[must_use]
pub fn extrapolate_decode(n_b: f64) -> (f64, f64, f64) {
    around(K_DECODE_MJ_PER_TOKEN_PER_B * n_b)
}

/// Extrapole ε_prefill (P5, P50, P95) comme `PREFILL_DECODE_RATIO × ε_decode`.
#[must_use]
pub fn extrapolate_prefill(n_b: f64) -> (f64, f64, f64) {
    around(K_DECODE_MJ_PER_TOKEN_PER_B * n_b * PREFILL_DECODE_RATIO)
}

/// Extrapole embodied amorti par requête.
#[must_use]
pub fn extrapolate_embodied(n_b: f64) -> (f64, f64, f64) {
    around(K_EMBODIED_G_PER_B * n_b)
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry statique
// ─────────────────────────────────────────────────────────────────────────────

/// Liste de tous les presets connus.
///
/// Les triplets sont **calculés explicitement** ici plutôt que via les helpers,
/// pour rendre les chiffres directement lisibles dans le code. Toute évolution
/// passe par la doc `docs/methodology/MODEL-PRESETS.md` puis par modification
/// ici.
pub static MODEL_REGISTRY: &[ModelPreset] = &[
    ModelPreset {
        id: "gpt-4o",
        display_name: "GPT-4o",
        provider: "OpenAI",
        family: "gpt-4",
        approx_params_billions: 200.0,
        openness: Openness::Closed,
        // Décode 200 × 25 = 5000 mJ/tok (≈ 5 J/tok)
        epsilon_prefill_mj: (1210.0, 2000.0, 3300.0),
        epsilon_decode_mj: (3030.0, 5000.0, 8250.0),
        // Embodied 200 × 0.00025 = 0.05 g/req
        embodied_g_per_req: (0.030, 0.050, 0.0825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "EcoLogits 2026-01",
            "HF AI Energy Score 2026",
            "Estimation taille — analyse publique 2024",
        ],
    },
    ModelPreset {
        id: "gpt-4o-mini",
        display_name: "GPT-4o mini",
        provider: "OpenAI",
        family: "gpt-4",
        approx_params_billions: 8.0,
        openness: Openness::Closed,
        // Décode 8 × 25 = 200 mJ/tok — extrapolation pure
        epsilon_prefill_mj: (48.5, 80.0, 132.0),
        epsilon_decode_mj: (121.0, 200.0, 330.0),
        embodied_g_per_req: (0.00121, 0.0020, 0.0033),
        calibration: CalibrationStatus::Extrapolated,
        sources: &["EcoLogits 2026-01", "Estimation taille — analyse publique 2024"],
    },
    ModelPreset {
        id: "claude-3-5-sonnet",
        display_name: "Claude 3.5 Sonnet",
        provider: "Anthropic",
        family: "claude-3",
        approx_params_billions: 200.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (1210.0, 2000.0, 3300.0),
        epsilon_decode_mj: (3030.0, 5000.0, 8250.0),
        embodied_g_per_req: (0.030, 0.050, 0.0825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &["EcoLogits 2026-01 (analogie modèles dense ~200B)"],
    },
    ModelPreset {
        id: "mistral-large-2",
        display_name: "Mistral Large 2",
        provider: "Mistral AI",
        family: "mistral-large",
        approx_params_billions: 123.0,
        openness: Openness::OpenWeights,
        // 123 × 25 = 3075
        epsilon_prefill_mj: (745.0, 1230.0, 2030.0),
        epsilon_decode_mj: (1864.0, 3075.0, 5074.0),
        embodied_g_per_req: (0.01864, 0.03075, 0.05074),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Mistral AI tech report 2024",
            "HF AI Energy Score 2026",
            "EcoLogits 2026-01",
        ],
    },
    ModelPreset {
        id: "mistral-medium-3",
        display_name: "Mistral Medium 3",
        provider: "Mistral AI",
        family: "mistral-medium",
        approx_params_billions: 30.0,
        openness: Openness::OpenWeights,
        // 30 × 25 = 750
        epsilon_prefill_mj: (182.0, 300.0, 495.0),
        epsilon_decode_mj: (455.0, 750.0, 1238.0),
        embodied_g_per_req: (0.00455, 0.0075, 0.01238),
        calibration: CalibrationStatus::Indicative,
        sources: &["Mistral AI 2024", "EcoLogits 2026-01"],
    },
    ModelPreset {
        id: "llama-3-1-70b",
        display_name: "Llama 3.1 70B",
        provider: "Meta",
        family: "llama-3",
        approx_params_billions: 70.0,
        openness: Openness::OpenWeights,
        // 70 × 25 = 1750
        epsilon_prefill_mj: (424.0, 700.0, 1155.0),
        epsilon_decode_mj: (1061.0, 1750.0, 2888.0),
        embodied_g_per_req: (0.0106, 0.0175, 0.02888),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 3.1 paper (Touvron et al. 2024)",
            "HF AI Energy Score 2026",
        ],
    },
    ModelPreset {
        id: "llama-3-1-8b",
        display_name: "Llama 3.1 8B",
        provider: "Meta",
        family: "llama-3",
        approx_params_billions: 8.0,
        openness: Openness::OpenWeights,
        // 8 × 25 = 200
        epsilon_prefill_mj: (48.5, 80.0, 132.0),
        epsilon_decode_mj: (121.0, 200.0, 330.0),
        embodied_g_per_req: (0.00121, 0.0020, 0.0033),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 3.1 paper (Touvron et al. 2024)",
            "HF AI Energy Score 2026",
        ],
    },
    ModelPreset {
        id: "gemini-2-0-flash",
        display_name: "Gemini 2.0 Flash",
        provider: "Google",
        family: "gemini-2",
        approx_params_billions: 32.0,
        openness: Openness::Closed,
        // 32 × 25 = 800
        epsilon_prefill_mj: (194.0, 320.0, 528.0),
        epsilon_decode_mj: (485.0, 800.0, 1320.0),
        embodied_g_per_req: (0.00485, 0.0080, 0.0132),
        calibration: CalibrationStatus::Extrapolated,
        sources: &["Google DeepMind 2025 (annonce publique)", "Analogie Mistral Medium"],
    },
];

/// Cherche un preset par son identifiant exact.
#[must_use]
pub fn find_preset(model_id: &str) -> Option<&'static ModelPreset> {
    MODEL_REGISTRY.iter().find(|p| p.id == model_id)
}

/// Retourne la liste de tous les modèles disponibles.
#[must_use]
pub fn available_models() -> Vec<&'static ModelPreset> {
    MODEL_REGISTRY.iter().collect()
}

impl EstimationParams {
    /// Construit un set de paramètres distributionnels calibré pour un modèle
    /// du registry interne.
    pub fn for_model(model_id: &str) -> EstimatorResult<Self> {
        let preset = find_preset(model_id).ok_or_else(|| {
            let known: Vec<&str> = MODEL_REGISTRY.iter().map(|p| p.id).collect();
            EstimatorError::Schema(format!(
                "modèle inconnu : {model_id:?}. Modèles connus : {known:?}"
            ))
        })?;
        Self::from_preset(preset)
    }

    /// Construit `EstimationParams` directement à partir d'un preset
    /// (utile pour les tests ou pour des presets custom).
    pub fn from_preset(preset: &ModelPreset) -> EstimatorResult<Self> {
        let epsilon_prefill = Distribution::log_normal_from_interval(
            preset.epsilon_prefill_mj.0,
            preset.epsilon_prefill_mj.1,
            preset.epsilon_prefill_mj.2,
        )?;
        let epsilon_decode = Distribution::log_normal_from_interval(
            preset.epsilon_decode_mj.0,
            preset.epsilon_decode_mj.1,
            preset.epsilon_decode_mj.2,
        )?;
        let embodied = Distribution::log_normal_from_interval(
            preset.embodied_g_per_req.0,
            preset.embodied_g_per_req.1,
            preset.embodied_g_per_req.2,
        )?;
        Ok(Self {
            epsilon_prefill_mj_per_token: epsilon_prefill,
            epsilon_decode_mj_per_token: epsilon_decode,
            pue: Distribution::Uniform { low: 1.1, high: 1.4 },
            // Par défaut : mix électrique France 2024 ADEME (~56 gCO₂eq/kWh).
            // Override possible via with_if_electrical().
            if_electrical_g_per_kwh: Distribution::Point { value: 56.0 },
            embodied_g_per_request: embodied,
            wue_l_per_kwh: Distribution::Uniform { low: 0.5, high: 2.5 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MonteCarloEngine;
    use chrono::Utc;
    use sobria_core::{EstimationRequest, Indicator};

    fn request_for(model_id: &str, t_in: u32, t_out: u32) -> EstimationRequest {
        EstimationRequest {
            model_id: model_id.into(),
            tokens_in: t_in,
            tokens_out_estimated: t_out,
            datacenter_id: None,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn registry_has_minimum_models() {
        assert!(MODEL_REGISTRY.len() >= 8);
    }

    #[test]
    fn registry_ids_are_unique() {
        let mut ids: Vec<&str> = MODEL_REGISTRY.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "doublons d'id dans MODEL_REGISTRY");
    }

    #[test]
    fn for_model_returns_valid_params_for_each_preset() {
        for preset in MODEL_REGISTRY {
            let params = EstimationParams::for_model(preset.id)
                .unwrap_or_else(|e| panic!("preset {} : {e}", preset.id));
            params
                .validate()
                .unwrap_or_else(|e| panic!("preset {} non valide : {e}", preset.id));
        }
    }

    #[test]
    fn for_model_unknown_returns_err() {
        let err = EstimationParams::for_model("modele-inconnu-zzz").unwrap_err();
        assert!(
            format!("{err}").contains("modèle inconnu"),
            "message d'erreur attendu, reçu : {err}"
        );
    }

    #[test]
    fn available_models_returns_at_least_eight() {
        assert!(available_models().len() >= 8);
    }

    #[test]
    fn find_preset_known_and_unknown() {
        assert!(find_preset("gpt-4o-mini").is_some());
        assert!(find_preset("ce-modele-nexiste-pas").is_none());
    }

    #[test]
    fn estimation_with_preset_yields_reasonable_co2eq() {
        // Pour GPT-4o-mini, 100 tokens in + 500 tokens out, mix FR (56 g/kWh par
        // défaut dans le preset) : on attend un P50 dans une plage raisonnable.
        // Borne supérieure très permissive — c'est un test de sanity, pas de
        // validation scientifique (laquelle est en C07).
        let params = EstimationParams::for_model("gpt-4o-mini").unwrap();
        let engine = MonteCarloEngine::new(42);
        let res = engine
            .estimate(&request_for("gpt-4o-mini", 100, 500), &params)
            .unwrap();
        let co2eq = res
            .indicators
            .iter()
            .find(|i| i.indicator == Indicator::Co2Eq)
            .unwrap();
        // GPT-4o-mini (~8 B) sur 600 tokens (FR mix) : ordre de grandeur ~10⁻⁴ g
        // — très petit, vu la taille modeste et le mix décarboné.
        assert!(
            co2eq.interval.p50 > 0.0 && co2eq.interval.p50 < 1.0,
            "P50 CO2eq attendu dans [0, 1] g, reçu {}",
            co2eq.interval.p50
        );
    }

    #[test]
    fn closed_models_marked_extrapolated() {
        for preset in MODEL_REGISTRY {
            if preset.openness == Openness::Closed {
                assert_eq!(
                    preset.calibration,
                    CalibrationStatus::Extrapolated,
                    "modèle fermé {} devrait être Extrapolated",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn around_helper_produces_ordered_triplet() {
        let (p5, p50, p95) = around(2.0);
        assert!(p5 < p50);
        assert!(p50 < p95);
        assert!((p50 - 2.0).abs() < 1e-12);
    }

    /// Vérifie que les valeurs hardcodées du registry sont cohérentes avec
    /// les formules d'extrapolation documentées. Si on modifie une valeur
    /// du registry sans toucher à la formule, ce test alerte.
    ///
    /// Tolérance : ±2 % (les valeurs sont hardcodées arrondies).
    #[test]
    fn registry_values_consistent_with_extrapolation_formulas() {
        for preset in MODEL_REGISTRY {
            let n_b = preset.approx_params_billions;
            let (_, expected_decode_p50, _) = extrapolate_decode(n_b);
            let (_, expected_prefill_p50, _) = extrapolate_prefill(n_b);
            let (_, expected_embodied_p50, _) = extrapolate_embodied(n_b);

            let actual_decode = preset.epsilon_decode_mj.1;
            let actual_prefill = preset.epsilon_prefill_mj.1;
            let actual_embodied = preset.embodied_g_per_req.1;

            let tol = 0.02; // 2 %
            assert!(
                (actual_decode - expected_decode_p50).abs() / expected_decode_p50 < tol,
                "{}: decode P50 incohérent (registry={actual_decode}, formule={expected_decode_p50})",
                preset.id
            );
            assert!(
                (actual_prefill - expected_prefill_p50).abs() / expected_prefill_p50 < tol,
                "{}: prefill P50 incohérent (registry={actual_prefill}, formule={expected_prefill_p50})",
                preset.id
            );
            assert!(
                (actual_embodied - expected_embodied_p50).abs() / expected_embodied_p50 < tol,
                "{}: embodied P50 incohérent (registry={actual_embodied}, formule={expected_embodied_p50})",
                preset.id
            );
        }
    }

    #[test]
    fn uncertainty_ratio_yields_expected_p5_p95() {
        let (p5, p50, p95) = around(10.0);
        // P95/P50 = UNCERTAINTY_RATIO et P50/P5 = UNCERTAINTY_RATIO
        assert!((p95 / p50 - UNCERTAINTY_RATIO).abs() < 1e-12);
        assert!((p50 / p5 - UNCERTAINTY_RATIO).abs() < 1e-12);
    }
}

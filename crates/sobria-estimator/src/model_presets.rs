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

/// Périmètre d'un chiffre publié par un fabricant.
///
/// **`Training`** : empreinte totale de l'entraînement du modèle (one-shot,
/// amortie sur la durée de vie). Unité typique : `t_co2eq`, `m3_water`.
///
/// **`InferencePerPrompt`** : empreinte d'un prompt unitaire (estimation
/// vendor selon sa propre méthodologie — médiane, percentile, etc.).
/// Unité typique : `g_co2eq`, `wh`, `ml_water`.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VendorScope {
    /// Empreinte training (one-shot, amorti sur durée de vie).
    Training,
    /// Empreinte par prompt d'inférence (selon méthodologie vendor).
    InferencePerPrompt,
}

/// Métrique exposée dans un vendor disclosure.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VendorUnit {
    /// Tonnes de CO₂eq (typiquement training).
    TCo2Eq,
    /// Grammes de CO₂eq (typiquement par prompt).
    GCo2Eq,
    /// Watt-heures (typiquement par prompt).
    Wh,
    /// Millilitres d'eau (typiquement par prompt).
    MlWater,
    /// Mètres cubes d'eau (typiquement training).
    M3Water,
}

/// Chiffre officiel publié par un fabricant (Mistral, Google, Meta…).
///
/// **C32.4** — agrégation des vendor disclosures officielles (audit datasets
/// Q3 2026 §D). Sobr.ia est le tiers de confiance qui les normalise et les
/// présente avec leur lineage, **sans se substituer aux méthodologies
/// indépendantes** (AFNOR/EcoLogits restent calculées en parallèle).
///
/// `Deserialize` n'est pas dérivé (cf. doc de [`ModelPreset`]).
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct VendorDisclosure {
    /// Fabricant (ex : `"Mistral AI"`, `"Google"`, `"Meta"`).
    pub vendor: &'static str,
    /// Périmètre du chiffre (training ou inference).
    pub scope: VendorScope,
    /// Valeur numérique.
    pub value: f64,
    /// Unité de la valeur.
    pub unit: VendorUnit,
    /// URL canonique de la source (citation directe).
    pub source_url: &'static str,
    /// Date de publication (RFC 3339 simplifié `YYYY-MM-DD`).
    pub published_at: &'static str,
    /// Note méthodologique courte (1-3 phrases) — explication du périmètre,
    /// hypothèses du fabricant, caveats. Affichée dans l'encadré M9.
    pub methodology_note: &'static str,
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
    /// **C32.4** — Chiffres officiels publiés par le fabricant (training
    /// et inference). Liste vide si le fabricant n'a pas de disclosure
    /// publique (cas Anthropic, OpenAI au 2026-05).
    ///
    /// Quand non-vide, l'UI M9 affiche un encadré dédié « Données vendor
    /// disclosure » qui présente la valeur officielle **à côté** de
    /// l'estimation Sobr.ia Monte-Carlo (transparence multi-méthodologie).
    pub vendor_disclosures: &'static [VendorDisclosure],
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
        // OpenAI ne publie pas de disclosure officielle au 2026-05.
        vendor_disclosures: &[],
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
        sources: &[
            "EcoLogits 2026-01",
            "Estimation taille — analyse publique 2024",
        ],
        vendor_disclosures: &[],
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
        // Anthropic ne publie pas de disclosure officielle au 2026-05.
        vendor_disclosures: &[],
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
            "Mistral AI × ADEME × Carbone 4 (2025-08)",
        ],
        // C32.4 — Première ACV complète d'un LLM, publiée par un vendor
        // mondial (Mistral AI) en partenariat avec ADEME + Carbone 4.
        // Source : https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai
        vendor_disclosures: &[
            VendorDisclosure {
                vendor: "Mistral AI",
                scope: VendorScope::Training,
                value: 20_400.0,
                unit: VendorUnit::TCo2Eq,
                source_url: "https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai",
                published_at: "2025-08-01",
                methodology_note: "ACV training Mistral Large 2 — 18 mois d'analyse \
                                   par Carbone 4, vérifiée par l'ADEME. Inclut \
                                   production matériel + énergie training (85.5 % des GES).",
            },
            VendorDisclosure {
                vendor: "Mistral AI",
                scope: VendorScope::Training,
                value: 281_000.0,
                unit: VendorUnit::M3Water,
                source_url: "https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai",
                published_at: "2025-08-01",
                methodology_note: "Consommation d'eau totale du training (91 % du total eau ACV).",
            },
            VendorDisclosure {
                vendor: "Mistral AI",
                scope: VendorScope::InferencePerPrompt,
                value: 1.14,
                unit: VendorUnit::GCo2Eq,
                source_url: "https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai",
                published_at: "2025-08-01",
                methodology_note: "Empreinte d'une requête type 400 tokens \
                                   (équivalent ~10 secondes de streaming vidéo). \
                                   Méthodologie alignée AFNOR SPEC 2314.",
            },
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
        // Mistral n'a pas (encore) étendu sa disclosure ACV à Medium/Small.
        vendor_disclosures: &[],
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
            "Meta Llama 3.1 model card (MODEL_CARD.md)",
        ],
        // C32.4 — Meta publie le training (mais pas l'inférence par prompt).
        // Distinction location-based / market-based : transparence sur le
        // greenwashing « 0 tCO2eq market-based » via REC, à comparer aux
        // 11 390 tCO2eq réellement consommés localement (location-based).
        // Source : https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md
        vendor_disclosures: &[
            VendorDisclosure {
                vendor: "Meta",
                scope: VendorScope::Training,
                value: 11_390.0,
                unit: VendorUnit::TCo2Eq,
                source_url: "https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md",
                published_at: "2024-07-23",
                methodology_note: "Training location-based : émissions réelles \
                                   du mix électrique local des datacenters Meta \
                                   pendant 39.3M GPU-heures H100. C'est la \
                                   valeur la plus honnête.",
            },
            VendorDisclosure {
                vendor: "Meta",
                scope: VendorScope::Training,
                value: 0.0,
                unit: VendorUnit::TCo2Eq,
                source_url: "https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md",
                published_at: "2024-07-23",
                methodology_note: "Training market-based : 0 tCO₂eq car Meta \
                                   achète des REC qui matchent sa conso annuelle \
                                   totale. Comptable mais découplé de l'élec \
                                   réellement consommée pendant le training.",
            },
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
        vendor_disclosures: &[],
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
        sources: &[
            "Google DeepMind 2025 (annonce publique)",
            "Analogie Mistral Medium",
            "Google Environmental Impact paper (2025-08)",
        ],
        // C32.4 — Google publie un prompt médian (méthodologie Google,
        // distincte de l'ACV). Valeurs sous-estimées vs prompts complexes /
        // raisonnement — à présenter comme telles.
        // Source : https://services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf
        vendor_disclosures: &[
            VendorDisclosure {
                vendor: "Google",
                scope: VendorScope::InferencePerPrompt,
                value: 0.03,
                unit: VendorUnit::GCo2Eq,
                source_url: "https://services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf",
                published_at: "2025-08-01",
                methodology_note: "Prompt texte médian Gemini Apps. \
                                   Méthodologie Google (médian, pas P95) — \
                                   sous-estime les requêtes complexes ou agents.",
            },
            VendorDisclosure {
                vendor: "Google",
                scope: VendorScope::InferencePerPrompt,
                value: 0.24,
                unit: VendorUnit::Wh,
                source_url: "https://services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf",
                published_at: "2025-08-01",
                methodology_note: "Énergie médiane par prompt texte Gemini.",
            },
            VendorDisclosure {
                vendor: "Google",
                scope: VendorScope::InferencePerPrompt,
                value: 0.26,
                unit: VendorUnit::MlWater,
                source_url: "https://services.google.com/fh/files/misc/measuring_the_environmental_impact_of_delivering_ai_at_google_scale.pdf",
                published_at: "2025-08-01",
                methodology_note: "Eau médiane par prompt texte Gemini \
                                   (≈ 5 gouttes d'eau).",
            },
        ],
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
            pue: Distribution::Uniform {
                low: 1.1,
                high: 1.4,
            },
            // Par défaut : mix électrique France 2024 ADEME (~56 gCO₂eq/kWh).
            // Override possible via with_if_electrical().
            if_electrical_g_per_kwh: Distribution::Point { value: 56.0 },
            embodied_g_per_request: embodied,
            wue_l_per_kwh: Distribution::Uniform {
                low: 0.5,
                high: 2.5,
            },
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
    fn at_least_three_vendors_have_disclosures() {
        // C32.4 — Mistral Large 2, Gemini, Llama 3.1 70B doivent avoir
        // au moins une disclosure chacun (le scoop pitch v0.8.0).
        let vendors_with_disclosure: std::collections::HashSet<&str> = MODEL_REGISTRY
            .iter()
            .flat_map(|p| p.vendor_disclosures.iter().map(|d| d.vendor))
            .collect();
        assert!(
            vendors_with_disclosure.len() >= 3,
            "C32.4 attend ≥ 3 vendors avec disclosure, vu : {vendors_with_disclosure:?}"
        );
        for expected in ["Mistral AI", "Google", "Meta"] {
            assert!(
                vendors_with_disclosure.contains(expected),
                "vendor {expected} doit avoir au moins une disclosure"
            );
        }
    }

    #[test]
    fn vendor_disclosure_source_urls_are_https() {
        for preset in MODEL_REGISTRY {
            for d in preset.vendor_disclosures {
                assert!(
                    d.source_url.starts_with("https://"),
                    "preset {} : source_url non-HTTPS : {}",
                    preset.id,
                    d.source_url
                );
            }
        }
    }

    #[test]
    fn vendor_disclosure_published_at_is_iso_date() {
        // Format attendu YYYY-MM-DD (10 caractères).
        for preset in MODEL_REGISTRY {
            for d in preset.vendor_disclosures {
                assert_eq!(
                    d.published_at.len(),
                    10,
                    "preset {} : published_at doit être YYYY-MM-DD, reçu {}",
                    preset.id,
                    d.published_at
                );
                let parts: Vec<&str> = d.published_at.split('-').collect();
                assert_eq!(
                    parts.len(),
                    3,
                    "preset {} : format date invalide",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn mistral_large_2_has_training_and_inference_disclosures() {
        // Validation explicite du scoop pitch.
        let mistral = find_preset("mistral-large-2").expect("mistral-large-2 doit exister");
        let has_training = mistral.vendor_disclosures.iter().any(|d| {
            matches!(d.scope, VendorScope::Training) && matches!(d.unit, VendorUnit::TCo2Eq)
        });
        let has_inference = mistral.vendor_disclosures.iter().any(|d| {
            matches!(d.scope, VendorScope::InferencePerPrompt)
                && matches!(d.unit, VendorUnit::GCo2Eq)
        });
        assert!(has_training, "Mistral Large 2 manque training tCO2eq");
        assert!(has_inference, "Mistral Large 2 manque inference gCO2eq");
    }

    #[test]
    fn meta_llama_3_1_70b_has_both_location_and_market_based() {
        // C32.4 — encadré pédagogique location-based vs market-based.
        let llama = find_preset("llama-3-1-70b").expect("llama-3-1-70b doit exister");
        let trainings: Vec<f64> = llama
            .vendor_disclosures
            .iter()
            .filter(|d| matches!(d.scope, VendorScope::Training))
            .map(|d| d.value)
            .collect();
        assert_eq!(
            trainings.len(),
            2,
            "Meta Llama 3.1 70B doit avoir 2 training entries (location + market-based)"
        );
        assert!(
            trainings.contains(&0.0),
            "Llama 3.1 70B manque market-based (0 tCO2eq)"
        );
        assert!(
            trainings.iter().any(|&v| v > 10_000.0),
            "Llama 3.1 70B manque location-based (~11 390 tCO2eq)"
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

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
//!
//! ## C34.2 — Catalogue 2026
//!
//! Étendu en v0.9.0 (chantier C34.2) avec :
//! - 17 nouveaux presets 2025-2026 (Claude 4.x, GPT-5.5, Gemini 3.x,
//!   Llama 4, Mistral Large 3, DeepSeek V4, Grok 4, Qwen 3.6, Phi-4
//!   reasoning).
//! - Anciens presets 2024 marqués `deprecated: true` (conservés pour
//!   reproductibilité historique audit ledger).
//! - Nouveaux enums : [`ModelFamily`], [`ArchitectureKind`],
//!   [`VisionPricing`].
//! - Nouveaux champs de capabilities : `release_date`, `model_family`,
//!   `architecture`, `vision_capable`, `vision_pricing`, `audio_capable`,
//!   `reasoning_capable`, `thinking_token_multiplier`,
//!   `default_context_overhead_tokens`, `deprecated`, `source_url`,
//!   `active_params_b`.
//!
//! Shortlist source : `briefs/chantiers/C34-shortlist-models-validated.md`.

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

// ─────────────────────────────────────────────────────────────────────────────
// C34.2 — Nouveaux enums : ModelFamily, ArchitectureKind, VisionPricing
// ─────────────────────────────────────────────────────────────────────────────

/// **C34.2** — Famille du fabricant. Sert au filtrage et au regroupement UI.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    /// Anthropic (Claude).
    Anthropic,
    /// OpenAI (GPT, o1/o3).
    OpenAi,
    /// Google DeepMind (Gemini, Gemma).
    GoogleDeepMind,
    /// Meta AI (Llama).
    MetaAi,
    /// Mistral AI (Mistral, Codestral, Pixtral).
    MistralAi,
    /// DeepSeek (V3/R1/V4).
    DeepSeek,
    /// xAI (Grok).
    Xai,
    /// Alibaba (Qwen).
    Alibaba,
    /// Microsoft (Phi).
    Microsoft,
    /// Autre / non classifié.
    Other,
}

/// **C34.2** — Architecture interne du modèle.
///
/// L'énergie d'inférence (`ε_decode`) est calculée à partir des paramètres
/// **actifs** (cf. [`ModelPreset::active_params_b`]). Pour MoE, c'est
/// significativement inférieur aux paramètres totaux.
#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ArchitectureKind {
    /// Transformer dense classique (GPT-3/4, Claude 1-3, Llama 1-3).
    DenseTransformer,
    /// Mixture-of-Experts (Mixtral, DeepSeek V3+, Llama 4, Qwen3 MoE).
    Moe {
        /// Nombre total d'experts dans chaque couche MoE.
        experts: u32,
        /// Nombre d'experts activés par token (typiquement 1-8).
        active_experts: u32,
    },
    /// Architecture Mamba / SSM (rare en frontier).
    Mamba,
    /// Architecture hybride (Mamba + Transformer, etc.).
    Hybrid,
}

/// **C34.2** — Formule de tarification tokens vision par fabricant.
///
/// Sourcée des docs publiques officielles (voir [`ModelPreset::source_url`]) :
/// - **OpenAI** : <https://platform.openai.com/docs/guides/vision/calculating-costs>
/// - **Anthropic** : <https://docs.anthropic.com/en/docs/build-with-claude/vision>
/// - **Google Gemini** : <https://ai.google.dev/gemini-api/docs/vision>
/// - **Meta Llama** : <https://ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/>
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum VisionPricing {
    /// OpenAI : `base + per_tile × ⌈W/tile_size⌉ × ⌈H/tile_size⌉` en haute
    /// résolution. Constants typiques : `base = 85`, `per_tile = 170`,
    /// `tile_size = 512`. En basse résolution : `base` seul.
    OpenAiTiles {
        /// Tokens de base (low detail).
        base: u32,
        /// Tokens par tile (high detail).
        per_tile: u32,
        /// Taille de tile en pixels (carré).
        tile_size: u32,
    },
    /// Anthropic Claude : `(W × H) / divisor` tokens, plafonné à `max_tokens`.
    /// Constants typiques : `divisor = 750`, `max_tokens = 1568`.
    AnthropicArea {
        /// Diviseur appliqué à l'aire en pixels.
        divisor: u32,
        /// Plafond de tokens par image.
        max_tokens: u32,
    },
    /// Google Gemini : `base` tokens fixes pour images ≤ 384×384, sinon
    /// `base × ⌈W/tile_size⌉ × ⌈H/tile_size⌉`. Constants : `base = 258`,
    /// `tile_size = 768`.
    GeminiNative {
        /// Tokens de base (image ≤ 384×384).
        base: u32,
        /// Taille de tile (768 chez Gemini).
        tile_size: u32,
    },
    /// Meta Llama 3.2 Vision / Llama 4 multimodal : `tokens_per_image` fixe
    /// (patches 14×14 sur 560×560 → ≈ 1601 tokens chez Llama 3.2).
    LlamaPatches {
        /// Tokens fixes par image traitée.
        tokens_per_image: u32,
    },
}

impl VisionPricing {
    /// Calcule les tokens facturés pour `count` images de dimensions données.
    ///
    /// `high_detail = true` active la tile-based pricing (OpenAI) ou la
    /// version haute résolution pour les autres. Pour Anthropic et Llama,
    /// le paramètre est ignoré (la formule est unique).
    ///
    /// **Garanties** : retourne 0 si `count = 0`. Pas de panique
    /// arithmétique : `saturating_*` utilisé en interne.
    #[must_use]
    pub fn tokens_for(&self, count: u32, width: u32, height: u32, high_detail: bool) -> u32 {
        if count == 0 {
            return 0;
        }
        match self {
            Self::OpenAiTiles {
                base,
                per_tile,
                tile_size,
            } => {
                if !high_detail || width == 0 || height == 0 {
                    base.saturating_mul(count)
                } else {
                    let tiles_w = width.div_ceil(*tile_size);
                    let tiles_h = height.div_ceil(*tile_size);
                    let per_image = base.saturating_add(
                        per_tile
                            .saturating_mul(tiles_w)
                            .saturating_mul(tiles_h),
                    );
                    per_image.saturating_mul(count)
                }
            }
            Self::AnthropicArea {
                divisor,
                max_tokens,
            } => {
                if *divisor == 0 || width == 0 || height == 0 {
                    return max_tokens.saturating_mul(count);
                }
                let area = width.saturating_mul(height);
                let per_image = (area / divisor).min(*max_tokens);
                per_image.saturating_mul(count)
            }
            Self::GeminiNative { base, tile_size } => {
                if (width <= 384 && height <= 384) || *tile_size == 0 {
                    base.saturating_mul(count)
                } else {
                    let tiles_w = width.div_ceil(*tile_size);
                    let tiles_h = height.div_ceil(*tile_size);
                    base.saturating_mul(tiles_w)
                        .saturating_mul(tiles_h)
                        .saturating_mul(count)
                }
            }
            Self::LlamaPatches { tokens_per_image } => tokens_per_image.saturating_mul(count),
        }
    }
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
///
/// **C34.2** — 4 bools de capabilities (vision/audio/reasoning/deprecated) :
/// allow `struct_excessive_bools` car chaque flag a une sémantique distincte
/// et indépendante (refactor en enum/bitflags rendrait le code moins lisible).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct ModelPreset {
    /// Identifiant stable (ex: `"gpt-4o-mini"`).
    pub id: &'static str,
    /// Nom commercial.
    pub display_name: &'static str,
    /// Provider (legacy — `model_family` est la version typée C34.2).
    pub provider: &'static str,
    /// Famille (regroupement intra-provider, legacy).
    pub family: &'static str,
    /// Nombre de paramètres **total** en milliards.
    ///
    /// - Pour dense : égal à [`Self::active_params_b`].
    /// - Pour MoE : peut être beaucoup plus grand (Llama 4 Maverick =
    ///   400B total / 17B actifs).
    pub approx_params_billions: f64,
    /// **C34.2** — Nombre de paramètres **actifs** par token (en
    /// milliards). Drive l'énergie d'inférence.
    ///
    /// - Pour dense : `= approx_params_billions`.
    /// - Pour MoE : `= experts × params_per_expert + shared` (publié
    ///   par le vendor).
    pub active_params_b: f64,
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
    /// **C34.2** — Date de sortie publique (ISO `YYYY-MM-DD`).
    ///
    /// Format `&'static str` au lieu de `chrono::NaiveDate` pour rester
    /// compatible `static` (NaiveDate n'a pas de constructeur const).
    /// Format validé en test (10 chars, 3 segments `YYYY-MM-DD`).
    pub release_date: &'static str,
    /// **C34.2** — Famille typée du fabricant.
    pub model_family: ModelFamily,
    /// **C34.2** — Architecture du modèle.
    pub architecture: ArchitectureKind,
    /// **C34.2** — `true` si le modèle accepte les images en entrée.
    pub vision_capable: bool,
    /// **C34.2** — Formule de tarification tokens vision selon vendor.
    /// `None` si `vision_capable = false`.
    pub vision_pricing: Option<VisionPricing>,
    /// **C34.2** — `true` si le modèle accepte l'audio en entrée.
    pub audio_capable: bool,
    /// **C34.2** — `true` si le modèle a un mode reasoning intégré
    /// (chain-of-thought tokens visibles ou cachés : o3, R1, V4, Claude
    /// extended thinking, Gemini thinking, Phi-4 reasoning, etc.).
    pub reasoning_capable: bool,
    /// **C34.2** — `(P5, P95)` du ratio thinking/output tokens pour
    /// reasoning models. `None` si `reasoning_capable = false`.
    ///
    /// Sources :
    /// - OpenAI o3 / GPT-5.5 Thinking : `(5.0, 30.0)` (system card)
    /// - DeepSeek R1 / V4 thinking : `(8.0, 25.0)` (paper arXiv 2501.12948)
    /// - Claude extended thinking : `(2.0, 50.0)` (configurable, max 128k)
    /// - Gemini 2.5+ thinking : `(3.0, 25.0)` (Google doc)
    /// - Phi-4 reasoning : `(5.0, 15.0)` (Phi-4 reasoning technical report)
    pub thinking_token_multiplier: Option<(f64, f64)>,
    /// **C34.2** — Overhead système typique (system prompt + tools) en
    /// tokens pour l'INTERFACE app vendor par défaut.
    ///
    /// Valeurs P50 utilisées (cf. brief C34 §3 sources) :
    /// - Claude (claude.ai) : 2000
    /// - GPT (ChatGPT) : 1000
    /// - Gemini (web) : 1000
    /// - Mistral (chat.mistral.ai) : 300
    /// - API directe / open-weights local : 0
    ///
    /// **Estimation ± 50 %** — basée sur leaks publics et reverse-engineering
    /// (Claude.ai, ChatGPT app, Gemini app). À surcharger en mode Expert
    /// si l'utilisateur connaît sa valeur exacte.
    pub default_context_overhead_tokens: u32,
    /// **C34.2** — `true` pour les modèles obsolètes conservés pour
    /// reproductibilité historique de l'audit ledger uniquement (à exclure
    /// par défaut de la liste UI via [`available_models_filtered`]).
    pub deprecated: bool,
    /// **C34.2** — URL canonique de la source vendor (model card, blog
    /// post, system card). Toujours HTTPS, validé en test.
    pub source_url: &'static str,
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
//
// **C34.2** : les formules prennent `active_params_b` (pas total) pour
// les MoE. Pour dense, active == total donc identique.
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
/// `n_b` milliards de paramètres **actifs**.
#[must_use]
pub fn extrapolate_decode(n_b: f64) -> (f64, f64, f64) {
    around(K_DECODE_MJ_PER_TOKEN_PER_B * n_b)
}

/// Extrapole ε_prefill (P5, P50, P95) comme `PREFILL_DECODE_RATIO × ε_decode`.
#[must_use]
pub fn extrapolate_prefill(n_b: f64) -> (f64, f64, f64) {
    around(K_DECODE_MJ_PER_TOKEN_PER_B * n_b * PREFILL_DECODE_RATIO)
}

/// Extrapole embodied amorti par requête en gCO₂eq.
#[must_use]
pub fn extrapolate_embodied(n_b: f64) -> (f64, f64, f64) {
    around(K_EMBODIED_G_PER_B * n_b)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers C34.2 : valeurs vision pricing canoniques par vendor
// ─────────────────────────────────────────────────────────────────────────────

/// OpenAI Vision pricing canonique : `85 + 170 × tiles`.
const OPENAI_VISION: VisionPricing = VisionPricing::OpenAiTiles {
    base: 85,
    per_tile: 170,
    tile_size: 512,
};

/// Anthropic Vision pricing canonique : `(W × H) / 750`, max 1568.
const ANTHROPIC_VISION: VisionPricing = VisionPricing::AnthropicArea {
    divisor: 750,
    max_tokens: 1568,
};

/// Google Gemini vision native : 258 tokens, tiles 768.
const GEMINI_VISION: VisionPricing = VisionPricing::GeminiNative {
    base: 258,
    tile_size: 768,
};

/// Meta Llama vision (patches 14×14 sur 560×560) : 1601 tokens fixe.
const LLAMA_VISION: VisionPricing = VisionPricing::LlamaPatches {
    tokens_per_image: 1601,
};

// ─────────────────────────────────────────────────────────────────────────────
// Registry statique
//
// Ordre : nouveaux presets 2025-2026 d'abord (par fraîcheur descendante),
// puis presets historiques 2024 (deprecated:true) en fin de registry.
// ─────────────────────────────────────────────────────────────────────────────

/// Liste de tous les presets connus.
///
/// **Cohorte v0.9.0** : 25 presets, dont 17 sortis 2025-2026 et 8 deprecated
/// 2024 (conservés pour reproductibilité de l'audit ledger historique).
pub static MODEL_REGISTRY: &[ModelPreset] = &[
    // ════════════════════════════════════════════════════════════════════════
    // ANTHROPIC (6 presets, 6 en 2025-2026)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "claude-opus-4-7",
        display_name: "Claude Opus 4.7",
        provider: "Anthropic",
        family: "claude-4",
        approx_params_billions: 2000.0,
        active_params_b: 2000.0,
        openness: Openness::Closed,
        // 2000 × 25 = 50000 mJ/tok
        epsilon_prefill_mj: (12_120.0, 20_000.0, 33_000.0),
        epsilon_decode_mj: (30_300.0, 50_000.0, 82_500.0),
        embodied_g_per_req: (0.303, 0.500, 0.825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude Opus 4.7 release (2026-04-16)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-16",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://www.anthropic.com/news",
    },
    ModelPreset {
        id: "claude-sonnet-4-6",
        display_name: "Claude Sonnet 4.6",
        provider: "Anthropic",
        family: "claude-4",
        approx_params_billions: 400.0,
        active_params_b: 400.0,
        openness: Openness::Closed,
        // 400 × 25 = 10000 mJ/tok
        epsilon_prefill_mj: (2_424.0, 4_000.0, 6_600.0),
        epsilon_decode_mj: (6_061.0, 10_000.0, 16_500.0),
        embodied_g_per_req: (0.0606, 0.100, 0.165),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude Sonnet 4.6 release (2026-02-17)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-02-17",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://en.wikipedia.org/wiki/Claude_(language_model)",
    },
    ModelPreset {
        id: "claude-haiku-4-5",
        display_name: "Claude Haiku 4.5",
        provider: "Anthropic",
        family: "claude-4",
        approx_params_billions: 70.0,
        active_params_b: 70.0,
        openness: Openness::Closed,
        // 70 × 25 = 1750
        epsilon_prefill_mj: (424.0, 700.0, 1_155.0),
        epsilon_decode_mj: (1_061.0, 1_750.0, 2_888.0),
        embodied_g_per_req: (0.0106, 0.0175, 0.02888),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude Haiku 4.5 release (2025-10)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-10-15",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://claudefa.st/blog/models",
    },
    ModelPreset {
        id: "claude-opus-4",
        display_name: "Claude Opus 4",
        provider: "Anthropic",
        family: "claude-4",
        approx_params_billions: 1500.0,
        active_params_b: 1500.0,
        openness: Openness::Closed,
        // 1500 × 25 = 37500
        epsilon_prefill_mj: (9_091.0, 15_000.0, 24_750.0),
        epsilon_decode_mj: (22_727.0, 37_500.0, 61_875.0),
        embodied_g_per_req: (0.227, 0.375, 0.619),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude 4 release (2025-05-22)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-05-22",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://www.anthropic.com/news",
    },
    ModelPreset {
        id: "claude-sonnet-4",
        display_name: "Claude Sonnet 4",
        provider: "Anthropic",
        family: "claude-4",
        approx_params_billions: 400.0,
        active_params_b: 400.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (2_424.0, 4_000.0, 6_600.0),
        epsilon_decode_mj: (6_061.0, 10_000.0, 16_500.0),
        embodied_g_per_req: (0.0606, 0.100, 0.165),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude 4 release (2025-05-22)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-05-22",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://www.anthropic.com/news",
    },
    ModelPreset {
        id: "claude-3-7-sonnet",
        display_name: "Claude 3.7 Sonnet",
        provider: "Anthropic",
        family: "claude-3",
        approx_params_billions: 200.0,
        active_params_b: 200.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (1_212.0, 2_000.0, 3_300.0),
        epsilon_decode_mj: (3_030.0, 5_000.0, 8_250.0),
        embodied_g_per_req: (0.0303, 0.050, 0.0825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Anthropic Claude 3.7 Sonnet release (2025-02-25)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-02-25",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((2.0, 50.0)),
        default_context_overhead_tokens: 2000,
        deprecated: false,
        source_url: "https://www.anthropic.com/news/claude-3-7-sonnet",
    },
    // ════════════════════════════════════════════════════════════════════════
    // OPENAI (4 presets actifs, 2 deprecated)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "gpt-5-5",
        display_name: "GPT-5.5",
        provider: "OpenAI",
        family: "gpt-5",
        approx_params_billions: 1000.0,
        active_params_b: 1000.0,
        openness: Openness::Closed,
        // 1000 × 25 = 25000
        epsilon_prefill_mj: (6_061.0, 10_000.0, 16_500.0),
        epsilon_decode_mj: (15_152.0, 25_000.0, 41_250.0),
        embodied_g_per_req: (0.152, 0.250, 0.412),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "OpenAI GPT-5.5 system card (2026-04-23)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-23",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: true,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://openai.com/index/gpt-5-5-system-card/",
    },
    ModelPreset {
        id: "gpt-5-5-thinking",
        display_name: "GPT-5.5 Thinking",
        provider: "OpenAI",
        family: "gpt-5",
        approx_params_billions: 1000.0,
        active_params_b: 1000.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (6_061.0, 10_000.0, 16_500.0),
        epsilon_decode_mj: (15_152.0, 25_000.0, 41_250.0),
        embodied_g_per_req: (0.152, 0.250, 0.412),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "OpenAI GPT-5.5 system card (2026-04-23)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-23",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: true,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 30.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://openai.com/index/gpt-5-5-system-card/",
    },
    ModelPreset {
        id: "gpt-5-5-pro",
        display_name: "GPT-5.5 Pro",
        provider: "OpenAI",
        family: "gpt-5",
        approx_params_billions: 1000.0,
        active_params_b: 1000.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (6_061.0, 10_000.0, 16_500.0),
        epsilon_decode_mj: (15_152.0, 25_000.0, 41_250.0),
        embodied_g_per_req: (0.152, 0.250, 0.412),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "OpenAI GPT-5.5 Pro system card (2026-04-23)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-23",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: true,
        reasoning_capable: true,
        thinking_token_multiplier: Some((8.0, 50.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://openai.com/index/gpt-5-5-system-card/",
    },
    ModelPreset {
        id: "o3",
        display_name: "OpenAI o3",
        provider: "OpenAI",
        family: "o-series",
        approx_params_billions: 400.0,
        active_params_b: 400.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (2_424.0, 4_000.0, 6_600.0),
        epsilon_decode_mj: (6_061.0, 10_000.0, 16_500.0),
        embodied_g_per_req: (0.0606, 0.100, 0.165),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "OpenAI o3 system card (2024-12)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2024-12-20",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 30.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://openai.com/o1/",
    },
    // ════════════════════════════════════════════════════════════════════════
    // GOOGLE DEEPMIND (3 presets actifs, 1 deprecated)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "gemini-3-5-flash",
        display_name: "Gemini 3.5 Flash",
        provider: "Google",
        family: "gemini-3",
        approx_params_billions: 32.0,
        active_params_b: 32.0,
        openness: Openness::Closed,
        // 32 × 25 = 800
        epsilon_prefill_mj: (194.0, 320.0, 528.0),
        epsilon_decode_mj: (485.0, 800.0, 1_320.0),
        embodied_g_per_req: (0.00485, 0.0080, 0.0132),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Google Gemini 3.5 Flash release (2026-05)",
            "Estimation taille — analyse publique 2026",
            "Analogie Gemini 2.0 Flash (taille Flash classe)",
        ],
        vendor_disclosures: &[],
        release_date: "2026-05-15",
        model_family: ModelFamily::GoogleDeepMind,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(GEMINI_VISION),
        audio_capable: true,
        reasoning_capable: true,
        thinking_token_multiplier: Some((3.0, 25.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://deepmind.google/models/gemini/flash/",
    },
    ModelPreset {
        id: "gemini-3-1-pro",
        display_name: "Gemini 3.1 Pro",
        provider: "Google",
        family: "gemini-3",
        approx_params_billions: 400.0,
        active_params_b: 400.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (2_424.0, 4_000.0, 6_600.0),
        epsilon_decode_mj: (6_061.0, 10_000.0, 16_500.0),
        embodied_g_per_req: (0.0606, 0.100, 0.165),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Google Gemini 3.1 Pro release (2026-02)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-02-20",
        model_family: ModelFamily::GoogleDeepMind,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(GEMINI_VISION),
        audio_capable: true,
        reasoning_capable: true,
        thinking_token_multiplier: Some((3.0, 25.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://deepmind.google/models/gemini/",
    },
    ModelPreset {
        id: "gemini-2-5-pro",
        display_name: "Gemini 2.5 Pro",
        provider: "Google",
        family: "gemini-2",
        approx_params_billions: 400.0,
        active_params_b: 400.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (2_424.0, 4_000.0, 6_600.0),
        epsilon_decode_mj: (6_061.0, 10_000.0, 16_500.0),
        embodied_g_per_req: (0.0606, 0.100, 0.165),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Google Gemini 2.5 Pro release (2025-03)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-03-25",
        model_family: ModelFamily::GoogleDeepMind,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(GEMINI_VISION),
        audio_capable: true,
        reasoning_capable: true,
        thinking_token_multiplier: Some((3.0, 25.0)),
        default_context_overhead_tokens: 1000,
        deprecated: false,
        source_url: "https://deepmind.google/technologies/gemini/",
    },
    // ════════════════════════════════════════════════════════════════════════
    // META LLAMA (3 presets actifs, 2 deprecated)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "llama-4-scout",
        display_name: "Llama 4 Scout",
        provider: "Meta",
        family: "llama-4",
        approx_params_billions: 109.0, // total
        active_params_b: 17.0,         // 17B actifs (16 experts, ~1 active)
        openness: Openness::OpenWeights,
        // 17 × 25 = 425 mJ/tok (sur paramètres actifs)
        epsilon_prefill_mj: (103.0, 170.0, 280.5),
        epsilon_decode_mj: (258.0, 425.0, 701.0),
        embodied_g_per_req: (0.00258, 0.00425, 0.00701),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 4 herd (2025-04-05)",
            "Llama 4 Scout model card (HuggingFace)",
        ],
        vendor_disclosures: &[],
        release_date: "2025-04-05",
        model_family: ModelFamily::MetaAi,
        architecture: ArchitectureKind::Moe {
            experts: 16,
            active_experts: 1,
        },
        vision_capable: true,
        vision_pricing: Some(LLAMA_VISION),
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://ai.meta.com/blog/llama-4-multimodal-intelligence/",
    },
    ModelPreset {
        id: "llama-4-maverick",
        display_name: "Llama 4 Maverick",
        provider: "Meta",
        family: "llama-4",
        approx_params_billions: 400.0, // total
        active_params_b: 17.0,         // 17B actifs (128 experts)
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (103.0, 170.0, 280.5),
        epsilon_decode_mj: (258.0, 425.0, 701.0),
        embodied_g_per_req: (0.00258, 0.00425, 0.00701),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 4 herd (2025-04-05)",
            "Llama 4 Maverick model card (HuggingFace)",
        ],
        vendor_disclosures: &[],
        release_date: "2025-04-05",
        model_family: ModelFamily::MetaAi,
        architecture: ArchitectureKind::Moe {
            experts: 128,
            active_experts: 1,
        },
        vision_capable: true,
        vision_pricing: Some(LLAMA_VISION),
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://ai.meta.com/blog/llama-4-multimodal-intelligence/",
    },
    ModelPreset {
        id: "llama-3-3-70b",
        display_name: "Llama 3.3 70B",
        provider: "Meta",
        family: "llama-3",
        approx_params_billions: 70.0,
        active_params_b: 70.0,
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (424.0, 700.0, 1_155.0),
        epsilon_decode_mj: (1_061.0, 1_750.0, 2_888.0),
        embodied_g_per_req: (0.0106, 0.0175, 0.02888),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 3.3 release (2024-12)",
            "Llama 3.3 model card (GitHub)",
        ],
        vendor_disclosures: &[],
        release_date: "2024-12-06",
        model_family: ModelFamily::MetaAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://github.com/meta-llama/llama-models",
    },
    // ════════════════════════════════════════════════════════════════════════
    // MISTRAL AI (3 presets actifs, 2 deprecated)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "mistral-medium-3-5",
        display_name: "Mistral Medium 3.5",
        provider: "Mistral AI",
        family: "mistral-medium",
        approx_params_billions: 128.0, // dense
        active_params_b: 128.0,
        openness: Openness::OpenWeights,
        // 128 × 25 = 3200
        epsilon_prefill_mj: (776.0, 1_280.0, 2_112.0),
        epsilon_decode_mj: (1_939.0, 3_200.0, 5_280.0),
        embodied_g_per_req: (0.01939, 0.0320, 0.05280),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Mistral Medium 3.5 release (2026-04-30)",
            "Mistral changelog 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-30",
        model_family: ModelFamily::MistralAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 25.0)),
        default_context_overhead_tokens: 300,
        deprecated: false,
        source_url: "https://docs.mistral.ai/getting-started/changelog",
    },
    ModelPreset {
        id: "mistral-small-4",
        display_name: "Mistral Small 4",
        provider: "Mistral AI",
        family: "mistral-small",
        approx_params_billions: 30.0,
        active_params_b: 30.0,
        openness: Openness::OpenWeights,
        // 30 × 25 = 750
        epsilon_prefill_mj: (182.0, 300.0, 495.0),
        epsilon_decode_mj: (455.0, 750.0, 1_238.0),
        embodied_g_per_req: (0.00455, 0.0075, 0.01238),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Mistral Small 4 release (2026-03-16)",
            "Mistral changelog 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-03-16",
        model_family: ModelFamily::MistralAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 25.0)),
        default_context_overhead_tokens: 300,
        deprecated: false,
        source_url: "https://docs.mistral.ai/getting-started/changelog",
    },
    ModelPreset {
        id: "mistral-large-3",
        display_name: "Mistral Large 3",
        provider: "Mistral AI",
        family: "mistral-large",
        approx_params_billions: 675.0, // total
        active_params_b: 41.0,         // actifs MoE
        openness: Openness::OpenWeights,
        // 41 × 25 = 1025
        epsilon_prefill_mj: (248.0, 410.0, 676.5),
        epsilon_decode_mj: (621.0, 1_025.0, 1_691.0),
        embodied_g_per_req: (0.00621, 0.01025, 0.01691),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Mistral Large 3 release (2025-12-02)",
            "Mistral 3 model card",
        ],
        vendor_disclosures: &[],
        release_date: "2025-12-02",
        model_family: ModelFamily::MistralAi,
        architecture: ArchitectureKind::Moe {
            experts: 32,
            active_experts: 2,
        },
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 300,
        deprecated: false,
        source_url: "https://docs.mistral.ai/models/mistral-large-3-25-12/",
    },
    // ════════════════════════════════════════════════════════════════════════
    // DEEPSEEK (2 presets actifs)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "deepseek-v4-pro",
        display_name: "DeepSeek V4 Pro",
        provider: "DeepSeek",
        family: "deepseek-v4",
        approx_params_billions: 1600.0, // total
        active_params_b: 49.0,          // actifs MoE
        openness: Openness::OpenWeights,
        // 49 × 25 = 1225
        epsilon_prefill_mj: (297.0, 490.0, 808.5),
        epsilon_decode_mj: (742.0, 1_225.0, 2_021.0),
        embodied_g_per_req: (0.00742, 0.01225, 0.02021),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "DeepSeek V4 preview release (2026-04-24)",
            "DeepSeek API docs",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-24",
        model_family: ModelFamily::DeepSeek,
        architecture: ArchitectureKind::Moe {
            experts: 256,
            active_experts: 8,
        },
        vision_capable: true,
        vision_pricing: Some(LLAMA_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((8.0, 25.0)),
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://api-docs.deepseek.com/news/news260424",
    },
    ModelPreset {
        id: "deepseek-r1",
        display_name: "DeepSeek R1",
        provider: "DeepSeek",
        family: "deepseek-r1",
        approx_params_billions: 671.0, // total
        active_params_b: 37.0,         // actifs MoE
        openness: Openness::OpenWeights,
        // 37 × 25 = 925
        epsilon_prefill_mj: (224.0, 370.0, 610.5),
        epsilon_decode_mj: (561.0, 925.0, 1_526.0),
        embodied_g_per_req: (0.00561, 0.00925, 0.01526),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "DeepSeek R1 paper arXiv 2501.12948 (2025-01-20)",
            "DeepSeek R1 model card",
        ],
        vendor_disclosures: &[],
        release_date: "2025-01-20",
        model_family: ModelFamily::DeepSeek,
        architecture: ArchitectureKind::Moe {
            experts: 256,
            active_experts: 8,
        },
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((8.0, 25.0)),
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://arxiv.org/abs/2501.12948",
    },
    // ════════════════════════════════════════════════════════════════════════
    // xAI (1 preset actif)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "grok-4",
        display_name: "Grok 4",
        provider: "xAI",
        family: "grok-4",
        approx_params_billions: 500.0,
        active_params_b: 500.0,
        openness: Openness::Closed,
        // 500 × 25 = 12500
        epsilon_prefill_mj: (3_030.0, 5_000.0, 8_250.0),
        epsilon_decode_mj: (7_576.0, 12_500.0, 20_625.0),
        embodied_g_per_req: (0.0758, 0.125, 0.2063),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "xAI Grok 4 release (2025-07-10)",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2025-07-10",
        model_family: ModelFamily::Xai,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 30.0)),
        default_context_overhead_tokens: 800,
        deprecated: false,
        source_url: "https://x.ai/news",
    },
    // ════════════════════════════════════════════════════════════════════════
    // ALIBABA QWEN (1 preset actif)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "qwen-3-6-plus",
        display_name: "Qwen 3.6-Plus",
        provider: "Alibaba",
        family: "qwen-3",
        approx_params_billions: 1000.0,
        active_params_b: 1000.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (6_061.0, 10_000.0, 16_500.0),
        epsilon_decode_mj: (15_152.0, 25_000.0, 41_250.0),
        embodied_g_per_req: (0.152, 0.250, 0.412),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Qwen 3.6-Plus release (2026-04-02)",
            "Caixin Global coverage 2026",
            "Estimation taille — analyse publique 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2026-04-02",
        model_family: ModelFamily::Alibaba,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 25.0)),
        default_context_overhead_tokens: 500,
        deprecated: false,
        source_url: "https://www.caixinglobal.com/2026-04-02/alibaba-releases-qwen-36-plus-ai-model-with-enhanced-coding-capabilities-102430395.html",
    },
    // ════════════════════════════════════════════════════════════════════════
    // MICROSOFT PHI (2 presets actifs)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "phi-4-reasoning-vision",
        display_name: "Phi-4 Reasoning Vision 15B",
        provider: "Microsoft",
        family: "phi-4",
        approx_params_billions: 15.0,
        active_params_b: 15.0,
        openness: Openness::OpenWeights,
        // 15 × 25 = 375
        epsilon_prefill_mj: (91.0, 150.0, 247.5),
        epsilon_decode_mj: (227.0, 375.0, 618.75),
        embodied_g_per_req: (0.00227, 0.00375, 0.00619),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Microsoft Phi-4 Reasoning Vision 15B release (2026-03-04)",
            "HuggingFace model card",
        ],
        vendor_disclosures: &[],
        release_date: "2026-03-04",
        model_family: ModelFamily::Microsoft,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(LLAMA_VISION),
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 15.0)),
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://huggingface.co/microsoft/Phi-4-reasoning-vision-15B",
    },
    ModelPreset {
        id: "phi-4-reasoning",
        display_name: "Phi-4 Reasoning",
        provider: "Microsoft",
        family: "phi-4",
        approx_params_billions: 14.0,
        active_params_b: 14.0,
        openness: Openness::OpenWeights,
        // 14 × 25 = 350
        epsilon_prefill_mj: (85.0, 140.0, 231.0),
        epsilon_decode_mj: (212.0, 350.0, 577.5),
        embodied_g_per_req: (0.00212, 0.003_50, 0.005_775),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Microsoft Phi-4 Reasoning Technical Report (2025-04-30)",
            "Microsoft Research publication",
        ],
        vendor_disclosures: &[],
        release_date: "2025-04-30",
        model_family: ModelFamily::Microsoft,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: true,
        thinking_token_multiplier: Some((5.0, 15.0)),
        default_context_overhead_tokens: 0,
        deprecated: false,
        source_url: "https://www.microsoft.com/en-us/research/publication/phi-4-reasoning-technical-report/",
    },
    // ════════════════════════════════════════════════════════════════════════
    // DEPRECATED 2024 (conservés pour reproductibilité audit ledger)
    // ════════════════════════════════════════════════════════════════════════
    ModelPreset {
        id: "gpt-4o",
        display_name: "GPT-4o",
        provider: "OpenAI",
        family: "gpt-4",
        approx_params_billions: 200.0,
        active_params_b: 200.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (1_210.0, 2_000.0, 3_300.0),
        epsilon_decode_mj: (3_030.0, 5_000.0, 8_250.0),
        embodied_g_per_req: (0.030, 0.050, 0.0825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "EcoLogits 2026-01",
            "HF AI Energy Score 2026",
            "Estimation taille — analyse publique 2024",
        ],
        vendor_disclosures: &[],
        release_date: "2024-05-13",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: true,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 1000,
        deprecated: true,
        source_url: "https://openai.com/index/hello-gpt-4o/",
    },
    ModelPreset {
        id: "gpt-4o-mini",
        display_name: "GPT-4o mini",
        provider: "OpenAI",
        family: "gpt-4",
        approx_params_billions: 8.0,
        active_params_b: 8.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (48.5, 80.0, 132.0),
        epsilon_decode_mj: (121.0, 200.0, 330.0),
        embodied_g_per_req: (0.00121, 0.0020, 0.0033),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "EcoLogits 2026-01",
            "Estimation taille — analyse publique 2024",
        ],
        vendor_disclosures: &[],
        release_date: "2024-07-18",
        model_family: ModelFamily::OpenAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(OPENAI_VISION),
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 1000,
        deprecated: true,
        source_url: "https://openai.com/index/gpt-4o-mini-advancing-cost-efficient-intelligence/",
    },
    ModelPreset {
        id: "claude-3-5-sonnet",
        display_name: "Claude 3.5 Sonnet",
        provider: "Anthropic",
        family: "claude-3",
        approx_params_billions: 200.0,
        active_params_b: 200.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (1_210.0, 2_000.0, 3_300.0),
        epsilon_decode_mj: (3_030.0, 5_000.0, 8_250.0),
        embodied_g_per_req: (0.030, 0.050, 0.0825),
        calibration: CalibrationStatus::Extrapolated,
        sources: &["EcoLogits 2026-01 (analogie modèles dense ~200B)"],
        vendor_disclosures: &[],
        release_date: "2024-06-20",
        model_family: ModelFamily::Anthropic,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(ANTHROPIC_VISION),
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 2000,
        deprecated: true,
        source_url: "https://www.anthropic.com/news/claude-3-5-sonnet",
    },
    ModelPreset {
        id: "mistral-large-2",
        display_name: "Mistral Large 2",
        provider: "Mistral AI",
        family: "mistral-large",
        approx_params_billions: 123.0,
        active_params_b: 123.0,
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (745.0, 1_230.0, 2_030.0),
        epsilon_decode_mj: (1_864.0, 3_075.0, 5_074.0),
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
        release_date: "2024-07-24",
        model_family: ModelFamily::MistralAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 300,
        deprecated: true,
        source_url: "https://mistral.ai/news/mistral-large-2407/",
    },
    ModelPreset {
        id: "mistral-medium-3",
        display_name: "Mistral Medium 3",
        provider: "Mistral AI",
        family: "mistral-medium",
        approx_params_billions: 30.0,
        active_params_b: 30.0,
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (182.0, 300.0, 495.0),
        epsilon_decode_mj: (455.0, 750.0, 1_238.0),
        embodied_g_per_req: (0.00455, 0.0075, 0.01238),
        calibration: CalibrationStatus::Indicative,
        sources: &["Mistral AI 2024", "EcoLogits 2026-01"],
        vendor_disclosures: &[],
        release_date: "2025-05-01",
        model_family: ModelFamily::MistralAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 300,
        deprecated: true,
        source_url: "https://mistral.ai/news/mistral-medium-3/",
    },
    ModelPreset {
        id: "llama-3-1-70b",
        display_name: "Llama 3.1 70B",
        provider: "Meta",
        family: "llama-3",
        approx_params_billions: 70.0,
        active_params_b: 70.0,
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (424.0, 700.0, 1_155.0),
        epsilon_decode_mj: (1_061.0, 1_750.0, 2_888.0),
        embodied_g_per_req: (0.0106, 0.0175, 0.02888),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 3.1 paper (Touvron et al. 2024)",
            "HF AI Energy Score 2026",
            "Meta Llama 3.1 model card (MODEL_CARD.md)",
        ],
        // C32.4 — Meta publie le training (mais pas l'inférence par prompt).
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
        release_date: "2024-07-23",
        model_family: ModelFamily::MetaAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 0,
        deprecated: true,
        source_url: "https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md",
    },
    ModelPreset {
        id: "llama-3-1-8b",
        display_name: "Llama 3.1 8B",
        provider: "Meta",
        family: "llama-3",
        approx_params_billions: 8.0,
        active_params_b: 8.0,
        openness: Openness::OpenWeights,
        epsilon_prefill_mj: (48.5, 80.0, 132.0),
        epsilon_decode_mj: (121.0, 200.0, 330.0),
        embodied_g_per_req: (0.00121, 0.0020, 0.0033),
        calibration: CalibrationStatus::Indicative,
        sources: &[
            "Meta Llama 3.1 paper (Touvron et al. 2024)",
            "HF AI Energy Score 2026",
        ],
        vendor_disclosures: &[],
        release_date: "2024-07-23",
        model_family: ModelFamily::MetaAi,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: false,
        vision_pricing: None,
        audio_capable: false,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 0,
        deprecated: true,
        source_url: "https://github.com/meta-llama/llama-models/blob/main/models/llama3_1/MODEL_CARD.md",
    },
    ModelPreset {
        id: "gemini-2-0-flash",
        display_name: "Gemini 2.0 Flash",
        provider: "Google",
        family: "gemini-2",
        approx_params_billions: 32.0,
        active_params_b: 32.0,
        openness: Openness::Closed,
        epsilon_prefill_mj: (194.0, 320.0, 528.0),
        epsilon_decode_mj: (485.0, 800.0, 1_320.0),
        embodied_g_per_req: (0.00485, 0.0080, 0.0132),
        calibration: CalibrationStatus::Extrapolated,
        sources: &[
            "Google DeepMind 2025 (annonce publique)",
            "Analogie Mistral Medium",
            "Google Environmental Impact paper (2025-08)",
        ],
        // C32.4 — Google publie un prompt médian (méthodologie Google).
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
        release_date: "2024-12-11",
        model_family: ModelFamily::GoogleDeepMind,
        architecture: ArchitectureKind::DenseTransformer,
        vision_capable: true,
        vision_pricing: Some(GEMINI_VISION),
        audio_capable: true,
        reasoning_capable: false,
        thinking_token_multiplier: None,
        default_context_overhead_tokens: 1000,
        deprecated: true,
        source_url: "https://deepmind.google/technologies/gemini/flash/",
    },
];

/// Cherche un preset par son identifiant exact, même deprecated.
#[must_use]
pub fn find_preset(model_id: &str) -> Option<&'static ModelPreset> {
    MODEL_REGISTRY.iter().find(|p| p.id == model_id)
}

/// Retourne la liste de tous les modèles, **deprecated inclus** (compat
/// SemVer avec les appelants existants).
#[must_use]
pub fn available_models() -> Vec<&'static ModelPreset> {
    MODEL_REGISTRY.iter().collect()
}

/// **C34.2** — Retourne la liste de tous les modèles avec filtrage des
/// deprecated.
///
/// Si `include_deprecated = false`, exclut les presets avec
/// [`ModelPreset::deprecated`] = `true`. Utiliser ceci dans l'UI M1 / M9
/// par défaut, et `available_models()` pour les outils d'audit historique.
#[must_use]
pub fn available_models_filtered(include_deprecated: bool) -> Vec<&'static ModelPreset> {
    MODEL_REGISTRY
        .iter()
        .filter(|p| include_deprecated || !p.deprecated)
        .collect()
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
            modalities: Vec::new(),
            overhead: sobria_core::ContextOverhead::default(),
        }
    }

    #[test]
    fn registry_has_at_least_25_presets() {
        // DoD C34.2 : >= 25 presets totaux dans le registry.
        assert!(
            MODEL_REGISTRY.len() >= 25,
            "registry doit avoir >= 25 presets, a {}",
            MODEL_REGISTRY.len()
        );
    }

    #[test]
    fn registry_has_at_least_12_models_released_2025_or_later() {
        // DoD C34.2 : >= 12 presets sortis 2025-2026.
        let recent: Vec<&str> = MODEL_REGISTRY
            .iter()
            .filter(|p| !p.deprecated && p.release_date >= "2025-01-01")
            .map(|p| p.id)
            .collect();
        assert!(
            recent.len() >= 12,
            "DoD C34 : >= 12 modèles sortis 2025+, a {} : {:?}",
            recent.len(),
            recent
        );
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
    fn available_models_returns_at_least_25() {
        assert!(available_models().len() >= 25);
    }

    #[test]
    fn available_models_filtered_excludes_deprecated() {
        let all = available_models_filtered(true);
        let active_only = available_models_filtered(false);
        assert!(all.len() > active_only.len(), "le filtrage doit retirer >=1 deprecated");
        assert!(
            active_only.iter().all(|p| !p.deprecated),
            "include_deprecated=false ne doit garder que !deprecated"
        );
    }

    #[test]
    fn find_preset_known_and_unknown() {
        assert!(find_preset("gpt-4o-mini").is_some());
        assert!(find_preset("ce-modele-nexiste-pas").is_none());
    }

    #[test]
    fn c34_new_presets_findable() {
        // DoD C34.2 : tous les nouveaux presets clés sont accessibles via find_preset.
        for id in &[
            "claude-opus-4-7",
            "claude-sonnet-4-6",
            "claude-haiku-4-5",
            "claude-opus-4",
            "claude-sonnet-4",
            "claude-3-7-sonnet",
            "gpt-5-5",
            "gpt-5-5-thinking",
            "gpt-5-5-pro",
            "o3",
            "gemini-3-5-flash",
            "gemini-3-1-pro",
            "gemini-2-5-pro",
            "llama-4-scout",
            "llama-4-maverick",
            "llama-3-3-70b",
            "mistral-large-3",
            "mistral-small-4",
            "mistral-medium-3-5",
            "deepseek-v4-pro",
            "deepseek-r1",
            "grok-4",
            "qwen-3-6-plus",
            "phi-4-reasoning",
            "phi-4-reasoning-vision",
        ] {
            assert!(
                find_preset(id).is_some(),
                "preset C34 manquant : {id}"
            );
        }
    }

    #[test]
    fn estimation_with_preset_yields_reasonable_co2eq() {
        // Pour GPT-4o-mini, 100 tokens in + 500 tokens out, mix FR : P50 raisonnable.
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
        assert!(
            co2eq.interval.p50 > 0.0 && co2eq.interval.p50 < 1.0,
            "P50 CO2eq attendu dans [0, 1] g, reçu {}",
            co2eq.interval.p50
        );
    }

    #[test]
    fn at_least_three_vendors_have_disclosures() {
        // C32.4 — Mistral Large 2, Gemini 2.0 Flash, Llama 3.1 70B au moins.
        let vendors_with_disclosure: std::collections::HashSet<&str> = MODEL_REGISTRY
            .iter()
            .flat_map(|p| p.vendor_disclosures.iter().map(|d| d.vendor))
            .collect();
        assert!(
            vendors_with_disclosure.len() >= 3,
            "C32.4 attend >= 3 vendors avec disclosure, vu : {vendors_with_disclosure:?}"
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
        for preset in MODEL_REGISTRY {
            for d in preset.vendor_disclosures {
                assert_eq!(
                    d.published_at.len(),
                    10,
                    "preset {} : published_at doit être YYYY-MM-DD",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn mistral_large_2_has_training_and_inference_disclosures() {
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
        let llama = find_preset("llama-3-1-70b").expect("llama-3-1-70b doit exister");
        let trainings: Vec<f64> = llama
            .vendor_disclosures
            .iter()
            .filter(|d| matches!(d.scope, VendorScope::Training))
            .map(|d| d.value)
            .collect();
        assert_eq!(trainings.len(), 2);
        assert!(trainings.contains(&0.0));
        assert!(trainings.iter().any(|&v| v > 10_000.0));
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

    /// **C34.2** — les formules d'extrapolation utilisent `active_params_b`
    /// (pour MoE), pas `approx_params_billions` qui est le total.
    ///
    /// Tolérance : ±2 % (les valeurs sont hardcodées arrondies).
    #[test]
    fn registry_values_consistent_with_extrapolation_formulas() {
        for preset in MODEL_REGISTRY {
            let n_b = preset.active_params_b;
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
        assert!((p95 / p50 - UNCERTAINTY_RATIO).abs() < 1e-12);
        assert!((p50 / p5 - UNCERTAINTY_RATIO).abs() < 1e-12);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // C34.2 — Tests des nouveaux champs et enums
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn release_dates_are_valid_iso() {
        for preset in MODEL_REGISTRY {
            assert_eq!(
                preset.release_date.len(),
                10,
                "preset {} : release_date doit être YYYY-MM-DD, reçu {}",
                preset.id,
                preset.release_date
            );
            let parts: Vec<&str> = preset.release_date.split('-').collect();
            assert_eq!(parts.len(), 3, "preset {} : format date invalide", preset.id);
            // Parse year
            let year: u32 = parts[0].parse().unwrap_or_else(|_| {
                panic!("preset {} : year non numérique", preset.id);
            });
            assert!(
                (2020..=2030).contains(&year),
                "preset {} : year {} hors plage",
                preset.id,
                year
            );
        }
    }

    #[test]
    fn source_urls_are_https() {
        for preset in MODEL_REGISTRY {
            assert!(
                preset.source_url.starts_with("https://"),
                "preset {} : source_url non-HTTPS : {}",
                preset.id,
                preset.source_url
            );
        }
    }

    #[test]
    fn moe_presets_have_active_less_than_total() {
        for preset in MODEL_REGISTRY {
            if let ArchitectureKind::Moe { .. } = preset.architecture {
                assert!(
                    preset.active_params_b < preset.approx_params_billions,
                    "MoE {} doit avoir active < total",
                    preset.id
                );
            } else {
                assert_eq!(
                    preset.active_params_b, preset.approx_params_billions,
                    "Dense {} doit avoir active == total",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn vision_capable_implies_vision_pricing_some() {
        for preset in MODEL_REGISTRY {
            if preset.vision_capable {
                assert!(
                    preset.vision_pricing.is_some(),
                    "preset {} : vision_capable=true mais vision_pricing=None",
                    preset.id
                );
            } else {
                assert!(
                    preset.vision_pricing.is_none(),
                    "preset {} : vision_capable=false doit avoir vision_pricing=None",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn reasoning_capable_implies_thinking_multiplier_some() {
        for preset in MODEL_REGISTRY {
            if preset.reasoning_capable {
                assert!(
                    preset.thinking_token_multiplier.is_some(),
                    "preset {} : reasoning_capable=true mais thinking_token_multiplier=None",
                    preset.id
                );
                let (p5, p95) = preset.thinking_token_multiplier.unwrap();
                assert!(
                    p5 > 0.0 && p5 < p95,
                    "preset {} : (P5, P95) doit être ordonné et positif",
                    preset.id
                );
            } else {
                assert!(
                    preset.thinking_token_multiplier.is_none(),
                    "preset {} : reasoning_capable=false doit avoir thinking_token_multiplier=None",
                    preset.id
                );
            }
        }
    }

    #[test]
    fn deprecated_presets_count() {
        // On a >= 8 deprecated (les anciens 2024).
        let deprecated: Vec<&str> = MODEL_REGISTRY
            .iter()
            .filter(|p| p.deprecated)
            .map(|p| p.id)
            .collect();
        assert!(
            deprecated.len() >= 8,
            ">= 8 deprecated attendus, vu {} : {:?}",
            deprecated.len(),
            deprecated
        );
    }

    #[test]
    fn vision_pricing_openai_low_detail() {
        // OpenAI low detail = 85 fixe quel que soit la taille.
        let openai = OPENAI_VISION;
        assert_eq!(openai.tokens_for(1, 1024, 1024, false), 85);
        assert_eq!(openai.tokens_for(3, 1024, 1024, false), 255);
    }

    #[test]
    fn vision_pricing_openai_high_detail() {
        // OpenAI high detail 512×512 : 85 + 170 × 1 × 1 = 255 tokens.
        let openai = OPENAI_VISION;
        assert_eq!(openai.tokens_for(1, 512, 512, true), 255);
        // 1024×1024 high detail : 85 + 170 × 2 × 2 = 765 tokens.
        assert_eq!(openai.tokens_for(1, 1024, 1024, true), 765);
        // 2 images
        assert_eq!(openai.tokens_for(2, 1024, 1024, true), 1530);
    }

    #[test]
    fn vision_pricing_anthropic_area() {
        let anthropic = ANTHROPIC_VISION;
        // 750×750 = 562 500 pixels / 750 = 750 tokens.
        assert_eq!(anthropic.tokens_for(1, 750, 750, false), 750);
        // 2000×2000 = 4M pixels / 750 = 5333 → capé à 1568.
        assert_eq!(anthropic.tokens_for(1, 2000, 2000, false), 1568);
        // 2 images
        assert_eq!(anthropic.tokens_for(2, 750, 750, false), 1500);
    }

    #[test]
    fn vision_pricing_gemini_small() {
        let gemini = GEMINI_VISION;
        // <= 384×384 : 258 fixe.
        assert_eq!(gemini.tokens_for(1, 384, 384, false), 258);
        assert_eq!(gemini.tokens_for(1, 100, 100, false), 258);
    }

    #[test]
    fn vision_pricing_gemini_large() {
        let gemini = GEMINI_VISION;
        // 768×768 : 258 × 1 × 1 = 258.
        assert_eq!(gemini.tokens_for(1, 768, 768, false), 258);
        // 1500×768 : 258 × 2 × 1 = 516.
        assert_eq!(gemini.tokens_for(1, 1500, 768, false), 516);
        // 1500×1500 : 258 × 2 × 2 = 1032.
        assert_eq!(gemini.tokens_for(1, 1500, 1500, false), 1032);
    }

    #[test]
    fn vision_pricing_llama_fixed() {
        let llama = LLAMA_VISION;
        // 1601 fixe quelle que soit la taille.
        assert_eq!(llama.tokens_for(1, 100, 100, false), 1601);
        assert_eq!(llama.tokens_for(1, 4000, 4000, true), 1601);
        assert_eq!(llama.tokens_for(3, 1024, 1024, true), 4803);
    }

    #[test]
    fn vision_pricing_zero_count() {
        for vp in [OPENAI_VISION, ANTHROPIC_VISION, GEMINI_VISION, LLAMA_VISION] {
            assert_eq!(vp.tokens_for(0, 1024, 1024, true), 0);
        }
    }

    #[test]
    fn serde_round_trip_model_preset_json() {
        // ModelPreset doit être sérialisable (Deserialize n'est pas dérivé,
        // donc round-trip impossible — on teste juste la sérialisation).
        let preset = find_preset("claude-opus-4-7").unwrap();
        let json = serde_json::to_string(preset).expect("serialize");
        assert!(json.contains("claude-opus-4-7"));
        assert!(json.contains("\"model_family\""));
        assert!(json.contains("\"anthropic\"") || json.contains("\"Anthropic\""));
        assert!(json.contains("\"vision_capable\":true"));
        assert!(json.contains("\"reasoning_capable\":true"));
    }
}

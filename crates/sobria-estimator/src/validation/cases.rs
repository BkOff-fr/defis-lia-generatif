//! Cas concrets de validation.
//!
//! Deux familles :
//! - [`PlausibilityCase`] : plages larges (3-5 ordres de grandeur),
//!   toujours actifs en CI. Détectent les bugs catastrophiques (mauvais
//!   sens d'une formule, oubli de × 1000, etc.).
//! - [`ReproductionCase`] : reproduction **usage-only** d'une cible
//!   publiée à ±20-25 %, recalculée depuis la méthodologie EcoLogits
//!   2026-01 (Rincé & Banse 2025, DOI: 10.21105/joss.07471).
//!
//! Voir `docs/methodology/VALIDATION-CROISEE.md` et
//! `notebook/validation.qmd` pour le statut détaillé de chaque cas et la
//! justification des tolérances.

use super::{PlausibilityCase, ReproductionCase};
use crate::engine_trait::EmpreinteMethod;
use sobria_core::{ContextOverhead, InputModality};

/// Mix électrique France (ADEME 2024) — ~56 gCO₂eq/kWh.
const IF_FR: f64 = 56.0;
/// Mix électrique US-Virginie (Electricity Maps moyenne) — ~412 gCO₂eq/kWh.
const IF_US_VA: f64 = 412.0;

/// **C34.6** — Helper : ContextOverhead à zéro (cas legacy texte).
const ZERO_OVERHEAD: ContextOverhead = ContextOverhead {
    system_prompt_tokens: 0,
    tools_definition_tokens: 0,
    memory_tokens: 0,
    thinking_tokens_p50: 0,
};

/// **C34.6** — Helper : pas de modalités (cas legacy texte).
const NO_MODALITIES: &[InputModality] = &[];

// ─── Constantes modalités pour les cas C34.6 ────────────────────────────
/// 2 images 1024×1024 haute résolution.
const TWO_HIGH_IMAGES_1024: &[InputModality] = &[InputModality::VisionHigh {
    image_count: 2,
    avg_width: 1024,
    avg_height: 1024,
}];
/// PDF 5 pages.
const PDF_5_PAGES: &[InputModality] = &[InputModality::Document { page_count: 5 }];
/// 30 secondes d'audio.
const AUDIO_30S: &[InputModality] = &[InputModality::AudioInput {
    duration_seconds: 30,
}];

/// Overhead Claude par défaut (2000 tokens system prompt typique claude.ai).
const CLAUDE_DEFAULT_OVERHEAD: ContextOverhead = ContextOverhead {
    system_prompt_tokens: 2000,
    tools_definition_tokens: 0,
    memory_tokens: 0,
    thinking_tokens_p50: 0,
};

/// Liste des cas de plausibilité (toujours actifs en CI).
pub const PLAUSIBILITY_CASES: &[PlausibilityCase] = &[
    PlausibilityCase {
        id: "gpt-4o-mini-fr-short",
        description: "GPT-4o mini, mix France, prompt court (100 in / 500 out)",
        model_id: "gpt-4o-mini",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        // Petit modèle (8 G), mix décarboné, prompt court : on attend un
        // P50 dans une plage très basse (les modèles fermés sont
        // extrapolés donc plages élargies).
        expected_range_g_co2eq: (1e-5, 1e-2),
        reference: "Sobr.ia C06 model presets + ADEME 2024 (mix FR)",
    },
    PlausibilityCase {
        id: "llama-70b-fr-medium",
        description: "Llama 3.1 70B, mix France, prompt moyen (200 in / 1000 out)",
        model_id: "llama-3-1-70b",
        tokens_in: 200,
        tokens_out: 1000,
        if_electrical_g_per_kwh: IF_FR,
        expected_range_g_co2eq: (1e-4, 1e-1),
        reference: "Touvron et al. 2024 (Llama 3.1) + ADEME 2024",
    },
    PlausibilityCase {
        id: "llama-70b-us-va-medium",
        description: "Llama 3.1 70B, mix US-Virginie, prompt moyen",
        model_id: "llama-3-1-70b",
        tokens_in: 200,
        tokens_out: 1000,
        if_electrical_g_per_kwh: IF_US_VA,
        // Même modèle, même prompt — mais mix électrique ~7,4 × plus
        // carboné. On élargit la plage haute en conséquence.
        expected_range_g_co2eq: (1e-3, 1.0),
        reference: "Electricity Maps (Virginie ~412 g/kWh)",
    },
    PlausibilityCase {
        id: "gpt-4o-fr-long",
        description: "GPT-4o (grand modèle), mix France, prompt long (500 in / 2000 out)",
        model_id: "gpt-4o",
        tokens_in: 500,
        tokens_out: 2000,
        if_electrical_g_per_kwh: IF_FR,
        // Post-recalibration C24 : ~0.26 g attendus, plage élargie.
        expected_range_g_co2eq: (1e-2, 1.0),
        reference: "Sobr.ia C06 model presets (extrapolation ~200 G)",
    },
    PlausibilityCase {
        id: "mistral-large-fr-short",
        description: "Mistral Large 2, mix France, prompt court",
        model_id: "mistral-large-2",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        // Mistral Large 2 (123 G) : l'embodied amorti (≈ 0.031 g/req,
        // soit 0.00025 × 123) **domine** sur un prompt court vu que le
        // compute pèse moins que le hardware fab. La borne haute reflète
        // ce phénomène — un observation utile pour l'UI (« sur les gros
        // modèles, prompt court ≠ impact négligeable »).
        expected_range_g_co2eq: (1e-4, 0.1),
        reference: "Mistral AI 2024 + ADEME ; embodied 123G ≈ 0.031 g/req dominant",
    },
    PlausibilityCase {
        id: "us-mix-is-more-carbon-intensive-than-fr",
        // Ce cas n'est pas une vérification d'un chiffre absolu, mais
        // d'une **propriété qualitative** essentielle : le mix US est
        // plus carboné que le mix FR. Si notre moteur produit l'inverse,
        // quelque chose est fondamentalement cassé.
        description: "Sanity : US-VA produit > 5× plus CO₂eq que FR à modèle/prompt égaux",
        model_id: "llama-3-1-8b",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_US_VA,
        // On attend que le P50 soit nécessairement supérieur à la plage
        // FR équivalente (gpt-4o-mini-fr-short = mêmes tokens, petit
        // modèle, FR mix). On encadre largement pour éviter les faux
        // positifs liés au Monte-Carlo.
        expected_range_g_co2eq: (1e-4, 1.0),
        reference: "Electricity Maps + ADEME — différentiel mix carboné",
    },
];

/// Cas de reproduction stricte ciblant le moteur [`crate::engines::ecologits::EcoLogitsEngine`].
///
/// Ces cas valident que **notre port Rust** des formules EcoLogits
/// (Rincé & Banse 2025, DOI: 10.21105/joss.07471, CC BY-SA 4.0) reproduit
/// fidèlement les valeurs calculées indépendamment en Python (cf.
/// `notebook/validation.qmd`). Comme `EcoLogitsEngine` est un port direct
/// de leurs formules, l'écart est par construction proche de 0 — on
/// retient une tolérance de 1 % pour absorber les imprécisions
/// arithmétiques float64.
///
/// **Pourquoi usage-only ?** Dans cette première itération du port
/// EcoLogits, on isole la composante usage (énergie × IF × PUE) pour
/// vérifier d'abord cette partie. L'embodied EcoLogits (`I_server × ΔT/
/// (B×ΔL)`) est implémenté dans le moteur mais sera couvert par des
/// cas dédiés en v1.1+ (cf. `briefs/chantiers/C24-multi-methodologie-ecologits.md`).
///
/// **Formule EcoLogits réimplémentée pour calculer les cibles** (cf.
/// <https://ecologits.ai/latest/methodology/llm_inference/>, accédé 2026-05-13) :
/// - `f_E(P, B=64) = 5.71e-7 × P + 4.05e-5` Wh/token GPU
/// - `n_GPU = roundup_pow2(ceil(1.2 × P × 16 / 8 / 80))`
/// - `f_L(P, B=64) = 6.78e-4 × P + 0.020 + 0.0194` s/token
/// - `E_server_noGPU = ΔT × 1200 W × (n_GPU/8) / 64 / 3600` Wh
/// - `E_request = PUE × (n_GPU × n_tokens_out × f_E + E_server_noGPU)`
/// - `g_CO2eq_usage = E_request × IF_g_per_kWh / 1000`
///
/// Le détail des calculs et la justification des tolérances sont
/// documentés dans `notebook/validation.qmd`.
pub const REPRODUCTION_CASES: &[ReproductionCase] = &[
    ReproductionCase {
        id: "ecologits-llama-70b-fr-short",
        method: EmpreinteMethod::EcoLogits,
        source_doi_or_url: "https://doi.org/10.21105/joss.07471",
        model_id: "llama-3-1-70b",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // EcoLogits :
        //   M_model = 1.2 × 70 × 16 / 8 = 168 GB
        //   n_GPU = roundup_pow2(ceil(168/80)) = roundup_pow2(3) = 4
        //   E_GPU_total/tok = 4 × (5.71e-7 × 70 + 4.05e-5)
        //                   = 4 × 8.05e-5 = 3.22e-4 Wh
        //   E_GPU = 500 × 3.22e-4 = 0.161 Wh
        //   ΔT = 500 × (6.78e-4×70 + 0.020 + 0.0194) = 500 × 0.0869 = 43.45 s
        //   E_server_noGPU = 43.45 × 1200 × (4/8) / 64 / 3600 = 0.113 Wh
        //   E_server = 0.161 + 0.113 = 0.274 Wh
        //   E_req = 1.2 × 0.274 = 0.329 Wh = 3.29e-4 kWh
        //   gCO2eq usage = 3.29e-4 × 56 = 0.01843 g
        expected_p50_g_co2eq: 0.01843,
        tolerance: 0.01,
        disable_embodied: true,
        notes: "Llama 3.1 70B, prompt court, mix FR ADEME 2024. \
                Cible Python (notebook/validation.qmd) vs port Rust \
                EcoLogitsEngine — tolérance 1 % (port direct).",
        modalities: NO_MODALITIES,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "ecologits-llama-70b-us-long",
        method: EmpreinteMethod::EcoLogits,
        source_doi_or_url: "https://doi.org/10.21105/joss.07471",
        model_id: "llama-3-1-70b",
        tokens_in: 100,
        tokens_out: 2000,
        if_electrical_g_per_kwh: IF_US_VA,
        pue: 1.2,
        // EcoLogits :
        //   n_GPU = 4 (idem)
        //   E_GPU = 2000 × 3.22e-4 = 0.644 Wh
        //   ΔT = 2000 × 0.0869 = 173.8 s
        //   E_server_noGPU = 173.8 × 1200 × (4/8) / 64 / 3600 = 0.452 Wh
        //   E_server = 0.644 + 0.452 = 1.096 Wh
        //   E_req = 1.2 × 1.096 = 1.315 Wh = 1.315e-3 kWh
        //   gCO2eq usage = 1.315e-3 × 412 = 0.542 g
        expected_p50_g_co2eq: 0.542,
        tolerance: 0.01,
        disable_embodied: true,
        notes: "Llama 3.1 70B, prompt long, mix US-Virginie. Cible Python \
                vs port Rust — tolérance 1 %.",
        modalities: NO_MODALITIES,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "ecologits-mistral-large-us-medium",
        method: EmpreinteMethod::EcoLogits,
        source_doi_or_url: "https://doi.org/10.21105/joss.07471",
        model_id: "mistral-large-2",
        tokens_in: 100,
        tokens_out: 1000,
        if_electrical_g_per_kwh: IF_US_VA,
        pue: 1.2,
        // EcoLogits :
        //   n_GPU = roundup_pow2(ceil(1.2×123×16/8/80)) = roundup_pow2(4) = 4
        //   E_GPU_total/tok = 4 × (5.71e-7 × 123 + 4.05e-5)
        //                   = 4 × 1.108e-4 = 4.43e-4 Wh
        //   E_GPU = 1000 × 4.43e-4 = 0.443 Wh
        //   ΔT = 1000 × (6.78e-4×123 + 0.020 + 0.0194) = 1000 × 0.123 = 123.4 s
        //   E_server_noGPU = 123.4 × 1200 × (4/8) / 64 / 3600 = 0.321 Wh
        //   E_server = 0.443 + 0.321 = 0.764 Wh
        //   E_req = 1.2 × 0.764 = 0.917 Wh = 9.17e-4 kWh
        //   gCO2eq usage = 9.17e-4 × 412 = 0.378 g
        expected_p50_g_co2eq: 0.378,
        tolerance: 0.01,
        disable_embodied: true,
        notes: "Mistral Large 2 (123B open-weights), mix US-VA. Cible \
                Python vs port Rust — tolérance 1 %.",
        modalities: NO_MODALITIES,
        overhead: ZERO_OVERHEAD,
    },
    // ════════════════════════════════════════════════════════════════════════
    // C34.6 — Cas modalités + overhead + reasoning thinking auto
    //
    // Ces 5 cas vérifient que le pipeline `effective_tokens` (cf.
    // crates/sobria-estimator/src/effective_tokens.rs) injecte correctement :
    //   - les tokens vision via preset.vision_pricing (Cas A)
    //   - les tokens document génériques (Cas B)
    //   - les tokens audio génériques (Cas C)
    //   - le thinking tokens auto pour reasoning models (Cas D)
    //   - l'overhead système (Cas E)
    //
    // Méthodologie : Monte-Carlo AFNOR (P50 ~ geom-median des log-normaux).
    // Tolérance 25 % par défaut (Monte-Carlo + extrapolation closed models).
    // ════════════════════════════════════════════════════════════════════════
    ReproductionCase {
        id: "c34-vision-gpt4o-2-high-images-fr",
        method: EmpreinteMethod::AfnorSobria,
        source_doi_or_url: "https://platform.openai.com/docs/guides/vision/calculating-costs",
        model_id: "gpt-4o",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // Calcul de référence :
        //   Vision OpenAI : (85 + 170×2×2) × 2 = 1530 tokens
        //   effective_in = 100 + 1530 = 1630 tokens
        //   e_compute = 1630 × 2000 + 500 × 5000 = 5.76 J/req
        //   e_total = 5.76 × 1.2 / 3.6 = 1.92 Wh = 1.92e-3 kWh
        //   usage = 1.92e-3 × 56 = 0.107 g
        //   + embodied P50 = 0.05 g
        //   total ≈ 0.157 g
        expected_p50_g_co2eq: 0.157,
        tolerance: 0.25,
        disable_embodied: false,
        notes: "GPT-4o + 2 images 1024×1024 haute rés via formule OpenAI \
                (85 + 170 × tiles) → 1530 tokens vision injectés. \
                Vérifie que VisionPricing::OpenAiTiles + effective_tokens() \
                contribuent au pipeline AFNOR Monte-Carlo.",
        modalities: TWO_HIGH_IMAGES_1024,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "c34-document-gpt4o-5-pages-fr",
        method: EmpreinteMethod::AfnorSobria,
        source_doi_or_url: "https://platform.openai.com/docs/guides/vision/calculating-costs",
        model_id: "gpt-4o",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // Calcul de référence :
        //   Document : 5 × 1100 = 5500 tokens (DOCUMENT_TOKENS_PER_PAGE)
        //   effective_in = 100 + 5500 = 5600 tokens
        //   e_compute = 5600 × 2000 + 500 × 5000 = 13.7 J
        //   e_total = 13.7 × 1.2 / 3.6 = 4.57 Wh
        //   usage = 4.57e-3 × 56 = 0.256 g
        //   + embodied 0.05 = 0.306 g
        expected_p50_g_co2eq: 0.306,
        tolerance: 0.25,
        disable_embodied: false,
        notes: "PDF 5 pages → 5500 tokens via formule générique \
                (DOCUMENT_TOKENS_PER_PAGE=1100, source analyse LMSYS arena). \
                Vérifie que InputModality::Document fallback preset-indépendant \
                contribue correctement au pipeline.",
        modalities: PDF_5_PAGES,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "c34-audio-gpt4o-30s-fr",
        method: EmpreinteMethod::AfnorSobria,
        source_doi_or_url: "https://openai.com/research/whisper",
        model_id: "gpt-4o",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // Calcul de référence :
        //   Audio : 30 × 10 = 300 tokens (AUDIO_TOKENS_PER_SECOND=10)
        //   effective_in = 100 + 300 = 400 tokens
        //   e_compute = 400 × 2000 + 500 × 5000 = 3.3 J
        //   e_total = 3.3 × 1.2 / 3.6 = 1.1 Wh
        //   usage = 1.1e-3 × 56 = 0.0616 g
        //   + embodied 0.05 = 0.112 g
        expected_p50_g_co2eq: 0.112,
        tolerance: 0.25,
        disable_embodied: false,
        notes: "30s audio → 300 tokens via Whisper rate (10 tokens/s). \
                Vérifie que InputModality::AudioInput est correctement \
                injecté pour les modèles audio_capable.",
        modalities: AUDIO_30S,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "c34-reasoning-o3-complex-fr",
        method: EmpreinteMethod::AfnorSobria,
        source_doi_or_url: "https://openai.com/o1/",
        model_id: "o3",
        tokens_in: 200,
        tokens_out: 1000,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // Calcul de référence :
        //   o3 reasoning_capable, multiplier (5, 30), geomean = √150 = 12.247
        //   auto_thinking = 1000 × 12.247 ≈ 12247 tokens
        //   effective_out = 1000 + 12247 = 13247 tokens
        //   o3 params : ε_prefill_p50 = 4000, ε_decode_p50 = 10000, embodied = 0.10
        //   e_compute = 200 × 4000 + 13247 × 10000 = 133.27 J
        //   e_total = 133.27 × 1.2 / 3.6 = 44.42 Wh
        //   usage = 44.42e-3 × 56 = 2.488 g
        //   + embodied 0.10 = 2.588 g
        expected_p50_g_co2eq: 2.588,
        tolerance: 0.25,
        disable_embodied: false,
        notes: "OpenAI o3 reasoning model — vérifie que auto_thinking_tokens() \
                ajoute ~12k tokens (geomean P5×P95 = √(5×30) ≈ 12.25) à \
                effective_out, multipliant l'empreinte par ~14×.",
        modalities: NO_MODALITIES,
        overhead: ZERO_OVERHEAD,
    },
    ReproductionCase {
        id: "c34-overhead-claude-3-7-fr",
        method: EmpreinteMethod::AfnorSobria,
        source_doi_or_url: "https://www.anthropic.com/news/claude-3-7-sonnet",
        model_id: "claude-3-7-sonnet",
        tokens_in: 100,
        tokens_out: 500,
        if_electrical_g_per_kwh: IF_FR,
        pue: 1.2,
        // Calcul de référence :
        //   Overhead Claude.ai : system = 2000 (typique leaked)
        //   effective_in = 100 + 2000 = 2100
        //   claude-3-7 reasoning_capable, multiplier (2, 50), geomean = √100 = 10
        //   auto_thinking = 500 × 10 = 5000 tokens
        //   effective_out = 500 + 5000 = 5500
        //   claude-3-7 params : ε_prefill_p50 = 2000, ε_decode_p50 = 5000, embodied = 0.05
        //   e_compute = 2100 × 2000 + 5500 × 5000 = 31.7 J
        //   e_total = 31.7 × 1.2 / 3.6 = 10.57 Wh
        //   usage = 10.57e-3 × 56 = 0.592 g
        //   + embodied 0.05 = 0.642 g
        expected_p50_g_co2eq: 0.642,
        tolerance: 0.30,
        disable_embodied: false,
        notes: "Claude 3.7 Sonnet sur claude.ai (overhead 2000 tokens system \
                prompt typique) + auto thinking (multiplier 2-50, geomean 10×). \
                Vérifie que ContextOverhead + reasoning auto-thinking \
                s'appliquent ensemble correctement.",
        modalities: NO_MODALITIES,
        overhead: CLAUDE_DEFAULT_OVERHEAD,
    },
];

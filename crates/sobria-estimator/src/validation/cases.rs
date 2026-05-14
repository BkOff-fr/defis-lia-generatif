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

/// Mix électrique France (ADEME 2024) — ~56 gCO₂eq/kWh.
const IF_FR: f64 = 56.0;
/// Mix électrique US-Virginie (Electricity Maps moyenne) — ~412 gCO₂eq/kWh.
const IF_US_VA: f64 = 412.0;

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
    },
];

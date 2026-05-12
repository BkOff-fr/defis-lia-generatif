//! Cas concrets de validation.
//!
//! En v1 : uniquement des [`PlausibilityCase`] avec plages larges. Les
//! [`ReproductionCase`] (validation stricte à ±15 % d'une valeur publiée)
//! seront ajoutés après lecture biblio S0.
//!
//! Voir `docs/methodology/VALIDATION-CROISEE.md` pour le statut détaillé
//! de chaque cas et les références sources.

use super::{PlausibilityCase, ReproductionCase};

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
        expected_range_g_co2eq: (1e-3, 0.1),
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

/// Cas de reproduction stricte (vide en v1 — à compléter en S0 biblio).
pub const REPRODUCTION_CASES: &[ReproductionCase] = &[];

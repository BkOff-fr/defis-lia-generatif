//! **C34.3** — Bridge entre [`sobria_core::InputModality`] +
//! [`sobria_core::ContextOverhead`] et [`crate::model_presets::VisionPricing`].
//!
//! Calcule les **effective tokens** (input et output) à utiliser dans les
//! formules d'énergie des engines. Sépare physiquement le côté **input**
//! (prefill : user tokens + system + tools + memory + modalités) et le côté
//! **output** (decode : output tokens + thinking pour reasoning models).
//!
//! ## Disclaimer
//!
//! L'overhead système (`system_prompt_tokens`, etc.) est une **estimation
//! ± 50 %** basée sur leaks publics et reverse-engineering. Le thinking
//! multiplier `(P5, P95)` provient des system cards vendor. Voir
//! [`crate::model_presets::ModelPreset::default_context_overhead_tokens`]
//! et [`crate::model_presets::ModelPreset::thinking_token_multiplier`].

use sobria_core::{ContextOverhead, EstimationRequest, InputModality};

use crate::model_presets::{ModelPreset, VisionPricing};

/// Calcule les tokens de la modalité, avec préférence au `vision_pricing`
/// du preset s'il est disponible. Fallback générique pour Text / Document /
/// AudioInput (preset-indépendant).
#[must_use]
pub fn modality_tokens(modality: &InputModality, preset_opt: Option<&ModelPreset>) -> u32 {
    match modality {
        InputModality::VisionLow { image_count } => preset_opt
            .and_then(|p| p.vision_pricing.as_ref())
            .map_or_else(
                || modality.default_token_count(),
                // Low detail : 512×512 typique, high_detail=false
                |vp: &VisionPricing| vp.tokens_for(*image_count, 512, 512, false),
            ),
        InputModality::VisionHigh {
            image_count,
            avg_width,
            avg_height,
        } => preset_opt
            .and_then(|p| p.vision_pricing.as_ref())
            .map_or_else(
                || modality.default_token_count(),
                |vp| vp.tokens_for(*image_count, *avg_width, *avg_height, true),
            ),
        // Text / Document / AudioInput : formule preset-indépendante (déjà
        // dans InputModality::default_token_count).
        _ => modality.default_token_count(),
    }
}

/// Somme des tokens de toutes les modalités d'une requête.
#[must_use]
pub fn modalities_total(modalities: &[InputModality], preset_opt: Option<&ModelPreset>) -> u32 {
    modalities
        .iter()
        .map(|m| modality_tokens(m, preset_opt))
        .fold(0u32, u32::saturating_add)
}

/// Calcule la valeur P50 du thinking_tokens à ajouter automatiquement quand
/// le modèle est reasoning et que l'utilisateur n'a pas fourni de valeur
/// explicite (`overhead.thinking_tokens_p50 == 0`).
///
/// Formule : `output_tokens × geometric_mean(P5, P95)`.
/// Géométrique car le ratio est log-normal (cf. system cards o3, R1, V4).
#[must_use]
pub fn auto_thinking_tokens(output_tokens: u32, preset_opt: Option<&ModelPreset>) -> u32 {
    let Some(preset) = preset_opt else {
        return 0;
    };
    if !preset.reasoning_capable {
        return 0;
    }
    let Some((p5, p95)) = preset.thinking_token_multiplier else {
        return 0;
    };
    if p5 <= 0.0 || p95 <= 0.0 || p5 > p95 {
        return 0;
    }
    let geomean = (p5 * p95).sqrt();
    let out = f64::from(output_tokens);
    // Saturating cast — si débordement, on plafonne à u32::MAX
    let result = out * geomean;
    if !result.is_finite() || result < 0.0 {
        0
    } else if result > f64::from(u32::MAX) {
        u32::MAX
    } else {
        result as u32
    }
}

/// Couple `(effective_input_tokens, effective_output_tokens)` calculé à
/// partir de la requête et du preset (si trouvable).
///
/// - `effective_in = tokens_in + overhead.total_input() + Σ modalities`
/// - `effective_out = tokens_out_estimated + overhead.total_output()
///                    + auto_thinking_tokens(output, preset)`
///
/// `auto_thinking_tokens` n'est ajouté que si `overhead.thinking_tokens_p50
/// == 0` (sinon l'utilisateur a fourni sa propre valeur en mode Expert).
#[must_use]
pub fn effective_tokens(
    request: &EstimationRequest,
    preset_opt: Option<&ModelPreset>,
) -> (u32, u32) {
    let modalities_tok = modalities_total(&request.modalities, preset_opt);
    let effective_in = request
        .tokens_in
        .saturating_add(request.overhead.total_input())
        .saturating_add(modalities_tok);

    let user_thinking = request.overhead.thinking_tokens_p50;
    let auto_thinking = if user_thinking == 0 {
        auto_thinking_tokens(request.tokens_out_estimated, preset_opt)
    } else {
        0
    };
    let effective_out = request
        .tokens_out_estimated
        .saturating_add(user_thinking)
        .saturating_add(auto_thinking);

    (effective_in, effective_out)
}

/// Helper : recalcule un `ContextOverhead` à partir des valeurs par défaut
/// du preset (système prompt, thinking auto). Utile pour l'UI M1 qui
/// pré-remplit les champs au choix du modèle.
#[must_use]
pub fn overhead_from_preset(preset: &ModelPreset, output_tokens: u32) -> ContextOverhead {
    ContextOverhead {
        system_prompt_tokens: preset.default_context_overhead_tokens,
        tools_definition_tokens: 0,
        memory_tokens: 0,
        thinking_tokens_p50: if preset.reasoning_capable {
            auto_thinking_tokens(output_tokens, Some(preset))
        } else {
            0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_presets::find_preset;
    use chrono::Utc;
    use sobria_core::EstimationRequest;

    fn base_request(model_id: &str, t_in: u32, t_out: u32) -> EstimationRequest {
        EstimationRequest {
            model_id: model_id.into(),
            tokens_in: t_in,
            tokens_out_estimated: t_out,
            datacenter_id: None,
            timestamp: Utc::now(),
            modalities: Vec::new(),
            overhead: ContextOverhead::default(),
        }
    }

    #[test]
    fn baseline_no_overhead_no_modalities_passes_through() {
        let req = base_request("gpt-4o", 100, 500);
        let preset = find_preset("gpt-4o");
        let (eff_in, eff_out) = effective_tokens(&req, preset);
        assert_eq!(eff_in, 100);
        assert_eq!(eff_out, 500);
    }

    #[test]
    fn overhead_total_input_added_to_in() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.overhead.system_prompt_tokens = 1000;
        req.overhead.tools_definition_tokens = 300;
        req.overhead.memory_tokens = 200;
        let preset = find_preset("gpt-4o");
        let (eff_in, eff_out) = effective_tokens(&req, preset);
        assert_eq!(eff_in, 100 + 1500);
        assert_eq!(eff_out, 500);
    }

    #[test]
    fn vision_high_uses_openai_pricing_for_gpt4o() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.modalities = vec![InputModality::VisionHigh {
            image_count: 1,
            avg_width: 1024,
            avg_height: 1024,
        }];
        let preset = find_preset("gpt-4o");
        let (eff_in, _) = effective_tokens(&req, preset);
        // OpenAI 1024×1024 high : 85 + 170 × 2 × 2 = 765
        assert_eq!(eff_in, 100 + 765);
    }

    #[test]
    fn vision_low_uses_openai_pricing_for_gpt4o() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.modalities = vec![InputModality::VisionLow { image_count: 2 }];
        let preset = find_preset("gpt-4o");
        let (eff_in, _) = effective_tokens(&req, preset);
        // OpenAI low : 85 fixe × 2 = 170
        assert_eq!(eff_in, 100 + 170);
    }

    #[test]
    fn vision_falls_back_to_generic_when_no_preset() {
        let mut req = base_request("unknown-model", 100, 500);
        req.modalities = vec![InputModality::VisionLow { image_count: 1 }];
        let (eff_in, _) = effective_tokens(&req, None);
        assert_eq!(eff_in, 100 + 300); // VISION_LOW_TOKENS_PER_IMAGE_FALLBACK
    }

    #[test]
    fn auto_thinking_added_for_reasoning_models() {
        // o3 : reasoning, multiplier (5.0, 30.0) → geomean = sqrt(150) ≈ 12.247
        let req = base_request("o3", 100, 1000);
        let preset = find_preset("o3");
        let (_, eff_out) = effective_tokens(&req, preset);
        // attendu : 1000 + 1000 × ≈12.247 = 13247
        assert!((13_200..=13_300).contains(&eff_out), "eff_out = {eff_out}");
    }

    #[test]
    fn auto_thinking_skipped_when_user_provided() {
        let mut req = base_request("o3", 100, 1000);
        req.overhead.thinking_tokens_p50 = 5000; // user explicit
        let preset = find_preset("o3");
        let (_, eff_out) = effective_tokens(&req, preset);
        // No auto-add; just user value
        assert_eq!(eff_out, 1000 + 5000);
    }

    #[test]
    fn no_thinking_for_non_reasoning_models() {
        let req = base_request("gpt-4o", 100, 1000);
        let preset = find_preset("gpt-4o");
        let (_, eff_out) = effective_tokens(&req, preset);
        assert_eq!(eff_out, 1000); // pas de reasoning, pas de thinking auto
    }

    #[test]
    fn document_5_pages_uses_generic_formula() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.modalities = vec![InputModality::Document { page_count: 5 }];
        let preset = find_preset("gpt-4o");
        let (eff_in, _) = effective_tokens(&req, preset);
        assert_eq!(eff_in, 100 + 5500); // 1100 × 5
    }

    #[test]
    fn audio_30s_uses_generic_formula() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.modalities = vec![InputModality::AudioInput {
            duration_seconds: 30,
        }];
        let preset = find_preset("gpt-4o");
        let (eff_in, _) = effective_tokens(&req, preset);
        assert_eq!(eff_in, 100 + 300); // 10 × 30
    }

    #[test]
    fn multiple_modalities_sum() {
        let mut req = base_request("gpt-4o", 100, 500);
        req.modalities = vec![
            InputModality::VisionLow { image_count: 1 },
            InputModality::Document { page_count: 2 },
            InputModality::AudioInput {
                duration_seconds: 10,
            },
        ];
        let preset = find_preset("gpt-4o");
        let (eff_in, _) = effective_tokens(&req, preset);
        // 100 (user) + 85 (vision low OpenAI) + 2200 (doc 2×1100) + 100 (audio 10×10)
        assert_eq!(eff_in, 100 + 85 + 2200 + 100);
    }

    #[test]
    fn overhead_from_preset_fills_system_prompt() {
        let preset = find_preset("claude-opus-4-7").unwrap();
        let oh = overhead_from_preset(preset, 500);
        assert_eq!(oh.system_prompt_tokens, 2000); // Claude default
        // claude-opus-4-7 is reasoning_capable, multiplier (2, 50) → geomean = 10
        // 500 × 10 = 5000
        assert!((4_900..=5_100).contains(&oh.thinking_tokens_p50));
    }

    #[test]
    fn overhead_from_preset_zero_thinking_for_non_reasoning() {
        let preset = find_preset("gpt-4o").unwrap();
        let oh = overhead_from_preset(preset, 500);
        assert_eq!(oh.system_prompt_tokens, 1000); // GPT default
        assert_eq!(oh.thinking_tokens_p50, 0);
    }
}

//! **C34.3** — Modalités d'INPUT d'un prompt.
//!
//! Permet de modéliser un prompt multimodal : texte + images + documents +
//! audio. Chaque variante augmente le nombre de tokens facturés au modèle.
//!
//! La conversion en tokens **dépend du modèle** (cf. `VisionPricing` côté
//! `sobria-estimator`). Cette crate fournit une estimation **générique**
//! pour les cas où le preset n'est pas connu — le moteur effectif utilise
//! `preset.vision_pricing.tokens_for(...)` quand disponible.
//!
//! ## Différence avec `ModelDomain`
//!
//! - [`crate::model::ModelDomain`] qualifie le **modèle entier** (LLM,
//!   Stable Diffusion, TTS, etc.).
//! - [`InputModality`] qualifie le **type d'input d'un prompt unitaire**
//!   sur un LLM multimodal (texte, images, documents, audio).
//!
//! Voir `briefs/chantiers/C34-catalogue-modalites-overhead.md` §2.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type d'input d'un prompt multimodal.
///
/// Sérialisation tagged JSON pour distinguer les variantes :
/// ```json
/// { "kind": "vision_high", "image_count": 2, "avg_width": 1024, "avg_height": 1024 }
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputModality {
    /// Texte uniquement (cas par défaut — pas de surcoût).
    Text,
    /// Images en **basse résolution** (OpenAI `low detail`). Les modèles
    /// vendor facturent typiquement quelques dizaines à centaines de tokens
    /// par image (85 pour OpenAI low, 258 pour Gemini ≤384×384, etc.).
    VisionLow {
        /// Nombre d'images dans le prompt.
        image_count: u32,
    },
    /// Images en **haute résolution**. Coût en tokens dépend des dimensions
    /// et du modèle (voir `VisionPricing` côté `sobria-estimator`).
    VisionHigh {
        /// Nombre d'images dans le prompt.
        image_count: u32,
        /// Largeur moyenne en pixels.
        avg_width: u32,
        /// Hauteur moyenne en pixels.
        avg_height: u32,
    },
    /// Document PDF — converti page par page en tokens via OCR/text-extract.
    Document {
        /// Nombre de pages du document.
        page_count: u32,
    },
    /// Audio en entrée — tokenisé à ~10 tokens/seconde (Whisper rate).
    AudioInput {
        /// Durée de l'audio en secondes.
        duration_seconds: u32,
    },
}

/// Tokens par seconde d'audio (Whisper rate moyen).
///
/// Source : OpenAI Whisper paper + Realtime API doc.
pub const AUDIO_TOKENS_PER_SECOND: u32 = 10;

/// Tokens estimés par page PDF (850 mots × 1.3 token/mot ≈ 1100).
///
/// Source : analyse empirique LMSYS arena uploads.
pub const DOCUMENT_TOKENS_PER_PAGE: u32 = 1100;

/// Tokens génériques par image basse résolution (fallback si pas de
/// `VisionPricing` preset).
///
/// Médiane entre OpenAI low (85), Gemini ≤384 (258), Anthropic carré ~750².
/// Source : doc vision officielle des 4 vendors principaux.
pub const VISION_LOW_TOKENS_PER_IMAGE_FALLBACK: u32 = 300;

/// Tokens génériques par image haute résolution 1024×1024 (fallback).
///
/// Médiane entre OpenAI high (765 pour 1024×1024), Anthropic carré (1568),
/// Gemini (1032 pour 1500×1500), Llama (1601). Source : doc vendor.
pub const VISION_HIGH_TOKENS_PER_IMAGE_FALLBACK: u32 = 1200;

impl InputModality {
    /// Estimation **générique** de tokens d'overhead, sans connaître le
    /// modèle. Utilisée comme fallback si le preset ne définit pas de
    /// `VisionPricing`.
    ///
    /// Les engines utilisent en priorité `preset.vision_pricing.tokens_for(...)`
    /// pour les variantes VisionLow/VisionHigh.
    #[must_use]
    pub fn default_token_count(&self) -> u32 {
        match self {
            Self::Text => 0,
            Self::VisionLow { image_count } => {
                VISION_LOW_TOKENS_PER_IMAGE_FALLBACK.saturating_mul(*image_count)
            },
            Self::VisionHigh { image_count, .. } => {
                VISION_HIGH_TOKENS_PER_IMAGE_FALLBACK.saturating_mul(*image_count)
            },
            Self::Document { page_count } => DOCUMENT_TOKENS_PER_PAGE.saturating_mul(*page_count),
            Self::AudioInput { duration_seconds } => {
                AUDIO_TOKENS_PER_SECOND.saturating_mul(*duration_seconds)
            },
        }
    }

    /// Indique si la modalité requiert que le modèle soit `vision_capable`.
    #[must_use]
    pub fn requires_vision(&self) -> bool {
        matches!(
            self,
            Self::VisionLow { .. } | Self::VisionHigh { .. } | Self::Document { .. }
        )
    }

    /// Indique si la modalité requiert que le modèle soit `audio_capable`.
    #[must_use]
    pub fn requires_audio(&self) -> bool {
        matches!(self, Self::AudioInput { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_has_zero_overhead() {
        assert_eq!(InputModality::Text.default_token_count(), 0);
    }

    #[test]
    fn vision_low_scales_with_count() {
        let m = InputModality::VisionLow { image_count: 3 };
        assert_eq!(m.default_token_count(), 900); // 300 × 3
    }

    #[test]
    fn vision_high_scales_with_count() {
        let m = InputModality::VisionHigh {
            image_count: 2,
            avg_width: 1024,
            avg_height: 1024,
        };
        assert_eq!(m.default_token_count(), 2400); // 1200 × 2
    }

    #[test]
    fn document_5_pages() {
        let m = InputModality::Document { page_count: 5 };
        assert_eq!(m.default_token_count(), 5500); // 1100 × 5
    }

    #[test]
    fn audio_30_seconds() {
        let m = InputModality::AudioInput {
            duration_seconds: 30,
        };
        assert_eq!(m.default_token_count(), 300); // 10 × 30
    }

    #[test]
    fn requires_vision_consistency() {
        assert!(!InputModality::Text.requires_vision());
        assert!(InputModality::VisionLow { image_count: 1 }.requires_vision());
        assert!(InputModality::VisionHigh {
            image_count: 1,
            avg_width: 512,
            avg_height: 512
        }
        .requires_vision());
        assert!(InputModality::Document { page_count: 1 }.requires_vision());
        assert!(!InputModality::AudioInput {
            duration_seconds: 10
        }
        .requires_vision());
    }

    #[test]
    fn requires_audio_consistency() {
        assert!(!InputModality::Text.requires_audio());
        assert!(InputModality::AudioInput {
            duration_seconds: 1
        }
        .requires_audio());
        assert!(!InputModality::VisionLow { image_count: 1 }.requires_audio());
    }

    #[test]
    fn serde_round_trip_text() {
        let m = InputModality::Text;
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#"{"kind":"text"}"#);
        let back: InputModality = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn serde_round_trip_vision_high() {
        let m = InputModality::VisionHigh {
            image_count: 2,
            avg_width: 1024,
            avg_height: 768,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: InputModality = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn saturating_handles_huge_counts() {
        // Empêche d'overflow u32 même avec un image_count démesuré.
        let m = InputModality::VisionLow {
            image_count: u32::MAX,
        };
        // saturating_mul ne panique pas
        let _ = m.default_token_count();
    }
}

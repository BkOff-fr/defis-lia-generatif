//! **C34.3** — Overhead système d'un prompt (tokens cachés).
//!
//! Modélise le coût invisible payé pour chaque requête à un LLM via une
//! interface app (Claude.ai, ChatGPT, Gemini, etc.) :
//!
//! - `system_prompt_tokens` : instructions cachées injectées par le vendor
//!   (~600-2500 selon vendor). Source : leaks publics + reverse-engineering.
//! - `tools_definition_tokens` : schémas JSON des outils activés (code
//!   interpreter, web search, etc.) — ~200-1500 tokens chacun.
//! - `memory_tokens` : tokens accumulés des tours précédents de la
//!   conversation (memory feature).
//! - `thinking_tokens_p50` : pour reasoning models (o3, R1, V4, Claude
//!   extended thinking…) — 5×-30× output tokens en P50.
//!
//! **Disclaimer** : « Estimation overhead système ± 50 % — basée sur leaks
//! publics et reverse-engineering interfaces vendor (Claude.ai, ChatGPT
//! app, Gemini app). À surcharger en mode Expert si vous connaissez votre
//! valeur exacte. »

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Overhead contexte invisible payé sur chaque requête.
///
/// Tous les champs ont `#[serde(default)]` pour rester compatible avec les
/// entries audit ledger antérieures à C34 (qui n'incluent pas ces champs).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
pub struct ContextOverhead {
    /// Tokens du system prompt caché de l'interface (`~2000` pour Claude.ai,
    /// `~1000` pour ChatGPT, `0` pour l'API directe).
    #[serde(default)]
    pub system_prompt_tokens: u32,
    /// Tokens des schémas JSON des outils activés (~200-1500 par outil).
    #[serde(default)]
    pub tools_definition_tokens: u32,
    /// Tokens des tours précédents accumulés (memory).
    #[serde(default)]
    pub memory_tokens: u32,
    /// **Thinking tokens** (P50) pour reasoning models. Ces tokens sont
    /// du **côté sortie** (decode) — pas du côté entrée. Ajoutés
    /// automatiquement par les engines quand
    /// [`crate::model::ModelDomain`] active reasoning.
    ///
    /// Note sémantique : `total()` inclut ces tokens par convention du
    /// brief C34 §2, mais [`Self::total_input`] / [`Self::total_output`]
    /// font la distinction physique correcte.
    #[serde(default)]
    pub thinking_tokens_p50: u32,
}

impl ContextOverhead {
    /// Constructeur explicite (alternatif à `Default::default()`).
    #[must_use]
    pub fn new(
        system_prompt_tokens: u32,
        tools_definition_tokens: u32,
        memory_tokens: u32,
        thinking_tokens_p50: u32,
    ) -> Self {
        Self {
            system_prompt_tokens,
            tools_definition_tokens,
            memory_tokens,
            thinking_tokens_p50,
        }
    }

    /// Somme **côté entrée** : system + tools + memory.
    ///
    /// Utilisé par les engines pour calculer `effective_input_tokens`.
    #[must_use]
    pub fn total_input(&self) -> u32 {
        self.system_prompt_tokens
            .saturating_add(self.tools_definition_tokens)
            .saturating_add(self.memory_tokens)
    }

    /// Somme **côté sortie** : thinking tokens P50.
    ///
    /// Utilisé par les engines pour calculer `effective_output_tokens`.
    #[must_use]
    pub fn total_output(&self) -> u32 {
        self.thinking_tokens_p50
    }

    /// Somme totale (input + output). Cohérent avec brief C34 §2 mais
    /// physiquement ambigu — préférer [`Self::total_input`] /
    /// [`Self::total_output`] pour les calculs énergétiques.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.total_input().saturating_add(self.total_output())
    }

    /// Indique si tous les champs sont à zéro (overhead inactif).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.system_prompt_tokens == 0
            && self.tools_definition_tokens == 0
            && self.memory_tokens == 0
            && self.thinking_tokens_p50 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zeros() {
        let oh = ContextOverhead::default();
        assert!(oh.is_empty());
        assert_eq!(oh.total(), 0);
        assert_eq!(oh.total_input(), 0);
        assert_eq!(oh.total_output(), 0);
    }

    #[test]
    fn total_input_excludes_thinking() {
        let oh = ContextOverhead::new(1000, 500, 200, 5000);
        assert_eq!(oh.total_input(), 1700);
        assert_eq!(oh.total_output(), 5000);
        assert_eq!(oh.total(), 6700);
    }

    #[test]
    fn saturating_handles_overflow() {
        let oh = ContextOverhead::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX);
        // Ne panique pas
        let _ = oh.total();
        let _ = oh.total_input();
        let _ = oh.total_output();
    }

    #[test]
    fn serde_round_trip() {
        let oh = ContextOverhead::new(2000, 800, 150, 3000);
        let json = serde_json::to_string(&oh).unwrap();
        let back: ContextOverhead = serde_json::from_str(&json).unwrap();
        assert_eq!(back, oh);
    }

    #[test]
    fn serde_deserialize_legacy_empty_json() {
        // Compat backward : un audit ledger v0.8.x sans ContextOverhead
        // doit pouvoir désérialiser en `default()`.
        let json = "{}";
        let oh: ContextOverhead = serde_json::from_str(json).unwrap();
        assert_eq!(oh, ContextOverhead::default());
    }

    #[test]
    fn serde_deserialize_partial_fields() {
        // Avec seulement un champ — les autres prennent 0 par défaut.
        let json = r#"{"system_prompt_tokens": 2000}"#;
        let oh: ContextOverhead = serde_json::from_str(json).unwrap();
        assert_eq!(oh.system_prompt_tokens, 2000);
        assert_eq!(oh.tools_definition_tokens, 0);
    }
}

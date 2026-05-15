//! Modèles d'IA générative et leurs caractéristiques.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Fournisseurs de modèles connus.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ModelProvider {
    /// OpenAI.
    OpenAI,
    /// Anthropic.
    Anthropic,
    /// Mistral AI.
    Mistral,
    /// Google.
    Google,
    /// Meta.
    Meta,
    /// Hugging Face (modèles ouverts hébergés).
    HuggingFace,
    /// Autre fournisseur (avec nom libre).
    Other(String),
}

/// Modalités d'IA générative. La v1.0 ne couvre que `Text`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// Texte uniquement (LLMs) — périmètre v1.0.
    Text,
    /// Image (Stable Diffusion, etc.) — v2.0.
    Image,
    /// Audio / voix — v2.0.
    Audio,
    /// Vidéo — v2.0.
    Video,
}

/// Fiche d'un modèle dans le référentiel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Model {
    /// Identifiant stable (ex: `"gpt-4o-mini"`).
    pub id: String,
    /// Nom commercial.
    pub name: String,
    /// Fournisseur.
    pub provider: ModelProvider,
    /// Modalité principale.
    pub modality: Modality,
    /// Nombre de paramètres (en milliards). `None` si non public.
    pub parameters_billions: Option<f64>,
    /// Contexte maximal en tokens.
    pub context_tokens: Option<u32>,
    /// Sources documentaires (URL ou DOI).
    pub sources: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_serializes() {
        let m = Model {
            id: "gpt-4o-mini".into(),
            name: "GPT-4o mini".into(),
            provider: ModelProvider::OpenAI,
            modality: Modality::Text,
            parameters_billions: Some(8.0),
            context_tokens: Some(128_000),
            sources: vec![
                "https://openai.com/index/gpt-4o-mini-advancing-cost-efficient-intelligence/"
                    .into(),
            ],
        };
        let _ = serde_json::to_string(&m).expect("serialize");
    }
}

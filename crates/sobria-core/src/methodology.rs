//! Méthodologies d'estimation d'empreinte LLM embarquées dans Sobr.ia.
//!
//! Sobr.ia propose un **catalogue de méthodologies scientifiques** que
//! l'utilisateur peut sélectionner pour ses calculs :
//!
//! - **AFNOR SPEC 2314 (Sobr.ia)** : référentiel français, formule
//!   linéaire-par-token + Monte-Carlo N=10⁴.
//! - **EcoLogits 2026-01** : port direct des formules Rincé & Banse
//!   ([doi:10.21105/joss.07471](https://doi.org/10.21105/joss.07471),
//!   CC BY-SA 4.0).
//!
//! Ce module définit uniquement l'identifiant énumératif. Les
//! implémentations (`EmpreinteEngine`, `MethodologyInfo`, factory) sont
//! dans le crate `sobria-estimator`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Identifiant stable d'une méthodologie d'empreinte LLM.
///
/// Sérialisé en `snake_case` pour stockage SQLite (table `audit_entries`,
/// `app_preferences`) et IPC.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EmpreinteMethod {
    /// Formule AFNOR SPEC 2314 (linéaire-par-token) + Monte-Carlo Sobr.ia.
    AfnorSobria,
    /// Méthodologie EcoLogits 2026-01 (port direct, CC BY-SA 4.0).
    ///
    /// Note : sérialisé en `"ecologits"` (mot unique, marque déposée) et non
    /// `"eco_logits"` produit par défaut par `rename_all = "snake_case"`.
    #[serde(rename = "ecologits")]
    EcoLogits,
}

impl EmpreinteMethod {
    /// Identifiant snake_case stable (pour persistence SQLite / IPC).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AfnorSobria => "afnor_sobria",
            Self::EcoLogits => "ecologits",
        }
    }

    /// Parse depuis l'identifiant snake_case. Retourne `None` si inconnu.
    ///
    /// Note : on n'implémente pas `std::str::FromStr` car la sémantique
    /// "retourne `None`" est plus parlante qu'une `Err` pour cet usage
    /// (préférences user / IPC).
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "afnor_sobria" => Some(Self::AfnorSobria),
            "ecologits" => Some(Self::EcoLogits),
            _ => None,
        }
    }

    /// Méthodologie par défaut au premier lancement.
    ///
    /// AFNOR SPEC 2314 est le référentiel français, choisi comme défaut
    /// "souverain" pour cette app made-in-France.
    #[must_use]
    pub fn default_method() -> Self {
        Self::AfnorSobria
    }

    /// Variantes connues (utilisé pour itérer dans les tests / l'UI).
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::AfnorSobria, Self::EcoLogits]
    }
}

impl Default for EmpreinteMethod {
    fn default() -> Self {
        Self::default_method()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_via_str() {
        for &m in EmpreinteMethod::all() {
            assert_eq!(EmpreinteMethod::from_str(m.as_str()), Some(m));
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert_eq!(EmpreinteMethod::from_str("bidon"), None);
    }

    #[test]
    fn default_is_afnor_sobria() {
        assert_eq!(EmpreinteMethod::default(), EmpreinteMethod::AfnorSobria);
    }

    #[test]
    fn serde_uses_snake_case() {
        let json = serde_json::to_string(&EmpreinteMethod::EcoLogits).unwrap();
        assert_eq!(json, "\"ecologits\"");
    }
}

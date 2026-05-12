//! Représentation d'un datacenter dans le référentiel.
//!
//! Le PUE (Power Usage Effectiveness) et le WUE (Water Usage Effectiveness)
//! sont des distributions d'incertitude — voir
//! [ADR-0004](../../docs/adr/ADR-0004-monte-carlo.md).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::SobriaResult,
    indicators::UncertaintyInterval,
    validation::validate_country_iso,
};

/// Fiche d'un datacenter dans le référentiel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Datacenter {
    /// Identifiant stable (ex: `"openai-us-east-virginia"`).
    pub id: String,
    /// Provider hébergeur (ex: `"AWS"`, `"Azure"`, `"OVHcloud"`).
    pub provider: String,
    /// Région (ex: `"us-east-1"`).
    pub region: String,
    /// Code pays ISO 3166-1 alpha-2 (ex: `"FR"`).
    pub country_iso: String,
    /// PUE — distribution d'incertitude. Valeur typique 1,1-1,6.
    pub pue: UncertaintyInterval,
    /// WUE — litres d'eau par kWh IT (optionnel, peu publié).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wue: Option<UncertaintyInterval>,
    /// Sources documentaires (URLs, DOIs, ou clés BibTeX).
    pub sources: Vec<String>,
}

impl Datacenter {
    /// Valide tous les invariants de la fiche.
    ///
    /// # Erreurs
    ///
    /// Retourne `SchemaValidation` si :
    /// - `country_iso` n'est pas un code ISO 3166-1 alpha-2 valide.
    /// - Un intervalle d'incertitude (PUE, WUE) est mal formé.
    pub fn validate(&self) -> SobriaResult<()> {
        validate_country_iso(&self.country_iso)?;
        self.pue.validate()?;
        if let Some(ref wue) = self.wue {
            wue.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Datacenter {
        Datacenter {
            id: "openai-us-east-virginia".into(),
            provider: "Azure".into(),
            region: "us-east-2".into(),
            country_iso: "US".into(),
            pue: UncertaintyInterval::new(1.2, 1.3, 1.5).unwrap(),
            wue: Some(UncertaintyInterval::new(1.0, 1.8, 3.2).unwrap()),
            sources: vec!["https://example.com".into()],
        }
    }

    #[test]
    fn datacenter_validates() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn datacenter_rejects_bad_country() {
        let mut d = sample();
        d.country_iso = "us".into();
        assert!(d.validate().is_err());
    }

    #[test]
    fn datacenter_serializes_round_trip() {
        let d = sample();
        let json = serde_json::to_string(&d).unwrap();
        let back: Datacenter = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn datacenter_wue_optional_omitted_in_json() {
        let mut d = sample();
        d.wue = None;
        let json = serde_json::to_string(&d).unwrap();
        assert!(!json.contains("wue"));
    }
}

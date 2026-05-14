//! Demande et résultat d'estimation d'un prompt.
//!
//! Le résultat embarque les hypothèses utilisées (cliquables dans l'UI)
//! et un seed reproductible — voir
//! [ADR-0004](../../docs/adr/ADR-0004-monte-carlo.md).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::{SobriaError, SobriaResult},
    indicators::{Equivalent, IndicatorValue},
    methodology::EmpreinteMethod,
};

/// Une requête d'estimation pour un usage unitaire.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EstimationRequest {
    /// Identifiant du modèle visé (ex: `"gpt-4o-mini"`).
    pub model_id: String,
    /// Tokens d'entrée mesurés ou estimés.
    pub tokens_in: u32,
    /// Tokens de sortie estimés (avec leur propre incertitude implicite).
    pub tokens_out_estimated: u32,
    /// Datacenter retenu (optionnel — défaut : géoloc M9 ou moyenne).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,
    /// Horodatage UTC de la requête.
    pub timestamp: DateTime<Utc>,
}

impl EstimationRequest {
    /// Valide les invariants.
    ///
    /// # Erreurs
    ///
    /// Retourne `SchemaValidation` si `tokens_in > 10_000_000` ou
    /// `tokens_out_estimated > 10_000_000` (garde-fou anti-abus).
    pub fn validate(&self) -> SobriaResult<()> {
        const MAX_TOKENS: u32 = 10_000_000;
        if self.tokens_in > MAX_TOKENS || self.tokens_out_estimated > MAX_TOKENS {
            return Err(SobriaError::SchemaValidation(format!(
                "tokens absurdes : in={}, out={}",
                self.tokens_in, self.tokens_out_estimated
            )));
        }
        if self.model_id.trim().is_empty() {
            return Err(SobriaError::SchemaValidation("model_id vide".into()));
        }
        Ok(())
    }
}

/// Une hypothèse utilisée par l'estimateur (affichée dans l'UI, cliquable
/// vers la source).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Hypothesis {
    /// Clé courte (ex: `"epsilon_decode_mJ_per_token"`).
    pub key: String,
    /// Valeur arbitraire (typiquement nombre, mais peut être objet).
    pub value: serde_json::Value,
    /// Source documentaire (URL, DOI, ou clé BibTeX).
    pub source: String,
}

/// Résultat complet d'une estimation, prêt à journaliser dans l'audit ledger.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EstimationResult {
    /// Méthodologie utilisée pour produire ce résultat. Tracé dans
    /// l'audit ledger (colonne `method`) pour permettre à l'utilisateur
    /// de filtrer ses estimations par méthodologie ou de régénérer un
    /// rapport historiquement cohérent (cf. ADR-0012, chantier C24).
    ///
    /// `serde(default)` pour rester compatible avec les entries audit
    /// produites **avant** l'introduction de ce champ (v0.3.x) : elles
    /// sont alors considérées comme `AfnorSobria` (seul moteur historique).
    #[serde(default)]
    pub method: EmpreinteMethod,
    /// Requête d'origine.
    pub request: EstimationRequest,
    /// Indicateurs calculés (CO₂eq, énergie, eau, métaux, coût).
    pub indicators: Vec<IndicatorValue>,
    /// Équivalents parlants pour l'UI.
    pub equivalents: Vec<Equivalent>,
    /// Hypothèses utilisées (sources cliquables).
    pub hypotheses: Vec<Hypothesis>,
    /// Horodatage UTC du calcul.
    pub computed_at: DateTime<Utc>,
    /// Seed Monte-Carlo utilisé — assure la reproductibilité.
    pub seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::{Indicator, UncertaintyInterval};

    fn sample_request() -> EstimationRequest {
        EstimationRequest {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 23,
            tokens_out_estimated: 720,
            datacenter_id: Some("openai-us-east-virginia".into()),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn request_validates() {
        assert!(sample_request().validate().is_ok());
    }

    #[test]
    fn request_rejects_empty_model() {
        let mut r = sample_request();
        r.model_id = "  ".into();
        assert!(r.validate().is_err());
    }

    #[test]
    fn request_rejects_absurd_tokens() {
        let mut r = sample_request();
        r.tokens_in = 999_999_999;
        assert!(r.validate().is_err());
    }

    #[test]
    fn result_serializes_round_trip() {
        let request = sample_request();
        let result = EstimationResult {
            method: EmpreinteMethod::EcoLogits,
            request,
            indicators: vec![IndicatorValue {
                indicator: Indicator::Co2Eq,
                interval: UncertaintyInterval::new(1.68, 2.14, 2.74).unwrap(),
                unit: "gCO2eq".into(),
                bins: None,
            }],
            equivalents: vec![Equivalent {
                label: "km en voiture thermique".into(),
                value: 0.017,
                source: "ADEME 2025".into(),
            }],
            hypotheses: vec![Hypothesis {
                key: "epsilon_decode_mJ_per_token".into(),
                value: serde_json::json!(1.8),
                source: "HF AI Energy Score 2026".into(),
            }],
            computed_at: Utc::now(),
            seed: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: EstimationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }

    #[test]
    fn result_deserializes_legacy_v03_without_method_field() {
        // Une entry audit v0.3.x (avant C24) n'a pas de champ `method` :
        // on doit pouvoir la désérialiser, le champ vaudra `AfnorSobria`
        // (méthodologie historique).
        let legacy_json = r#"{
            "request": {
                "model_id": "gpt-4o-mini",
                "tokens_in": 10,
                "tokens_out_estimated": 50,
                "timestamp": "2026-04-01T10:00:00Z"
            },
            "indicators": [],
            "equivalents": [],
            "hypotheses": [],
            "computed_at": "2026-04-01T10:00:00Z",
            "seed": 42
        }"#;
        let back: EstimationResult = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(back.method, EmpreinteMethod::AfnorSobria);
    }
}

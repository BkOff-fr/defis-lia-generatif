//! Paramètres distributionnels du modèle de calcul.
//!
//! Voir CDC §9 et ADR-0004 pour la formule de référence.
//! Voir `briefs/chantiers/C05-estimator-monte-carlo.md` §4.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    distributions::Distribution,
    error::{EstimatorError, EstimatorResult},
};

/// Ensemble distributionnel des paramètres physiques d'une estimation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EstimationParams {
    /// Énergie/token de prefill (input), en milli-joules par token.
    pub epsilon_prefill_mj_per_token: Distribution,
    /// Énergie/token de decode (output), en milli-joules par token.
    pub epsilon_decode_mj_per_token: Distribution,
    /// PUE du datacenter (sans unité, typiquement entre 1.05 et 1.6).
    pub pue: Distribution,
    /// Facteur d'émission électrique, en gCO₂eq/kWh.
    pub if_electrical_g_per_kwh: Distribution,
    /// Embodied carbon amorti par requête, en gCO₂eq.
    pub embodied_g_per_request: Distribution,
    /// Water Usage Effectiveness, en litres par kWh IT.
    pub wue_l_per_kwh: Distribution,
}

impl EstimationParams {
    /// Construit un set de paramètres conservateur "par défaut" — valeurs
    /// raisonnables à défaut de données spécifiques. À utiliser uniquement
    /// dans les tests et la démonstration.
    ///
    /// Valeurs sourcées de manière indicative :
    /// - ε_prefill / ε_decode : ordre de grandeur HF AI Energy Score 2026.
    /// - PUE : datacenter moderne, plage 1.1-1.4.
    /// - IF élec : mix France 2024 (ADEME ~56 g/kWh).
    /// - Embodied / req : Gupta 2022 amorti sur ~1 G req.
    /// - WUE : Mytton 2021 plage moyenne.
    #[must_use]
    pub fn conservative_default() -> Self {
        Self {
            // 1 mJ/token médian, ±50 %
            epsilon_prefill_mj_per_token: Distribution::LogNormal {
                mu: (1.0_f64).ln(),
                sigma: 0.25,
            },
            // 2 mJ/token médian (decode plus coûteux que prefill)
            epsilon_decode_mj_per_token: Distribution::LogNormal {
                mu: (2.0_f64).ln(),
                sigma: 0.3,
            },
            pue: Distribution::Uniform { low: 1.1, high: 1.4 },
            if_electrical_g_per_kwh: Distribution::Point { value: 56.0 },
            embodied_g_per_request: Distribution::LogNormal {
                mu: (0.02_f64).ln(),
                sigma: 0.4,
            },
            wue_l_per_kwh: Distribution::Uniform { low: 0.5, high: 2.5 },
        }
    }

    /// Vérifie que toutes les distributions sont valides.
    pub fn validate(&self) -> EstimatorResult<()> {
        self.epsilon_prefill_mj_per_token
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("epsilon_prefill: {e}")))?;
        self.epsilon_decode_mj_per_token
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("epsilon_decode: {e}")))?;
        self.pue
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("pue: {e}")))?;
        self.if_electrical_g_per_kwh
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("if_electrical: {e}")))?;
        self.embodied_g_per_request
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("embodied: {e}")))?;
        self.wue_l_per_kwh
            .validate()
            .map_err(|e| EstimatorError::Validation(format!("wue: {e}")))?;
        Ok(())
    }

    /// Builder fluide : remplace `pue`.
    #[must_use]
    pub fn with_pue(mut self, d: Distribution) -> Self {
        self.pue = d;
        self
    }

    /// Builder fluide : remplace `if_electrical_g_per_kwh`.
    #[must_use]
    pub fn with_if_electrical(mut self, d: Distribution) -> Self {
        self.if_electrical_g_per_kwh = d;
        self
    }

    /// Builder fluide : remplace `epsilon_decode_mj_per_token`.
    #[must_use]
    pub fn with_epsilon_decode(mut self, d: Distribution) -> Self {
        self.epsilon_decode_mj_per_token = d;
        self
    }

    /// Builder fluide : remplace `embodied_g_per_request`.
    #[must_use]
    pub fn with_embodied(mut self, d: Distribution) -> Self {
        self.embodied_g_per_request = d;
        self
    }

    /// Builder fluide : remplace `wue_l_per_kwh`.
    #[must_use]
    pub fn with_wue(mut self, d: Distribution) -> Self {
        self.wue_l_per_kwh = d;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_default_is_valid() {
        let p = EstimationParams::conservative_default();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn builder_chains() {
        let p = EstimationParams::conservative_default()
            .with_pue(Distribution::Point { value: 1.3 })
            .with_if_electrical(Distribution::Point { value: 412.0 });
        assert_eq!(p.pue, Distribution::Point { value: 1.3 });
        assert_eq!(p.if_electrical_g_per_kwh, Distribution::Point { value: 412.0 });
        // Les autres champs ne sont pas modifiés
        assert!(p.validate().is_ok());
    }

    #[test]
    fn validate_catches_bad_subparam() {
        let p = EstimationParams::conservative_default()
            .with_pue(Distribution::Uniform { low: 5.0, high: 1.0 });
        let err = p.validate().unwrap_err();
        assert!(format!("{err}").contains("pue"));
    }

    #[test]
    fn serde_round_trip() {
        let p = EstimationParams::conservative_default();
        let json = serde_json::to_string(&p).unwrap();
        let back: EstimationParams = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }
}

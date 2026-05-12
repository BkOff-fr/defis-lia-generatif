//! Facteurs d'émission électriques par pays/année.
//!
//! Source primaire : ADEME Base Empreinte + RTE eco2mix pour la France live.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::SobriaResult,
    indicators::UncertaintyInterval,
    validation::{validate_country_iso, validate_year},
};

/// Facteur d'émission électrique d'un pays sur une période donnée.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EmissionFactor {
    /// Code pays ISO 3166-1 alpha-2 (ex: `"FR"`).
    pub country_iso: String,
    /// Année calendaire de référence.
    pub year: u16,
    /// Intensité carbone (gCO₂eq/kWh) avec incertitude.
    pub g_co2eq_per_kwh: UncertaintyInterval,
    /// Source documentaire (URL ou DOI).
    pub source: String,
    /// Date de début de validité.
    pub valid_from: DateTime<Utc>,
    /// Date de fin de validité (None si toujours en cours).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
}

impl EmissionFactor {
    /// Valide les invariants.
    ///
    /// # Erreurs
    ///
    /// Retourne `SchemaValidation` si :
    /// - `country_iso` n'est pas valide.
    /// - `year` est hors plage [1990, 2100].
    /// - L'intervalle CO₂eq est mal formé.
    /// - `valid_until` est antérieur à `valid_from`.
    pub fn validate(&self) -> SobriaResult<()> {
        validate_country_iso(&self.country_iso)?;
        validate_year(self.year)?;
        self.g_co2eq_per_kwh.validate()?;
        if let Some(end) = self.valid_until {
            if end < self.valid_from {
                return Err(crate::error::SobriaError::SchemaValidation(format!(
                    "valid_until ({end}) antérieur à valid_from ({})",
                    self.valid_from
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_fr() -> EmissionFactor {
        EmissionFactor {
            country_iso: "FR".into(),
            year: 2025,
            // Source ADEME : ~56 gCO₂eq/kWh pour la France 2024.
            g_co2eq_per_kwh: UncertaintyInterval::new(45.0, 56.0, 75.0).unwrap(),
            source: "ADEME Base Empreinte 2025".into(),
            valid_from: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            valid_until: Some(Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap()),
        }
    }

    #[test]
    fn emission_factor_validates() {
        assert!(sample_fr().validate().is_ok());
    }

    #[test]
    fn emission_factor_rejects_end_before_start() {
        let mut f = sample_fr();
        f.valid_until = Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
        assert!(f.validate().is_err());
    }

    #[test]
    fn emission_factor_serializes_round_trip() {
        let f = sample_fr();
        let json = serde_json::to_string(&f).unwrap();
        let back: EmissionFactor = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }
}

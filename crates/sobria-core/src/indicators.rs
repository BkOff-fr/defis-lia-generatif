//! Indicateurs environnementaux et leur restitution.
//!
//! Voir CDC §4.2 (indicateurs calculés) et ADR-0004 (incertitude).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Indicateurs environnementaux supportés.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Indicator {
    /// Équivalent CO₂ (gCO₂eq).
    Co2Eq,
    /// Énergie consommée (Wh).
    Energy,
    /// Eau (litres).
    Water,
    /// Métaux critiques (mg équivalent terre rare).
    CriticalMetals,
    /// Coût utilisateur facturé (€).
    Cost,
}

/// Intervalle d'incertitude calculé par Monte-Carlo (ADR-0004).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct UncertaintyInterval {
    /// P5 — borne basse de l'intervalle à 90 %.
    pub p5: f64,
    /// P50 — médiane.
    pub p50: f64,
    /// P95 — borne haute de l'intervalle à 90 %.
    pub p95: f64,
}

/// Valeur d'un indicateur, accompagnée de son unité et de son incertitude.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndicatorValue {
    /// Indicateur concerné.
    pub indicator: Indicator,
    /// Intervalle d'incertitude restitué.
    pub interval: UncertaintyInterval,
    /// Unité humainement lisible (ex: `"gCO2eq"`, `"Wh"`).
    pub unit: String,
}

/// Équivalent parlant pour l'utilisateur final.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Equivalent {
    /// Description courte (ex: `"km en voiture thermique"`).
    pub label: String,
    /// Valeur de l'équivalent.
    pub value: f64,
    /// Source de la conversion.
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncertainty_interval_serializes_round_trip() {
        let i = UncertaintyInterval { p5: 1.5, p50: 2.0, p95: 2.7 };
        let json = serde_json::to_string(&i).unwrap();
        let back: UncertaintyInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(back, i);
    }
}

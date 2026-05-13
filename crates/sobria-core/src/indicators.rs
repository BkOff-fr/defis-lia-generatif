//! Indicateurs environnementaux et leur restitution.
//!
//! Voir CDC §4.2 (indicateurs calculés) et ADR-0004 (incertitude).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{SobriaError, SobriaResult};

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

impl Indicator {
    /// Unité SI par défaut associée à l'indicateur.
    #[must_use]
    pub fn default_unit(self) -> &'static str {
        match self {
            Self::Co2Eq => "gCO2eq",
            Self::Energy => "Wh",
            Self::Water => "L",
            Self::CriticalMetals => "mg",
            Self::Cost => "EUR",
        }
    }
}

/// Intervalle d'incertitude restitué par Monte-Carlo (ADR-0004).
///
/// Invariants garantis par construction via [`Self::new`] :
/// - `p5`, `p50`, `p95` sont finis et positifs (≥ 0).
/// - `p5 ≤ p50 ≤ p95`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct UncertaintyInterval {
    /// P5 — borne basse de l'intervalle à 90 %.
    pub p5: f64,
    /// P50 — médiane.
    pub p50: f64,
    /// P95 — borne haute de l'intervalle à 90 %.
    pub p95: f64,
}

impl UncertaintyInterval {
    /// Construit un intervalle après validation des invariants.
    pub fn new(p5: f64, p50: f64, p95: f64) -> SobriaResult<Self> {
        if !(p5.is_finite() && p50.is_finite() && p95.is_finite()) {
            return Err(SobriaError::SchemaValidation(
                "p5/p50/p95 doivent être finis (pas NaN ni infini)".into(),
            ));
        }
        if p5 < 0.0 || p50 < 0.0 || p95 < 0.0 {
            return Err(SobriaError::SchemaValidation(format!(
                "p5/p50/p95 doivent être positifs : p5={p5}, p50={p50}, p95={p95}"
            )));
        }
        if !(p5 <= p50 && p50 <= p95) {
            return Err(SobriaError::SchemaValidation(format!(
                "ordre violé (attendu p5 ≤ p50 ≤ p95) : p5={p5}, p50={p50}, p95={p95}"
            )));
        }
        Ok(Self { p5, p50, p95 })
    }

    /// Construit un intervalle ponctuel (sans incertitude).
    pub fn point(value: f64) -> SobriaResult<Self> {
        Self::new(value, value, value)
    }

    /// Largeur relative de l'intervalle (P95 - P5) / P50.
    #[must_use]
    pub fn relative_width(&self) -> Option<f64> {
        if self.p50 == 0.0 {
            None
        } else {
            Some((self.p95 - self.p5) / self.p50)
        }
    }

    /// Revalide un intervalle (utile après désérialisation).
    pub fn validate(&self) -> SobriaResult<()> {
        Self::new(self.p5, self.p50, self.p95).map(|_| ())
    }
}

/// Histogramme de la distribution Monte-Carlo d'un indicateur.
///
/// `counts.len()` bins **équi-width** entre `min` et `max` (bornes
/// incluses). Permet à l'UI de rendre une distribution fidèle plutôt
/// qu'une approximation analytique depuis P5/P50/P95.
///
/// Présent dans l'audit ledger (sérialisé dans `EstimationResult`) pour
/// que la traçabilité réglementaire CSRD inclue la distribution complète,
/// pas uniquement les quantiles agrégés.
///
/// Voir `briefs/chantiers/C09-tauri-integration.md` (option A1).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct DistributionBins {
    /// Borne basse de l'histogramme (≤ tous les samples).
    pub min: f64,
    /// Borne haute de l'histogramme (≥ tous les samples).
    pub max: f64,
    /// Comptes par bin. `counts.iter().sum() == N` (nombre de tirages).
    pub counts: Vec<u32>,
}

/// Valeur d'un indicateur, accompagnée de son unité, de son incertitude
/// et éventuellement de son histogramme distributionnel.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct IndicatorValue {
    /// Indicateur concerné.
    pub indicator: Indicator,
    /// Intervalle d'incertitude restitué.
    pub interval: UncertaintyInterval,
    /// Unité humainement lisible.
    pub unit: String,
    /// Histogramme de la distribution Monte-Carlo (optionnel).
    ///
    /// Absent pour les entrées d'audit antérieures à la v0.2 (rétro-compat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bins: Option<DistributionBins>,
}

/// Équivalent parlant pour l'utilisateur final.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Equivalent {
    /// Description courte.
    pub label: String,
    /// Valeur de l'équivalent.
    pub value: f64,
    /// Source documentaire.
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn interval_new_ok() {
        let i = UncertaintyInterval::new(1.5, 2.0, 2.7).unwrap();
        assert_eq!(i.p50, 2.0);
    }

    #[test]
    fn interval_new_rejects_nan() {
        assert!(UncertaintyInterval::new(f64::NAN, 1.0, 2.0).is_err());
        assert!(UncertaintyInterval::new(1.0, f64::INFINITY, 2.0).is_err());
    }

    #[test]
    fn interval_new_rejects_negative() {
        assert!(UncertaintyInterval::new(-0.1, 1.0, 2.0).is_err());
    }

    #[test]
    fn interval_new_rejects_unordered() {
        assert!(UncertaintyInterval::new(2.0, 1.0, 3.0).is_err());
        assert!(UncertaintyInterval::new(1.0, 3.0, 2.0).is_err());
    }

    #[test]
    fn interval_point() {
        let p = UncertaintyInterval::point(42.0).unwrap();
        assert_eq!(p.p5, 42.0);
        assert_eq!(p.p95, 42.0);
        assert_eq!(p.relative_width(), Some(0.0));
    }

    #[test]
    fn interval_relative_width_zero_p50() {
        let i = UncertaintyInterval::new(0.0, 0.0, 0.0).unwrap();
        assert_eq!(i.relative_width(), None);
    }

    #[test]
    fn interval_serializes_round_trip() {
        let i = UncertaintyInterval::new(1.5, 2.0, 2.7).unwrap();
        let json = serde_json::to_string(&i).unwrap();
        let back: UncertaintyInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(back, i);
    }

    #[test]
    fn indicator_default_units() {
        assert_eq!(Indicator::Co2Eq.default_unit(), "gCO2eq");
        assert_eq!(Indicator::Energy.default_unit(), "Wh");
        assert_eq!(Indicator::Water.default_unit(), "L");
    }

    #[test]
    fn indicator_serializes_snake_case() {
        let s = serde_json::to_string(&Indicator::Co2Eq).unwrap();
        assert_eq!(s, "\"co2_eq\"");
        let s2 = serde_json::to_string(&Indicator::CriticalMetals).unwrap();
        assert_eq!(s2, "\"critical_metals\"");
    }

    proptest! {
        #[test]
        fn prop_interval_invariants(
            p5 in 0.0_f64..1e9,
            offset_a in 0.0_f64..1e9,
            offset_b in 0.0_f64..1e9,
        ) {
            let p50 = p5 + offset_a;
            let p95 = p50 + offset_b;
            let i = UncertaintyInterval::new(p5, p50, p95).unwrap();
            prop_assert!(i.p5 <= i.p50);
            prop_assert!(i.p50 <= i.p95);
            prop_assert!(i.validate().is_ok());
        }
    }
}

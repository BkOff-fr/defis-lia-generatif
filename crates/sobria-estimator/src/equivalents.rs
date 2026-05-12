//! Conversions des résultats en équivalents parlants pour l'UI.
//!
//! Toutes les conversions sont sourcées (commentaire avec référence).
//! Voir CDC §4.2 (équivalents parlants).

use sobria_core::Equivalent;

/// Facteur d'émission moyen d'une voiture thermique européenne, en gCO₂eq/km.
/// Source : ADEME Base Empreinte 2024, voiture particulière essence moyenne.
const CAR_G_CO2EQ_PER_KM: f64 = 192.0;

/// Énergie typique d'une douche chaude de 5 min (8 L/min, ΔT=30°C, ~chauffe-eau électrique).
/// Source : ADEME — guide eau chaude sanitaire 2023.
const SHOWER_WH: f64 = 1_750.0;

/// Énergie typique d'un écran 24" pendant une heure (~25 W moyens).
/// Source : ADEME — guide écrans 2023.
const SCREEN_HOUR_WH: f64 = 25.0;

/// Convertit un résultat CO₂eq (en grammes) en équivalent kilomètres voiture.
#[must_use]
pub fn co2eq_to_car_km(g_co2eq: f64) -> Equivalent {
    Equivalent {
        label: "km en voiture thermique".into(),
        value: g_co2eq / CAR_G_CO2EQ_PER_KM,
        source: "ADEME Base Empreinte 2024 — voiture essence moyenne (192 gCO₂eq/km)".into(),
    }
}

/// Convertit un résultat énergie (en Wh) en équivalent secondes de douche chaude.
#[must_use]
pub fn energy_wh_to_shower_seconds(wh: f64) -> Equivalent {
    let ratio = wh / SHOWER_WH;
    Equivalent {
        label: "secondes de douche chaude (5 min ≈ 1 750 Wh)".into(),
        value: ratio * 300.0, // 5 min = 300 s
        source: "ADEME — guide ECS 2023, douche 5 min standard".into(),
    }
}

/// Convertit un résultat énergie (en Wh) en équivalent heures d'écran 24".
#[must_use]
pub fn energy_wh_to_screen_hours(wh: f64) -> Equivalent {
    Equivalent {
        label: "heures d'écran 24\" (~25 W)".into(),
        value: wh / SCREEN_HOUR_WH,
        source: "ADEME — guide écrans 2023, 25 W moyens".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_km_for_known_value() {
        // 192 gCO₂eq = 1 km
        let e = co2eq_to_car_km(192.0);
        assert!((e.value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn shower_seconds_for_known_value() {
        // 1 750 Wh = 300 s
        let e = energy_wh_to_shower_seconds(1_750.0);
        assert!((e.value - 300.0).abs() < 1e-9);
    }

    #[test]
    fn screen_hours_for_known_value() {
        // 25 Wh = 1 h
        let e = energy_wh_to_screen_hours(25.0);
        assert!((e.value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn equivalents_have_non_empty_source() {
        let e = co2eq_to_car_km(100.0);
        assert!(!e.source.is_empty());
        assert!(!e.label.is_empty());
    }
}

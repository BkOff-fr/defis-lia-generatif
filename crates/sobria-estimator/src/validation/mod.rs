//! Validation croisée du moteur Monte-Carlo.
//!
//! Voir `briefs/chantiers/C07-validation-croisee.md` et
//! `docs/methodology/VALIDATION-CROISEE.md`.
//!
//! Deux niveaux :
//! - [`PlausibilityCase`] : ordre de grandeur (plage large), toujours actif.
//! - [`ReproductionCase`] : reproduction stricte d'une valeur publiée à ±X %.

use chrono::Utc;
use serde::Serialize;
use sobria_core::{EstimationRequest, Indicator};

use crate::{
    distributions::Distribution, engine::MonteCarloEngine, engine_trait::EmpreinteMethod,
    engines::ecologits::EcoLogitsEngine, error::EstimatorResult, params::EstimationParams,
};

pub mod cases;

/// Cas de test "ordre de grandeur" (plage large).
#[derive(Debug, Clone, Copy)]
pub struct PlausibilityCase {
    /// Identifiant stable du cas.
    pub id: &'static str,
    /// Description humaine.
    pub description: &'static str,
    /// Modèle (preset registry).
    pub model_id: &'static str,
    /// Tokens d'entrée.
    pub tokens_in: u32,
    /// Tokens de sortie.
    pub tokens_out: u32,
    /// Facteur d'émission électrique (gCO₂eq/kWh).
    pub if_electrical_g_per_kwh: f64,
    /// Plage attendue (min, max) en gCO₂eq pour le P50.
    pub expected_range_g_co2eq: (f64, f64),
    /// Référence documentaire (paper, doc EcoLogits, etc.).
    pub reference: &'static str,
}

/// Cas de test "reproduction stricte" (±X % d'une valeur publiée).
#[derive(Debug, Clone, Copy)]
pub struct ReproductionCase {
    /// Identifiant stable.
    pub id: &'static str,
    /// Méthodologie ciblée. Le test invoque le moteur correspondant via
    /// la factory [`crate::engines::factory::engine_for`].
    pub method: EmpreinteMethod,
    /// DOI ou URL de la source.
    pub source_doi_or_url: &'static str,
    /// Modèle visé.
    pub model_id: &'static str,
    /// Tokens d'entrée.
    pub tokens_in: u32,
    /// Tokens de sortie.
    pub tokens_out: u32,
    /// Facteur d'émission électrique utilisé dans la référence.
    pub if_electrical_g_per_kwh: f64,
    /// PUE utilisé dans la référence.
    pub pue: f64,
    /// P50 cible (gCO₂eq).
    pub expected_p50_g_co2eq: f64,
    /// Tolérance relative (ex: 0.15 pour ±15 %).
    pub tolerance: f64,
    /// Si `true`, désactive la composante embodied dans notre estimateur
    /// (Point 0). Utile pour comparaisons usage-only entre méthodologies
    /// qui allouent l'embodied différemment.
    pub disable_embodied: bool,
    /// Notes méthodologiques.
    pub notes: &'static str,
}

/// Type d'un cas validé.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationKind {
    /// Test d'ordre de grandeur.
    Plausibility,
    /// Test de reproduction stricte.
    Reproduction,
}

/// Rapport d'un cas validé.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    /// Identifiant du cas.
    pub case_id: String,
    /// Type de cas.
    pub kind: ValidationKind,
    /// `true` si le test passe.
    pub passed: bool,
    /// P50 calculé par le moteur (gCO₂eq).
    pub computed_p50_g_co2eq: f64,
    /// Valeur attendue (selon le type de cas) — bornée ou ponctuelle.
    pub expected: Expectation,
    /// Message humain résumant le résultat.
    pub message: String,
}

/// Valeur attendue.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expectation {
    /// Plage [min, max] (plausibility).
    Range {
        /// Borne basse.
        min: f64,
        /// Borne haute.
        max: f64,
    },
    /// Valeur ponctuelle ± tolérance relative.
    PointWithinTolerance {
        /// Valeur cible.
        target: f64,
        /// Tolérance relative.
        tolerance: f64,
    },
}

/// Helper : construit un `EstimationRequest` pour un cas.
fn request_for(model_id: &str, tokens_in: u32, tokens_out: u32) -> EstimationRequest {
    EstimationRequest {
        model_id: model_id.into(),
        tokens_in,
        tokens_out_estimated: tokens_out,
        datacenter_id: None,
        timestamp: Utc::now(),
    }
}

/// Exécute un cas de plausibilité.
pub fn run_plausibility(case: &PlausibilityCase) -> EstimatorResult<ValidationReport> {
    let params =
        EstimationParams::for_model(case.model_id)?.with_if_electrical(Distribution::Point {
            value: case.if_electrical_g_per_kwh,
        });
    let engine = MonteCarloEngine::new(42).with_n(2_000);
    let result = engine.estimate(
        &request_for(case.model_id, case.tokens_in, case.tokens_out),
        &params,
    )?;
    let co2eq = result
        .indicators
        .iter()
        .find(|i| i.indicator == Indicator::Co2Eq)
        .ok_or_else(|| crate::error::EstimatorError::Other("CO2eq absent du résultat".into()))?;
    let p50 = co2eq.interval.p50;
    let (min, max) = case.expected_range_g_co2eq;
    let passed = p50 >= min && p50 <= max;
    let message = if passed {
        format!("{} : OK ({p50:.6} ∈ [{min}, {max}] g CO₂eq)", case.id)
    } else {
        format!(
            "{} : KO — P50={p50:.6} hors plage [{min}, {max}] g CO₂eq",
            case.id
        )
    };
    Ok(ValidationReport {
        case_id: case.id.into(),
        kind: ValidationKind::Plausibility,
        passed,
        computed_p50_g_co2eq: p50,
        expected: Expectation::Range { min, max },
        message,
    })
}

/// Exécute un cas de reproduction stricte.
///
/// Le moteur utilisé est déterminé par `case.method` :
/// - `EmpreinteMethod::AfnorSobria` → [`MonteCarloEngine`] (10⁴ tirages)
/// - `EmpreinteMethod::EcoLogits`   → [`EcoLogitsEngine`] (déterministe)
pub fn run_reproduction(case: &ReproductionCase) -> EstimatorResult<ValidationReport> {
    let mut params = EstimationParams::for_model(case.model_id)?
        .with_if_electrical(Distribution::Point {
            value: case.if_electrical_g_per_kwh,
        })
        .with_pue(Distribution::Point { value: case.pue });
    if case.disable_embodied {
        // Comparaison usage-only : on neutralise notre amortissement
        // embodied pour rester homogène avec la valeur cible (qui n'inclut
        // pas l'embodied ou l'inclut via une formule différente).
        params = params.with_embodied(Distribution::Point { value: 0.0 });
    }
    let request = request_for(case.model_id, case.tokens_in, case.tokens_out);
    let result = match case.method {
        EmpreinteMethod::AfnorSobria => {
            let engine = MonteCarloEngine::new(42).with_n(10_000);
            engine.estimate(&request, &params)?
        },
        EmpreinteMethod::EcoLogits => {
            let engine = EcoLogitsEngine::new(42);
            engine.estimate(&request, &params)?
        },
    };
    let co2eq = result
        .indicators
        .iter()
        .find(|i| i.indicator == Indicator::Co2Eq)
        .ok_or_else(|| crate::error::EstimatorError::Other("CO2eq absent du résultat".into()))?;
    let p50 = co2eq.interval.p50;
    let relative_error = (p50 - case.expected_p50_g_co2eq).abs() / case.expected_p50_g_co2eq;
    let passed = relative_error <= case.tolerance;
    let message = if passed {
        format!(
            "{} : OK (P50={p50:.4}, cible={:.4}, err={:.1}%, tol={:.1}%)",
            case.id,
            case.expected_p50_g_co2eq,
            relative_error * 100.0,
            case.tolerance * 100.0
        )
    } else {
        format!(
            "{} : KO — P50={p50:.4}, cible={:.4}, err={:.1}% > tol={:.1}%",
            case.id,
            case.expected_p50_g_co2eq,
            relative_error * 100.0,
            case.tolerance * 100.0
        )
    };
    Ok(ValidationReport {
        case_id: case.id.into(),
        kind: ValidationKind::Reproduction,
        passed,
        computed_p50_g_co2eq: p50,
        expected: Expectation::PointWithinTolerance {
            target: case.expected_p50_g_co2eq,
            tolerance: case.tolerance,
        },
        message,
    })
}

/// Exécute tous les cas de plausibilité connus.
#[must_use]
pub fn run_all_plausibility() -> Vec<ValidationReport> {
    cases::PLAUSIBILITY_CASES
        .iter()
        .filter_map(|c| run_plausibility(c).ok())
        .collect()
}

/// Exécute tous les cas de reproduction connus.
#[must_use]
pub fn run_all_reproduction() -> Vec<ValidationReport> {
    cases::REPRODUCTION_CASES
        .iter()
        .filter_map(|c| run_reproduction(c).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_plausibility_cases_pass() {
        for case in cases::PLAUSIBILITY_CASES {
            let report = run_plausibility(case)
                .unwrap_or_else(|e| panic!("erreur de validation pour {} : {e}", case.id));
            assert!(report.passed, "{}", report.message);
        }
    }

    #[test]
    fn run_all_plausibility_returns_at_least_five() {
        let reports = run_all_plausibility();
        assert!(
            reports.len() >= 5,
            "attendu ≥ 5 rapports, reçu {}",
            reports.len()
        );
        for r in &reports {
            assert!(r.passed, "{}", r.message);
        }
    }

    #[test]
    fn at_least_three_reproduction_cases_registered() {
        assert!(
            cases::REPRODUCTION_CASES.len() >= 3,
            "attendu ≥ 3 cas de reproduction, reçu {}",
            cases::REPRODUCTION_CASES.len()
        );
    }

    /// Vérifie que chaque [`ReproductionCase`] passe sa tolérance contre
    /// le moteur ciblé par `case.method`.
    ///
    /// Pour les cas `EcoLogits` : le port Rust doit reproduire les
    /// formules officielles à <1 % (port direct → l'écart attendu est
    /// purement numérique float64).
    #[test]
    fn all_reproduction_cases_pass() {
        for case in cases::REPRODUCTION_CASES {
            let report = run_reproduction(case)
                .unwrap_or_else(|e| panic!("erreur de validation pour {} : {e}", case.id));
            assert!(report.passed, "{}", report.message);
        }
    }

    #[test]
    fn all_reproduction_cases_cite_a_doi_or_url() {
        for case in cases::REPRODUCTION_CASES {
            let s = case.source_doi_or_url;
            assert!(
                s.starts_with("https://") || s.starts_with("doi:") || s.starts_with("10."),
                "cas {} : source non citée correctement ({s:?})",
                case.id
            );
        }
    }
}

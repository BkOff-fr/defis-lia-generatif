//! Distributions de probabilité utilisées par le moteur Monte-Carlo.
//!
//! Voir ADR-0004 §"Distributions par paramètre" et CDC §9.2.
//!
//! Toutes les distributions échantillonnent des valeurs **positives ou
//! nulles** (les variables physiques traitées sont toujours ≥ 0).

use rand::Rng;
use rand_distr::{Distribution as RandDistribution, LogNormal as RdLogNormal, Normal as RdNormal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{EstimatorError, EstimatorResult};

/// Z-score correspondant au quantile 0.95 de la loi normale standard.
/// Utilisé par le calibrage log-normale depuis un intervalle P5-P95.
const Z_95: f64 = 1.644_853_626_951_472_7;

/// Distribution paramétrable pour Monte-Carlo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Distribution {
    /// Valeur déterministe (sans incertitude).
    Point {
        /// La valeur retournée à chaque tirage.
        value: f64,
    },
    /// Uniforme bornée [low, high].
    Uniform {
        /// Borne inférieure.
        low: f64,
        /// Borne supérieure.
        high: f64,
    },
    /// Loi normale (tronquée à 0 si tirage négatif).
    Normal {
        /// Moyenne.
        mean: f64,
        /// Écart-type.
        std: f64,
    },
    /// Loi log-normale, paramétrée par `μ` et `σ` de la log.
    LogNormal {
        /// Paramètre μ (moyenne de la log).
        mu: f64,
        /// Paramètre σ (écart-type de la log).
        sigma: f64,
    },
}

impl Distribution {
    /// Tire une valeur dans la distribution.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match *self {
            Self::Point { value } => value,
            Self::Uniform { low, high } => rng.gen_range(low..=high),
            Self::Normal { mean, std } => {
                let dist = RdNormal::new(mean, std).expect("Normal std > 0 garanti par validate");
                dist.sample(rng).max(0.0)
            },
            Self::LogNormal { mu, sigma } => {
                let dist =
                    RdLogNormal::new(mu, sigma).expect("LogNormal sigma > 0 garanti par validate");
                dist.sample(rng)
            },
        }
    }

    /// Construit une `LogNormal` calibrée pour passer par `p50` (médiane)
    /// et dont l'intervalle P5-P95 approche `[p5, p95]`.
    ///
    /// Formule : `μ = ln(p50)`, `σ = ln(p95/p5) / (2 × z_{0.95})`
    /// avec `z_{0.95} ≈ 1.6449`.
    pub fn log_normal_from_interval(p5: f64, p50: f64, p95: f64) -> EstimatorResult<Self> {
        if p5 <= 0.0 || p50 <= 0.0 || p95 <= 0.0 {
            return Err(EstimatorError::Schema(format!(
                "log_normal_from_interval : quantiles doivent être > 0 ; p5={p5}, p50={p50}, p95={p95}"
            )));
        }
        if !(p5 <= p50 && p50 <= p95) {
            return Err(EstimatorError::Schema(format!(
                "log_normal_from_interval : ordre p5 ≤ p50 ≤ p95 violé ; p5={p5}, p50={p50}, p95={p95}"
            )));
        }
        let mu = p50.ln();
        let sigma = (p95 / p5).ln() / (2.0 * Z_95);
        if sigma <= 0.0 {
            return Err(EstimatorError::Schema(
                "log_normal_from_interval : σ calculé ≤ 0 (p5 = p95 ?)".into(),
            ));
        }
        Ok(Self::LogNormal { mu, sigma })
    }

    /// Vérifie les invariants de la distribution.
    pub fn validate(&self) -> EstimatorResult<()> {
        match *self {
            Self::Point { value } => {
                if !value.is_finite() {
                    return Err(EstimatorError::Schema(format!(
                        "Point : valeur non finie ({value})"
                    )));
                }
                Ok(())
            },
            Self::Uniform { low, high } => {
                if !(low.is_finite() && high.is_finite()) {
                    return Err(EstimatorError::Schema("Uniform : bornes non finies".into()));
                }
                if low > high {
                    return Err(EstimatorError::Schema(format!(
                        "Uniform : low={low} > high={high}"
                    )));
                }
                Ok(())
            },
            Self::Normal { mean: _, std } => {
                if !std.is_finite() || std <= 0.0 {
                    return Err(EstimatorError::Schema(format!(
                        "Normal : std doit être > 0 (reçu {std})"
                    )));
                }
                Ok(())
            },
            Self::LogNormal { mu, sigma } => {
                if !mu.is_finite() {
                    return Err(EstimatorError::Schema(format!(
                        "LogNormal : μ non fini ({mu})"
                    )));
                }
                if !sigma.is_finite() || sigma <= 0.0 {
                    return Err(EstimatorError::Schema(format!(
                        "LogNormal : σ doit être > 0 (reçu {sigma})"
                    )));
                }
                Ok(())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};

    #[test]
    fn point_always_returns_same_value() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        let d = Distribution::Point { value: 2.5 };
        for _ in 0..100 {
            assert!((d.sample(&mut rng) - 2.5).abs() < 1e-12);
        }
    }

    #[test]
    fn uniform_in_bounds() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
        let d = Distribution::Uniform {
            low: 1.0,
            high: 2.0,
        };
        for _ in 0..1000 {
            let v = d.sample(&mut rng);
            assert!((1.0..=2.0).contains(&v));
        }
    }

    #[test]
    fn normal_truncated_to_zero() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
        let d = Distribution::Normal {
            mean: 0.0,
            std: 10.0,
        };
        let mut hits = 0;
        for _ in 0..1000 {
            if d.sample(&mut rng) == 0.0 {
                hits += 1;
            }
        }
        assert!(hits > 100, "{hits} tirages tronqués à 0 (attendu ≥ 100)");
    }

    #[test]
    fn log_normal_strictly_positive() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(99);
        let d = Distribution::LogNormal {
            mu: 0.0,
            sigma: 1.0,
        };
        for _ in 0..1000 {
            assert!(d.sample(&mut rng) > 0.0);
        }
    }

    #[test]
    fn log_normal_from_interval_passes_through_median() {
        let d = Distribution::log_normal_from_interval(1.0, 2.0, 4.0).unwrap();
        if let Distribution::LogNormal { mu, sigma } = d {
            assert!((mu - 2.0_f64.ln()).abs() < 1e-12);
            let expected_sigma = (4.0_f64).ln() / (2.0 * Z_95);
            assert!((sigma - expected_sigma).abs() < 1e-12);
        } else {
            panic!("Attendu LogNormal");
        }
    }

    #[test]
    fn log_normal_from_interval_rejects_invalid() {
        assert!(Distribution::log_normal_from_interval(-1.0, 2.0, 4.0).is_err());
        assert!(Distribution::log_normal_from_interval(1.0, 0.5, 4.0).is_err());
        assert!(Distribution::log_normal_from_interval(1.0, 1.0, 1.0).is_err());
    }

    #[test]
    fn validate_catches_bad_params() {
        assert!(Distribution::Point { value: f64::NAN }.validate().is_err());
        assert!(Distribution::Uniform {
            low: 5.0,
            high: 1.0
        }
        .validate()
        .is_err());
        assert!(Distribution::Normal {
            mean: 0.0,
            std: 0.0
        }
        .validate()
        .is_err());
        assert!(Distribution::Normal {
            mean: 0.0,
            std: -1.0
        }
        .validate()
        .is_err());
        assert!(Distribution::LogNormal {
            mu: 0.0,
            sigma: 0.0
        }
        .validate()
        .is_err());
    }

    #[test]
    fn serde_round_trip() {
        for d in [
            Distribution::Point { value: 1.2 },
            Distribution::Uniform {
                low: 0.5,
                high: 1.5,
            },
            Distribution::Normal {
                mean: 1.0,
                std: 0.1,
            },
            Distribution::LogNormal {
                mu: 0.0,
                sigma: 0.5,
            },
        ] {
            let json = serde_json::to_string(&d).unwrap();
            let back: Distribution = serde_json::from_str(&json).unwrap();
            assert_eq!(back, d);
        }
    }
}

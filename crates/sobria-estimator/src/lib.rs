//! # sobria-estimator
//!
//! Moteur de calcul scientifique de Sobr.ia — Monte-Carlo N=10⁴ pour la
//! formule AFNOR SPEC 2314 (voir ADR-0004).
//!
//! ## Exemple
//!
//! ```no_run
//! use sobria_core::EstimationRequest;
//! use sobria_estimator::{EstimationParams, MonteCarloEngine};
//! use chrono::Utc;
//!
//! let request = EstimationRequest {
//!     model_id: "gpt-4o-mini".into(),
//!     tokens_in: 100,
//!     tokens_out_estimated: 500,
//!     datacenter_id: None,
//!     timestamp: Utc::now(),
//! };
//! let params = EstimationParams::for_model("gpt-4o-mini").unwrap();
//! let engine = MonteCarloEngine::default();
//! let result = engine.estimate(&request, &params).unwrap();
//! assert_eq!(result.indicators.len(), 3);
//! ```

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]

pub mod distributions;
pub mod engine;
pub mod equivalents;
pub mod error;
pub mod model_presets;
pub mod params;
pub mod validation;

pub use distributions::Distribution;
pub use engine::{MonteCarloEngine, DEFAULT_N};
pub use error::{EstimatorError, EstimatorResult};
pub use model_presets::{
    available_models, find_preset, CalibrationStatus, ModelPreset, Openness, MODEL_REGISTRY,
};
pub use params::EstimationParams;
pub use validation::{
    run_all_plausibility, run_all_reproduction, run_plausibility, run_reproduction, Expectation,
    PlausibilityCase, ReproductionCase, ValidationKind, ValidationReport,
};

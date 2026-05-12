//! # sobria-core
//!
//! Types et traits partagés par les crates de l'écosystème Sobr.ia.
//!
//! Crate fondation, **sans** dépendance lourde (pas de runtime async, pas de
//! HTTP, pas de base de données). Tous les types publics sont sérialisables
//! et porteurs d'un `JsonSchema` pour validation cross-language.
//!
//! Voir :
//! - [`docs/CAHIER-DES-CHARGES-v1.0.md`](../../docs/CAHIER-DES-CHARGES-v1.0.md)
//! - [`docs/adr/ADR-0001-rust-tauri.md`](../../docs/adr/ADR-0001-rust-tauri.md)
//! - [`docs/adr/ADR-0004-monte-carlo.md`](../../docs/adr/ADR-0004-monte-carlo.md)
//!
//! ## Exemple
//!
//! ```
//! use sobria_core::{Indicator, IndicatorValue, UncertaintyInterval};
//!
//! let interval = UncertaintyInterval::new(1.68, 2.14, 2.74).unwrap();
//! let value = IndicatorValue {
//!     indicator: Indicator::Co2Eq,
//!     interval,
//!     unit: "gCO2eq".into(),
//! };
//! assert_eq!(value.interval.p50, 2.14);
//! ```

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod datacenter;
pub mod emission;
pub mod error;
pub mod estimation;
pub mod indicators;
pub mod model;
pub mod validation;

pub use datacenter::Datacenter;
pub use emission::EmissionFactor;
pub use error::{SobriaError, SobriaResult};
pub use estimation::{EstimationRequest, EstimationResult, Hypothesis};
pub use indicators::{Equivalent, Indicator, IndicatorValue, UncertaintyInterval};
pub use model::{Modality, Model, ModelProvider};
pub use validation::{validate_country_iso, validate_year};

/// Version courante du schéma de référentiel — voir [`crate::validation`].
pub const SCHEMA_VERSION: &str = "1";

/// Seed Monte-Carlo par défaut — voir
/// [`docs/adr/ADR-0004-monte-carlo.md`](../../docs/adr/ADR-0004-monte-carlo.md).
pub const DEFAULT_SEED: u64 = 42;

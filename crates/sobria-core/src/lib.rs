//! # sobria-core
//!
//! Types et traits partagés par les crates de l'écosystème Sobr.ia.
//!
//! Voir le cahier des charges (`docs/CAHIER-DES-CHARGES-v1.0.md`) et les ADR.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod indicators;
pub mod model;

pub use error::{SobriaError, SobriaResult};
pub use indicators::{Equivalent, Indicator, IndicatorValue, UncertaintyInterval};
pub use model::{Model, ModelProvider, Modality};

/// Version courante du schéma de référentiel.
pub const SCHEMA_VERSION: &str = "1";

/// Seed Monte-Carlo par défaut — voir ADR-0004.
pub const DEFAULT_SEED: u64 = 42;

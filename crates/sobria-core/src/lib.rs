//! # sobria-core
//!
//! Types et traits partagés par les crates de l'écosystème Sobr.ia.
//!
//! Crate fondation, **sans** dépendance lourde (pas de runtime async, pas de
//! HTTP, pas de base de données). Tous les types publics sont sérialisables
//! et porteurs d'un `JsonSchema` pour validation cross-language.
//!
//! Voir :
//! - `docs/CAHIER-DES-CHARGES-v1.0.md`
//! - `docs/adr/ADR-0001-rust-tauri.md`
//! - `docs/adr/ADR-0004-monte-carlo.md`

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]

pub mod datacenter;
pub mod emission;
pub mod error;
pub mod estimation;
pub mod indicators;
pub mod model;
pub mod preferences;
pub mod validation;

pub use datacenter::Datacenter;
pub use emission::EmissionFactor;
pub use error::{SobriaError, SobriaResult};
pub use estimation::{EstimationRequest, EstimationResult, Hypothesis};
pub use indicators::{
    DistributionBins, Equivalent, Indicator, IndicatorValue, UncertaintyInterval,
};
pub use model::{Modality, Model, ModelProvider};
pub use preferences::{ModuleId, Persona};
pub use validation::{validate_country_iso, validate_year};

/// Version courante du schéma de référentiel.
pub const SCHEMA_VERSION: &str = "1";

/// Seed Monte-Carlo par défaut — voir ADR-0004.
pub const DEFAULT_SEED: u64 = 42;

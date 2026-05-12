//! # sobria-ingest
//!
//! Pipeline médaillon Sobr.ia — voir
//! [ADR-0009](../../docs/adr/ADR-0009-medallion-architecture.md).
//!
//! Trois couches automatiquement orchestrées via le trait [`DataLayer`] :
//!
//! - 🟫 **Copper** — récupération brute, immutable, datée, hashée.
//! - 🥈 **Silver** — Parquet typé, validé, schéma versionné.
//! - 🥇 **Gold** — `referentiel.sqlite` + `analytics.parquet` consommés par l'app.
//!
//! ## Modules
//!
//! - [`hash`] : SHA-256 streaming (gros fichiers).
//! - [`manifest`] : `manifest.json` Copper, format stable.
//! - [`context`] : contexte d'exécution du pipeline.
//! - [`layer`] : trait `DataLayer`.
//! - [`registry`] : orchestrateur des sources.
//! - [`error`] : erreurs publiques de la crate.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod context;
pub mod download;
pub mod error;
pub mod hash;
pub mod layer;
pub mod manifest;
pub mod registry;

pub use context::Context;
pub use error::{IngestError, IngestResult};
pub use layer::{
    CopperSnapshot, DataLayer, GoldContribution, SilverEntity, SourceMeta,
};
pub use manifest::{CopperManifest, ManifestFileEntry};
pub use registry::LayerRegistry;

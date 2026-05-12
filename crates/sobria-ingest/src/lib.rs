//! # sobria-ingest
//!
//! Pipeline médaillon Sobr.ia — voir [ADR-0009](../../docs/adr/ADR-0009-medallion-architecture.md).
//!
//! Trois couches automatiquement orchestrées via le trait [`DataLayer`] :
//!
//! - 🟫 **Copper** — récupération brute, immutable, datée, hashée.
//! - 🥈 **Silver** — Parquet typé, validé, schéma versionné.
//! - 🥇 **Gold** — `referentiel.sqlite` + `analytics.parquet` consommés par l'app.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod context;
pub mod layer;
pub mod registry;

// pub mod sources;  // chaque source aura son module (à implémenter en S2-S3)

pub use context::Context;
pub use layer::{
    CopperSnapshot, DataLayer, GoldContribution, SilverEntity, SourceMeta,
};
pub use registry::LayerRegistry;

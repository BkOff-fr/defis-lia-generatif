//! # sobria-ingest
//!
//! Pipeline médaillon Sobr.ia — voir ADR-0009.
//!
//! Trois couches : Copper → Silver → Gold, orchestrées via le trait
//! [`DataLayer`].

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::duration_suboptimal_units)]
#![allow(clippy::needless_pass_by_value)]

pub mod context;
pub mod download;
pub mod error;
pub mod gold;
pub mod hash;
pub mod layer;
pub mod lineage;
pub mod manifest;
pub mod registry;
pub mod sources;

pub use context::Context;
pub use download::{DownloadOutcome, DownloadStatus, Downloader};
pub use error::{IngestError, IngestResult};
pub use gold::{assemble_gold, GoldArtifacts};
pub use layer::{
    CopperSnapshot, DataLayer, GoldContribution, HealthReport, SilverEntity, SourceMeta,
};
pub use lineage::{CopperRef, GoldLineage, SilverLineage};
pub use manifest::{CopperManifest, ManifestFileEntry};
pub use registry::{LayerRegistry, PipelineReport, StepResult};
pub use sources::{
    discover_datasets, fetch_industrial_sites, fetch_rte_mix, write_artifact_json, ComparIASource,
    DatasetMatch, IndustrialSite, RegionMeta, RteIrisSource, RteMixArtifact, RteMixSourceTotals,
    TerritoireFrArtifact,
};

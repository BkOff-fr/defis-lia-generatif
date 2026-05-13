//! Sources concrètes du pipeline médaillon.

pub mod comparia;
pub mod rte_iris;
pub mod territoire_fr;

pub use comparia::ComparIASource;
pub use rte_iris::RteIrisSource;
pub use territoire_fr::{
    discover_datasets, fetch_industrial_sites, fetch_rte_mix, write_artifact_json, DatasetMatch,
    IndustrialSite, RegionMeta, RteMixArtifact, RteMixSourceTotals, TerritoireFrArtifact,
};

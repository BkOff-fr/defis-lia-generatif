//! # sobria-geoloc
//!
//! Géolocalisation et données territoriales pour Sobr.ia.
//!
//! - Dataset embarqué des 28 datacenters européens (voir [`datacenters`]).
//! - Loader du dataset RTE/NaTran/Teréga « sites industriels par IRIS »
//!   (voir [`territoire_fr`]).
//! - Génération du Sankey énergétique national à partir du mix RTE eco2mix
//!   (voir [`sankey_fr`]).
//!
//! Voir CDC §7.1 (M12 / M20) et `briefs/chantiers/C12-*` + `C13-*`.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

pub mod datacenters;
pub mod sankey_fr;
pub mod territoire_fr;

pub use datacenters::{
    aggregate_by_country, all_datacenters, find_datacenter, CountryAggregate, DatacenterRecord,
};
pub use sankey_fr::{
    generate_sankey_fr, load_rte_mix, RteMixArtifact, RteMixMeta, RteMixSourceTotals, SankeyData,
    SankeyFrError, SankeyFrResult, SankeyLink, SankeyNode,
};
pub use territoire_fr::{
    aggregate_by_region, find_site_by_code_iris, load_territoire_fr, ArtifactMeta, IndustrialSite,
    IndustrialSiteSummary, RegionFrAggregate, RegionMeta, TerritoireFrArtifact, TerritoireFrError,
    TerritoireFrResult,
};

/// Version de la crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

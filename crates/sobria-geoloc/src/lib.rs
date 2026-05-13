//! # sobria-geoloc
//!
//! Géolocalisation des datacenters et helpers cartographiques de Sobr.ia.
//!
//! - Dataset embarqué des 28 datacenters européens (voir [`datacenters`]).
//! - Agrégation par pays pour la vue dézoomée.
//! - Profils horaires 24h (ENTSO-E) joints au moment du chargement.
//!
//! Voir CDC §7.1 (M12) et `briefs/chantiers/C12-datacenters-europe.md`.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_precision_loss)]

pub mod datacenters;

pub use datacenters::{
    aggregate_by_country, all_datacenters, find_datacenter, CountryAggregate, DatacenterRecord,
};

/// Version de la crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

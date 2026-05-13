//! # sobria-export
//!
//! Génération de rapports PDF et JSON-LD PROV-O pour la conformité CSRD,
//! AGEC et AFNOR SPEC 2314.
//!
//! Voir `briefs/chantiers/C14-rapport-csrd-agec.md` et CDC v1.3 §4 M22.
//!
//! **Politique** : toutes les valeurs proviennent de l'audit ledger
//! Sobr.ia (entrées chaînées SHA-256, déjà produites par l'estimateur
//! Monte-Carlo). Aucune donnée n'est synthétisée par ce module.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

pub mod datasheet;
pub mod error;
pub mod provo;
pub mod report;
pub mod summary;

pub use datasheet::{
    build_datasheet, Composition, DatasheetArtifact, DatasheetOptions, ProjectMeta,
};
pub use error::{ExportError, ExportResult};
pub use provo::{build_provo_jsonld, ProvOOptions};
pub use report::{generate_report, ReportArtifacts, ReportRequest};
pub use summary::ReportSummary;

/// Version de la crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

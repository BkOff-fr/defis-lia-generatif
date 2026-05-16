//! Exports du `sobria-team-aggregator` (C28.5).
//!
//! Trois formats à destination de l'admin :
//!
//! - [`csrd`]  : PDF AFNOR SPEC 2314 + sidecar PROV-O via `sobria-export`
//!   (les estimations team sont converties en `AuditEntry` shims).
//! - [`prov_o`]: variant team-specific du PROV-O (per-user `prov:Agent`,
//!   per-estimation `prov:Activity`).
//! - [`csv`]   : dump RFC 4180 plat de la fenêtre demandée.

pub mod csrd;
pub mod csv;
pub mod prov_o;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Paramètres communs des 3 exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    /// Nom d'organisation à mettre dans l'en-tête PDF + PROV-O.
    #[serde(default)]
    pub entity_name: Option<String>,
    /// Anonymise les fingerprints/display_names dans PROV-O + CSV
    /// (le PDF agrégé ne dévoile pas d'identité par défaut).
    #[serde(default)]
    pub anonymize: bool,
}

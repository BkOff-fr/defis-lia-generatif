//! # sobria-audit
//!
//! Ledger ACID SQLite WAL chaîné SHA-256 pour la traçabilité réglementaire
//! (voir CDC §5.7 et `briefs/chantiers/C08-audit-ledger.md`).
//!
//! ## Exemple
//!
//! ```no_run
//! # fn _doc() -> Result<(), Box<dyn std::error::Error>> {
//! use sobria_audit::AuditLedger;
//! use sobria_core::EstimationResult;
//!
//! let mut ledger = AuditLedger::open(std::path::Path::new("audit.sqlite"))?;
//! # let result: EstimationResult = todo!();
//! let entry = ledger.append(&result)?;
//! let report = ledger.verify_chain()?;
//! assert!(report.valid);
//! # Ok(()) }
//! ```

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]

pub mod entry;
pub mod error;
pub mod ledger;

pub use entry::{AuditEntry, PURGED_SENTINEL};
pub use error::{AuditError, AuditResult};
pub use ledger::{AuditLedger, IntegrityReport};

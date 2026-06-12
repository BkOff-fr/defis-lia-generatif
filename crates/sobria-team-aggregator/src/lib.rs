//! Sobr.ia — `sobria-team-aggregator` (binaire HTTPS self-hosted).
//!
//! Voir ADR-0013 Phase 2 et `briefs/chantiers/C28-mode-equipe-self-hosted.md`.
//! C28.1 (cet incrément) livre le bootstrap : crate + CLI `init`/`serve`,
//! TLS auto-signé via rcgen, schéma SQLite v1, admin initial Argon2id et
//! route `/health`. Les API d'auth, d'admin et d'analytics sont prévues
//! dans les sous-chantiers suivants (C28.2 → C28.5).

#![forbid(unsafe_code)]

pub mod alerts;
pub mod cli;
pub mod commands;
pub mod config;
pub mod crypto;
pub mod error;
pub mod exports;
pub mod policy;
pub mod server;
pub mod storage;

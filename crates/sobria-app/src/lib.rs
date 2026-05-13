//! # sobria-app — façade IPC Tauri
//!
//! Cette bibliothèque expose le **cœur métier** sous forme de fonctions
//! testables, indépendamment du runtime Tauri. Le binaire `sobria-app`
//! enregistre ces fonctions comme commandes IPC via `#[tauri::command]`,
//! mais les fonctions internes (`logic::*`) sont **directement appelables
//! depuis les tests** sans démarrer la fenêtre native.
//!
//! Voir `briefs/chantiers/C09-tauri-integration.md`.

#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_pass_by_value)]

pub mod batch;
pub mod dashboard;
pub mod dto;
pub mod error;
pub mod goals_store;
pub mod logic;
pub mod preferences_store;
pub mod project_store;
pub mod state;

pub use error::{IpcError, IpcResult};
pub use state::AppState;

//! Commandes CLI du `sobria-team-aggregator`.
//!
//! - [`init`]  : prépare un data dir vierge (DB + cert + admin).
//! - [`serve`] : lance le serveur HTTPS axum.
//! - [`code`]  : crée / liste / révoque des enrollment codes (C28.2).
//! - [`admin`] : reset-password + list (C29.2).

pub mod admin;
pub mod code;
pub mod config;
pub mod init;
pub mod serve;

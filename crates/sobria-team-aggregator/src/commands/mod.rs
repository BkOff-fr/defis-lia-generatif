//! Commandes CLI du `sobria-team-aggregator`.
//!
//! - [`init`] : prépare un data dir vierge (DB + cert + admin).
//! - [`serve`] : lance le serveur HTTPS axum.
//! - [`code`] : crée / liste / révoque des enrollment codes (C28.2).
//!
//! D'autres commandes (`user`, `admin reset-password`) viendront avec C28.3+.

pub mod code;
pub mod init;
pub mod serve;

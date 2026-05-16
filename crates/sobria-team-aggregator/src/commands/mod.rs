//! Commandes CLI du `sobria-team-aggregator`.
//!
//! - [`init`] : prépare un data dir vierge (DB + cert + admin).
//! - [`serve`] : lance le serveur HTTPS axum.
//!
//! D'autres commandes (`code`, `user`, `admin`) viendront avec C28.2 → C28.3.

pub mod init;
pub mod serve;

//! Primitives cryptographiques du `sobria-team-aggregator`.
//!
//! - [`password`] : Argon2id PHC (passwords admin + futurs users).
//! - [`secret`]   : OS RNG → hex (JWT signing key, futurs refresh tokens).
//! - [`tls`]      : génération de cert auto-signé via `rcgen` (backend ring).

pub mod password;
pub mod secret;
pub mod tls;

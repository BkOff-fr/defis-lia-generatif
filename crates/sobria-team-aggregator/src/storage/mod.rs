//! Couche persistance du `sobria-team-aggregator`.
//!
//! [`Storage`] encapsule une `rusqlite::Connection` mono-thread (WAL +
//! foreign_keys activés). Le multi-thread `Send` est garanti par axum via
//! `Arc<Mutex<Storage>>` (la connexion n'est pas `Sync`). Pour les charges
//! C28.2 ce verrou global suffit ; on passera à un pool en C28.4 si les
//! benchs le demandent.
//!
//! Les helpers par entité vivent dans les submodules (`admins`,
//! `enrollment_codes`, `users`, `tokens`, `estimations`) sous forme de
//! fonctions libres prenant `&Connection`. La struct `Storage` ne porte
//! que l'ouverture/migrations + l'accès au KV `config`.

pub mod admins;
pub mod analytics;
pub mod enrollment_codes;
pub mod estimations;
pub mod schema;
pub mod tokens;
pub mod users;

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AggregatorResult;

/// Façade SQLite mono-connexion (cf. doc du module).
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Ouvre `team.sqlite` (le crée si absent) et applique le schéma v1.
    pub fn open(path: &Path) -> AggregatorResult<Self> {
        let conn = Connection::open(path)?;
        schema::install(&conn)?;
        Ok(Self { conn })
    }

    /// Ouvre une base en mémoire (tests).
    pub fn open_in_memory() -> AggregatorResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::install(&conn)?;
        Ok(Self { conn })
    }

    /// Accès partagé en lecture-écriture à la connexion pour les helpers
    /// par entité (`users::insert(storage.connection(), …)`).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Lit une valeur du KV `config`.
    pub fn get_config(&self, key: &str) -> AggregatorResult<Option<String>> {
        let value = self
            .conn
            .query_row(
                "SELECT value FROM config WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(value)
    }

    /// Pose une valeur du KV `config` (upsert).
    pub fn set_config(&self, key: &str, value: &str) -> AggregatorResult<()> {
        self.conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_set_get_roundtrip() {
        let s = Storage::open_in_memory().unwrap();
        assert!(s.get_config("jwt_secret").unwrap().is_none());
        s.set_config("jwt_secret", "deadbeef").unwrap();
        assert_eq!(
            s.get_config("jwt_secret").unwrap().as_deref(),
            Some("deadbeef")
        );
        s.set_config("jwt_secret", "cafebabe").unwrap();
        assert_eq!(
            s.get_config("jwt_secret").unwrap().as_deref(),
            Some("cafebabe")
        );
    }
}

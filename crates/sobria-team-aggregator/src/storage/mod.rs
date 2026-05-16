//! Couche persistance du `sobria-team-aggregator`.
//!
//! Cette couche enveloppe une `rusqlite::Connection` mono-thread (WAL activé)
//! pour l'API REST et la CLI. Pour C28.1, on n'expose que :
//!
//! - [`Storage::open`] : ouvre + applique les migrations v1.
//! - [`Storage::set_config`] / [`Storage::get_config`] : KV store interne.
//! - [`Storage::insert_admin`] : crée l'admin initial (Argon2id PHC déjà calculé).
//! - [`Storage::admin_count`] / [`Storage::find_admin_by_username`] : utilitaires.
//!
//! Les helpers utilisateurs / tokens / estimations / analytics arrivent dans
//! les sous-chantiers C28.2 → C28.5.

pub mod schema;

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AggregatorResult;

/// DTO d'admin tel que stocké en base.
#[derive(Debug, Clone)]
pub struct AdminRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Façade SQLite mono-connexion. Le multi-thread `Send` est garanti par axum
/// via `Arc<Mutex<Storage>>` (la connexion n'est pas `Sync`).
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
    #[cfg(test)]
    pub fn open_in_memory() -> AggregatorResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::install(&conn)?;
        Ok(Self { conn })
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

    /// Insère un nouvel admin (le hash Argon2id PHC doit être pré-calculé).
    pub fn insert_admin(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
        created_at: DateTime<Utc>,
    ) -> AggregatorResult<()> {
        self.conn.execute(
            "INSERT INTO admins (id, username, password_hash, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, username, password_hash, created_at.to_rfc3339()],
        )?;
        Ok(())
    }

    /// Nombre total d'admins (sert au quickstart pour savoir s'il faut en créer un).
    pub fn admin_count(&self) -> AggregatorResult<u32> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM admins", [], |row| row.get(0))?;
        Ok(n.max(0) as u32)
    }

    /// Cherche un admin par username.
    pub fn find_admin_by_username(&self, username: &str) -> AggregatorResult<Option<AdminRow>> {
        let row = self
            .conn
            .query_row(
                "SELECT id, username, password_hash, created_at, last_login_at
                 FROM admins WHERE username = ?1",
                params![username],
                |row| {
                    let created_at_s: String = row.get(3)?;
                    let last_login_s: Option<String> = row.get(4)?;
                    Ok(AdminRow {
                        id: row.get(0)?,
                        username: row.get(1)?,
                        password_hash: row.get(2)?,
                        created_at: DateTime::parse_from_rfc3339(&created_at_s)
                            .map_err(|e| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    3,
                                    rusqlite::types::Type::Text,
                                    Box::new(e),
                                )
                            })?
                            .with_timezone(&Utc),
                        last_login_at: match last_login_s {
                            Some(s) => Some(
                                DateTime::parse_from_rfc3339(&s)
                                    .map_err(|e| {
                                        rusqlite::Error::FromSqlConversionFailure(
                                            4,
                                            rusqlite::types::Type::Text,
                                            Box::new(e),
                                        )
                                    })?
                                    .with_timezone(&Utc),
                            ),
                            None => None,
                        },
                    })
                },
            )
            .optional()?;
        Ok(row)
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

    #[test]
    fn admin_insert_and_lookup() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(s.admin_count().unwrap(), 0);

        s.insert_admin("a-1", "alice", "$argon2id$fake-phc", Utc::now())
            .unwrap();

        assert_eq!(s.admin_count().unwrap(), 1);
        let row = s.find_admin_by_username("alice").unwrap().unwrap();
        assert_eq!(row.id, "a-1");
        assert_eq!(row.password_hash, "$argon2id$fake-phc");
        assert!(row.last_login_at.is_none());

        assert!(s.find_admin_by_username("bob").unwrap().is_none());
    }

    #[test]
    fn admin_username_is_unique() {
        let s = Storage::open_in_memory().unwrap();
        s.insert_admin("a-1", "alice", "h", Utc::now()).unwrap();
        let dup = s.insert_admin("a-2", "alice", "h", Utc::now());
        assert!(dup.is_err());
    }
}

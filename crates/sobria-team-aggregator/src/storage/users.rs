//! Helpers SQL pour la table `users`.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AggregatorResult;

/// Employé enrôlé. `password_hash` est un PHC Argon2id (cf. `crypto::password`).
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub enrollment_code_id: Option<String>,
    pub fingerprint: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Insère un nouvel utilisateur (le hash Argon2id doit être pré-calculé).
pub fn insert(
    conn: &Connection,
    id: &str,
    enrollment_code_id: Option<&str>,
    fingerprint: &str,
    password_hash: &str,
    display_name: Option<&str>,
    created_at: DateTime<Utc>,
) -> AggregatorResult<()> {
    conn.execute(
        "INSERT INTO users
            (id, enrollment_code_id, fingerprint, password_hash, display_name, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            enrollment_code_id,
            fingerprint,
            password_hash,
            display_name,
            created_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Cherche un user par id (ULID).
pub fn find_by_id(conn: &Connection, id: &str) -> AggregatorResult<Option<UserRow>> {
    let row = conn
        .query_row(
            "SELECT id, enrollment_code_id, fingerprint, password_hash,
                    display_name, created_at, last_seen_at
             FROM users WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

/// Cherche un user par fingerprint (unicité garantie par le schéma).
pub fn find_by_fingerprint(
    conn: &Connection,
    fingerprint: &str,
) -> AggregatorResult<Option<UserRow>> {
    let row = conn
        .query_row(
            "SELECT id, enrollment_code_id, fingerprint, password_hash,
                    display_name, created_at, last_seen_at
             FROM users WHERE fingerprint = ?1",
            params![fingerprint],
            map_row,
        )
        .optional()?;
    Ok(row)
}

/// Met à jour `last_seen_at` (idempotent).
pub fn touch_last_seen(conn: &Connection, id: &str, now: DateTime<Utc>) -> AggregatorResult<()> {
    conn.execute(
        "UPDATE users SET last_seen_at = ?1 WHERE id = ?2",
        params![now.to_rfc3339(), id],
    )?;
    Ok(())
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<UserRow> {
    let created_at_s: String = row.get(5)?;
    let last_seen_s: Option<String> = row.get(6)?;
    Ok(UserRow {
        id: row.get(0)?,
        enrollment_code_id: row.get(1)?,
        fingerprint: row.get(2)?,
        password_hash: row.get(3)?,
        display_name: row.get(4)?,
        created_at: parse_ts(&created_at_s, 5)?,
        last_seen_at: last_seen_s.map(|s| parse_ts(&s, 6)).transpose()?,
    })
}

fn parse_ts(s: &str, col: usize) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    #[test]
    fn insert_and_lookup_by_fingerprint() {
        let s = Storage::open_in_memory().unwrap();
        insert(
            s.connection(),
            "u-1",
            None,
            "chrome-mac-abc",
            "$argon2id$h",
            Some("Alice"),
            Utc::now(),
        )
        .unwrap();
        let row = find_by_fingerprint(s.connection(), "chrome-mac-abc")
            .unwrap()
            .unwrap();
        assert_eq!(row.id, "u-1");
        assert_eq!(row.display_name.as_deref(), Some("Alice"));
        assert!(find_by_fingerprint(s.connection(), "unknown")
            .unwrap()
            .is_none());
    }

    #[test]
    fn fingerprint_is_unique() {
        let s = Storage::open_in_memory().unwrap();
        insert(s.connection(), "u-1", None, "fp", "h", None, Utc::now()).unwrap();
        let dup = insert(s.connection(), "u-2", None, "fp", "h", None, Utc::now());
        assert!(dup.is_err());
    }
}

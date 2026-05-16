//! Helpers SQL pour la table `users`.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::AggregatorResult;
use crate::storage::estimations::UsageTotals;

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

/// Vue user + totaux d'usage (admin liste).
#[derive(Debug, Clone, Serialize)]
pub struct UserWithTotals {
    pub id: String,
    pub fingerprint: String,
    pub display_name: Option<String>,
    pub enrollment_code_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub totals: UsageTotals,
}

/// Liste tous les users avec leurs totaux (LEFT JOIN estimations).
/// Tri par dernière activité décroissante (user actif en haut).
pub fn list_all_with_totals(conn: &Connection) -> AggregatorResult<Vec<UserWithTotals>> {
    let mut stmt = conn.prepare(
        "SELECT u.id, u.fingerprint, u.display_name, u.enrollment_code_id,
                u.created_at, u.last_seen_at,
                COUNT(e.id),
                COALESCE(SUM(e.tokens_in), 0),
                COALESCE(SUM(e.tokens_out), 0),
                COALESCE(SUM(e.gco2eq_p50), 0.0),
                COALESCE(SUM(e.water_ml), 0.0),
                COALESCE(SUM(e.energy_wh), 0.0)
         FROM users u
         LEFT JOIN estimations e ON e.user_id = u.id
         GROUP BY u.id, u.fingerprint, u.display_name, u.enrollment_code_id,
                  u.created_at, u.last_seen_at
         ORDER BY COALESCE(u.last_seen_at, u.created_at) DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let created_at_s: String = row.get(4)?;
        let last_seen_s: Option<String> = row.get(5)?;
        Ok(UserWithTotals {
            id: row.get(0)?,
            fingerprint: row.get(1)?,
            display_name: row.get(2)?,
            enrollment_code_id: row.get(3)?,
            created_at: parse_ts(&created_at_s, 4)?,
            last_seen_at: last_seen_s.map(|s| parse_ts(&s, 5)).transpose()?,
            totals: UsageTotals {
                count: row.get::<_, i64>(6)?.max(0) as u64,
                tokens_in: row.get::<_, i64>(7)?.max(0) as u64,
                tokens_out: row.get::<_, i64>(8)?.max(0) as u64,
                gco2eq_p50_g: row.get(9)?,
                water_ml: row.get(10)?,
                energy_wh: row.get(11)?,
            },
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
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

    #[test]
    fn list_all_with_totals_includes_users_without_estimations() {
        use crate::storage::estimations;
        let s = Storage::open_in_memory().unwrap();
        insert(
            s.connection(),
            "u-1",
            None,
            "fp-1",
            "h",
            Some("Alice"),
            Utc::now(),
        )
        .unwrap();
        insert(s.connection(), "u-2", None, "fp-2", "h", None, Utc::now()).unwrap();

        // u-1 a 2 estimations, u-2 aucune.
        for (id, g) in [("e-1", 0.4), ("e-2", 0.6)] {
            let est = estimations::NewEstimation {
                id,
                user_id: "u-1",
                ts: Utc::now(),
                method: "afnor_sobria",
                model_id: "m",
                tokens_in: 10,
                tokens_out: 20,
                gco2eq_p50: g,
                gco2eq_p5: None,
                gco2eq_p95: None,
                water_ml: 0.0,
                energy_wh: 0.0,
                region: None,
                raw_payload_json: "{}",
                received_at: Utc::now(),
            };
            estimations::insert(s.connection(), &est).unwrap();
        }

        let rows = list_all_with_totals(s.connection()).unwrap();
        assert_eq!(rows.len(), 2);

        let by_id: std::collections::HashMap<_, _> =
            rows.iter().map(|r| (r.id.clone(), r.clone())).collect();
        assert_eq!(by_id["u-1"].totals.count, 2);
        assert!((by_id["u-1"].totals.gco2eq_p50_g - 1.0).abs() < 1e-9);
        assert_eq!(by_id["u-2"].totals.count, 0);
        assert_eq!(by_id["u-2"].totals.gco2eq_p50_g, 0.0);
    }
}

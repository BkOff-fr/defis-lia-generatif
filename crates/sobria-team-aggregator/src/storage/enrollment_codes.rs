//! Helpers SQL pour la table `enrollment_codes`.
//!
//! Pattern d'usage :
//!
//! 1. Un admin appelle [`create_batch`] avec N codes 12 chiffres + un TTL.
//!    Chaque code est hashé Argon2id (cf. `crypto::password`) et inséré.
//!    L'admin reçoit en clair UNE seule fois la liste (pour distribution).
//! 2. À l'enroll, [`verify_active_code`] itère tous les codes encore valides
//!    et vérifie le PHC. Coût O(N×argon2). Acceptable pour N≤quelques milliers
//!    (les /enroll restent rares dans une vie d'équipe).
//! 3. [`mark_used`] verrouille le code (single-use) en posant `used_at` +
//!    `used_by_user_id`.
//! 4. [`revoke`] permet à l'admin d'annuler un code non utilisé.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::crypto::password::{self, verify_password};
use crate::error::AggregatorResult;

/// Vue d'un code en base (le clear text n'est jamais accessible).
#[derive(Debug, Clone)]
pub struct EnrollmentCodeRow {
    pub id: String,
    pub code_hash: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by_user_id: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl EnrollmentCodeRow {
    /// `true` si le code est utilisable pour enroll (non utilisé, non révoqué,
    /// non expiré).
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.used_at.is_none() && self.revoked_at.is_none() && self.expires_at > now
    }
}

/// Code en clair + métadonnées renvoyés par [`create_batch`] (à présenter
/// à l'admin UNE seule fois).
#[derive(Debug, Clone)]
pub struct CreatedCode {
    pub id: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

/// Insère un seul code (`code` en clair, hashé Argon2id avant insertion).
pub fn insert(
    conn: &Connection,
    id: &str,
    code: &str,
    created_by: &str,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> AggregatorResult<()> {
    let hash = password::hash_password(code)?;
    conn.execute(
        "INSERT INTO enrollment_codes
            (id, code_hash, created_by, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            hash,
            created_by,
            created_at.to_rfc3339(),
            expires_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Itère tous les codes actifs et retourne celui qui matche `candidate`.
///
/// O(N) en nombre de codes actifs (l'iter Argon2 est lent : ~50 ms/verify).
/// La verif retourne le row complet ; à l'appelant de poser `mark_used` avant
/// d'émettre des tokens.
pub fn verify_active_code(
    conn: &Connection,
    candidate: &str,
    now: DateTime<Utc>,
) -> AggregatorResult<Option<EnrollmentCodeRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, code_hash, created_by, created_at, expires_at,
                used_at, used_by_user_id, revoked_at
         FROM enrollment_codes
         WHERE used_at IS NULL
           AND revoked_at IS NULL
           AND expires_at > ?1",
    )?;
    let rows = stmt.query_map(params![now.to_rfc3339()], map_row)?;
    for row in rows {
        let row = row?;
        if verify_password(&row.code_hash, candidate) {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

/// Marque un code comme consommé (single-use). Renvoie `false` si le code
/// est introuvable, déjà utilisé, ou révoqué (transaction safe).
pub fn mark_used(
    conn: &Connection,
    code_id: &str,
    user_id: &str,
    now: DateTime<Utc>,
) -> AggregatorResult<bool> {
    let n = conn.execute(
        "UPDATE enrollment_codes
         SET used_at = ?1, used_by_user_id = ?2
         WHERE id = ?3
           AND used_at IS NULL
           AND revoked_at IS NULL",
        params![now.to_rfc3339(), user_id, code_id],
    )?;
    Ok(n == 1)
}

/// Révoque un code (admin → empêche enroll futur).
pub fn revoke(conn: &Connection, code_id: &str, now: DateTime<Utc>) -> AggregatorResult<bool> {
    let n = conn.execute(
        "UPDATE enrollment_codes
         SET revoked_at = ?1
         WHERE id = ?2 AND revoked_at IS NULL",
        params![now.to_rfc3339(), code_id],
    )?;
    Ok(n == 1)
}

/// Liste tous les codes (admin view).
pub fn list_all(conn: &Connection) -> AggregatorResult<Vec<EnrollmentCodeRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, code_hash, created_by, created_at, expires_at,
                used_at, used_by_user_id, revoked_at
         FROM enrollment_codes
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Cherche un code par id (utilisé par CLI `code revoke`).
pub fn find_by_id(conn: &Connection, id: &str) -> AggregatorResult<Option<EnrollmentCodeRow>> {
    let row = conn
        .query_row(
            "SELECT id, code_hash, created_by, created_at, expires_at,
                    used_at, used_by_user_id, revoked_at
             FROM enrollment_codes WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<EnrollmentCodeRow> {
    let created_at_s: String = row.get(3)?;
    let expires_at_s: String = row.get(4)?;
    let used_at_s: Option<String> = row.get(5)?;
    let revoked_at_s: Option<String> = row.get(7)?;
    Ok(EnrollmentCodeRow {
        id: row.get(0)?,
        code_hash: row.get(1)?,
        created_by: row.get(2)?,
        created_at: parse_ts(&created_at_s, 3)?,
        expires_at: parse_ts(&expires_at_s, 4)?,
        used_at: used_at_s.map(|s| parse_ts(&s, 5)).transpose()?,
        used_by_user_id: row.get(6)?,
        revoked_at: revoked_at_s.map(|s| parse_ts(&s, 7)).transpose()?,
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
    use crate::storage::{admins, Storage};
    use chrono::Duration;

    fn setup_admin(s: &Storage) -> String {
        admins::insert(s.connection(), "a-1", "admin", "h", Utc::now()).unwrap();
        "a-1".to_string()
    }

    #[test]
    fn insert_and_verify_active_code() {
        let s = Storage::open_in_memory().unwrap();
        let admin_id = setup_admin(&s);
        let now = Utc::now();
        let exp = now + Duration::days(7);

        insert(s.connection(), "c-1", "384729104857", &admin_id, now, exp).unwrap();

        // Le code correct est retrouvé.
        let row = verify_active_code(s.connection(), "384729104857", now)
            .unwrap()
            .expect("code trouvé");
        assert_eq!(row.id, "c-1");

        // Un mauvais code → None.
        let none = verify_active_code(s.connection(), "000000000000", now).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn expired_code_is_rejected() {
        let s = Storage::open_in_memory().unwrap();
        let admin_id = setup_admin(&s);
        let now = Utc::now();
        let exp = now - Duration::seconds(1);

        insert(
            s.connection(),
            "c-1",
            "111111111111",
            &admin_id,
            now - Duration::days(8),
            exp,
        )
        .unwrap();
        let none = verify_active_code(s.connection(), "111111111111", now).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn mark_used_makes_code_inactive() {
        let s = Storage::open_in_memory().unwrap();
        let admin_id = setup_admin(&s);
        let now = Utc::now();
        insert(
            s.connection(),
            "c-1",
            "222222222222",
            &admin_id,
            now,
            now + Duration::days(7),
        )
        .unwrap();

        // Crée un user factice (FK sur users non testée ici car FK off ne s'applique
        // pas à des id arbitraires si l'on désactive ; on garde le contrat dur).
        crate::storage::users::insert(
            s.connection(),
            "u-1",
            Some("c-1"),
            "chrome-mac-abc",
            "$argon2id$fake",
            None,
            now,
        )
        .unwrap();

        let ok = mark_used(s.connection(), "c-1", "u-1", now).unwrap();
        assert!(ok);

        // Code re-vérifié → plus actif.
        let none = verify_active_code(s.connection(), "222222222222", now).unwrap();
        assert!(none.is_none());

        // Re-mark_used du même code → false (single-use).
        let again = mark_used(s.connection(), "c-1", "u-1", now).unwrap();
        assert!(!again);
    }

    #[test]
    fn revoke_makes_code_inactive() {
        let s = Storage::open_in_memory().unwrap();
        let admin_id = setup_admin(&s);
        let now = Utc::now();
        insert(
            s.connection(),
            "c-1",
            "333333333333",
            &admin_id,
            now,
            now + Duration::days(7),
        )
        .unwrap();

        assert!(revoke(s.connection(), "c-1", now).unwrap());
        let none = verify_active_code(s.connection(), "333333333333", now).unwrap();
        assert!(none.is_none());
        // Re-revoke → false.
        assert!(!revoke(s.connection(), "c-1", now).unwrap());
    }
}

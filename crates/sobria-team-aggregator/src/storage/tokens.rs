//! Helpers SQL pour la table `tokens` (refresh tokens auth — admin OU user).
//!
//! ## Format selector + verifier
//!
//! Un refresh token exposé au client est une chaîne `<token_id>.<secret>` :
//!
//! - `token_id` (ULID) est l'`id` du row, exposé en clair → permet une
//!   lookup O(1) par index PRIMARY KEY.
//! - `secret` (UUID v4 hex 32 chars) est hashé Argon2id et stocké dans
//!   `refresh_token_hash`.
//!
//! La verif est donc O(1) : on cherche le row par `token_id`, puis
//! `argon2_verify(stored_hash, candidate_secret)`. Aucune itération.
//! Le `CHECK ((user_id IS NULL) <> (admin_id IS NULL))` au niveau SQL
//! garantit l'XOR rôle.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::crypto::password;
use crate::error::AggregatorResult;

/// Vue d'un refresh token en base.
#[derive(Debug, Clone)]
pub struct TokenRow {
    pub id: String,
    pub user_id: Option<String>,
    pub admin_id: Option<String>,
    pub refresh_token_hash: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl TokenRow {
    /// `true` si le token est utilisable (non révoqué, non expiré).
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}

/// Insère un refresh token (hash Argon2id pré-calculé) pour un user OU admin.
///
/// `user_id` et `admin_id` doivent être en XOR — le CHECK SQL le garantit.
pub fn insert(
    conn: &Connection,
    id: &str,
    user_id: Option<&str>,
    admin_id: Option<&str>,
    refresh_token_hash: &str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> AggregatorResult<()> {
    conn.execute(
        "INSERT INTO tokens
            (id, user_id, admin_id, refresh_token_hash, issued_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            user_id,
            admin_id,
            refresh_token_hash,
            issued_at.to_rfc3339(),
            expires_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Cherche un token par id (selector).
pub fn find_by_id(conn: &Connection, id: &str) -> AggregatorResult<Option<TokenRow>> {
    let row = conn
        .query_row(
            "SELECT id, user_id, admin_id, refresh_token_hash, issued_at,
                    expires_at, revoked_at
             FROM tokens WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

/// Révoque un token (rotation auto à chaque /refresh).
pub fn revoke(conn: &Connection, id: &str, now: DateTime<Utc>) -> AggregatorResult<bool> {
    let n = conn.execute(
        "UPDATE tokens SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
        params![now.to_rfc3339(), id],
    )?;
    Ok(n == 1)
}

/// Vérifie un refresh token au format `<id>.<secret>`. Retourne le row si OK
/// (encore actif + verif Argon2id OK), `None` sinon.
///
/// L'appelant est responsable de la rotation (révoquer ce row + émettre un nouveau).
pub fn verify_refresh_token(
    conn: &Connection,
    refresh_token: &str,
    now: DateTime<Utc>,
) -> AggregatorResult<Option<TokenRow>> {
    let Some((id, secret)) = refresh_token.split_once('.') else {
        return Ok(None);
    };
    let Some(row) = find_by_id(conn, id)? else {
        return Ok(None);
    };
    if !row.is_active(now) {
        return Ok(None);
    }
    if !password::verify_password(&row.refresh_token_hash, secret) {
        return Ok(None);
    }
    Ok(Some(row))
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<TokenRow> {
    let issued_at_s: String = row.get(4)?;
    let expires_at_s: String = row.get(5)?;
    let revoked_at_s: Option<String> = row.get(6)?;
    Ok(TokenRow {
        id: row.get(0)?,
        user_id: row.get(1)?,
        admin_id: row.get(2)?,
        refresh_token_hash: row.get(3)?,
        issued_at: parse_ts(&issued_at_s, 4)?,
        expires_at: parse_ts(&expires_at_s, 5)?,
        revoked_at: revoked_at_s.map(|s| parse_ts(&s, 6)).transpose()?,
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
    use crate::storage::{admins, users, Storage};
    use chrono::Duration;

    #[test]
    fn verify_round_trip_for_user() {
        let s = Storage::open_in_memory().unwrap();
        users::insert(s.connection(), "u-1", None, "fp", "h", None, Utc::now()).unwrap();

        let token_id = "01HZZZ00000000000000000001";
        let secret = "the-uuid-secret-part";
        let hash = password::hash_password(secret).unwrap();
        let now = Utc::now();
        let exp = now + Duration::days(7);
        insert(s.connection(), token_id, Some("u-1"), None, &hash, now, exp).unwrap();

        // Bon token → trouve.
        let combined = format!("{token_id}.{secret}");
        let row = verify_refresh_token(s.connection(), &combined, now)
            .unwrap()
            .expect("doit matcher");
        assert_eq!(row.user_id.as_deref(), Some("u-1"));

        // Mauvais secret → None.
        let wrong = format!("{token_id}.wrong");
        assert!(verify_refresh_token(s.connection(), &wrong, now)
            .unwrap()
            .is_none());

        // ID inconnu → None.
        let unknown = format!("not-a-real-id.{secret}");
        assert!(verify_refresh_token(s.connection(), &unknown, now)
            .unwrap()
            .is_none());

        // Token sans `.` → None.
        assert!(verify_refresh_token(s.connection(), "garbage", now)
            .unwrap()
            .is_none());
    }

    #[test]
    fn revoked_token_is_inactive() {
        let s = Storage::open_in_memory().unwrap();
        admins::insert(s.connection(), "a-1", "alice", "h", Utc::now()).unwrap();
        let hash = password::hash_password("secret-abc").unwrap();
        let now = Utc::now();
        insert(
            s.connection(),
            "t-1",
            None,
            Some("a-1"),
            &hash,
            now,
            now + Duration::days(7),
        )
        .unwrap();

        assert!(revoke(s.connection(), "t-1", now).unwrap());
        let combined = "t-1.secret-abc";
        let row = verify_refresh_token(s.connection(), combined, now).unwrap();
        assert!(row.is_none(), "le row révoqué ne doit pas vérifier");
    }

    #[test]
    fn expired_token_is_inactive() {
        let s = Storage::open_in_memory().unwrap();
        users::insert(s.connection(), "u-1", None, "fp", "h", None, Utc::now()).unwrap();
        let hash = password::hash_password("xx").unwrap();
        let issued = Utc::now() - Duration::days(8);
        let expires = Utc::now() - Duration::seconds(1);
        insert(
            s.connection(),
            "t-x",
            Some("u-1"),
            None,
            &hash,
            issued,
            expires,
        )
        .unwrap();
        assert!(verify_refresh_token(s.connection(), "t-x.xx", Utc::now())
            .unwrap()
            .is_none());
    }
}

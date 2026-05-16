//! Émission de refresh tokens (format `<token_id>.<secret>`).
//!
//! Voir `storage::tokens` pour le pattern selector + verifier.
//! TTL 7 jours (cf. brief C28.2).

use chrono::{DateTime, Duration, Utc};
use ulid::Ulid;
use uuid::Uuid;

use crate::crypto::password;
use crate::error::AggregatorResult;
use crate::storage::{tokens, Storage};

/// TTL des refresh tokens.
pub const REFRESH_TTL: Duration = Duration::days(7);

/// Token émis (à renvoyer au client). Le `combined` est la valeur opaque
/// à conserver côté client.
#[derive(Debug, Clone)]
pub struct IssuedRefreshToken {
    pub combined: String,
    pub id: String,
    pub expires_at: DateTime<Utc>,
}

/// Émet un refresh token pour un user OU admin (XOR garanti par le caller).
pub fn issue(
    storage: &Storage,
    user_id: Option<&str>,
    admin_id: Option<&str>,
    now: DateTime<Utc>,
) -> AggregatorResult<IssuedRefreshToken> {
    debug_assert!(user_id.is_some() ^ admin_id.is_some(), "user XOR admin");
    let token_id = Ulid::new().to_string();
    let secret = Uuid::new_v4().simple().to_string();
    let hash = password::hash_password(&secret)?;
    let expires_at = now + REFRESH_TTL;
    tokens::insert(
        storage.connection(),
        &token_id,
        user_id,
        admin_id,
        &hash,
        now,
        expires_at,
    )?;
    Ok(IssuedRefreshToken {
        combined: format!("{token_id}.{secret}"),
        id: token_id,
        expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{users, Storage};

    #[test]
    fn issue_round_trips_through_verify() {
        let s = Storage::open_in_memory().unwrap();
        users::insert(s.connection(), "u-1", None, "fp", "h", None, Utc::now()).unwrap();

        let now = Utc::now();
        let issued = issue(&s, Some("u-1"), None, now).unwrap();
        assert!(issued.combined.contains('.'));

        let row = tokens::verify_refresh_token(s.connection(), &issued.combined, now)
            .unwrap()
            .expect("doit vérifier");
        assert_eq!(row.user_id.as_deref(), Some("u-1"));
        assert!(row.admin_id.is_none());
    }
}

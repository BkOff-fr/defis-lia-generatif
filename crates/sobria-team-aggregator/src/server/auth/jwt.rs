//! Émission + vérification des access tokens JWT HS256.
//!
//! Claims : `{ sub, role, iat, exp }`. TTL 24 h (cf. brief C28.2).
//! Le secret HS256 est lu depuis `config.jwt_signing_key` (hex 64 chars,
//! posé à `init`).

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::server::error::ApiError;

/// TTL des access tokens (24 h).
pub const ACCESS_TTL: Duration = Duration::hours(24);

/// Rôle stocké dans le JWT (le serveur connaît la vérité — c'est juste un hint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
        }
    }
}

/// Claims signés.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Sujet : `user_id` (ULID) ou `admin_id` (ULID) selon `role`.
    pub sub: String,
    /// `"user"` | `"admin"`.
    pub role: Role,
    /// Issued at (Unix seconds).
    pub iat: i64,
    /// Expires at (Unix seconds).
    pub exp: i64,
}

/// Émet un access token JWT HS256.
pub fn issue(
    signing_key_hex: &str,
    sub: &str,
    role: Role,
    now: DateTime<Utc>,
) -> Result<String, ApiError> {
    let key = decode_hex(signing_key_hex)?;
    let claims = Claims {
        sub: sub.to_string(),
        role,
        iat: now.timestamp(),
        exp: (now + ACCESS_TTL).timestamp(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(&key))
        .map_err(|e| ApiError::InternalMsg(format!("jwt encode: {e}")))
}

/// Vérifie un access token et retourne ses claims si OK.
pub fn verify(signing_key_hex: &str, token: &str) -> Result<Claims, ApiError> {
    let key = decode_hex(signing_key_hex)?;
    let mut validation = Validation::default();
    // Pas de leeway : exp strict. Le client refresh dès qu'il reçoit 401.
    validation.leeway = 0;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(&key), &validation)
        .map_err(|_| ApiError::InvalidToken)?;
    Ok(data.claims)
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, ApiError> {
    if !hex.len().is_multiple_of(2) {
        return Err(ApiError::InternalMsg("jwt_signing_key hex impair".into()));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| ApiError::InternalMsg(format!("jwt_signing_key non-hex: {e}")))?;
        out.push(byte);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

    #[test]
    fn issue_then_verify_roundtrip() {
        let now = Utc::now();
        let token = issue(FAKE_KEY, "u-1", Role::User, now).unwrap();
        let claims = verify(FAKE_KEY, &token).unwrap();
        assert_eq!(claims.sub, "u-1");
        assert_eq!(claims.role, Role::User);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn verify_rejects_wrong_signing_key() {
        let other_key: String = "ff".repeat(32);
        let token = issue(FAKE_KEY, "u-1", Role::User, Utc::now()).unwrap();
        assert!(verify(&other_key, &token).is_err());
    }

    #[test]
    fn verify_rejects_expired_token() {
        // Émis très loin dans le passé → exp < now.
        let past = Utc::now() - Duration::hours(48);
        let token = issue(FAKE_KEY, "u-1", Role::User, past).unwrap();
        assert!(verify(FAKE_KEY, &token).is_err());
    }

    #[test]
    fn role_admin_roundtrips() {
        let token = issue(FAKE_KEY, "a-1", Role::Admin, Utc::now()).unwrap();
        let claims = verify(FAKE_KEY, &token).unwrap();
        assert_eq!(claims.role, Role::Admin);
    }
}

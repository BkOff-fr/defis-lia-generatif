//! Sobr.ia — pairing extension navigateur ↔ app desktop (C27.5.c).
//!
//! Logique pure du pairing par **code à 6 chiffres** TTL 5 min, single-use,
//! comparaison constant-time. Indépendant de SQLite et de Tauri — la
//! persistance des `device_pairings` viendra en C27.5.d quand on intégrera
//! le module à `AppState`.
//!
//! ## Sécurité
//!
//! - **Code 6 chiffres** : 20 bits d'entropie, OS RNG (`rand::thread_rng`).
//!   Suffisant pour usage local éphémère (TTL 5 min, single-use).
//! - **Secret partagé** : 32 octets random, hashé en SHA-256 + sel pour
//!   stockage. **TODO C27.6** : passer à Argon2id (params 15000/2/1) quand
//!   on ajoute la dépendance crypto. Pour l'instant SHA-256 + sel 16 octets
//!   est acceptable (le secret n'est pas un mot de passe humain, c'est
//!   un random 256-bit).
//! - **Comparaison code** : constant-time pour éviter le timing attack
//!   (même si le risque est faible sur du local).
//!
//! ## Usage prévu (intégration C27.5.d)
//!
//! ```ignore
//! // Côté UI (/parametres → Extension navigateur), bouton "Régénérer" :
//! let pairing_code = generate_pairing_code(&mut state.codes);
//! // affiche pairing_code.code (6 chiffres) à l'utilisateur pendant 5 min
//!
//! // Quand le bridge transmet le code saisi côté extension :
//! let (id, secret) = verify_pairing_code(&mut state.codes, &code, &fingerprint)?;
//! // → on insère dans device_pairings (id, fingerprint, hash(secret), now)
//! // → on retourne `secret` à l'extension via le bridge
//!
//! // À chaque estimation reçue :
//! let pairing = verify_secret(&db, &secret, &fingerprint)?;
//! // → on insère dans extension_events
//! ```

use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

/// TTL d'un code de pairing affiché côté app. Au-delà, il est régénéré.
pub const CODE_TTL: Duration = Duration::minutes(5);

/// Longueur du secret partagé en octets (256 bits, encodé hex → 64 chars).
pub const SECRET_BYTES: usize = 32;

/// Longueur du sel SHA-256 pour le hash du secret.
pub const SALT_BYTES: usize = 16;

/// Erreurs de la couche pairing.
#[derive(Debug, thiserror::Error)]
pub enum PairingError {
    #[error("code invalide ou expiré")]
    InvalidCode,
    #[error("code à 6 chiffres requis")]
    Malformed,
}

/// Code de pairing en attente d'utilisation (in-memory, jamais persisté).
#[derive(Debug, Clone)]
pub struct PendingCode {
    /// Les 6 chiffres affichés à l'utilisateur.
    pub code: String,
    /// Instant après lequel le code n'est plus accepté.
    pub expires_at: DateTime<Utc>,
}

impl PendingCode {
    /// Construit un code aléatoire 6 chiffres TTL 5 min.
    #[must_use]
    pub fn new() -> Self {
        Self::with_expiry(Utc::now() + CODE_TTL)
    }

    /// Construit un code avec une expiration donnée (utile pour les tests).
    #[must_use]
    pub fn with_expiry(expires_at: DateTime<Utc>) -> Self {
        let mut rng = thread_rng();
        // 6 chiffres en chaîne, padding 0 si nécessaire (ex: "042039").
        let n: u32 = rng.gen_range(0..1_000_000);
        Self {
            code: format!("{n:06}"),
            expires_at,
        }
    }

    /// `true` si le code est encore valide (non expiré).
    #[must_use]
    pub fn is_valid(&self, now: DateTime<Utc>) -> bool {
        now <= self.expires_at
    }
}

impl Default for PendingCode {
    fn default() -> Self {
        Self::new()
    }
}

/// Secret partagé entre l'extension et l'app desktop, généré à la validation
/// du code. Stocké en clair côté extension (chrome.storage.local sandbox),
/// hashé côté app (SHA-256 + sel pour v0.6.0, Argon2id en C27.6).
#[derive(Debug, Clone)]
pub struct PairingSecret {
    /// Secret 32 octets encodé en hex (64 caractères ASCII).
    pub secret_hex: String,
    /// Hash SHA-256 du secret, à stocker côté app.
    pub secret_hash: String,
    /// Sel utilisé pour le hash (16 octets hex).
    pub salt_hex: String,
}

impl PairingSecret {
    /// Génère un secret aléatoire + son hash.
    #[must_use]
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut secret = [0u8; SECRET_BYTES];
        rng.fill(&mut secret);
        let mut salt = [0u8; SALT_BYTES];
        rng.fill(&mut salt);
        Self {
            secret_hex: encode_hex(&secret),
            secret_hash: hash_with_salt(&secret, &salt),
            salt_hex: encode_hex(&salt),
        }
    }

    /// Vérifie qu'un secret en clair correspond à ce hash+sel (constant-time).
    ///
    /// Utilisé côté app pour authentifier chaque message ingéré du bridge.
    #[must_use]
    pub fn verify_against(stored_hash: &str, stored_salt_hex: &str, candidate_hex: &str) -> bool {
        let Some(candidate) = decode_hex(candidate_hex) else {
            return false;
        };
        let Some(salt) = decode_hex(stored_salt_hex) else {
            return false;
        };
        let recomputed = hash_with_salt(&candidate, &salt);
        constant_time_eq(recomputed.as_bytes(), stored_hash.as_bytes())
    }
}

impl Default for PairingSecret {
    fn default() -> Self {
        Self::new()
    }
}

/// Vérifie un code soumis contre un code attendu, en constant-time.
///
/// Retourne `Err(Malformed)` si la forme n'est pas 6 chiffres ASCII,
/// `Err(InvalidCode)` si la comparaison échoue ou si le code a expiré.
pub fn verify_code(
    pending: &PendingCode,
    submitted: &str,
    now: DateTime<Utc>,
) -> Result<(), PairingError> {
    if submitted.len() != 6 || !submitted.chars().all(|c| c.is_ascii_digit()) {
        return Err(PairingError::Malformed);
    }
    if !pending.is_valid(now) {
        return Err(PairingError::InvalidCode);
    }
    if constant_time_eq(submitted.as_bytes(), pending.code.as_bytes()) {
        Ok(())
    } else {
        Err(PairingError::InvalidCode)
    }
}

// ─── Helpers privés ──────────────────────────────────────────────────────────

/// SHA-256(secret || salt) — encodé hex.
fn hash_with_salt(secret: &[u8], salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    hasher.update(salt);
    encode_hex(&hasher.finalize())
}

/// Encodage hexadécimal lowercase.
fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Décodage hex lowercase (`None` si invalide).
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut iter = s.chars();
    while let (Some(a), Some(b)) = (iter.next(), iter.next()) {
        let hi = a.to_digit(16)?;
        let lo = b.to_digit(16)?;
        out.push(u8::try_from(hi * 16 + lo).ok()?);
    }
    Some(out)
}

/// Comparaison constant-time de deux slices d'octets.
///
/// Évite le timing attack même sur du local. Implémentation maison —
/// `subtle::ConstantTimeEq` ferait pareil mais ajoute une dépendance.
#[must_use]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn pending_code_is_6_digits() {
        let code = PendingCode::new();
        assert_eq!(code.code.len(), 6);
        assert!(code.code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn code_is_valid_just_after_creation() {
        let code = PendingCode::new();
        assert!(code.is_valid(Utc::now()));
    }

    #[test]
    fn code_is_invalid_after_ttl() {
        let past = Utc::now() - Duration::seconds(1);
        let code = PendingCode::with_expiry(past);
        assert!(!code.is_valid(Utc::now()));
    }

    #[test]
    fn verify_code_accepts_matching_code() {
        let code = PendingCode::with_expiry(Utc::now() + Duration::minutes(5));
        let res = verify_code(&code, &code.code, Utc::now());
        assert!(res.is_ok());
    }

    #[test]
    fn verify_code_rejects_wrong_code() {
        let code = PendingCode {
            code: "123456".into(),
            expires_at: Utc::now() + Duration::minutes(5),
        };
        let err = verify_code(&code, "654321", Utc::now()).unwrap_err();
        assert!(matches!(err, PairingError::InvalidCode));
    }

    #[test]
    fn verify_code_rejects_expired_code() {
        let code = PendingCode {
            code: "123456".into(),
            expires_at: Utc::now() - Duration::seconds(1),
        };
        let err = verify_code(&code, "123456", Utc::now()).unwrap_err();
        assert!(matches!(err, PairingError::InvalidCode));
    }

    #[test]
    fn verify_code_rejects_malformed() {
        let code = PendingCode {
            code: "123456".into(),
            expires_at: Utc::now() + Duration::minutes(5),
        };
        assert!(matches!(
            verify_code(&code, "abc123", Utc::now()),
            Err(PairingError::Malformed)
        ));
        assert!(matches!(
            verify_code(&code, "12345", Utc::now()),
            Err(PairingError::Malformed)
        ));
        assert!(matches!(
            verify_code(&code, "1234567", Utc::now()),
            Err(PairingError::Malformed)
        ));
    }

    #[test]
    fn pairing_secret_is_64_hex_chars() {
        let s = PairingSecret::new();
        assert_eq!(s.secret_hex.len(), 64);
        assert!(s.secret_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(s.salt_hex.len(), 32);
        assert_eq!(s.secret_hash.len(), 64); // SHA-256
    }

    #[test]
    fn verify_against_accepts_matching_secret() {
        let s = PairingSecret::new();
        assert!(PairingSecret::verify_against(
            &s.secret_hash,
            &s.salt_hex,
            &s.secret_hex
        ));
    }

    #[test]
    fn verify_against_rejects_wrong_secret() {
        let s = PairingSecret::new();
        // Modifie 1 caractère du secret → hash différent.
        let mut tampered = s.secret_hex.clone();
        tampered.replace_range(0..1, "f");
        // Si le 1er caractère était déjà 'f', on prend 'e' à la place.
        if tampered == s.secret_hex {
            tampered.replace_range(0..1, "e");
        }
        assert!(!PairingSecret::verify_against(
            &s.secret_hash,
            &s.salt_hex,
            &tampered
        ));
    }

    #[test]
    fn verify_against_rejects_malformed_hex() {
        let s = PairingSecret::new();
        assert!(!PairingSecret::verify_against(
            &s.secret_hash,
            &s.salt_hex,
            "not-hex"
        ));
    }

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn two_pending_codes_are_different() {
        // Avec 10^6 possibilités, la collision est rare. On en tire 5 et on
        // vérifie qu'au moins 4 sont distincts (test probabiliste).
        let codes: Vec<String> = (0..5).map(|_| PendingCode::new().code).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert!(
            unique.len() >= 4,
            "trop de collisions sur 5 tirages : {codes:?}"
        );
    }

    #[test]
    fn two_pairing_secrets_are_different() {
        let s1 = PairingSecret::new();
        let s2 = PairingSecret::new();
        assert_ne!(s1.secret_hex, s2.secret_hex);
        assert_ne!(s1.secret_hash, s2.secret_hash);
        assert_ne!(s1.salt_hex, s2.salt_hex);
    }
}

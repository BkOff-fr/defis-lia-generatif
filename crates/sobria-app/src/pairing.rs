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
//! - **Secret partagé** : 32 octets random, hashé en **Argon2id** (params
//!   par défaut de la crate `argon2 = 0.5`, déterminés par l'OWASP cheat
//!   sheet) pour stockage. Le hash retourné est une **PHC string** qui
//!   contient les paramètres + le sel + le hash en base64, et qui se
//!   suffit à elle-même côté `verify_password`. Patch C27 v0.6.0 —
//!   remplace l'implémentation SHA-256 + sel manuel précédente.
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

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};

/// TTL d'un code de pairing affiché côté app. Au-delà, il est régénéré.
pub const CODE_TTL: Duration = Duration::minutes(5);

/// Longueur du secret partagé en octets (256 bits, encodé hex → 64 chars).
pub const SECRET_BYTES: usize = 32;

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
/// du code. Stocké en clair côté extension (`chrome.storage.local` sandbox),
/// hashé côté app en **Argon2id** (PHC string) depuis le patch C27 v0.6.0.
///
/// La PHC string (`$argon2id$v=19$m=...,t=...,p=...$<salt-b64>$<hash-b64>`)
/// embarque déjà ses paramètres et son sel, donc on n'a **plus besoin** de
/// stocker `salt_hex` séparément en base.
#[derive(Debug, Clone)]
pub struct PairingSecret {
    /// Secret 32 octets encodé en hex (64 caractères ASCII).
    pub secret_hex: String,
    /// Hash Argon2id du secret au format PHC, à stocker côté app dans
    /// `device_pairings.secret_hash`.
    pub secret_hash: String,
}

impl PairingSecret {
    /// Génère un secret aléatoire + son hash Argon2id.
    ///
    /// Panique uniquement si l'OS RNG sous-jacent à `SaltString::generate`
    /// échoue (ce qui implique un OS dans un état non récupérable).
    #[must_use]
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut secret = [0u8; SECRET_BYTES];
        rng.fill(&mut secret);
        let secret_hex = encode_hex(&secret);
        let secret_hash = hash_secret(&secret_hex);
        Self {
            secret_hex,
            secret_hash,
        }
    }

    /// Vérifie qu'un secret en clair (hex) correspond à un hash Argon2id PHC
    /// stocké. La comparaison est constant-time côté `argon2`.
    ///
    /// Retourne `false` si le hash est malformé (PHC invalide) ou si le
    /// secret candidat ne matche pas.
    #[must_use]
    pub fn verify_against(stored_phc: &str, candidate_hex: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(stored_phc) else {
            return false;
        };
        Argon2::default()
            .verify_password(candidate_hex.as_bytes(), &parsed)
            .is_ok()
    }
}

impl Default for PairingSecret {
    fn default() -> Self {
        Self::new()
    }
}

/// `true` si `stored_hash` ressemble à un hash Argon2id PHC (préfixe
/// `$argon2id$`). Utilisé par la migration v2 → v3 pour repérer et
/// révoquer les anciens hashs SHA-256 hex.
#[must_use]
pub fn is_argon2id_phc(stored_hash: &str) -> bool {
    stored_hash.starts_with("$argon2id$")
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

/// Hash Argon2id (PHC string) d'un secret en clair. Sel généré par OS RNG.
///
/// Les paramètres laissés par défaut côté `argon2 = 0.5` (m=19456, t=2, p=1)
/// correspondent à la recommandation OWASP — voir
/// <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id>.
fn hash_secret(secret_hex: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret_hex.as_bytes(), &salt)
        .expect("argon2 hash : OS RNG indisponible (état non récupérable)")
        .to_string()
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
    fn pairing_secret_is_64_hex_chars_with_argon2id_phc() {
        let s = PairingSecret::new();
        assert_eq!(s.secret_hex.len(), 64);
        assert!(s.secret_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            is_argon2id_phc(&s.secret_hash),
            "secret_hash doit être une PHC Argon2id : {}",
            s.secret_hash
        );
    }

    #[test]
    fn verify_against_accepts_matching_secret() {
        let s = PairingSecret::new();
        assert!(PairingSecret::verify_against(&s.secret_hash, &s.secret_hex));
    }

    #[test]
    fn verify_against_rejects_wrong_secret() {
        let s = PairingSecret::new();
        // Modifie 1 caractère du secret → la vérif Argon2 doit échouer.
        let mut tampered = s.secret_hex.clone();
        tampered.replace_range(0..1, "f");
        if tampered == s.secret_hex {
            tampered.replace_range(0..1, "e");
        }
        assert!(!PairingSecret::verify_against(&s.secret_hash, &tampered));
    }

    #[test]
    fn verify_against_rejects_malformed_phc() {
        // Un hash legacy SHA-256+sel n'est pas une PHC string Argon2 → false.
        let s = PairingSecret::new();
        assert!(!PairingSecret::verify_against(
            "not-a-phc-string",
            &s.secret_hex
        ));
        assert!(!PairingSecret::verify_against(
            "0123456789abcdef".repeat(4).as_str(),
            &s.secret_hex
        ));
    }

    #[test]
    fn is_argon2id_phc_detects_legacy_sha256() {
        // Migration v2 → v3 : les anciens hashs SHA-256 hex (64 chars) ne
        // doivent jamais être confondus avec une PHC Argon2id.
        assert!(!is_argon2id_phc(&"a".repeat(64)));
        assert!(is_argon2id_phc(
            "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$Zm9v"
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
        assert_ne!(
            s1.secret_hash, s2.secret_hash,
            "Argon2 doit produire deux PHC strings différentes (sels distincts)"
        );
    }
}

//! Hash + vérification de mots de passe (Argon2id PHC).
//!
//! On reprend exactement le pattern de `sobria-app::pairing` : paramètres
//! par défaut d'`argon2 = 0.5` (OWASP : m=19456, t=2, p=1), sel généré par
//! l'OS RNG, hash retourné en PHC string auto-portante. Aucune dépendance
//! à OpenSSL.
//!
//! Voir <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id>.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::{AggregatorError, AggregatorResult};

/// Hash Argon2id (PHC string) d'un mot de passe.
pub fn hash_password(password: &str) -> AggregatorResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AggregatorError::Crypto(format!("argon2 hash: {e}")))?
        .to_string();
    Ok(phc)
}

/// `true` si `candidate` correspond à `stored_phc`.
///
/// Renvoie `false` si la PHC est malformée — pas d'erreur explicite pour
/// éviter d'exposer la cause d'un échec d'auth.
#[must_use]
pub fn verify_password(stored_phc: &str, candidate: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(candidate.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrip() {
        let phc = hash_password("hunter2-correct-horse").unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(verify_password(&phc, "hunter2-correct-horse"));
        assert!(!verify_password(&phc, "wrong-password"));
    }

    #[test]
    fn verify_rejects_malformed_phc() {
        assert!(!verify_password("not-a-phc", "anything"));
        assert!(!verify_password("", "anything"));
    }

    #[test]
    fn two_hashes_of_same_password_differ() {
        let a = hash_password("same").unwrap();
        let b = hash_password("same").unwrap();
        assert_ne!(a, b, "sels distincts => PHC distincts");
        assert!(verify_password(&a, "same"));
        assert!(verify_password(&b, "same"));
    }
}

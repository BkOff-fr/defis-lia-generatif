//! Validateurs simples réutilisés à travers le projet.

use crate::error::{SobriaError, SobriaResult};

/// Vérifie qu'une chaîne est un code pays ISO 3166-1 alpha-2.
///
/// # Erreurs
///
/// Retourne `SchemaValidation` si la chaîne ne fait pas exactement 2
/// caractères majuscules ASCII.
pub fn validate_country_iso(code: &str) -> SobriaResult<()> {
    if code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(SobriaError::SchemaValidation(format!(
            "code pays ISO 3166-1 alpha-2 attendu, reçu : {code:?}"
        )))
    }
}

/// Vérifie qu'une année se situe dans la plage admissible du projet
/// (référentiel raisonnable : 1990 à 2100).
///
/// # Erreurs
///
/// Retourne `SchemaValidation` hors plage.
pub fn validate_year(year: u16) -> SobriaResult<()> {
    if (1990..=2100).contains(&year) {
        Ok(())
    } else {
        Err(SobriaError::SchemaValidation(format!(
            "année hors plage [1990, 2100] : {year}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_iso_ok() {
        assert!(validate_country_iso("FR").is_ok());
        assert!(validate_country_iso("US").is_ok());
        assert!(validate_country_iso("DE").is_ok());
    }

    #[test]
    fn country_iso_rejects_lower() {
        assert!(validate_country_iso("fr").is_err());
    }

    #[test]
    fn country_iso_rejects_size() {
        assert!(validate_country_iso("FRA").is_err());
        assert!(validate_country_iso("F").is_err());
        assert!(validate_country_iso("").is_err());
    }

    #[test]
    fn year_ok() {
        assert!(validate_year(2026).is_ok());
        assert!(validate_year(1990).is_ok());
        assert!(validate_year(2100).is_ok());
    }

    #[test]
    fn year_rejects() {
        assert!(validate_year(1989).is_err());
        assert!(validate_year(2101).is_err());
    }
}

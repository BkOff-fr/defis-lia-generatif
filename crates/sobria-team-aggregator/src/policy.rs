//! Politique de visibilité du déploiement (ADR-0016, C44).
//!
//! Trois régimes, choisis par l'ORGANISATION (pas par l'admin au fil de
//! l'eau — la valeur vit en `config` et le mode `identified` exige une
//! attestation tracée) :
//!
//! - `anonymous`  : agrégats k-anonymes uniquement, aucune identification
//!   individuelle côté admin (les opt-in sont ignorés à l'affichage).
//! - `opt_in`     : défaut — comportement ADR-0015 (k-anonymat + partage
//!   identifié contrôlé par chaque salarié).
//! - `identified` : vues nominatives intégrales ; l'activation exige une
//!   attestation CSE/salariés informés (stockée en base, affichée).

use serde::{Deserialize, Serialize};

use crate::error::{AggregatorError, AggregatorResult};
use crate::storage::Storage;

/// Clé config de la politique.
pub const POLICY_KEY: &str = "visibility_policy";
/// Clé config de l'attestation (JSON `{text, set_at, via}`).
pub const ATTESTATION_KEY: &str = "visibility_attestation";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityPolicy {
    Anonymous,
    OptIn,
    Identified,
}

impl VisibilityPolicy {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::OptIn => "opt_in",
            Self::Identified => "identified",
        }
    }

    pub fn parse(s: &str) -> AggregatorResult<Self> {
        match s {
            "anonymous" => Ok(Self::Anonymous),
            "opt_in" => Ok(Self::OptIn),
            "identified" => Ok(Self::Identified),
            other => Err(AggregatorError::Config(format!(
                "visibility_policy inconnue: {other} (anonymous | opt_in | identified)"
            ))),
        }
    }
}

/// Lit la politique effective (défaut `opt_in` si absente/illisible).
pub fn load(storage: &Storage) -> VisibilityPolicy {
    storage
        .get_config(POLICY_KEY)
        .ok()
        .flatten()
        .and_then(|v| VisibilityPolicy::parse(&v).ok())
        .unwrap_or(VisibilityPolicy::OptIn)
}

/// Attestation stockée avec le mode `identified`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attestation {
    pub text: String,
    /// RFC 3339.
    pub set_at: String,
    /// Origine (`cli`, `api`).
    pub via: String,
}

pub fn load_attestation(storage: &Storage) -> Option<Attestation> {
    storage
        .get_config(ATTESTATION_KEY)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str(&v).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_and_default() {
        for p in [
            VisibilityPolicy::Anonymous,
            VisibilityPolicy::OptIn,
            VisibilityPolicy::Identified,
        ] {
            assert_eq!(VisibilityPolicy::parse(p.as_str()).unwrap(), p);
        }
        assert!(VisibilityPolicy::parse("nominatif").is_err());
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(load(&s), VisibilityPolicy::OptIn);
        s.set_config(POLICY_KEY, "identified").unwrap();
        assert_eq!(load(&s), VisibilityPolicy::Identified);
        s.set_config(POLICY_KEY, "garbage").unwrap();
        assert_eq!(load(&s), VisibilityPolicy::OptIn);
    }
}

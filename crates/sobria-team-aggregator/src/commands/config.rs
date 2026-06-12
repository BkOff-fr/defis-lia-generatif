//! Commande CLI `config` (C38.x — ADR-0015).
//!
//! Lecture/écriture validée des clés de configuration **runtime** stockées
//! dans la table `config` (KV). Seules les clés de l'allow-list sont
//! accessibles — les clés internes (`jwt_signing_key`, …) ne passent JAMAIS
//! par cette commande.

use anyhow::{bail, Result};
use chrono::Utc;

use crate::config::DataPaths;
use crate::policy::{self, Attestation, VisibilityPolicy};
use crate::storage::Storage;

/// Nom CLI de la clé politique (traitée à part : valeur non entière,
/// attestation exigée pour `identified` — ADR-0016).
pub const POLICY_KEY_NAME: &str = "visibility_policy";

/// Lit la politique effective + son attestation éventuelle.
pub fn get_policy(paths: &DataPaths) -> Result<(&'static str, Option<Attestation>)> {
    let storage = Storage::open(&paths.db())?;
    let pol = policy::load(&storage);
    Ok((pol.as_str(), policy::load_attestation(&storage)))
}

/// Écrit la politique. `identified` exige une attestation non vide ;
/// les autres modes purgent l'attestation stockée.
pub fn set_policy(paths: &DataPaths, value: &str, attest: Option<&str>) -> Result<&'static str> {
    let pol = VisibilityPolicy::parse(value)?;
    let storage = Storage::open(&paths.db())?;
    match pol {
        VisibilityPolicy::Identified => {
            let text = attest
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "le mode `identified` exige --attest \"…\" : déclaration que le CSE \
                         (L2312-38) et les salariés (L1222-4) ont été informés — cf. ADR-0016"
                    )
                })?;
            let attestation = Attestation {
                text: text.to_string(),
                set_at: Utc::now().to_rfc3339(),
                via: "cli".into(),
            };
            storage.set_config(
                policy::ATTESTATION_KEY,
                &serde_json::to_string(&attestation)?,
            )?;
        },
        _ => {
            // Retour à un mode protecteur : l'attestation n'a plus d'objet.
            storage.set_config(policy::ATTESTATION_KEY, "")?;
        },
    }
    storage.set_config(policy::POLICY_KEY, pol.as_str())?;
    Ok(pol.as_str())
}

/// Clé runtime exposée à l'opérateur : (nom, défaut, plancher, description).
pub struct RuntimeKey {
    pub key: &'static str,
    pub default: i64,
    pub floor: i64,
    pub description: &'static str,
}

/// Allow-list des clés opérateur (ADR-0015).
pub const RUNTIME_KEYS: &[RuntimeKey] = &[
    RuntimeKey {
        key: "k_anonymity_min",
        default: 5,
        floor: 3,
        description: "Seuil k-anonymat des analytics équipe (utilisateurs actifs minimum)",
    },
    RuntimeKey {
        key: "retention_days",
        default: 730,
        floor: 30,
        description: "Rétention des estimations en jours (purge quotidienne au-delà)",
    },
];

fn find_key(key: &str) -> Result<&'static RuntimeKey> {
    RUNTIME_KEYS.iter().find(|k| k.key == key).map_or_else(
        || {
            bail!(
                "clé inconnue: {key} (clés autorisées: {})",
                RUNTIME_KEYS
                    .iter()
                    .map(|k| k.key)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        Ok,
    )
}

/// Lit la valeur effective d'une clé (défaut si absente).
pub fn get(paths: &DataPaths, key: &str) -> Result<i64> {
    let spec = find_key(key)?;
    let storage = Storage::open(&paths.db())?;
    let value = storage
        .get_config(key)?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(spec.default);
    Ok(value.max(spec.floor))
}

/// Écrit une clé après validation (entier ≥ plancher).
pub fn set(paths: &DataPaths, key: &str, value: &str) -> Result<i64> {
    let spec = find_key(key)?;
    let parsed: i64 = value
        .parse()
        .map_err(|_| anyhow::anyhow!("valeur invalide pour {key}: {value} (entier attendu)"))?;
    if parsed < spec.floor {
        bail!(
            "{key} = {parsed} refusé : plancher {} ({})",
            spec.floor,
            spec.description
        );
    }
    let storage = Storage::open(&paths.db())?;
    storage.set_config(key, &parsed.to_string())?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init;
    use tempfile::tempdir;

    fn paths() -> (tempfile::TempDir, DataPaths) {
        let dir = tempdir().unwrap();
        let p = DataPaths::new(dir.path());
        init::run(&p, "admin", "pw-not-secret-test", false).unwrap();
        (dir, p)
    }

    #[test]
    fn get_returns_default_then_set_value() {
        let (_d, p) = paths();
        assert_eq!(get(&p, "k_anonymity_min").unwrap(), 5);
        assert_eq!(set(&p, "k_anonymity_min", "7").unwrap(), 7);
        assert_eq!(get(&p, "k_anonymity_min").unwrap(), 7);
    }

    #[test]
    fn set_rejects_below_floor_and_unknown_keys() {
        let (_d, p) = paths();
        assert!(set(&p, "k_anonymity_min", "2").is_err());
        assert!(set(&p, "retention_days", "5").is_err());
        assert!(set(&p, "jwt_signing_key", "1234").is_err());
        assert!(get(&p, "nope").is_err());
    }

    #[test]
    fn set_policy_requires_attestation_for_identified() {
        let (_d, p) = paths();
        let (pol, att) = get_policy(&p).unwrap();
        assert_eq!(pol, "opt_in");
        assert!(att.is_none());
        // identified sans attestation → refus
        assert!(set_policy(&p, "identified", None).is_err());
        assert!(set_policy(&p, "identified", Some("  ")).is_err());
        // avec attestation → ok, attestation stockée
        assert_eq!(
            set_policy(&p, "identified", Some("CSE informé le 2026-06-12")).unwrap(),
            "identified"
        );
        let (pol, att) = get_policy(&p).unwrap();
        assert_eq!(pol, "identified");
        assert_eq!(att.unwrap().text, "CSE informé le 2026-06-12");
        // retour à opt_in → attestation purgée
        set_policy(&p, "opt_in", None).unwrap();
        let (_, att) = get_policy(&p).unwrap();
        assert!(att.is_none());
        // valeur inconnue → refus
        assert!(set_policy(&p, "nominatif", Some("x")).is_err());
    }

    #[test]
    fn get_floors_corrupted_low_values() {
        let (_d, p) = paths();
        let storage = Storage::open(&p.db()).unwrap();
        storage.set_config("k_anonymity_min", "1").unwrap();
        assert_eq!(get(&p, "k_anonymity_min").unwrap(), 3);
    }
}

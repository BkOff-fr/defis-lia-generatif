//! Persistance des préférences utilisateur dans `referentiel.sqlite`.
//!
//! Voir [ADR-0010](../../docs/adr/ADR-0010-personas-and-module-gating.md)
//! et `briefs/chantiers/C10-onboarding-personas.md` §2.
//!
//! Schéma :
//! ```sql
//! CREATE TABLE IF NOT EXISTS app_preferences (
//!     key         TEXT PRIMARY KEY,
//!     value       TEXT NOT NULL,        -- JSON sérialisé
//!     updated_at  TEXT NOT NULL          -- RFC 3339 UTC
//! );
//! ```
//!
//! Quatre clés réservées en v1.0 : `persona`, `enabled_modules`,
//! `onboarded`, `lang`. Toute autre clé est ignorée par les consommateurs
//! (forward-compatibility).

use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use sobria_core::{EmpreinteMethod, ModuleId, Persona};
use tracing::info;

use crate::error::AppError;

/// Clé de la table `app_preferences` qui stocke le persona courant.
pub const KEY_PERSONA: &str = "persona";
/// Clé qui stocke la liste JSON des modules activés.
pub const KEY_ENABLED_MODULES: &str = "enabled_modules";
/// Clé qui stocke le flag d'onboarding (`"true"` / `"false"`).
pub const KEY_ONBOARDED: &str = "onboarded";
/// Clé qui stocke la langue courante (`"fr"` / `"en"`).
pub const KEY_LANG: &str = "lang";
/// Clé qui stocke la méthodologie par défaut de l'utilisateur (C24).
pub const KEY_DEFAULT_METHOD: &str = "default_method";
/// Clé qui stocke la liste JSON des méthodologies affichées en référence (C24).
pub const KEY_ALSO_SHOW_METHODS: &str = "also_show_methods";
/// Clé qui stocke l'id du datacenter sélectionné par défaut (C25).
pub const KEY_DEFAULT_DATACENTER: &str = "default_datacenter_id";

/// Représentation interne des préférences avant traversée IPC.
///
/// Les `Option<_>` traduisent l'absence physique de la clé en SQLite
/// (table vide ou clé non-écrite). Le caller (`logic::*`) applique
/// alors les valeurs par défaut.
#[derive(Debug, Clone, Default)]
pub struct StoredPreferences {
    /// Persona sélectionné lors de l'onboarding.
    pub persona: Option<Persona>,
    /// Modules activés (chevauche / dépasse le bundle persona).
    pub enabled_modules: Option<Vec<ModuleId>>,
    /// Vrai si l'utilisateur a complété l'onboarding.
    pub onboarded: Option<bool>,
    /// Langue courante (`"fr"` ou `"en"`).
    pub lang: Option<String>,
    /// Méthodologie utilisée par défaut pour les calculs (C24).
    /// `None` → fallback `EmpreinteMethod::default_method()`
    /// (= `AfnorSobria`, le référentiel français).
    pub default_method: Option<EmpreinteMethod>,
    /// Méthodologies additionnelles affichées en panneau "Voir aussi"
    /// à côté du résultat principal (C24). `None` → liste vide (aucune
    /// référence comparée par défaut, l'utilisateur active manuellement).
    pub also_show_methods: Option<Vec<EmpreinteMethod>>,
    /// Dernier datacenter sélectionné par l'utilisateur dans /estimate,
    /// /comparer ou /simuler (C25). `None` = aucun choisi → l'estimation
    /// utilise les `EstimationParams` par défaut.
    pub default_datacenter_id: Option<String>,
}

/// Magasin de préférences adossé à SQLite (`referentiel.sqlite`).
pub struct PreferencesStore {
    conn: Connection,
}

impl PreferencesStore {
    /// Ouvre (ou crée) la base `referentiel.sqlite` et crée la table
    /// `app_preferences` si elle n'existe pas.
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS app_preferences (
                key         TEXT PRIMARY KEY,
                value       TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            ",
        )?;
        info!(path = %path.display(), "préférences: ouverture du store");
        Ok(Self { conn })
    }

    /// Lit l'intégralité des 4 clés réservées en une seule passe.
    pub fn read_all(&self) -> Result<StoredPreferences, AppError> {
        let persona = self
            .read_raw(KEY_PERSONA)?
            .map(|v| serde_json::from_str::<Persona>(&v))
            .transpose()
            .map_err(AppError::from)?;
        let enabled_modules = self
            .read_raw(KEY_ENABLED_MODULES)?
            .map(|v| serde_json::from_str::<Vec<ModuleId>>(&v))
            .transpose()
            .map_err(AppError::from)?;
        let onboarded = self
            .read_raw(KEY_ONBOARDED)?
            .map(|v| matches!(v.as_str(), "true"));
        let lang = self.read_raw(KEY_LANG)?;
        let default_method = self
            .read_raw(KEY_DEFAULT_METHOD)?
            .map(|v| serde_json::from_str::<EmpreinteMethod>(&v))
            .transpose()
            .map_err(AppError::from)?;
        let also_show_methods = self
            .read_raw(KEY_ALSO_SHOW_METHODS)?
            .map(|v| serde_json::from_str::<Vec<EmpreinteMethod>>(&v))
            .transpose()
            .map_err(AppError::from)?;
        let default_datacenter_id = self.read_raw(KEY_DEFAULT_DATACENTER)?;
        Ok(StoredPreferences {
            persona,
            enabled_modules,
            onboarded,
            lang,
            default_method,
            also_show_methods,
            default_datacenter_id,
        })
    }

    /// Écrit les 4 clés en une transaction. Crée ou met à jour (UPSERT).
    pub fn write_all(&mut self, prefs: &StoredPreferences) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let tx = self.conn.transaction()?;

        // persona : null → DELETE, sinon UPSERT JSON.
        match &prefs.persona {
            Some(p) => {
                let v = serde_json::to_string(p)?;
                upsert(&tx, KEY_PERSONA, &v, &now)?;
            }
            None => delete(&tx, KEY_PERSONA)?,
        }

        // enabled_modules
        if let Some(modules) = &prefs.enabled_modules {
            let v = serde_json::to_string(modules)?;
            upsert(&tx, KEY_ENABLED_MODULES, &v, &now)?;
        }

        // onboarded
        if let Some(flag) = prefs.onboarded {
            let v = if flag { "true" } else { "false" };
            upsert(&tx, KEY_ONBOARDED, v, &now)?;
        }

        // lang
        if let Some(lang) = &prefs.lang {
            upsert(&tx, KEY_LANG, lang, &now)?;
        }

        // default_method (C24)
        if let Some(method) = &prefs.default_method {
            let v = serde_json::to_string(method)?;
            upsert(&tx, KEY_DEFAULT_METHOD, &v, &now)?;
        }

        // also_show_methods (C24)
        if let Some(methods) = &prefs.also_show_methods {
            let v = serde_json::to_string(methods)?;
            upsert(&tx, KEY_ALSO_SHOW_METHODS, &v, &now)?;
        }

        // default_datacenter_id (C25)
        if let Some(id) = &prefs.default_datacenter_id {
            upsert(&tx, KEY_DEFAULT_DATACENTER, id, &now)?;
        }

        tx.commit()?;
        Ok(())
    }

    fn read_raw(&self, key: &str) -> Result<Option<String>, AppError> {
        let v: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM app_preferences WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v)
    }

    fn write_raw(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO app_preferences (key, value, updated_at) \
             VALUES (?1, ?2, ?3) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    fn delete_raw(&mut self, key: &str) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM app_preferences WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Persiste l'id du datacenter par défaut (C25). `None` efface la clé.
    pub fn set_default_datacenter_id(&mut self, id: Option<&str>) -> Result<(), AppError> {
        match id {
            Some(v) => self.write_raw(KEY_DEFAULT_DATACENTER, v),
            None => self.delete_raw(KEY_DEFAULT_DATACENTER),
        }
    }
}

fn upsert(tx: &rusqlite::Transaction<'_>, key: &str, value: &str, now: &str) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO app_preferences (key, value, updated_at) \
         VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    Ok(())
}

fn delete(tx: &rusqlite::Transaction<'_>, key: &str) -> Result<(), AppError> {
    tx.execute("DELETE FROM app_preferences WHERE key = ?1", params![key])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, PreferencesStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = PreferencesStore::open(&tmp.path().join("referentiel.sqlite")).unwrap();
        (tmp, store)
    }

    #[test]
    fn empty_store_returns_all_none() {
        let (_tmp, store) = open_temp();
        let prefs = store.read_all().unwrap();
        assert!(prefs.persona.is_none());
        assert!(prefs.enabled_modules.is_none());
        assert!(prefs.onboarded.is_none());
        assert!(prefs.lang.is_none());
        assert!(prefs.default_method.is_none());
        assert!(prefs.also_show_methods.is_none());
    }

    #[test]
    fn write_then_read_round_trip() {
        let (_tmp, mut store) = open_temp();
        let written = StoredPreferences {
            persona: Some(Persona::Enterprise),
            enabled_modules: Some(vec![ModuleId::M1, ModuleId::M22]),
            onboarded: Some(true),
            lang: Some("fr".into()),
            default_method: Some(EmpreinteMethod::EcoLogits),
            also_show_methods: Some(vec![EmpreinteMethod::AfnorSobria]),
            default_datacenter_id: None,
        };
        store.write_all(&written).unwrap();
        let read = store.read_all().unwrap();
        assert_eq!(read.persona, Some(Persona::Enterprise));
        assert_eq!(read.enabled_modules, Some(vec![ModuleId::M1, ModuleId::M22]));
        assert_eq!(read.onboarded, Some(true));
        assert_eq!(read.lang.as_deref(), Some("fr"));
        assert_eq!(read.default_method, Some(EmpreinteMethod::EcoLogits));
        assert_eq!(read.also_show_methods, Some(vec![EmpreinteMethod::AfnorSobria]));
    }

    #[test]
    fn write_overwrites_previous() {
        let (_tmp, mut store) = open_temp();
        store
            .write_all(&StoredPreferences {
                persona: Some(Persona::Student),
                enabled_modules: Some(vec![ModuleId::M1]),
                onboarded: Some(false),
                lang: Some("fr".into()),
                ..StoredPreferences::default()
            })
            .unwrap();
        store
            .write_all(&StoredPreferences {
                persona: Some(Persona::Researcher),
                enabled_modules: Some(vec![ModuleId::M1, ModuleId::M17]),
                onboarded: Some(true),
                lang: Some("en".into()),
                ..StoredPreferences::default()
            })
            .unwrap();
        let read = store.read_all().unwrap();
        assert_eq!(read.persona, Some(Persona::Researcher));
        assert_eq!(read.enabled_modules, Some(vec![ModuleId::M1, ModuleId::M17]));
        assert_eq!(read.onboarded, Some(true));
        assert_eq!(read.lang.as_deref(), Some("en"));
    }

    #[test]
    fn write_persona_none_deletes_row() {
        let (_tmp, mut store) = open_temp();
        store
            .write_all(&StoredPreferences {
                persona: Some(Persona::Enterprise),
                ..StoredPreferences::default()
            })
            .unwrap();
        assert!(store.read_all().unwrap().persona.is_some());
        store
            .write_all(&StoredPreferences::default())
            .unwrap();
        assert!(store.read_all().unwrap().persona.is_none());
    }

    #[test]
    fn default_datacenter_id_round_trip() {
        let (_tmp, mut store) = open_temp();
        assert!(store.read_all().unwrap().default_datacenter_id.is_none());

        store
            .set_default_datacenter_id(Some("aws-us-east-1"))
            .unwrap();
        let back = store.read_all().unwrap().default_datacenter_id;
        assert_eq!(back.as_deref(), Some("aws-us-east-1"));

        // Reverting to None must also persist.
        store.set_default_datacenter_id(None).unwrap();
        assert!(store.read_all().unwrap().default_datacenter_id.is_none());
    }

    #[test]
    fn store_persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("referentiel.sqlite");
        {
            let mut s = PreferencesStore::open(&path).unwrap();
            s.write_all(&StoredPreferences {
                persona: Some(Persona::ProTech),
                enabled_modules: Some(vec![ModuleId::M1, ModuleId::M3]),
                onboarded: Some(true),
                lang: Some("fr".into()),
                default_method: Some(EmpreinteMethod::EcoLogits),
                ..StoredPreferences::default()
            })
            .unwrap();
        }
        let s2 = PreferencesStore::open(&path).unwrap();
        let p = s2.read_all().unwrap();
        assert_eq!(p.persona, Some(Persona::ProTech));
        assert_eq!(p.onboarded, Some(true));
        assert_eq!(p.default_method, Some(EmpreinteMethod::EcoLogits));
    }
}

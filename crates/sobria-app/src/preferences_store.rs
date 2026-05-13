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
use sobria_core::{ModuleId, Persona};
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

/// Représentation interne des préférences avant traversée IPC.
///
/// Les `Option<_>` traduisent l'absence physique de la clé en SQLite
/// (table vide ou clé non-écrite). Le caller (`logic::*`) applique
/// alors les valeurs par défaut.
#[derive(Debug, Clone, Default)]
pub struct StoredPreferences {
    pub persona: Option<Persona>,
    pub enabled_modules: Option<Vec<ModuleId>>,
    pub onboarded: Option<bool>,
    pub lang: Option<String>,
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
        Ok(StoredPreferences {
            persona,
            enabled_modules,
            onboarded,
            lang,
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
    }

    #[test]
    fn write_then_read_round_trip() {
        let (_tmp, mut store) = open_temp();
        let written = StoredPreferences {
            persona: Some(Persona::Enterprise),
            enabled_modules: Some(vec![ModuleId::M1, ModuleId::M22]),
            onboarded: Some(true),
            lang: Some("fr".into()),
        };
        store.write_all(&written).unwrap();
        let read = store.read_all().unwrap();
        assert_eq!(read.persona, Some(Persona::Enterprise));
        assert_eq!(read.enabled_modules, Some(vec![ModuleId::M1, ModuleId::M22]));
        assert_eq!(read.onboarded, Some(true));
        assert_eq!(read.lang.as_deref(), Some("fr"));
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
            })
            .unwrap();
        store
            .write_all(&StoredPreferences {
                persona: Some(Persona::Researcher),
                enabled_modules: Some(vec![ModuleId::M1, ModuleId::M17]),
                onboarded: Some(true),
                lang: Some("en".into()),
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
                enabled_modules: None,
                onboarded: None,
                lang: None,
            })
            .unwrap();
        assert!(store.read_all().unwrap().persona.is_some());
        store
            .write_all(&StoredPreferences {
                persona: None,
                enabled_modules: None,
                onboarded: None,
                lang: None,
            })
            .unwrap();
        assert!(store.read_all().unwrap().persona.is_none());
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
            })
            .unwrap();
        }
        let s2 = PreferencesStore::open(&path).unwrap();
        let p = s2.read_all().unwrap();
        assert_eq!(p.persona, Some(Persona::ProTech));
        assert_eq!(p.onboarded, Some(true));
    }
}

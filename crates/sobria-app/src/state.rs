//! State partagé entre les commandes IPC.
//!
//! `AppState` est instancié une fois au démarrage de Tauri et passé à
//! chaque commande via `tauri::State<'_, AppState>`. Le `AuditLedger`
//! est derrière un `Mutex` parce que les écritures (`append`) doivent
//! être sérielles pour garantir la chaîne SHA-256.

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use sobria_audit::AuditLedger;
use sobria_estimator::MonteCarloEngine;
use tracing::info;

use crate::{
    error::AppError, goals_store::PersonalGoalsStore, preferences_store::PreferencesStore,
};

/// State partagé de l'application.
pub struct AppState {
    /// Racine de données (`~/.local/share/sobria` sur Linux, etc.).
    pub data_root: PathBuf,
    /// Chemin absolu du ledger SQLite.
    pub audit_path: PathBuf,
    /// Chemin absolu de la base référentiel (préférences + futurs catalogues).
    pub referentiel_path: PathBuf,
    /// Ledger d'audit (toutes les écritures sérielles).
    pub ledger: Mutex<AuditLedger>,
    /// Store de préférences utilisateur — voir ADR-0010.
    pub preferences: Mutex<PreferencesStore>,
    /// Store des objectifs eco-budget — voir brief C19 (M25).
    pub goals: Mutex<PersonalGoalsStore>,
    /// Moteur Monte-Carlo (immuable, partageable).
    pub estimator: MonteCarloEngine,
}

impl AppState {
    /// Construit un `AppState` en initialisant la racine de données, le
    /// ledger d'audit et le store de préférences.
    ///
    /// Si `data_root` est `None`, on utilise `dirs::data_dir() / "sobria"`.
    pub fn init(data_root: Option<PathBuf>) -> Result<Self, AppError> {
        let data_root = data_root
            .or_else(|| dirs::data_dir().map(|d| d.join("sobria")))
            .ok_or_else(|| {
                AppError::Internal("impossible de déterminer la data dir système".into())
            })?;
        std::fs::create_dir_all(&data_root)?;

        let audit_path = data_root.join("audit.sqlite");
        info!(path = %audit_path.display(), "audit: ouverture du ledger");
        let ledger = AuditLedger::open(&audit_path)?;

        let referentiel_path = data_root.join("referentiel.sqlite");
        info!(path = %referentiel_path.display(), "préférences: ouverture du référentiel");
        let preferences = PreferencesStore::open(&referentiel_path)?;
        let goals = PersonalGoalsStore::open(&referentiel_path)?;

        let estimator = MonteCarloEngine::default();
        Ok(Self {
            data_root,
            audit_path,
            referentiel_path,
            ledger: Mutex::new(ledger),
            preferences: Mutex::new(preferences),
            goals: Mutex::new(goals),
            estimator,
        })
    }

    /// Construit un `AppState` dans un dossier explicite (utile pour les tests).
    pub fn init_in(dir: &Path) -> Result<Self, AppError> {
        Self::init(Some(dir.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_in_creates_audit_sqlite() {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::init_in(tmp.path()).unwrap();
        assert!(state.audit_path.exists());
        assert_eq!(state.data_root, tmp.path());
        // Ledger ouvert : len() == 0
        let ledger = state.ledger.lock().unwrap();
        assert_eq!(ledger.len().unwrap(), 0);
    }

    #[test]
    fn init_in_creates_referentiel_sqlite() {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::init_in(tmp.path()).unwrap();
        assert!(state.referentiel_path.exists());
        // Préférences vides au boot
        let store = state.preferences.lock().unwrap();
        let prefs = store.read_all().unwrap();
        assert!(prefs.persona.is_none());
        assert!(prefs.onboarded.is_none());
    }

    #[test]
    fn init_in_idempotent_on_existing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let _s1 = AppState::init_in(tmp.path()).unwrap();
        let _s2 = AppState::init_in(tmp.path()).unwrap();
        // Pas de panic, pas d'erreur.
    }
}

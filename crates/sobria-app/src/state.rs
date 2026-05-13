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

use crate::error::AppError;

/// State partagé de l'application.
pub struct AppState {
    /// Racine de données (`~/.local/share/sobria` sur Linux, etc.).
    pub data_root: PathBuf,
    /// Chemin absolu du ledger SQLite.
    pub audit_path: PathBuf,
    /// Ledger d'audit (toutes les écritures sérielles).
    pub ledger: Mutex<AuditLedger>,
    /// Moteur Monte-Carlo (immuable, partageable).
    pub estimator: MonteCarloEngine,
}

impl AppState {
    /// Construit un `AppState` en initialisant la racine de données + le ledger.
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
        let estimator = MonteCarloEngine::default();
        Ok(Self {
            data_root,
            audit_path,
            ledger: Mutex::new(ledger),
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
    fn init_in_idempotent_on_existing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let _s1 = AppState::init_in(tmp.path()).unwrap();
        let _s2 = AppState::init_in(tmp.path()).unwrap();
        // Pas de panic, pas d'erreur.
    }
}

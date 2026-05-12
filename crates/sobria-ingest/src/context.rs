//! Contexte d'exécution du pipeline médaillon.

use std::path::PathBuf;

/// Contexte injecté à toutes les étapes du pipeline.
#[derive(Debug, Clone)]
pub struct Context {
    /// Racine des données (par défaut `data/`).
    pub data_root: PathBuf,
    /// Mode incrémental — ne ré-ingère que les snapshots modifiés.
    pub incremental: bool,
    /// Seed Monte-Carlo (utile pour les sources stochastiques).
    pub seed: u64,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            data_root: PathBuf::from("data"),
            incremental: false,
            seed: sobria_core::DEFAULT_SEED,
        }
    }
}

impl Context {
    /// Chemin Copper pour une source donnée.
    #[must_use]
    pub fn copper_root(&self, source_id: &str) -> PathBuf {
        self.data_root.join("copper").join(source_id)
    }

    /// Chemin Silver pour une source donnée.
    #[must_use]
    pub fn silver_root(&self, source_id: &str) -> PathBuf {
        self.data_root.join("silver").join(source_id)
    }

    /// Chemin Gold (sortie unifiée).
    #[must_use]
    pub fn gold_root(&self) -> PathBuf {
        self.data_root.join("gold")
    }
}

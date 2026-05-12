//! Registry — orchestre toutes les sources du pipeline médaillon.

use std::sync::Arc;

use crate::layer::DataLayer;

/// Liste de toutes les sources connues.
pub struct LayerRegistry {
    sources: Vec<Arc<dyn DataLayer>>,
}

impl LayerRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// Construit le registre standard avec toutes les sources de production.
    ///
    /// Implémenté en S2-S3. Voir `docs/sources/CATALOGUE-SOURCES.md`.
    #[must_use]
    pub fn standard() -> Self {
        // TODO(sobria-003): instancier les 8 sources (ComparIA, RTE IRIS, ADEME, …).
        Self::new()
    }

    /// Enregistre une source.
    pub fn register(&mut self, source: Arc<dyn DataLayer>) {
        self.sources.push(source);
    }

    /// Itère sur les sources enregistrées.
    pub fn sources(&self) -> impl Iterator<Item = &Arc<dyn DataLayer>> {
        self.sources.iter()
    }
}

impl Default for LayerRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

//! Factory pour instancier le bon moteur selon [`EmpreinteMethod`].

use sobria_core::DEFAULT_SEED;

use crate::{
    engine::MonteCarloEngine,
    engine_trait::{EmpreinteEngine, EmpreinteMethod},
    engines::ecologits::EcoLogitsEngine,
};

/// Instancie le moteur correspondant à une méthodologie.
///
/// Le seed `DEFAULT_SEED` (42) est utilisé partout pour garantir la
/// reproductibilité ; surchargeable au runtime via `SOBRIA_SEED` env var
/// (cf. CLAUDE.md §6).
#[must_use]
pub fn engine_for(method: EmpreinteMethod) -> Box<dyn EmpreinteEngine> {
    match method {
        EmpreinteMethod::AfnorSobria => Box::new(MonteCarloEngine::new(DEFAULT_SEED)),
        EmpreinteMethod::EcoLogits => Box::new(EcoLogitsEngine::new(DEFAULT_SEED)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_returns_correct_method() {
        for &method in &[EmpreinteMethod::AfnorSobria, EmpreinteMethod::EcoLogits] {
            let eng = engine_for(method);
            assert_eq!(eng.method(), method);
        }
    }

    #[test]
    fn factory_engines_expose_methodology_info() {
        for &method in &[EmpreinteMethod::AfnorSobria, EmpreinteMethod::EcoLogits] {
            let eng = engine_for(method);
            let info = eng.methodology_info();
            assert_eq!(info.method, method);
            assert!(!info.display_name.is_empty());
        }
    }
}

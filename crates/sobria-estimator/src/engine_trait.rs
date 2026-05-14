//! Trait commun à toutes les méthodologies d'empreinte LLM embarquées
//! dans Sobr.ia (ADR-0012 à venir).
//!
//! Sobr.ia v1.0+ propose un **catalogue de méthodologies scientifiques**
//! sélectionnables par l'utilisateur :
//!
//! - **AFNOR SPEC 2314 (Sobr.ia)** : formule linéaire-par-token + Monte-Carlo,
//!   méthode propre alignée sur le référentiel français AFNOR SPEC 2314.
//! - **EcoLogits 2026-01** : port direct des formules EcoLogits (Rincé &
//!   Banse 2025, [DOI:10.21105/joss.07471](https://doi.org/10.21105/joss.07471),
//!   licence CC BY-SA 4.0).
//! - *(v1.1+ prévu)* BoaVizta, HF AI Energy Score, Custom user engine.
//!
//! ## UX
//!
//! L'utilisateur sélectionne **une** méthodologie par défaut dans
//! `Settings → Méthodologies`. Il peut éventuellement activer d'autres
//! méthodologies en référence : les résultats correspondants apparaissent
//! dans un panneau "Voir aussi" à côté du résultat principal.
//!
//! L'audit ledger trace systématiquement la méthodologie utilisée pour
//! chaque estimation (champ `method` dans la table `audit_entries`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sobria_core::{EstimationRequest, EstimationResult};

use crate::{error::EstimatorResult, params::EstimationParams};

/// Re-export du type énumératif défini dans `sobria-core::methodology`.
///
/// On garde le type central dans `sobria-core` pour pouvoir l'utiliser
/// dans `EstimationResult` (sérialisé dans l'audit ledger) et dans les
/// préférences user, **sans** créer de dépendance circulaire vers
/// `sobria-estimator`.
pub use sobria_core::EmpreinteMethod;

/// Statut de calibration d'une méthodologie au sein de Sobr.ia.
///
/// Distinct de [`crate::model_presets::CalibrationStatus`] (qui s'applique
/// à un *modèle* dans un *preset*) — ici on parle du statut de la
/// *méthodologie* elle-même.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MethodologyCalibration {
    /// Méthodologie peer-reviewed, reproductible à <1 % de l'implémentation
    /// de référence publiée (test cross-check automatique en CI).
    PeerReviewedReproduced,
    /// Méthodologie publique de référence, implémentée par Sobr.ia avec
    /// validation par plausibilité (ordres de grandeur) — la calibration
    /// chiffrée est en cours.
    PublicMethodCalibrationPending,
    /// Méthodologie embarquée pour exploration / pédagogie, valeurs à
    /// considérer comme indicatives uniquement.
    Indicative,
}

/// Description statique d'une méthodologie embarquée.
///
/// Renvoyée par [`AVAILABLE_METHODS`] au runtime pour afficher le
/// catalogue dans `Settings → Méthodologies`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct MethodologyInfo {
    /// Identifiant stable.
    pub method: EmpreinteMethod,
    /// Nom affiché en UI.
    pub display_name: &'static str,
    /// Courte description (1-2 phrases) destinée à un public non-technique.
    pub short_description: &'static str,
    /// DOI ou URL de référence académique / open-source.
    pub reference_url: &'static str,
    /// Identifiant DOI normalisé si dispo (sinon `None`).
    pub doi: Option<&'static str>,
    /// License de la méthodologie publiée.
    pub license: &'static str,
    /// Statut de calibration dans Sobr.ia.
    pub calibration: MethodologyCalibration,
    /// Année de publication de la méthodologie de référence.
    pub year_published: u16,
    /// Organisation qui maintient la méthodologie de référence.
    pub maintained_by: &'static str,
}

/// Catalogue complet des méthodologies disponibles dans Sobr.ia v1.0.
///
/// Ajout d'une nouvelle méthodologie en v1.1+ :
/// 1. Étendre l'enum [`EmpreinteMethod`].
/// 2. Ajouter l'entry ici.
/// 3. Implémenter [`EmpreinteEngine`] dans `crates/sobria-estimator/src/engines/`.
/// 4. Câbler dans la factory [`engine_for`].
pub static AVAILABLE_METHODS: &[MethodologyInfo] = &[
    MethodologyInfo {
        method: EmpreinteMethod::AfnorSobria,
        display_name: "AFNOR SPEC 2314 — Sobr.ia",
        short_description: "Référentiel français de mesure de l'empreinte LLM. \
                            Formule linéaire-par-token + Monte-Carlo N=10⁴.",
        reference_url: "https://norminfo.afnor.org/norme/AFNOR%20SPEC%202314/",
        doi: None,
        license: "AFNOR SPEC publique ; code Sobr.ia sous licence MIT",
        calibration: MethodologyCalibration::PublicMethodCalibrationPending,
        year_published: 2024,
        maintained_by: "Sobr.ia (impl.) / AFNOR (spec)",
    },
    MethodologyInfo {
        method: EmpreinteMethod::EcoLogits,
        display_name: "EcoLogits 2026-01",
        short_description: "Méthodologie open peer-reviewed, port direct des \
                            formules Rincé & Banse (JOSS 2025). Référence \
                            internationale de l'estimation LLM.",
        reference_url: "https://ecologits.ai/latest/methodology/llm_inference/",
        doi: Some("10.21105/joss.07471"),
        license: "CC BY-SA 4.0 (méthodologie) ; code Sobr.ia sous licence MIT",
        calibration: MethodologyCalibration::PeerReviewedReproduced,
        year_published: 2025,
        maintained_by: "GenAI Impact (Rincé & Banse)",
    },
];

/// Trait commun aux moteurs d'estimation d'empreinte.
///
/// Toute méthodologie embarquée dans Sobr.ia implémente ce trait, ce qui
/// permet à la couche `sobria-app` d'invoquer indifféremment le moteur
/// AFNOR ou EcoLogits selon la préférence utilisateur.
pub trait EmpreinteEngine: Send + Sync {
    /// Identifiant stable de la méthodologie.
    fn method(&self) -> EmpreinteMethod;

    /// Méta-info pour affichage UI / dossier candidature.
    fn methodology_info(&self) -> &'static MethodologyInfo {
        AVAILABLE_METHODS
            .iter()
            .find(|m| m.method == self.method())
            .expect("AVAILABLE_METHODS doit couvrir tous les EmpreinteMethod variants")
    }

    /// Lance l'estimation pour une requête + jeu de paramètres.
    ///
    /// Les `EstimationParams` ne sont pas tous honorés par toutes les
    /// méthodologies (EcoLogits a ses propres coefficients internes), cf.
    /// la documentation de chaque implémentation.
    fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult>;
}

/// Récupère les infos d'une méthodologie par identifiant.
#[must_use]
pub fn info_for(method: EmpreinteMethod) -> &'static MethodologyInfo {
    AVAILABLE_METHODS
        .iter()
        .find(|m| m.method == method)
        .expect("AVAILABLE_METHODS doit couvrir tous les EmpreinteMethod variants")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_methods_covers_all_variants() {
        // Chaque variant de l'enum doit avoir une entrée dans AVAILABLE_METHODS.
        // Si on ajoute un variant à l'enum mais qu'on oublie le registry,
        // ce test échoue.
        for &method in EmpreinteMethod::all() {
            assert!(
                AVAILABLE_METHODS.iter().any(|m| m.method == method),
                "AVAILABLE_METHODS ne couvre pas {method:?}"
            );
        }
    }

    #[test]
    fn info_for_returns_matching_entry() {
        for &method in EmpreinteMethod::all() {
            let info = info_for(method);
            assert_eq!(info.method, method);
        }
    }

    #[test]
    fn ecologits_entry_has_doi() {
        let info = info_for(EmpreinteMethod::EcoLogits);
        assert_eq!(info.doi, Some("10.21105/joss.07471"));
        assert!(info.reference_url.starts_with("https://"));
    }
}

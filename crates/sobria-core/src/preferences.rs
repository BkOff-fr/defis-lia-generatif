//! Personas et identifiants stables des modules — voir ADR-0010 et
//! CDC v1.3 §3 (personas) + §4 (25 modules).
//!
//! Ces types sont **fermés** : ajouter un persona ou un module nécessite
//! un bump de version de schéma (chantier dédié + ADR mini-incrément).
//!
//! Ils vivent dans `sobria-core` parce qu'ils sont consommés à la fois par
//! `sobria-app` (préférences IPC) et par tout futur frontend tiers
//! (extension navigateur, CLI, etc.).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Les cinq personas figés en v1.3 du cahier des charges.
///
/// Un utilisateur choisit un et un seul persona à la fois ; il peut
/// néanmoins personnaliser librement ses modules au-delà du bundle
/// par défaut. Voir ADR-0010 §"Principes" pour la justification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Persona {
    /// Étudiant·e ou simple curieux·se — découverte, apprentissage, usage perso.
    Student,
    /// Professionnel·le tech (dev, ML eng, intégrateur).
    ProTech,
    /// Entreprise — DSI, RSE, scope 3 IA, reporting CSRD.
    Enterprise,
    /// Collectivité ou service public — territorial, marchés publics.
    PublicSector,
    /// Chercheur·se ou journaliste — reproductibilité, comparaisons sourcées.
    Researcher,
}

impl Persona {
    /// Itère sur les 5 personas dans l'ordre canonique du CDC §3.
    #[must_use]
    pub fn all() -> &'static [Persona] {
        &[
            Persona::Student,
            Persona::ProTech,
            Persona::Enterprise,
            Persona::PublicSector,
            Persona::Researcher,
        ]
    }

    /// Bundle de modules pré-cochés au moment où l'utilisateur sélectionne
    /// ce persona dans l'onboarding. L'utilisateur peut ensuite ajouter
    /// ou retirer librement n'importe quel module.
    ///
    /// **Périmètre v1.0 figé** (cf. ADR-0011 — réduction périmètre) :
    /// seuls **13 modules** sont en cible v1.0 — M1, M3, M7, M8, M9, M12,
    /// M13, M14, M15, M17, M20, M22, M25. Les autres `ModuleId` restent
    /// dans l'enum (forward compat, activables manuellement) mais ne
    /// figurent dans aucun bundle par défaut.
    ///
    /// **Invariants garantis par les tests** :
    /// - le bundle est non vide,
    /// - `ModuleId::M1` (Estimer) est présent dans tous les bundles,
    /// - aucun doublon,
    /// - aucune référence à un identifiant `M4` (réservé en v1.3),
    /// - tous les modules d'un bundle sont dans le set v1.0 (13 IDs).
    #[must_use]
    pub fn default_modules(self) -> Vec<ModuleId> {
        use ModuleId::{
            M1, M12, M13, M14, M15, M17, M20, M22, M25, M3, M7, M8, M9,
        };
        match self {
            // Étudiant / Curieux : apprendre, comprendre, suivre, fixer.
            Persona::Student => vec![M1, M8, M13, M14, M15, M25],
            // Pro tech : estimer, comparer, journal, refs, méthodo.
            Persona::ProTech => vec![M1, M3, M7, M8, M9, M13, M14],
            // Entreprise : toute la chaîne compliance + dashboard + budget.
            Persona::Enterprise => {
                vec![M1, M7, M12, M14, M15, M17, M20, M22, M25]
            },
            // Collectivité : focus territoire FR + reporting.
            Persona::PublicSector => vec![M1, M8, M12, M14, M17, M20, M22],
            // Chercheur / Journaliste : méthodologie + reproductibilité.
            Persona::Researcher => vec![M1, M3, M7, M8, M9, M14, M17],
        }
    }

    /// Liste des modules en cible v1.0 (13 IDs). Voir ADR-0011.
    #[must_use]
    pub fn v1_0_modules() -> &'static [ModuleId] {
        use ModuleId::{M1, M12, M13, M14, M15, M17, M20, M22, M25, M3, M7, M8, M9};
        &[M1, M3, M7, M8, M9, M12, M13, M14, M15, M17, M20, M22, M25]
    }
}

/// Identifiants stables des 24 modules visibles en v1.3 du CDC.
///
/// L'identifiant `M4` est volontairement **omis** : il était utilisé en
/// v1.2 et a été retiré dans v1.3 pour éviter la confusion. Tout code
/// produisant ou consommant la string `"m4"` est invalide.
///
/// L'ordre déclaratif ci-dessous correspond à l'ordre par défaut dans
/// le rail UI (voir `web/src/routes/+layout.svelte`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModuleId {
    /// M1 — Estimer un prompt (cœur produit, dans tous les bundles).
    M1,
    /// M2 — Workbench multi-prompts.
    M2,
    /// M3 — Comparer modèles (benchmark côte-à-côte).
    M3,
    // M4 — réservé / retiré v1.3.
    /// M5 — Exporter rapport (PDF/CSV/JSON).
    M5,
    /// M6 — Géoloc datacenter unitaire.
    M6,
    /// M7 — Journal d'audit (ledger chaîné SHA-256).
    M7,
    /// M8 — Méthodologie interactive.
    M8,
    /// M9 — Référentiel modèles (catalogue 25+).
    M9,
    /// M10 — Importer batch (CSV/JSON multi-prompts).
    M10,
    /// M11 — Extension navigateur (capture vie réelle).
    M11,
    /// M12 — Datacenters Europe (carte OpenStreetMap + drill-down).
    M12,
    /// M13 — Simulateur « Et si...? » (7 leviers temps réel).
    M13,
    /// M14 — À propos / Crédits / Licences.
    M14,
    /// M15 — Tableau de bord personnel (jour/semaine/mois).
    M15,
    /// M16 — Forecaster 12 mois (bande d'incertitude × sliders).
    M16,
    /// M17 — Empreinte projet (reproductibilité scientifique).
    M17,
    /// M18 — Batch CSV → rapport agrégé.
    M18,
    /// M19 — Équipe / multi-utilisateurs (RBAC léger).
    M19,
    /// M20 — Territoire FR (RTE IRIS, Sankey énergétique).
    M20,
    /// M21 — Alertes & seuils (notifs locales).
    M21,
    /// M22 — Rapport CSRD / AGEC / AFNOR SPEC 2314 (PDF signé + PROV-O).
    M22,
    /// M23 — Marchés publics IA frugale (cahiers des charges types).
    M23,
    /// M24 — Apprendre (mini-cours, best practices).
    M24,
    /// M25 — Objectifs & habitudes (eco-budget personnel).
    M25,
}

impl ModuleId {
    /// Énumère les 24 modules dans l'ordre canonique du rail UI.
    #[must_use]
    pub fn all() -> &'static [ModuleId] {
        use ModuleId::{
            M1, M10, M11, M12, M13, M14, M15, M16, M17, M18, M19, M2, M20, M21, M22, M23, M24,
            M25, M3, M5, M6, M7, M8, M9,
        };
        &[
            M1, M2, M3, M5, M6, M7, M8, M9, M10, M11, M12, M13, M14, M15, M16, M17, M18, M19,
            M20, M21, M22, M23, M24, M25,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_personas_have_non_empty_bundles() {
        for p in Persona::all() {
            assert!(
                !p.default_modules().is_empty(),
                "persona {p:?} a un bundle vide"
            );
        }
    }

    #[test]
    fn m1_is_in_every_bundle() {
        for p in Persona::all() {
            assert!(
                p.default_modules().contains(&ModuleId::M1),
                "persona {p:?} ne contient pas M1 (point fixe)"
            );
        }
    }

    #[test]
    fn bundles_have_no_duplicates() {
        for p in Persona::all() {
            let bundle = p.default_modules();
            let unique: HashSet<_> = bundle.iter().collect();
            assert_eq!(
                bundle.len(),
                unique.len(),
                "doublons dans le bundle {p:?} : {bundle:?}"
            );
        }
    }

    #[test]
    fn all_modules_excludes_m4() {
        let json_ids: Vec<String> = ModuleId::all()
            .iter()
            .map(|m| serde_json::to_string(m).unwrap())
            .collect();
        for id in &json_ids {
            assert_ne!(id, "\"m4\"", "M4 doit être réservé / absent");
        }
        // Pour mémoire, la liste doit aussi contenir m1 et m25
        assert!(json_ids.contains(&"\"m1\"".to_string()));
        assert!(json_ids.contains(&"\"m25\"".to_string()));
    }

    #[test]
    fn all_modules_count_is_24() {
        // 25 - 1 (M4 réservé) = 24 modules visibles.
        assert_eq!(ModuleId::all().len(), 24);
    }

    #[test]
    fn persona_serde_round_trip() {
        for p in Persona::all() {
            let json = serde_json::to_string(p).unwrap();
            let back: Persona = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *p);
        }
    }

    #[test]
    fn persona_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&Persona::ProTech).unwrap(),
            "\"pro_tech\""
        );
        assert_eq!(
            serde_json::to_string(&Persona::PublicSector).unwrap(),
            "\"public_sector\""
        );
    }

    #[test]
    fn module_id_serde_round_trip() {
        for m in ModuleId::all() {
            let json = serde_json::to_string(m).unwrap();
            let back: ModuleId = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *m);
        }
    }

    #[test]
    fn module_id_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&ModuleId::M1).unwrap(), "\"m1\"");
        assert_eq!(serde_json::to_string(&ModuleId::M25).unwrap(), "\"m25\"");
    }

    #[test]
    fn module_id_deserialize_m4_fails() {
        let result: Result<ModuleId, _> = serde_json::from_str("\"m4\"");
        assert!(result.is_err(), "m4 doit être rejeté à la désérialisation");
    }

    #[test]
    fn deserialize_unknown_persona_fails() {
        let result: Result<Persona, _> = serde_json::from_str("\"hacker\"");
        assert!(result.is_err());
    }

    #[test]
    fn each_persona_bundle_modules_are_in_all_modules() {
        let all: HashSet<ModuleId> = ModuleId::all().iter().copied().collect();
        for p in Persona::all() {
            for m in p.default_modules() {
                assert!(
                    all.contains(&m),
                    "module {m:?} du bundle {p:?} pas dans ModuleId::all()"
                );
            }
        }
    }

    #[test]
    fn v1_0_modules_returns_13() {
        // ADR-0011 : périmètre v1.0 figé à 13 modules essentiels.
        assert_eq!(Persona::v1_0_modules().len(), 13);
    }

    #[test]
    fn v1_0_modules_includes_pivot_modules() {
        let v1 = Persona::v1_0_modules();
        // Modules clés du pitch
        for required in [
            ModuleId::M1,  // Estimer
            ModuleId::M7,  // Journal d'audit
            ModuleId::M20, // Territoire FR (différenciateur)
            ModuleId::M22, // Rapport CSRD
            ModuleId::M17, // Empreinte projet (datasheet Gebru)
        ] {
            assert!(
                v1.contains(&required),
                "module pivot {required:?} absent du périmètre v1.0"
            );
        }
    }

    #[test]
    fn v1_0_modules_excludes_deferred_modules() {
        let v1: HashSet<ModuleId> = Persona::v1_0_modules().iter().copied().collect();
        // Modules différés v1.1+ (cf. ADR-0011)
        for deferred in [
            ModuleId::M2,  // Workbench
            ModuleId::M5,  // Rapports génériques
            ModuleId::M6,  // Géoloc unitaire
            ModuleId::M10, // Import logs
            ModuleId::M11, // Extension navigateur
            ModuleId::M16, // Forecaster (backend prêt mais UI v1.1)
            ModuleId::M18, // Batch CSV (idem)
            ModuleId::M19, // Équipe
            ModuleId::M21, // Alertes
            ModuleId::M23, // Marchés publics
            ModuleId::M24, // Apprendre
        ] {
            assert!(
                !v1.contains(&deferred),
                "module {deferred:?} ne doit pas être dans le périmètre v1.0"
            );
        }
    }

    #[test]
    fn all_persona_bundles_only_reference_v1_0_modules() {
        let v1: HashSet<ModuleId> = Persona::v1_0_modules().iter().copied().collect();
        for p in Persona::all() {
            for m in p.default_modules() {
                assert!(
                    v1.contains(&m),
                    "persona {p:?} référence {m:?} hors périmètre v1.0"
                );
            }
        }
    }
}

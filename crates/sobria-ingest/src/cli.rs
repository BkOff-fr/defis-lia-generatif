//! Helpers exposés au binaire CLI `sobria-ingest`.
//!
//! Extraits dans un module dédié pour pouvoir être testés depuis
//! `tests/cli_pipeline.rs` (les fonctions privées de `main.rs` sont
//! inaccessibles depuis l'intégration).

use std::path::PathBuf;

use anyhow::{anyhow, Context as _, Result};

use crate::{Context, CopperSnapshot, LayerRegistry, StepResult};

/// Construit le contexte d'exécution en honorant les variables d'environnement
/// `SOBRIA_DATA_ROOT` (défaut `./data`) et `SOBRIA_SEED` (défaut
/// [`sobria_core::DEFAULT_SEED`]). Crée le dossier `data_root` au besoin.
pub fn build_context(incremental: bool) -> Result<Context> {
    let data_root =
        std::env::var("SOBRIA_DATA_ROOT").map_or_else(|_| PathBuf::from("data"), PathBuf::from);
    let seed = std::env::var("SOBRIA_SEED")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(sobria_core::DEFAULT_SEED);
    build_context_with(data_root, seed, incremental)
}

/// Variante testable de [`build_context`] où les paramètres sont injectés
/// explicitement (utilisée pour éviter les flakies sur `std::env` partagé
/// entre tests parallèles).
pub fn build_context_with(data_root: PathBuf, seed: u64, incremental: bool) -> Result<Context> {
    std::fs::create_dir_all(&data_root)
        .with_context(|| format!("creating data root {}", data_root.display()))?;
    Ok(Context {
        data_root,
        incremental,
        seed,
    })
}

/// Filtre [`LayerRegistry::standard`] pour ne garder qu'une source ciblée.
/// Renvoie le registre standard complet si `source_id` est `None`.
///
/// # Erreurs
///
/// Échoue si `source_id` est `Some` mais ne correspond à aucune source
/// enregistrée par défaut.
pub fn filter_registry(source_id: Option<&str>) -> Result<LayerRegistry> {
    let standard = LayerRegistry::standard();
    let Some(target) = source_id else {
        return Ok(standard);
    };
    let mut reg = LayerRegistry::new();
    let mut found = false;
    for s in standard.sources() {
        if s.id() == target {
            reg.register(s.clone());
            found = true;
            break;
        }
    }
    if !found {
        let known: Vec<&str> = LayerRegistry::standard()
            .sources()
            .map(|s| s.id())
            .collect();
        return Err(anyhow!(
            "source inconnue: '{target}'. Sources disponibles : {}",
            known.join(", ")
        ));
    }
    Ok(reg)
}

/// Liste les identifiants des sources du registre standard. Utile pour
/// affichage CLI et tests.
#[must_use]
pub fn standard_source_ids() -> Vec<&'static str> {
    LayerRegistry::standard()
        .sources()
        .map(|s| s.id())
        .collect()
}

/// Renvoie le dossier du snapshot Copper **le plus récent** d'une source
/// donnée — basé sur le tri lexicographique (qui équivaut au tri
/// chronologique pour des noms `YYYY-MM-DD`).
///
/// Renvoie `Ok(None)` si :
/// - le dossier `data/copper/<source>/` est inexistant,
/// - aucun de ses sous-dossiers ne contient de `manifest.json`.
pub fn latest_copper_snapshot(ctx: &Context, source_id: &str) -> std::io::Result<Option<PathBuf>> {
    let copper_root = ctx.copper_root(source_id);
    if !copper_root.exists() {
        return Ok(None);
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&copper_root)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("manifest.json").exists())
        .collect();
    if entries.is_empty() {
        return Ok(None);
    }
    entries.sort();
    Ok(entries.pop())
}

/// Reconstruit les [`CopperSnapshot`] de toutes les sources d'un registre
/// depuis les snapshots Copper persistants — **sans** ré-ingérer.
///
/// Pour chaque source :
/// - succès : on charge le `manifest.json` du snapshot le plus récent et on
///   vérifie l'intégrité (`CopperManifest::verify_files`).
/// - échec : message explicite indiquant la commande à lancer pour produire
///   un Copper.
///
/// Utilisée par la sous-commande `silver` pour permettre une promotion
/// déterministe à partir d'un Copper figé sur disque (cf. ADR-0009 et
/// chantier C26.2).
pub async fn rehydrate_copper(
    ctx: &Context,
    registry: &LayerRegistry,
) -> Vec<StepResult<CopperSnapshot>> {
    let mut out = Vec::with_capacity(registry.len());
    for source in registry.sources() {
        let id = source.id();
        match latest_copper_snapshot(ctx, id) {
            Ok(Some(snapshot_dir)) => {
                match CopperSnapshot::from_manifest(&snapshot_dir).await {
                    Ok(snap) => out.push(StepResult::ok(id, snap)),
                    Err(e) => out.push(StepResult::err(
                        id,
                        format!("from_manifest({}): {e}", snapshot_dir.display()),
                    )),
                }
            },
            Ok(None) => out.push(StepResult::err(
                id,
                format!(
                    "aucun snapshot Copper sous {} — lancez `cargo run -p sobria-ingest -- copper --source {id}` ou `pipeline run` d'abord",
                    ctx.copper_root(id).display()
                ),
            )),
            Err(e) => out.push(StepResult::err(id, format!("scan copper_root: {e}"))),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_registry_contains_tier1_sources() {
        let ids = standard_source_ids();
        assert!(
            ids.contains(&"comparia"),
            "ComparIA Tier 1 manquant : {ids:?}"
        );
        assert!(
            ids.contains(&"rte-iris"),
            "RTE IRIS Tier 1 manquant : {ids:?}"
        );
    }

    #[test]
    fn filter_registry_none_returns_full_standard() {
        let reg = filter_registry(None).unwrap();
        assert!(reg.len() >= 2, "≥ 2 sources Tier 1 attendues");
    }

    #[test]
    fn filter_registry_selects_one_source() {
        let reg = filter_registry(Some("comparia")).unwrap();
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.sources().next().unwrap().id(), "comparia");
    }

    #[test]
    fn filter_registry_rejects_unknown_source() {
        // `LayerRegistry` n'implémente pas `Debug` (les `dyn DataLayer` non plus),
        // ce qui empêche `unwrap_err()` qui requiert `T: Debug`. On extrait
        // l'erreur via let…else.
        let Err(err) = filter_registry(Some("does-not-exist")) else {
            panic!("expected error for unknown source");
        };
        let msg = format!("{err}");
        assert!(msg.contains("source inconnue"), "msg : {msg}");
        assert!(
            msg.contains("comparia"),
            "doit lister les sources connues : {msg}"
        );
    }

    #[test]
    fn build_context_with_creates_data_root() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("nested").join("data");
        assert!(!target.exists());
        let ctx = build_context_with(target.clone(), 42, false).unwrap();
        assert_eq!(ctx.data_root, target);
        assert!(target.exists(), "le dossier doit être créé");
        assert!(!ctx.incremental);
        assert_eq!(ctx.seed, 42);
    }

    #[test]
    fn build_context_with_honors_incremental_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = build_context_with(tmp.path().to_path_buf(), 1337, true).unwrap();
        assert!(ctx.incremental);
        assert_eq!(ctx.seed, 1337);
    }

    #[test]
    fn latest_copper_snapshot_returns_none_when_source_dir_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = build_context_with(tmp.path().to_path_buf(), 42, false).unwrap();
        let res = latest_copper_snapshot(&ctx, "comparia").unwrap();
        assert!(res.is_none(), "pas de snapshot → None");
    }

    #[test]
    fn latest_copper_snapshot_picks_most_recent_dir_with_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = build_context_with(tmp.path().to_path_buf(), 42, false).unwrap();
        let copper_root = ctx.copper_root("comparia");
        std::fs::create_dir_all(copper_root.join("2025-12-31")).unwrap();
        std::fs::create_dir_all(copper_root.join("2026-05-15")).unwrap();
        std::fs::create_dir_all(copper_root.join("2026-01-01")).unwrap();
        // Seul 2026-05-15 a un manifest valide → c'est lui qui doit gagner même
        // si "2026-05-15" est le tri max.
        std::fs::write(copper_root.join("2025-12-31").join("manifest.json"), "{}").unwrap();
        std::fs::write(copper_root.join("2026-05-15").join("manifest.json"), "{}").unwrap();
        // 2026-01-01 n'a pas de manifest, doit être ignoré.

        let res = latest_copper_snapshot(&ctx, "comparia").unwrap().unwrap();
        assert!(
            res.ends_with("2026-05-15"),
            "doit prendre le plus récent : {res:?}"
        );
    }

    #[test]
    fn latest_copper_snapshot_skips_directories_without_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = build_context_with(tmp.path().to_path_buf(), 42, false).unwrap();
        let copper_root = ctx.copper_root("rte-iris");
        std::fs::create_dir_all(copper_root.join("2026-05-15")).unwrap();
        // Pas de manifest → snapshot incomplet, doit être considéré inexistant.
        let res = latest_copper_snapshot(&ctx, "rte-iris").unwrap();
        assert!(res.is_none(), "pas de manifest → None");
    }
}

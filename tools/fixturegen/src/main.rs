//! Générateur de fixtures pour le mode démo web (C37).
//!
//! Produit des JSON consommés par `web/src/lib/demo/` quand l'app tourne
//! hors contexte Tauri (démo web déployée). Toutes les valeurs sortent du
//! VRAI moteur (`sobria-estimator`, seed 42, N=10 000) — aucune valeur
//! inventée, conformément à CLAUDE.md §13.
//!
//! Usage : `cargo run --release` → écrit dans `$OUT_DIR_FIXTURES` (défaut
//! `./fixtures`).

use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use sobria_core::{EmpreinteMethod, EstimationRequest};
use sobria_estimator::{engine_for, EstimationParams, AVAILABLE_METHODS, MODEL_REGISTRY};

/// Grille de tailles (tokens_in, tokens_out_estimated) couvrant un prompt
/// court, moyen, long. Les valeurs sont des points de grille de démo —
/// le frontend affiche toujours la requête écho (pas celle de l'utilisateur).
const SIZES: &[(u32, u32)] = &[(300, 150), (1200, 800), (4000, 2000)];

// Tous les modèles du registry (y compris deprecated 2024 : le sélecteur
// peut les présenter, l'historique d'audit les référence).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var("OUT_DIR_FIXTURES").unwrap_or_else(|_| "fixtures".into());
    std::fs::create_dir_all(&out)?;

    // Horodatage fixe → fixtures reproductibles au bit près.
    let ts = Utc.with_ymd_and_hms(2026, 6, 12, 0, 0, 0).unwrap();

    // ── 1. Catalogue modèles (mirror de `list_models`) ──────────────────
    let presets: Vec<Value> = MODEL_REGISTRY
        .iter()
        .map(|p| serde_json::to_value(p))
        .collect::<Result<_, _>>()?;
    std::fs::write(
        format!("{out}/models.json"),
        serde_json::to_string_pretty(&presets)?,
    )?;
    eprintln!("models.json: {} presets", presets.len());

    // ── 2. Méthodologies (mirror de `list_methodologies`) ───────────────
    let methods: Vec<Value> = AVAILABLE_METHODS
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()?;
    std::fs::write(
        format!("{out}/methodologies.json"),
        serde_json::to_string_pretty(&methods)?,
    )?;

    // ── 3. Estimations précalculées (vrai Monte-Carlo, seed 42) ─────────
    let mut estimates: Vec<Value> = Vec::new();
    for preset in MODEL_REGISTRY.iter() {
        let model_id = &preset.id;
        let params = EstimationParams::for_model(model_id)?;
        for method in [EmpreinteMethod::AfnorSobria, EmpreinteMethod::EcoLogits] {
            let engine = engine_for(method);
            for &(t_in, t_out) in SIZES {
                let req = EstimationRequest {
                    model_id: (*model_id).to_string(),
                    tokens_in: t_in,
                    tokens_out_estimated: t_out,
                    datacenter_id: None,
                    timestamp: ts,
                    modalities: vec![],
                    overhead: Default::default(),
                };
                let result = engine.estimate(&req, &params)?;
                let mut v = serde_json::to_value(&result)?;
                // Sentinel "non journalisé" — même convention que
                // `estimate_for_comparison` (audit_id = 0).
                v["audit_id"] = json!(0);
                estimates.push(v);
            }
        }
        eprintln!("estimates: {model_id} ok");
    }
    std::fs::write(
        format!("{out}/estimates.json"),
        serde_json::to_string_pretty(&estimates)?,
    )?;
    eprintln!("estimates.json: {} résultats", estimates.len());

    Ok(())
}

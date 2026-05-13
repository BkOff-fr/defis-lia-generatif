//! Binaire de l'application Tauri Sobr.ia — voir ADR-0001 et
//! `briefs/chantiers/C09-tauri-integration.md`.
//!
//! Le runtime Tauri se contente d'enregistrer les commandes IPC et de
//! charger le bundle frontend (SvelteKit produit par `web/`). Toute la
//! logique métier vit dans `sobria_app::logic` et est testable sans Tauri.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::doc_markdown)]

use sobria_app::{
    dto::{
        AppPreferencesDto, AuditEntrySummaryDto, EstimationRequestDto, EstimationResultDto,
        IntegrityReportDto, MetaInfo, ModelPresetDto, SimulationRequestDto, SimulationResultDto,
    },
    logic, AppState, IpcResult,
};
use tauri::Manager;
use tracing::info;

// ─────────────────────────────────────────────────────────────────────────────
// Commandes IPC — délégation pure vers `logic::*`
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
fn meta_info(state: tauri::State<'_, AppState>) -> IpcResult<MetaInfo> {
    logic::meta_info(state.inner())
}

#[tauri::command]
fn list_models() -> IpcResult<Vec<ModelPresetDto>> {
    logic::list_models()
}

#[tauri::command]
fn estimate_prompt(
    req: EstimationRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<EstimationResultDto> {
    logic::estimate_prompt(req, state.inner())
}

#[tauri::command]
fn verify_audit(state: tauri::State<'_, AppState>) -> IpcResult<IntegrityReportDto> {
    logic::verify_audit(state.inner())
}

#[tauri::command]
fn list_audit_entries(
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppState>,
) -> IpcResult<Vec<AuditEntrySummaryDto>> {
    logic::list_audit_entries(limit, offset, state.inner())
}

#[tauri::command]
fn export_audit_ndjson(
    path: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<usize> {
    logic::export_audit_ndjson(std::path::Path::new(&path), state.inner())
}

#[tauri::command]
fn get_app_preferences(state: tauri::State<'_, AppState>) -> IpcResult<AppPreferencesDto> {
    logic::get_app_preferences(state.inner())
}

#[tauri::command]
fn set_app_preferences(
    prefs: AppPreferencesDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<()> {
    logic::set_app_preferences(prefs, state.inner())
}

#[tauri::command]
fn simulate_scenarios(
    req: SimulationRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<SimulationResultDto> {
    logic::simulate_scenarios(req, state.inner())
}

// ─────────────────────────────────────────────────────────────────────────────
// Entrée principale
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("sobria=info,info")),
        )
        .init();
    info!("Sobr.ia démarre — v{}", logic::APP_VERSION);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::init(None).map_err(|e| {
                tracing::error!(error = %e, "AppState init failed");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            meta_info,
            list_models,
            estimate_prompt,
            verify_audit,
            list_audit_entries,
            export_audit_ndjson,
            get_app_preferences,
            set_app_preferences,
            simulate_scenarios,
        ])
        .run(tauri::generate_context!())
        .expect("erreur lors du démarrage de Sobr.ia");
}

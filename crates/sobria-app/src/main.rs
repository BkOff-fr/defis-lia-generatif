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
        AppPreferencesDto, AuditEntrySummaryDto, BenchmarkRequestDto, BenchmarkResultDto,
        CountryAggregateDto, CsrdReportRequestDto, CsrdReportResultDto, DatacenterDetailDto,
        DatacenterSummaryDto, EstimationRequestDto, EstimationResultDto,
        IndustrialSiteSummaryDto, IntegrityReportDto, MetaInfo, ModelPresetDto,
        RegionFrAggregateDto, SankeyDataDto, SimulationRequestDto, SimulationResultDto,
        YearlyForecastRequestDto, YearlyForecastResultDto,
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

#[tauri::command]
fn forecast_yearly_budget(
    req: YearlyForecastRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<YearlyForecastResultDto> {
    logic::forecast_yearly_budget(req, state.inner())
}

#[tauri::command]
fn benchmark_models(
    req: BenchmarkRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<BenchmarkResultDto> {
    logic::benchmark_models(req, state.inner())
}

#[tauri::command]
fn list_datacenters() -> IpcResult<Vec<DatacenterSummaryDto>> {
    logic::list_datacenters()
}

#[tauri::command]
fn get_datacenter_detail(
    id: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<DatacenterDetailDto> {
    logic::get_datacenter_detail(&id, state.inner())
}

#[tauri::command]
fn aggregate_datacenters_by_country() -> IpcResult<Vec<CountryAggregateDto>> {
    logic::aggregate_datacenters_by_country()
}

#[tauri::command]
fn list_industrial_sites_fr(
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppState>,
) -> IpcResult<Vec<IndustrialSiteSummaryDto>> {
    logic::list_industrial_sites_fr(limit, offset, state.inner())
}

#[tauri::command]
fn get_industrial_site_fr(
    code_iris: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<IndustrialSiteSummaryDto> {
    logic::get_industrial_site_fr(&code_iris, state.inner())
}

#[tauri::command]
fn aggregate_industrial_sites_by_region(
    state: tauri::State<'_, AppState>,
) -> IpcResult<Vec<RegionFrAggregateDto>> {
    logic::aggregate_industrial_sites_by_region(state.inner())
}

#[tauri::command]
fn sankey_fr_data(state: tauri::State<'_, AppState>) -> IpcResult<SankeyDataDto> {
    logic::sankey_fr_data(state.inner())
}

#[tauri::command]
fn export_csrd_report(
    req: CsrdReportRequestDto,
    output_dir: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<CsrdReportResultDto> {
    logic::export_csrd_report(req, std::path::Path::new(&output_dir), state.inner())
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
            forecast_yearly_budget,
            benchmark_models,
            list_datacenters,
            get_datacenter_detail,
            aggregate_datacenters_by_country,
            list_industrial_sites_fr,
            get_industrial_site_fr,
            aggregate_industrial_sites_by_region,
            sankey_fr_data,
            export_csrd_report,
        ])
        .run(tauri::generate_context!())
        .expect("erreur lors du démarrage de Sobr.ia");
}

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

use std::path::PathBuf;

use sobria_app::{
    bridge_install::{self, BridgeStatus, BrowserKind},
    bridge_server,
    dto::{
        AppPreferencesDto, AuditEntrySummaryDto, BatchRequestDto, BatchResultDto,
        BenchmarkRequestDto, BenchmarkResultDto, BudgetStatusDto, CountryAggregateDto,
        CreateProjectDto, CsrdReportRequestDto, CsrdReportResultDto, DashboardSummaryDto,
        DatacenterDetailDto, DatacenterSummaryDto, DatasheetDto, EstimationRequestDto,
        EstimationResultDto, ExtensionEventDto, IndustrialSiteSummaryDto, IntegrityReportDto,
        MetaInfo, MethodologyInfoDto, ModelDetailDto, ModelPresetDto, PairingCodeDto, PairingDto,
        PairingSecretDto, PersonalGoalDto, ProjectDto, ReferentielReloadResultDto,
        ReferentielStatusDto, RegionFrAggregateDto, SankeyDataDto, SimulationRequestDto,
        SimulationResultDto, UpdateProjectDto, VendorComparisonRowDto, YearlyForecastRequestDto,
        YearlyForecastResultDto,
    },
    logic,
    team_client::{self, EnrollRequest, TeamClient},
    team_settings::{self, TeamStatus},
    AppState, IpcError, IpcResult,
};
use tauri::Manager;
use tracing::info;

/// Chemin attendu du binaire `sobria-bridge` à côté de l'app. Utilisé pour
/// remplir le manifest natif et indiquer à l'extension où trouver le pont.
/// Si l'exécutable courant n'est pas localisable, on retombe sur `None` et
/// les IPC d'install rejettent proprement.
fn resolve_bridge_path() -> Option<PathBuf> {
    let current = std::env::current_exe().ok()?;
    let dir = current.parent()?.to_path_buf();
    let name = if cfg!(target_os = "windows") {
        "sobria-bridge.exe"
    } else {
        "sobria-bridge"
    };
    Some(dir.join(name))
}

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
fn list_methodologies() -> IpcResult<Vec<MethodologyInfoDto>> {
    logic::list_methodologies()
}

#[tauri::command]
fn get_model_detail(id: String, state: tauri::State<'_, AppState>) -> IpcResult<ModelDetailDto> {
    logic::get_model_detail(&id, state.inner())
}

/// **C32.4** — Liste agrégée des vendor disclosures par fabricant pour la
/// table comparaison M9 (5 lignes, Mistral / Google / Meta / Anthropic / OpenAI).
#[tauri::command]
fn list_vendor_comparison() -> IpcResult<Vec<VendorComparisonRowDto>> {
    logic::list_vendor_comparison()
}

#[tauri::command]
fn estimate_prompt(
    req: EstimationRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<EstimationResultDto> {
    logic::estimate_prompt(req, state.inner())
}

#[tauri::command]
fn estimate_for_comparison(
    req: EstimationRequestDto,
    method: sobria_core::EmpreinteMethod,
) -> IpcResult<EstimationResultDto> {
    logic::estimate_for_comparison(req, method)
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
fn export_audit_ndjson(path: String, state: tauri::State<'_, AppState>) -> IpcResult<usize> {
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

#[tauri::command]
fn get_dashboard_summary(
    period: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<DashboardSummaryDto> {
    logic::get_dashboard_summary(&period, state.inner())
}

#[tauri::command]
fn list_personal_goals(state: tauri::State<'_, AppState>) -> IpcResult<Vec<PersonalGoalDto>> {
    logic::list_personal_goals(state.inner())
}

#[tauri::command]
fn set_personal_goal(goal: PersonalGoalDto, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    logic::set_personal_goal(goal, state.inner())
}

#[tauri::command]
fn delete_personal_goal(
    indicator: String,
    period: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<()> {
    logic::delete_personal_goal(&indicator, &period, state.inner())
}

#[tauri::command]
fn get_budget_status(state: tauri::State<'_, AppState>) -> IpcResult<Vec<BudgetStatusDto>> {
    logic::get_budget_status(state.inner())
}

#[tauri::command]
fn list_projects(state: tauri::State<'_, AppState>) -> IpcResult<Vec<ProjectDto>> {
    logic::list_projects(state.inner())
}

#[tauri::command]
fn get_project(id: i64, state: tauri::State<'_, AppState>) -> IpcResult<ProjectDto> {
    logic::get_project(id, state.inner())
}

#[tauri::command]
fn create_project(
    req: CreateProjectDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<ProjectDto> {
    logic::create_project(req, state.inner())
}

#[tauri::command]
fn update_project(
    id: i64,
    req: UpdateProjectDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<ProjectDto> {
    logic::update_project(id, req, state.inner())
}

#[tauri::command]
fn delete_project(id: i64, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    logic::delete_project(id, state.inner())
}

#[tauri::command]
fn generate_project_datasheet(
    id: i64,
    state: tauri::State<'_, AppState>,
) -> IpcResult<DatasheetDto> {
    logic::generate_project_datasheet(id, state.inner())
}

#[tauri::command]
fn run_batch_from_csv(
    req: BatchRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<BatchResultDto> {
    logic::run_batch_from_csv(req, state.inner())
}

#[tauri::command]
fn get_referentiel_status() -> IpcResult<ReferentielStatusDto> {
    logic::get_referentiel_status()
}

#[tauri::command]
fn reload_referentiel() -> IpcResult<ReferentielReloadResultDto> {
    logic::reload_referentiel()
}

// ─── C27.5 — extension navigateur ────────────────────────────────────────────

#[tauri::command]
fn regenerate_pairing_code(state: tauri::State<'_, AppState>) -> IpcResult<PairingCodeDto> {
    logic::regenerate_pairing_code(state.inner())
}

#[tauri::command]
fn get_pairing_code_status(state: tauri::State<'_, AppState>) -> IpcResult<Option<PairingCodeDto>> {
    logic::get_pairing_code_status(state.inner())
}

#[tauri::command]
fn verify_pairing_code(
    code: String,
    fingerprint: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<PairingSecretDto> {
    logic::verify_pairing_code(state.inner(), &code, &fingerprint)
}

#[tauri::command]
fn list_pairings(state: tauri::State<'_, AppState>) -> IpcResult<Vec<PairingDto>> {
    logic::list_pairings(state.inner())
}

#[tauri::command]
fn revoke_pairing(id: String, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    logic::revoke_pairing(state.inner(), &id)
}

#[tauri::command]
fn list_extension_events(
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppState>,
) -> IpcResult<Vec<ExtensionEventDto>> {
    logic::list_extension_events(state.inner(), limit, offset)
}

#[tauri::command]
fn drain_extension_spool(state: tauri::State<'_, AppState>) -> IpcResult<usize> {
    logic::drain_extension_spool(state.inner())
}

// ─── C27 patch v0.6.0 — auto-install bridge ─────────────────────────────────

#[tauri::command]
fn bridge_status() -> BridgeStatus {
    bridge_install::bridge_status(resolve_bridge_path())
}

#[tauri::command]
fn install_extension_bridge(
    browsers: Vec<BrowserKind>,
    extension_id: String,
) -> IpcResult<Vec<PathBuf>> {
    let bridge_path = resolve_bridge_path().ok_or_else(|| {
        IpcError::new(
            "internal",
            "binaire sobria-bridge introuvable à côté de l'app",
        )
    })?;
    let mut written = Vec::with_capacity(browsers.len());
    for b in browsers {
        let path = bridge_install::install_native_manifest(b, &bridge_path, &extension_id)
            .map_err(|e| IpcError::new("internal", format!("install {}: {e}", b.id())))?;
        written.push(path);
    }
    Ok(written)
}

// ─── C28.6 — Mode Équipe (self-hosted aggregator) ────────────────────────────

#[tauri::command]
fn team_status(state: tauri::State<'_, AppState>) -> IpcResult<TeamStatus> {
    let store = state
        .team_settings
        .lock()
        .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
    store
        .snapshot()
        .map_err(|e| IpcError::new("internal", format!("team_status: {e}")))
}

#[tauri::command]
fn team_set_url(url: String, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    if !url.starts_with("https://") {
        return Err(IpcError::new(
            "bad_request",
            "url doit commencer par https://",
        ));
    }
    let trimmed = url.trim_end_matches('/').to_string();
    let store = state
        .team_settings
        .lock()
        .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
    store
        .set(team_settings::KEY_URL, &trimmed)
        .map_err(|e| IpcError::new("internal", format!("team_set_url: {e}")))?;
    Ok(())
}

#[tauri::command]
fn team_set_mode(mode: String, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    let normalized = team_settings::TeamMode::parse(&mode);
    let store = state
        .team_settings
        .lock()
        .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
    store
        .set(team_settings::KEY_MODE, normalized.as_str())
        .map_err(|e| IpcError::new("internal", format!("team_set_mode: {e}")))?;
    Ok(())
}

#[tauri::command]
fn team_set_accept_invalid_certs(accept: bool, state: tauri::State<'_, AppState>) -> IpcResult<()> {
    let store = state
        .team_settings
        .lock()
        .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
    store
        .set(
            team_settings::KEY_ACCEPT_INVALID_CERTS,
            if accept { "1" } else { "0" },
        )
        .map_err(|e| IpcError::new("internal", format!("team_set_accept: {e}")))?;
    Ok(())
}

#[tauri::command]
async fn team_ping(state: tauri::State<'_, AppState>) -> IpcResult<team_client::HealthResponse> {
    let cfg = {
        let store = state
            .team_settings
            .lock()
            .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
        team_client::ClientConfig::read(&store).map_err(team_client::map_team_err)?
    };
    let client = TeamClient::new(cfg).map_err(team_client::map_team_err)?;
    let resp = client.ping().await.map_err(team_client::map_team_err)?;
    // C29.1 — toute connexion réussie met à jour `last_seen_at`.
    if let Ok(store) = state.team_settings.lock() {
        let _ = store.mark_seen_now();
    }
    Ok(resp)
}

#[tauri::command]
async fn team_enroll(
    code: String,
    password: String,
    fingerprint: String,
    display_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> IpcResult<team_client::EnrollResponse> {
    let req = EnrollRequest {
        code,
        password,
        fingerprint,
        display_name,
    };
    let cfg = {
        let store = state
            .team_settings
            .lock()
            .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
        team_client::ClientConfig::read(&store).map_err(team_client::map_team_err)?
    };
    let client = TeamClient::new(cfg).map_err(team_client::map_team_err)?;
    let (resp, side) = client
        .enroll(req)
        .await
        .map_err(team_client::map_team_err)?;
    // Persiste tokens + side effects en re-lockant le store.
    {
        let store = state
            .team_settings
            .lock()
            .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
        team_client::persist::save_tokens(&store, &resp.access_token, &resp.refresh_token)
            .map_err(team_client::map_team_err)?;
        team_client::persist::save_enroll_side_effects(&store, &side)
            .map_err(team_client::map_team_err)?;
    }
    Ok(resp)
}

#[tauri::command]
fn team_logout(state: tauri::State<'_, AppState>) -> IpcResult<()> {
    let store = state
        .team_settings
        .lock()
        .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
    team_client::logout_local(&store)
        .map_err(|e| IpcError::new("internal", format!("team_logout: {e}")))
}

#[tauri::command]
async fn team_push_estimation(
    payload: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> IpcResult<bool> {
    // Best-effort : retourne `false` si dispatch non éligible (mode=local
    // ou non enrôlé), pas une erreur.
    let (snapshot, cfg) = {
        let store = state
            .team_settings
            .lock()
            .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
        let snap = store
            .snapshot()
            .map_err(|e| IpcError::new("internal", format!("snapshot: {e}")))?;
        let cfg = if team_client::should_dispatch_team(&snap) {
            Some(team_client::ClientConfig::read(&store).map_err(team_client::map_team_err)?)
        } else {
            None
        };
        (snap, cfg)
    };
    let Some(cfg) = cfg else {
        return Ok(false);
    };
    let _ = snapshot;
    let client = TeamClient::new(cfg).map_err(team_client::map_team_err)?;
    let outcome = client
        .push_estimation(&payload)
        .await
        .map_err(team_client::map_team_err)?;
    if let Some((access, refresh)) = outcome.new_tokens {
        let store = state
            .team_settings
            .lock()
            .map_err(|_| IpcError::new("internal", "team_settings mutex poisoned"))?;
        team_client::persist::save_tokens(&store, &access, &refresh)
            .map_err(team_client::map_team_err)?;
    }
    // C29.1 — push réussi : compteur + timestamp.
    if outcome.data.ack {
        if let Ok(store) = state.team_settings.lock() {
            let _ = store.increment_estimations_sent();
            let _ = store.mark_seen_now();
        }
    }
    Ok(outcome.data.ack)
}

#[tauri::command]
fn uninstall_extension_bridge(browsers: Vec<BrowserKind>) -> IpcResult<()> {
    for b in browsers {
        bridge_install::uninstall_native_manifest(b)
            .map_err(|e| IpcError::new("internal", format!("uninstall {}: {e}", b.id())))?;
    }
    Ok(())
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

            // C27 patch v0.6.0 — démarre le socket server pour le forward
            // bridge ↔ app en temps réel (Unix socket / Windows named pipe).
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = bridge_server::run(app_handle).await {
                    tracing::warn!(error = %e, "bridge_server: arrêt anormal");
                }
            });

            // C27 patch v0.6.0 — poller de fallback offline : draine toutes
            // les 5 s le spool écrit par le bridge quand l'app était fermée
            // ou que le socket était injoignable.
            let spool_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = tokio::time::interval(std::time::Duration::from_secs(5));
                ticker.tick().await; // saute le premier tick immédiat
                loop {
                    ticker.tick().await;
                    let state = spool_handle.state::<AppState>();
                    if let Err(e) = logic::drain_extension_spool(state.inner()) {
                        tracing::debug!(error = %e, "spool poller: drain a échoué");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            meta_info,
            list_models,
            list_methodologies,
            get_model_detail,
            list_vendor_comparison,
            estimate_prompt,
            estimate_for_comparison,
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
            get_dashboard_summary,
            list_personal_goals,
            set_personal_goal,
            delete_personal_goal,
            get_budget_status,
            list_projects,
            get_project,
            create_project,
            update_project,
            delete_project,
            generate_project_datasheet,
            run_batch_from_csv,
            get_referentiel_status,
            reload_referentiel,
            regenerate_pairing_code,
            get_pairing_code_status,
            verify_pairing_code,
            list_pairings,
            revoke_pairing,
            list_extension_events,
            drain_extension_spool,
            bridge_status,
            install_extension_bridge,
            uninstall_extension_bridge,
            team_status,
            team_set_url,
            team_set_mode,
            team_set_accept_invalid_certs,
            team_ping,
            team_enroll,
            team_logout,
            team_push_estimation,
        ])
        .run(tauri::generate_context!())
        .expect("erreur lors du démarrage de Sobr.ia");
}

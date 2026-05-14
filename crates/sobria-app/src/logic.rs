//! Logique métier des commandes IPC, **testable sans Tauri**.
//!
//! Chaque commande `#[tauri::command]` du binaire `sobria-app` se réduit
//! à un appel vers une fonction ici. Cela permet :
//! - des tests unitaires rapides (`cargo test -p sobria-app`),
//! - une réutilisation possible côté CLI plus tard (chantier C10),
//! - une frontière propre `IpcError` ↔ logique interne.

use chrono::Utc;
use sobria_core::{EstimationRequest, Indicator, ModuleId};
use sobria_estimator::{
    available_models, find_preset, Distribution, EstimationParams,
};
use sobria_geoloc::{
    aggregate_by_country, aggregate_by_region, all_datacenters, find_datacenter,
    find_site_by_code_iris, generate_sankey_fr, load_rte_mix, load_territoire_fr, DatacenterRecord,
    TerritoireFrArtifact, TerritoireFrError,
};
use tracing::{debug, info};

use crate::{
    batch,
    dashboard,
    dto::{
        AppPreferencesDto, AuditEntrySummaryDto, BatchAggregateDto, BatchModelAggregateDto,
        BatchRequestDto, BatchResultDto, BenchmarkOutcomeDto, BenchmarkRequestDto,
        BenchmarkResultDto, BudgetStatusDto, CompositionDto, CountryAggregateDto,
        CreateProjectDto, CsrdReportRequestDto, CsrdReportResultDto, DashboardSummaryDto,
        DatacenterDetailDto, DatacenterSummaryDto, DatasheetDto, EstimationRequestDto,
        EstimationResultDto, IndustrialSiteSummaryDto, IntegrityReportDto, MetaInfo,
        ModelDetailDto, ModelPresetDto, PersonalGoalDto, ProjectDto, RegionFrAggregateDto,
        SankeyDataDto, SimulationRequestDto, SimulationResultDto, TripletDto, UpdateProjectDto,
        YearlyForecastRequestDto, YearlyForecastResultDto,
    },
    error::{AppError, IpcError, IpcResult},
    goals_store::{GoalIndicator, GoalPeriod, PersonalGoal},
    preferences_store::StoredPreferences,
    project_store::{NewProject, Project, ProjectUpdate},
    state::AppState,
};

/// Constante exposée par `meta_info`.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Métadonnées runtime.
/// Polish G (C24) — Helper interne : résout la méthodologie effective
/// pour les estimations « ambient » (fiches statiques M9/M12, simulateur
/// M13, forecaster M16) qui n'ont pas de `method` explicite dans la
/// requête frontend.
///
/// Lit `prefs.default_method` avec fallback `EmpreinteMethod::default_method()`
/// si erreur disque ou store vide. Lecture best-effort — ne propage pas
/// l'erreur, car ces écrans doivent rester utilisables même si le store
/// de préférences est cassé.
fn user_default_method(state: &AppState) -> sobria_core::EmpreinteMethod {
    state
        .preferences
        .lock()
        .ok()
        .and_then(|store| store.read_all().ok())
        .and_then(|p| p.default_method)
        .unwrap_or_else(sobria_core::EmpreinteMethod::default_method)
}

pub fn meta_info(state: &AppState) -> IpcResult<MetaInfo> {
    Ok(MetaInfo {
        app_version: APP_VERSION.into(),
        estimator_seed: state.estimator.seed(),
        estimator_n: state.estimator.n(),
        audit_path: state.audit_path.display().to_string(),
        data_root: state.data_root.display().to_string(),
    })
}

/// Liste les presets de modèles disponibles (≥ 8 — voir C06).
pub fn list_models() -> IpcResult<Vec<ModelPresetDto>> {
    Ok(available_models().into_iter().map(Into::into).collect())
}

/// Liste les méthodologies d'empreinte LLM embarquées dans Sobr.ia (C24).
///
/// Cette commande IPC alimente la page `/methodologies` (catalogue) et
/// les sélecteurs Settings → "Méthodologie par défaut" / "Voir aussi".
/// Pas d'effet de bord — lecture seule du registry compile-time.
pub fn list_methodologies() -> IpcResult<Vec<crate::dto::MethodologyInfoDto>> {
    Ok(sobria_estimator::AVAILABLE_METHODS
        .iter()
        .map(crate::dto::MethodologyInfoDto::from)
        .collect())
}

/// Détaille un modèle (M9 / C18) avec ses params distributionnels et un
/// baseline contextuel calculé sur prompt référence 100/500 tokens.
///
/// **Pas journalisé** dans l'audit ledger — c'est une fiche statique
/// pédagogique, pas un acte d'estimation utilisateur.
pub fn get_model_detail(model_id: &str, state: &AppState) -> IpcResult<ModelDetailDto> {
    let preset = find_preset(model_id).ok_or_else(|| {
        IpcError::new("not_found", format!("modèle '{model_id}' inconnu"))
    })?;
    let openness = match preset.openness {
        sobria_estimator::Openness::Open => "open",
        sobria_estimator::Openness::OpenWeights => "open_weights",
        sobria_estimator::Openness::Closed => "closed",
    };
    let calibration = match preset.calibration {
        sobria_estimator::CalibrationStatus::Validated => "validated",
        sobria_estimator::CalibrationStatus::Indicative => "indicative",
        sobria_estimator::CalibrationStatus::Extrapolated => "extrapolated",
    };

    // Baseline pour contexte (sans journalisation).
    // Polish G — Honore la méthodologie par défaut de l'utilisateur,
    // sinon la fiche modèle affiche un baseline AFNOR alors que l'user
    // calcule en EcoLogits (mensonge visuel).
    let params = EstimationParams::for_model(model_id).map_err(AppError::from)?;
    let req = sobria_core::EstimationRequest {
        model_id: model_id.to_string(),
        tokens_in: 100,
        tokens_out_estimated: 500,
        datacenter_id: None,
        timestamp: Utc::now(),
    };
    let engine = sobria_estimator::engine_for(user_default_method(state));
    let result = engine.estimate(&req, &params).map_err(AppError::from)?;
    let co2 = result
        .indicators
        .iter()
        .find(|i| matches!(i.indicator, Indicator::Co2Eq))
        .ok_or_else(|| AppError::Internal("CO2eq indicator manquant".into()))?;
    let energy_p50 = result
        .indicators
        .iter()
        .find(|i| matches!(i.indicator, Indicator::Energy))
        .map_or(0.0, |i| i.interval.p50);
    let water_p50 = result
        .indicators
        .iter()
        .find(|i| matches!(i.indicator, Indicator::Water))
        .map_or(0.0, |i| i.interval.p50);

    Ok(ModelDetailDto {
        id: preset.id.into(),
        display_name: preset.display_name.into(),
        provider: preset.provider.into(),
        family: preset.family.into(),
        approx_params_billions: preset.approx_params_billions,
        openness: openness.into(),
        calibration: calibration.into(),
        sources: preset.sources.iter().map(|s| (*s).to_string()).collect(),
        epsilon_prefill_mj_per_token: TripletDto {
            p5: preset.epsilon_prefill_mj.0,
            p50: preset.epsilon_prefill_mj.1,
            p95: preset.epsilon_prefill_mj.2,
        },
        epsilon_decode_mj_per_token: TripletDto {
            p5: preset.epsilon_decode_mj.0,
            p50: preset.epsilon_decode_mj.1,
            p95: preset.epsilon_decode_mj.2,
        },
        embodied_g_per_request: TripletDto {
            p5: preset.embodied_g_per_req.0,
            p50: preset.embodied_g_per_req.1,
            p95: preset.embodied_g_per_req.2,
        },
        baseline_co2eq_p5_g: co2.interval.p5,
        baseline_co2eq_p50_g: co2.interval.p50,
        baseline_co2eq_p95_g: co2.interval.p95,
        baseline_energy_wh_p50: energy_p50,
        baseline_water_l_p50: water_p50,
    })
}

/// Estime un prompt + journalise dans le ledger.
///
/// Étapes :
/// 1. Validation de la requête.
/// 2. Lookup `EstimationParams` pour le `model_id`.
/// 3. Monte-Carlo via `MonteCarloEngine::estimate`.
/// 4. Append au ledger (lock acquis).
/// 5. Retourne le DTO + `audit_id`.
pub fn estimate_prompt(
    req: EstimationRequestDto,
    state: &AppState,
) -> IpcResult<EstimationResultDto> {
    // Vérif modèle connu avant toute conversion (erreur stable côté UI).
    if find_preset(&req.model_id).is_none() {
        return Err(IpcError::from(AppError::UnknownModel(req.model_id.clone())));
    }

    // Validation basique avant Monte-Carlo (le moteur valide aussi).
    if req.tokens_in == 0 && req.tokens_out_estimated == 0 {
        return Err(IpcError::from(AppError::InvalidRequest(
            "tokens_in et tokens_out_estimated sont tous les deux nuls".into(),
        )));
    }

    let model_id = req.model_id.clone();
    let method_override = req.method;
    let core_req = req.into_core(Utc::now());
    let params = EstimationParams::for_model(&model_id).map_err(AppError::from)?;

    // Choix de la méthodologie : surcharge explicite > préférence user > défaut.
    let method = method_override.unwrap_or_else(|| {
        // Lecture best-effort des préférences ; si erreur, fallback default.
        state
            .preferences
            .lock()
            .ok()
            .and_then(|store| store.read_all().ok())
            .and_then(|p| p.default_method)
            .unwrap_or_else(sobria_core::EmpreinteMethod::default_method)
    });
    let engine = sobria_estimator::engine_for(method);

    let result = engine
        .estimate(&core_req, &params)
        .map_err(AppError::from)?;

    let mut ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let entry = ledger.append(&result).map_err(AppError::from)?;
    let audit_id = entry.id;
    drop(ledger);

    info!(
        model = %model_id,
        audit_id,
        co2eq_p50 = ?result.indicators.first().map(|i| i.interval.p50),
        "estimate_prompt: ok"
    );
    Ok(EstimationResultDto::from_result(&result, audit_id))
}

/// Lance une estimation **éphémère** pour le panneau "Voir aussi" (C24).
///
/// Différence avec [`estimate_prompt`] :
/// - La méthodologie est **toujours** spécifiée par l'appelant (pas de
///   fallback sur la préférence user — c'est explicitement une
///   comparaison).
/// - Le résultat **n'est PAS journalisé** dans l'audit ledger.
///
/// Pourquoi ne pas logguer ? Le panneau "Voir aussi" fait tourner 1 à N
/// méthodologies alternatives à chaque estimation principale, ce qui
/// multiplierait le volume du ledger sans valeur ajoutée pour
/// l'utilisateur : ces calculs sont *exploratoires*, pas des décisions.
/// L'estimation principale (la méthodo par défaut) reste, elle, dans le
/// ledger via [`estimate_prompt`] — c'est elle qui sert pour les
/// rapports CSRD, le dashboard et le datasheet Gebru.
///
/// L'`audit_id` du DTO retourné est `0` (sentinel "non journalisé").
///
/// Cette fonction est *stateless* (pas d'accès au `AppState`) — un
/// comparatif n'a pas à lire les préférences user ni à toucher au
/// ledger. C'est volontaire.
pub fn estimate_for_comparison(
    req: EstimationRequestDto,
    method: sobria_core::EmpreinteMethod,
) -> IpcResult<EstimationResultDto> {
    if find_preset(&req.model_id).is_none() {
        return Err(IpcError::from(AppError::UnknownModel(req.model_id.clone())));
    }
    if req.tokens_in == 0 && req.tokens_out_estimated == 0 {
        return Err(IpcError::from(AppError::InvalidRequest(
            "tokens_in et tokens_out_estimated sont tous les deux nuls".into(),
        )));
    }

    let model_id = req.model_id.clone();
    let core_req = req.into_core(Utc::now());
    let params = EstimationParams::for_model(&model_id).map_err(AppError::from)?;
    let engine = sobria_estimator::engine_for(method);
    let result = engine
        .estimate(&core_req, &params)
        .map_err(AppError::from)?;

    debug!(
        model = %model_id,
        method = method.as_str(),
        co2eq_p50 = ?result.indicators.first().map(|i| i.interval.p50),
        "estimate_for_comparison: ok (no ledger write)"
    );
    // audit_id = 0 → sentinel "estimation éphémère, non journalisée".
    Ok(EstimationResultDto::from_result(&result, 0))
}

/// Vérifie l'intégrité de la chaîne d'audit.
pub fn verify_audit(state: &AppState) -> IpcResult<IntegrityReportDto> {
    let ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let report = ledger.verify_chain().map_err(AppError::from)?;
    debug!(valid = report.valid, total = report.total_entries, "audit: verify");
    Ok((&report).into())
}

/// Liste les entrées d'audit (résumé) avec pagination.
///
/// `limit` est borné à 1000 pour protéger l'UI.
pub fn list_audit_entries(
    limit: u32,
    offset: u32,
    state: &AppState,
) -> IpcResult<Vec<AuditEntrySummaryDto>> {
    let limit = limit.min(1000) as usize;
    let offset = offset as usize;
    let ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    // L'API publique du ledger n'expose pas (encore) une lecture paginée :
    // on fait via export NDJSON en mémoire et tranche. Acceptable au volume
    // attendu (audit < 10⁵ entrées). À factoriser en C10 si besoin.
    let mut buf: Vec<u8> = Vec::new();
    ledger.export_ndjson(&mut buf).map_err(AppError::from)?;
    drop(ledger);
    let text = String::from_utf8(buf).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut out: Vec<AuditEntrySummaryDto> = Vec::with_capacity(limit);
    for (i, line) in text.lines().enumerate() {
        if i < offset {
            continue;
        }
        if out.len() >= limit {
            break;
        }
        let entry: sobria_audit::AuditEntry =
            serde_json::from_str(line).map_err(AppError::from)?;
        out.push(AuditEntrySummaryDto::from_entry(&entry));
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// datacenters (C12 — M12)
// ─────────────────────────────────────────────────────────────────────────────

/// Liste tous les datacenters européens connus (28 en v1.0).
pub fn list_datacenters() -> IpcResult<Vec<DatacenterSummaryDto>> {
    Ok(all_datacenters().iter().map(Into::into).collect())
}

/// Retourne le détail d'un datacenter avec un baseline calculé (gpt-4o-mini
/// 100/500 tokens) pour les barres de l'UI drill-down.
pub fn get_datacenter_detail(id: &str, state: &AppState) -> IpcResult<DatacenterDetailDto> {
    let dc = find_datacenter(id).ok_or_else(|| {
        IpcError::new("not_found", format!("datacenter '{id}' inconnu"))
            .with_details(serde_json::json!({ "id": id }))
    })?;
    let (co2, energy, water) = compute_baseline_for_dc(dc, state).map_err(IpcError::from)?;
    Ok(DatacenterDetailDto {
        id: dc.id.clone(),
        name: dc.name.clone(),
        operator: dc.operator.clone(),
        country_iso: dc.country_iso.clone(),
        city: dc.city.clone(),
        lat: dc.lat,
        lon: dc.lon,
        pue: dc.pue,
        if_electrical_g_per_kwh: dc.if_electrical_g_per_kwh,
        wue_l_per_kwh: dc.wue_l_per_kwh,
        capacity_mw: dc.capacity_mw,
        sources: dc.sources.clone(),
        hourly_profile_24h: dc.hourly_profile_24h.clone(),
        baseline_co2eq_p50_g: co2,
        baseline_energy_wh_p50: energy,
        baseline_water_l_p50: water,
    })
}

/// Agrège les 28 datacenters par pays (13 pays en v1.0).
pub fn aggregate_datacenters_by_country() -> IpcResult<Vec<CountryAggregateDto>> {
    Ok(aggregate_by_country().iter().map(Into::into).collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// territoire_fr (C13 — M20 Territoire FR)
// ─────────────────────────────────────────────────────────────────────────────

/// Chemin attendu pour le JSON ODRÉ produit par `sobria-ingest`.
fn territoire_fr_path(state: &AppState) -> std::path::PathBuf {
    state.data_root.join("territoire_fr.json")
}

fn rte_mix_path(state: &AppState) -> std::path::PathBuf {
    state.data_root.join("rte_mix_fr.json")
}

fn territoire_or_data_not_ingested(e: TerritoireFrError) -> IpcError {
    match e {
        TerritoireFrError::NotFound(p) => IpcError::new(
            "data_not_ingested",
            format!(
                "données territoire FR absentes ({}). Lance : \
                 cargo run -p sobria-ingest -- fetch territoire-fr",
                p.display()
            ),
        ),
        TerritoireFrError::Json(e) => {
            IpcError::new("io_error", format!("territoire_fr.json corrompu : {e}"))
        },
        TerritoireFrError::Schema(m) => IpcError::new(
            "data_not_ingested",
            format!("territoire_fr.json non conforme : {m}"),
        ),
        TerritoireFrError::Io(e) => IpcError::new("io_error", e.to_string()),
    }
}

fn sankey_or_data_not_ingested(e: sobria_geoloc::SankeyFrError) -> IpcError {
    match e {
        sobria_geoloc::SankeyFrError::NotFound(p) => IpcError::new(
            "data_not_ingested",
            format!(
                "données RTE mix absentes ({}). Lance : \
                 cargo run -p sobria-ingest -- fetch rte-mix --year 2023",
                p.display()
            ),
        ),
        sobria_geoloc::SankeyFrError::Json(e) => {
            IpcError::new("io_error", format!("rte_mix_fr.json corrompu : {e}"))
        },
        sobria_geoloc::SankeyFrError::Schema(m) => {
            IpcError::new("data_not_ingested", format!("rte_mix_fr.json non conforme : {m}"))
        },
        sobria_geoloc::SankeyFrError::Io(e) => IpcError::new("io_error", e.to_string()),
    }
}

fn load_or_err(state: &AppState) -> IpcResult<TerritoireFrArtifact> {
    load_territoire_fr(&territoire_fr_path(state)).map_err(territoire_or_data_not_ingested)
}

/// Liste paginée des sites industriels (top par consommation totale décroissante).
pub fn list_industrial_sites_fr(
    limit: u32,
    offset: u32,
    state: &AppState,
) -> IpcResult<Vec<IndustrialSiteSummaryDto>> {
    let artifact = load_or_err(state)?;
    let limit = (limit as usize).min(1000);
    let offset = offset as usize;
    let dtos: Vec<IndustrialSiteSummaryDto> = artifact
        .industrial_sites
        .iter()
        .skip(offset)
        .take(limit)
        .map(Into::into)
        .collect();
    Ok(dtos)
}

/// Détail d'un site IRIS par son code.
pub fn get_industrial_site_fr(
    code_iris: &str,
    state: &AppState,
) -> IpcResult<IndustrialSiteSummaryDto> {
    let artifact = load_or_err(state)?;
    let site = find_site_by_code_iris(&artifact, code_iris).ok_or_else(|| {
        IpcError::new("not_found", format!("site IRIS '{code_iris}' inconnu"))
    })?;
    Ok(site.into())
}

/// Agrégation des sites industriels par région ISO.
pub fn aggregate_industrial_sites_by_region(
    state: &AppState,
) -> IpcResult<Vec<RegionFrAggregateDto>> {
    let artifact = load_or_err(state)?;
    Ok(aggregate_by_region(&artifact).iter().map(Into::into).collect())
}

/// Génère les données du Sankey énergétique national à partir du mix RTE chargé.
pub fn sankey_fr_data(state: &AppState) -> IpcResult<SankeyDataDto> {
    let mix = load_rte_mix(&rte_mix_path(state)).map_err(sankey_or_data_not_ingested)?;
    let sankey = generate_sankey_fr(&mix);
    Ok((&sankey).into())
}

// ─────────────────────────────────────────────────────────────────────────────
// rapport CSRD/AGEC (C14 — M22)
// ─────────────────────────────────────────────────────────────────────────────

/// Génère un rapport CSRD/AGEC pour la période demandée et écrit les
/// fichiers `report.pdf` + `provo.jsonld` dans `output_dir`.
///
/// La période est inclusive sur `period_start` et exclusive sur `period_end`.
pub fn export_csrd_report(
    req: CsrdReportRequestDto,
    output_dir: &std::path::Path,
    state: &AppState,
) -> IpcResult<CsrdReportResultDto> {
    // 1. Parse les dates ISO 8601.
    let period_start = chrono::DateTime::parse_from_rfc3339(&req.period_start)
        .map_err(|e| {
            IpcError::from(AppError::InvalidRequest(format!(
                "period_start invalide (attendu RFC 3339) : {e}"
            )))
        })?
        .with_timezone(&Utc);
    let period_end = chrono::DateTime::parse_from_rfc3339(&req.period_end)
        .map_err(|e| {
            IpcError::from(AppError::InvalidRequest(format!(
                "period_end invalide (attendu RFC 3339) : {e}"
            )))
        })?
        .with_timezone(&Utc);
    if period_end <= period_start {
        return Err(IpcError::from(AppError::InvalidRequest(
            "period_end doit être strictement après period_start".into(),
        )));
    }
    if req.organization_name.trim().is_empty() {
        return Err(IpcError::from(AppError::InvalidRequest(
            "organization_name ne doit pas être vide".into(),
        )));
    }
    if !matches!(req.locale.as_str(), "fr" | "en") {
        return Err(IpcError::from(AppError::InvalidRequest(format!(
            "locale '{}' non supportée (fr|en)",
            req.locale
        ))));
    }

    // 2. Lit toutes les entrées de l'audit ledger.
    let entries = read_all_audit_entries(state)?;

    // 3. Génère le rapport.
    let export_req = sobria_export::ReportRequest {
        period_start,
        period_end,
        organization_name: req.organization_name.clone(),
        locale: req.locale.clone(),
        app_version: APP_VERSION.into(),
        estimator_seed: state.estimator.seed(),
        estimator_n: state.estimator.n(),
    };
    let artifacts = sobria_export::generate_report(&export_req, &entries)
        .map_err(|e| match e {
            sobria_export::ExportError::EmptyPeriod => IpcError::new(
                "empty_period",
                "aucune entrée d'audit dans la période demandée",
            ),
            other => IpcError::new("export_error", other.to_string()),
        })?;

    // 4. Écrit les artefacts sur disque.
    std::fs::create_dir_all(output_dir).map_err(AppError::from)?;
    let pdf_path = output_dir.join("report.pdf");
    let provo_path = output_dir.join("provo.jsonld");
    std::fs::write(&pdf_path, &artifacts.pdf_bytes).map_err(AppError::from)?;
    let provo_text = serde_json::to_string_pretty(&artifacts.provo_jsonld)
        .map_err(AppError::from)?;
    std::fs::write(&provo_path, provo_text).map_err(AppError::from)?;

    info!(
        pdf = %pdf_path.display(),
        provo = %provo_path.display(),
        sha256 = %artifacts.pdf_sha256,
        "rapport CSRD généré"
    );

    Ok(CsrdReportResultDto {
        pdf_path: pdf_path.display().to_string(),
        provo_path: provo_path.display().to_string(),
        pdf_sha256: artifacts.pdf_sha256,
        audit_entries_count: artifacts.audit_entries_count,
        total_requests: artifacts.summary.total_requests,
        total_co2eq_g_p50: artifacts.summary.total_co2eq_g_p50,
        total_energy_wh_p50: artifacts.summary.total_energy_wh_p50,
        total_water_l_p50: artifacts.summary.total_water_l_p50,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// dashboard + eco-budget (C19 — M15 + M25)
// ─────────────────────────────────────────────────────────────────────────────

/// Construit le résumé dashboard pour la période donnée.
pub fn get_dashboard_summary(
    period_str: &str,
    state: &AppState,
) -> IpcResult<DashboardSummaryDto> {
    let period = dashboard::DashboardPeriod::parse(period_str).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "period '{period_str}' inconnue (attendu: today | last_7_days | this_month | last_month | this_year)"
        )))
    })?;
    let entries = read_all_audit_entries(state)?;
    let summary = dashboard::aggregate(&entries, period, Utc::now());
    Ok((&summary).into())
}

/// Liste tous les objectifs personnels (M25).
pub fn list_personal_goals(state: &AppState) -> IpcResult<Vec<PersonalGoalDto>> {
    let store = state
        .goals
        .lock()
        .map_err(|e| AppError::Poisoned(format!("goals: {e}")))?;
    let goals = store.list_all().map_err(IpcError::from)?;
    Ok(goals.into_iter().map(goal_to_dto).collect())
}

/// UPSERT d'un objectif personnel.
pub fn set_personal_goal(dto: PersonalGoalDto, state: &AppState) -> IpcResult<()> {
    let goal = dto_to_goal(&dto)?;
    goal.validate().map_err(IpcError::from)?;
    let mut store = state
        .goals
        .lock()
        .map_err(|e| AppError::Poisoned(format!("goals: {e}")))?;
    store.upsert(&goal).map_err(IpcError::from)?;
    info!(
        indicator = goal.indicator.as_str(),
        period = goal.period.as_str(),
        value_max = goal.value_max,
        "goals: set"
    );
    Ok(())
}

/// Supprime un objectif (idempotent).
pub fn delete_personal_goal(
    indicator: &str,
    period: &str,
    state: &AppState,
) -> IpcResult<()> {
    let i = GoalIndicator::parse(indicator).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "indicator '{indicator}' inconnu"
        )))
    })?;
    let p = GoalPeriod::parse(period).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "period '{period}' inconnue"
        )))
    })?;
    let mut store = state
        .goals
        .lock()
        .map_err(|e| AppError::Poisoned(format!("goals: {e}")))?;
    store.delete(i, p).map_err(IpcError::from)?;
    Ok(())
}

/// Statut budget pour chaque objectif défini.
pub fn get_budget_status(state: &AppState) -> IpcResult<Vec<BudgetStatusDto>> {
    let goals = list_personal_goals(state)?;
    if goals.is_empty() {
        return Ok(vec![]);
    }
    let entries = read_all_audit_entries(state)?;
    let now = Utc::now();
    let mut out = Vec::with_capacity(goals.len());
    for g in goals {
        let (start, end) = match GoalPeriod::parse(&g.period).unwrap_or(GoalPeriod::Monthly) {
            GoalPeriod::Daily => (
                dashboard::DashboardPeriod::Today.window(now).0,
                now,
            ),
            GoalPeriod::Weekly => (now - chrono::Duration::days(7), now),
            GoalPeriod::Monthly => (
                dashboard::DashboardPeriod::ThisMonth.window(now).0,
                now,
            ),
        };
        let agg = sum_indicator_in_window(&entries, &g.indicator, start, end);
        let consumed_pct = if g.value_max > 0.0 {
            (agg / g.value_max) * 100.0
        } else {
            0.0
        };
        let status = if consumed_pct > 100.0 {
            "exceeded"
        } else if consumed_pct >= 70.0 {
            "warning"
        } else {
            "ok"
        };
        out.push(BudgetStatusDto {
            goal: g,
            current_value: agg,
            period_start: start.to_rfc3339(),
            period_end: end.to_rfc3339(),
            consumed_pct,
            status: status.into(),
            remaining: 0.0, // remplace ligne suivante
        });
        // calcule remaining APRÈS push pour éviter le borrow ; on patche
        let last = out.last_mut().unwrap();
        last.remaining = last.goal.value_max - last.current_value;
    }
    Ok(out)
}

fn goal_to_dto(g: PersonalGoal) -> PersonalGoalDto {
    PersonalGoalDto {
        indicator: g.indicator.as_str().into(),
        period: g.period.as_str().into(),
        value_max: g.value_max,
        unit: g.unit,
    }
}

fn dto_to_goal(dto: &PersonalGoalDto) -> IpcResult<PersonalGoal> {
    let indicator = GoalIndicator::parse(&dto.indicator).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "indicator '{}' inconnu",
            dto.indicator
        )))
    })?;
    let period = GoalPeriod::parse(&dto.period).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "period '{}' inconnue",
            dto.period
        )))
    })?;
    Ok(PersonalGoal {
        indicator,
        period,
        value_max: dto.value_max,
        unit: dto.unit.clone(),
    })
}

/// Somme un indicateur sur une fenêtre donnée. Indicator = "co2eq" | "energy" | "water".
fn sum_indicator_in_window(
    entries: &[sobria_audit::AuditEntry],
    indicator: &str,
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> f64 {
    let mut sum = 0.0;
    for entry in entries {
        if entry.timestamp < start || entry.timestamp >= end {
            continue;
        }
        if entry.is_purged() {
            continue;
        }
        let Ok(result) = serde_json::from_str::<sobria_core::EstimationResult>(
            &entry.estimation_result_json,
        ) else {
            continue;
        };
        let target = match indicator {
            "co2eq" => Indicator::Co2Eq,
            "energy" => Indicator::Energy,
            "water" => Indicator::Water,
            _ => continue,
        };
        sum += result
            .indicators
            .iter()
            .find(|i| i.indicator == target)
            .map_or(0.0, |i| i.interval.p50);
    }
    sum
}

// ─────────────────────────────────────────────────────────────────────────────
// batch CSV (C21 — M18)
// ─────────────────────────────────────────────────────────────────────────────

fn batch_error_to_ipc(e: batch::BatchError) -> IpcError {
    match e {
        batch::BatchError::FileNotFound(p) => {
            IpcError::new("invalid_request", format!("fichier introuvable : {}", p.display()))
        },
        batch::BatchError::Format(m) => IpcError::new("invalid_request", m),
        batch::BatchError::TooManyRows { got, max } => IpcError::new(
            "invalid_request",
            format!("trop de lignes : {got} (max {max})"),
        ),
        batch::BatchError::TooManyRejections {
            rejected,
            total,
            limit_pct,
        } => IpcError::new(
            "invalid_request",
            format!(
                "{rejected}/{total} lignes rejetées (> {limit_pct}%) — vérifiez le format"
            ),
        ),
        batch::BatchError::EmptyBatch => {
            IpcError::new("invalid_request", "le CSV ne contient aucune ligne de données")
        },
        batch::BatchError::Csv(e) => IpcError::new("invalid_request", format!("csv : {e}")),
        batch::BatchError::Io(e) => IpcError::new("io_error", e.to_string()),
    }
}

/// Lance un batch CSV : parse → estimate par ligne → agrégat + export optionnel.
#[allow(clippy::cast_precision_loss)] // batch ≤ 1000 lignes, OK pour f64
pub fn run_batch_from_csv(
    req: BatchRequestDto,
    state: &AppState,
) -> IpcResult<BatchResultDto> {
    let input_path = std::path::PathBuf::from(&req.input_csv_path);
    let rows = batch::parse_csv(&input_path).map_err(batch_error_to_ipc)?;
    let total_input = rows.len();

    let mut output_rows: Vec<batch::BatchOutputRow> = Vec::with_capacity(total_input);
    let mut rejected: u32 = 0;
    let mut first_audit_id: i64 = i64::MAX;
    let mut last_audit_id: i64 = 0;

    for (idx, row) in rows.into_iter().enumerate() {
        let est_req = EstimationRequestDto {
            model_id: row.model_id.clone(),
            tokens_in: row.tokens_in,
            tokens_out_estimated: row.tokens_out,
            datacenter_id: row.datacenter_id.clone(),
            method: None,
        };
        match estimate_prompt(est_req, state) {
            Ok(dto) => {
                let co2 = pick_indicator_dto(&dto, "co2eq");
                let energy = pick_indicator_dto(&dto, "energy");
                let water = pick_indicator_dto(&dto, "water");
                let row_idx = u32::try_from(idx + 1).unwrap_or(u32::MAX);
                first_audit_id = first_audit_id.min(dto.audit_id);
                last_audit_id = last_audit_id.max(dto.audit_id);
                output_rows.push(batch::BatchOutputRow {
                    row_index: row_idx,
                    model_id: row.model_id,
                    tokens_in: row.tokens_in,
                    tokens_out: row.tokens_out,
                    datacenter_id: row.datacenter_id,
                    co2eq_p5_g: co2.0,
                    co2eq_p50_g: co2.1,
                    co2eq_p95_g: co2.2,
                    energy_wh_p50: energy.1,
                    water_l_p50: water.1,
                    audit_id: dto.audit_id,
                });
            },
            Err(e) => {
                rejected = rejected.saturating_add(1);
                debug!(idx, error = %e.message, "batch: row rejected");
            },
        }
    }

    batch::check_rejection_ratio(rejected as usize, total_input)
        .map_err(batch_error_to_ipc)?;

    // Agrégation
    let aggregate = if output_rows.is_empty() {
        BatchAggregateDto {
            total_co2eq_g_p50: 0.0,
            total_energy_wh_p50: 0.0,
            total_water_l_p50: 0.0,
            avg_co2eq_g_p50: 0.0,
            min_co2eq_g_p50: 0.0,
            max_co2eq_g_p50: 0.0,
        }
    } else {
        let total_co2: f64 = output_rows.iter().map(|r| r.co2eq_p50_g).sum();
        let total_energy: f64 = output_rows.iter().map(|r| r.energy_wh_p50).sum();
        let total_water: f64 = output_rows.iter().map(|r| r.water_l_p50).sum();
        let n = output_rows.len() as f64;
        let min_co2 = output_rows
            .iter()
            .map(|r| r.co2eq_p50_g)
            .fold(f64::INFINITY, f64::min);
        let max_co2 = output_rows
            .iter()
            .map(|r| r.co2eq_p50_g)
            .fold(f64::NEG_INFINITY, f64::max);
        BatchAggregateDto {
            total_co2eq_g_p50: total_co2,
            total_energy_wh_p50: total_energy,
            total_water_l_p50: total_water,
            avg_co2eq_g_p50: total_co2 / n,
            min_co2eq_g_p50: min_co2,
            max_co2eq_g_p50: max_co2,
        }
    };

    let by_model = aggregate_by_model(&output_rows);

    // Export optionnel
    let output_path = if let Some(out) = req.output_csv_path.as_ref() {
        let path = std::path::PathBuf::from(out);
        batch::export_results_csv(&path, &output_rows).map_err(batch_error_to_ipc)?;
        Some(out.clone())
    } else {
        None
    };

    let rows_processed = u32::try_from(output_rows.len()).unwrap_or(u32::MAX);
    info!(
        rows_processed,
        rejected,
        total_input,
        "batch CSV terminé"
    );

    Ok(BatchResultDto {
        rows_processed,
        rows_rejected: rejected,
        aggregate,
        by_model,
        output_csv_path: output_path,
        first_audit_id: if first_audit_id == i64::MAX { 0 } else { first_audit_id },
        last_audit_id,
    })
}

/// Extrait (P5, P50, P95) d'un indicateur du DTO (par son id string).
fn pick_indicator_dto(dto: &EstimationResultDto, id: &str) -> (f64, f64, f64) {
    dto.indicators
        .iter()
        .find(|i| i.indicator == id)
        .map_or((0.0, 0.0, 0.0), |i| (i.p5, i.p50, i.p95))
}

fn aggregate_by_model(rows: &[batch::BatchOutputRow]) -> Vec<BatchModelAggregateDto> {
    use std::collections::HashMap;
    let mut grouped: HashMap<String, (u32, f64)> = HashMap::new();
    for r in rows {
        let entry = grouped.entry(r.model_id.clone()).or_insert((0, 0.0));
        entry.0 = entry.0.saturating_add(1);
        entry.1 += r.co2eq_p50_g;
    }
    let mut out: Vec<BatchModelAggregateDto> = grouped
        .into_iter()
        .map(|(model_id, (count, total))| BatchModelAggregateDto {
            model_id,
            count,
            total_co2eq_g_p50: total,
            avg_co2eq_g_p50: total / f64::from(count.max(1)),
        })
        .collect();
    out.sort_by(|a, b| {
        b.total_co2eq_g_p50
            .partial_cmp(&a.total_co2eq_g_p50)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// projects + datasheet (C20 — M17 Empreinte projet)
// ─────────────────────────────────────────────────────────────────────────────

/// Liste tous les projets enregistrés (ordre ID décroissant).
pub fn list_projects(state: &AppState) -> IpcResult<Vec<ProjectDto>> {
    let store = state
        .projects
        .lock()
        .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
    let projects = store.list_all().map_err(IpcError::from)?;
    Ok(projects.into_iter().map(project_to_dto).collect())
}

/// Récupère un projet par son ID.
pub fn get_project(id: i64, state: &AppState) -> IpcResult<ProjectDto> {
    let store = state
        .projects
        .lock()
        .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
    let p = store
        .get(id)
        .map_err(IpcError::from)?
        .ok_or_else(|| IpcError::new("not_found", format!("projet {id} inconnu")))?;
    Ok(project_to_dto(p))
}

/// Crée un nouveau projet et retourne sa version persistée.
pub fn create_project(req: CreateProjectDto, state: &AppState) -> IpcResult<ProjectDto> {
    let period_start = parse_rfc3339(&req.period_start, "period_start")?;
    let period_end = parse_rfc3339(&req.period_end, "period_end")?;
    let mut store = state
        .projects
        .lock()
        .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
    let p = store
        .create(NewProject {
            name: req.name,
            description: req.description,
            period_start,
            period_end,
            tags: req.tags,
        })
        .map_err(IpcError::from)?;
    info!(id = p.id, name = %p.name, "project: created");
    Ok(project_to_dto(p))
}

/// Met à jour un projet existant (champs optionnels, dates non modifiables).
pub fn update_project(
    id: i64,
    req: UpdateProjectDto,
    state: &AppState,
) -> IpcResult<ProjectDto> {
    let mut store = state
        .projects
        .lock()
        .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
    let p = store
        .update(
            id,
            ProjectUpdate {
                name: req.name,
                description: req.description,
                tags: req.tags,
            },
        )
        .map_err(IpcError::from)?;
    Ok(project_to_dto(p))
}

/// Supprime un projet (idempotent).
pub fn delete_project(id: i64, state: &AppState) -> IpcResult<()> {
    let mut store = state
        .projects
        .lock()
        .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
    store.delete(id).map_err(IpcError::from)?;
    Ok(())
}

/// Génère le datasheet Gebru JSON-LD pour un projet existant.
pub fn generate_project_datasheet(id: i64, state: &AppState) -> IpcResult<DatasheetDto> {
    // 1. Récupère le projet.
    let project = {
        let store = state
            .projects
            .lock()
            .map_err(|e| AppError::Poisoned(format!("projects: {e}")))?;
        store
            .get(id)
            .map_err(IpcError::from)?
            .ok_or_else(|| IpcError::new("not_found", format!("projet {id} inconnu")))?
    };

    // 2. Filtre les entrées du ledger sur la période.
    let all_entries = read_all_audit_entries(state)?;
    let in_period: Vec<sobria_audit::AuditEntry> = all_entries
        .into_iter()
        .filter(|e| e.timestamp >= project.period_start && e.timestamp < project.period_end)
        .collect();

    // 3. Construit le datasheet.
    let meta = sobria_export::ProjectMeta {
        id: project.id,
        name: project.name.clone(),
        description: project.description.clone(),
        period_start: project.period_start,
        period_end: project.period_end,
        tags: project.tags.clone(),
        created_at: project.created_at,
    };
    let opts = sobria_export::DatasheetOptions::default();
    let artifact = sobria_export::build_datasheet(&meta, &in_period, &opts);

    info!(
        project_id = project.id,
        entries = in_period.len(),
        sha = %artifact.sha256,
        "datasheet généré"
    );

    Ok(DatasheetDto {
        project: project_to_dto(project),
        jsonld: artifact.jsonld,
        composition: CompositionDto::from(&artifact.composition),
        sha256: artifact.sha256,
    })
}

fn project_to_dto(p: Project) -> ProjectDto {
    ProjectDto {
        id: p.id,
        name: p.name,
        description: p.description,
        period_start: p.period_start.to_rfc3339(),
        period_end: p.period_end.to_rfc3339(),
        tags: p.tags,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }
}

fn parse_rfc3339(s: &str, field: &str) -> IpcResult<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| {
            IpcError::from(AppError::InvalidRequest(format!(
                "{field} invalide (attendu RFC 3339) : {e}"
            )))
        })
}

/// Lit toutes les entrées du ledger en mémoire (export ndjson + parse).
/// OK au volume v1.0 (<10⁵ entrées).
fn read_all_audit_entries(state: &AppState) -> IpcResult<Vec<sobria_audit::AuditEntry>> {
    let ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let mut buf: Vec<u8> = Vec::new();
    ledger.export_ndjson(&mut buf).map_err(AppError::from)?;
    drop(ledger);
    let text = String::from_utf8(buf).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut out: Vec<sobria_audit::AuditEntry> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: sobria_audit::AuditEntry =
            serde_json::from_str(line).map_err(AppError::from)?;
        out.push(entry);
    }
    Ok(out)
}

/// Calcule un baseline (CO₂eq, énergie, eau) pour un DC donné avec un prompt
/// de référence gpt-4o-mini 100/500 tokens. Les params PUE/IF/WUE du DC
/// sont injectés comme `Distribution::Point` (déterministes).
///
/// **Détermination cross-DC** : pour que les comparaisons entre DC soient
/// numériquement comparables (même graine Monte-Carlo, mêmes tirages),
/// on force tous les paramètres "scalaires" en `Point`. Pour les DC qui
/// ne publient pas leur WUE, on injecte une valeur médiane littéraire
/// (1.5 L/kWh — Mytton 2021) afin d'éviter qu'une distribution `Uniform`
/// fasse dériver l'état du RNG entre deux DC et masque la différence
/// de mix électrique. Cette WUE par défaut ne réapparaît pas dans le DTO
/// retourné (`wue_l_per_kwh` reste `None` côté front).
fn compute_baseline_for_dc(
    dc: &DatacenterRecord,
    state: &AppState,
) -> Result<(f64, f64, f64), AppError> {
    const REF_MODEL: &str = "gpt-4o-mini";
    const REF_TOKENS_IN: u32 = 100;
    const REF_TOKENS_OUT: u32 = 500;
    /// Médian Mytton 2021 — utilisé uniquement pour les DC qui ne publient
    /// pas leur WUE. Préserve la déterminisme RNG entre DC.
    const WUE_DEFAULT_L_PER_KWH: f64 = 1.5;

    let mut params = EstimationParams::for_model(REF_MODEL)?;
    params.pue = Distribution::Point { value: dc.pue };
    params.if_electrical_g_per_kwh = Distribution::Point {
        value: dc.if_electrical_g_per_kwh,
    };
    params.wue_l_per_kwh = Distribution::Point {
        value: dc.wue_l_per_kwh.unwrap_or(WUE_DEFAULT_L_PER_KWH),
    };
    params.validate()?;

    let req = EstimationRequest {
        model_id: REF_MODEL.into(),
        tokens_in: REF_TOKENS_IN,
        tokens_out_estimated: REF_TOKENS_OUT,
        datacenter_id: Some(dc.id.clone()),
        timestamp: Utc::now(),
    };
    // Polish G — baseline DC respecte la méthodologie user
    let engine = sobria_estimator::engine_for(user_default_method(state));
    let result = engine.estimate(&req, &params)?;
    let co2 = pick_p50(&result, Indicator::Co2Eq);
    let energy = pick_p50(&result, Indicator::Energy);
    let water = pick_p50(&result, Indicator::Water);
    Ok((co2, energy, water))
}

fn pick_p50(result: &sobria_core::EstimationResult, ind: Indicator) -> f64 {
    result
        .indicators
        .iter()
        .find(|i| i.indicator == ind)
        .map_or(0.0, |i| i.interval.p50)
}

/// Borne haute du nombre de modèles autorisés dans un benchmark.
pub const MAX_BENCHMARK_MODELS: usize = 20;

/// Compare N modèles (1..=`MAX_BENCHMARK_MODELS`) sur un même prompt (M3 / C17).
///
/// Boucle simplement sur `estimate_prompt` puis calcule des classements
/// par P50 ascendant pour CO₂eq / énergie / eau. Chaque appel journalise
/// dans l'audit ledger (1 entrée par modèle).
#[allow(clippy::too_many_lines)] // flow linéaire : validations + N estimations + 3 rankings
pub fn benchmark_models(
    req: BenchmarkRequestDto,
    state: &AppState,
) -> IpcResult<BenchmarkResultDto> {
    // 1. Validations.
    if req.model_ids.is_empty() {
        return Err(IpcError::from(AppError::InvalidRequest(
            "model_ids vide".into(),
        )));
    }
    if req.model_ids.len() > MAX_BENCHMARK_MODELS {
        return Err(IpcError::from(AppError::InvalidRequest(format!(
            "trop de modèles : {} (max {MAX_BENCHMARK_MODELS})",
            req.model_ids.len()
        ))));
    }
    let mut seen = std::collections::HashSet::new();
    for m in &req.model_ids {
        if !seen.insert(m.clone()) {
            return Err(IpcError::from(AppError::InvalidRequest(format!(
                "model_id '{m}' en doublon"
            ))));
        }
    }
    let mut unknown: Vec<&str> = Vec::new();
    for m in &req.model_ids {
        if find_preset(m).is_none() {
            unknown.push(m.as_str());
        }
    }
    if !unknown.is_empty() {
        return Err(IpcError::from(AppError::UnknownModel(unknown.join(", "))));
    }

    // 2. Estimation + journalisation de chaque modèle.
    //
    // Polish D — On lit la méthodologie par défaut **une fois** au lieu
    // de laisser chaque `estimate_prompt` la relire depuis SQLite. Ça :
    // 1. évite N appels disque (1 par modèle benchmarké),
    // 2. garantit que TOUS les modèles utilisent la **même** méthodo
    //    même si l'utilisateur change ses préférences pendant le calcul
    //    (race-condition mitigation — un benchmark doit être cohérent).
    let user_method = state
        .preferences
        .lock()
        .ok()
        .and_then(|store| store.read_all().ok())
        .and_then(|p| p.default_method)
        .unwrap_or_else(sobria_core::EmpreinteMethod::default_method);

    let mut tmp: Vec<(BenchmarkOutcomeDto, f64, f64, f64)> = Vec::with_capacity(req.model_ids.len());
    for model_id in &req.model_ids {
        let est_req = EstimationRequestDto {
            model_id: model_id.clone(),
            tokens_in: req.tokens_in,
            tokens_out_estimated: req.tokens_out_estimated,
            datacenter_id: req.datacenter_id.clone(),
            // Méthodo explicite — pas de fallback IPC, on a déjà résolu.
            method: Some(user_method),
        };
        let result_dto = estimate_prompt(est_req, state)?;
        let preset = find_preset(model_id)
            .ok_or_else(|| IpcError::from(AppError::UnknownModel(model_id.clone())))?;
        let co2_p50 = pick_p50_from_dto(&result_dto, "co2eq");
        let energy_p50 = pick_p50_from_dto(&result_dto, "energy");
        let water_p50 = pick_p50_from_dto(&result_dto, "water");
        let outcome = BenchmarkOutcomeDto {
            model_id: preset.id.into(),
            display_name: preset.display_name.into(),
            provider: preset.provider.into(),
            family: preset.family.into(),
            openness: match preset.openness {
                sobria_estimator::Openness::Open => "open",
                sobria_estimator::Openness::OpenWeights => "open_weights",
                sobria_estimator::Openness::Closed => "closed",
            }
            .into(),
            calibration: match preset.calibration {
                sobria_estimator::CalibrationStatus::Validated => "validated",
                sobria_estimator::CalibrationStatus::Indicative => "indicative",
                sobria_estimator::CalibrationStatus::Extrapolated => "extrapolated",
            }
            .into(),
            result: result_dto,
            rank_co2eq: 0, // assigné ci-dessous
            rank_energy: 0,
            rank_water: 0,
        };
        tmp.push((outcome, co2_p50, energy_p50, water_p50));
    }

    // 3. Classements ascendant (rang 1 = meilleur = plus bas P50).
    let ranking_co2 = rank_ascending(&tmp, |v| v.1);
    let ranking_energy = rank_ascending(&tmp, |v| v.2);
    let ranking_water = rank_ascending(&tmp, |v| v.3);
    let rank_map_co2 = build_rank_map(&ranking_co2);
    let rank_map_energy = build_rank_map(&ranking_energy);
    let rank_map_water = build_rank_map(&ranking_water);

    let mut outcomes: Vec<BenchmarkOutcomeDto> = tmp
        .into_iter()
        .map(|(mut o, _, _, _)| {
            o.rank_co2eq = rank_map_co2.get(o.model_id.as_str()).copied().unwrap_or(0);
            o.rank_energy = rank_map_energy.get(o.model_id.as_str()).copied().unwrap_or(0);
            o.rank_water = rank_map_water.get(o.model_id.as_str()).copied().unwrap_or(0);
            o
        })
        .collect();
    // Garde l'ordre demandé par l'utilisateur dans `outcomes` (utile pour l'UI).
    let order: Vec<String> = req.model_ids.clone();
    outcomes.sort_by_key(|o| {
        order
            .iter()
            .position(|m| m == &o.model_id)
            .unwrap_or(usize::MAX)
    });

    info!(
        models = req.model_ids.len(),
        winner_co2 = %ranking_co2.first().map_or("", String::as_str),
        "benchmark_models: ok"
    );

    Ok(BenchmarkResultDto {
        outcomes,
        ranking_by_co2eq_p50: ranking_co2,
        ranking_by_energy_p50: ranking_energy,
        ranking_by_water_p50: ranking_water,
        tokens_in: req.tokens_in,
        tokens_out_estimated: req.tokens_out_estimated,
    })
}

fn pick_p50_from_dto(dto: &EstimationResultDto, indicator_id: &str) -> f64 {
    dto.indicators
        .iter()
        .find(|i| i.indicator == indicator_id)
        .map_or(f64::NAN, |i| i.p50)
}

fn rank_ascending<F>(
    items: &[(BenchmarkOutcomeDto, f64, f64, f64)],
    key: F,
) -> Vec<String>
where
    F: Fn(&(BenchmarkOutcomeDto, f64, f64, f64)) -> f64,
{
    let mut indices: Vec<usize> = (0..items.len()).collect();
    indices.sort_by(|a, b| {
        key(&items[*a])
            .partial_cmp(&key(&items[*b]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    indices
        .into_iter()
        .map(|i| items[i].0.model_id.clone())
        .collect()
}

fn build_rank_map(ranking: &[String]) -> std::collections::HashMap<String, u32> {
    ranking
        .iter()
        .enumerate()
        .map(|(i, m)| {
            // Borné par MAX_BENCHMARK_MODELS (= 20) → cast safe.
            let rank = u32::try_from(i).unwrap_or(u32::MAX).saturating_add(1);
            (m.clone(), rank)
        })
        .collect()
}

/// Lance un forecast 12 mois (M16 / C15) avec bande d'incertitude
/// P5/P50/P95 et superposition de scénarios de croissance.
///
/// Étapes :
/// 1. Validation modèle baseline.
/// 2. `sobria_estimator::forecast_yearly()` (1 Monte-Carlo + projections déterministes).
/// 3. Journalisation du baseline dans le ledger (1 entrée).
/// 4. Conversion DTO.
pub fn forecast_yearly_budget(
    req: YearlyForecastRequestDto,
    state: &AppState,
) -> IpcResult<YearlyForecastResultDto> {
    if find_preset(&req.baseline.model_id).is_none() {
        return Err(IpcError::from(AppError::UnknownModel(
            req.baseline.model_id.clone(),
        )));
    }
    let core_req = req.into_core(Utc::now());
    // Polish G — Forecaster honore la méthodologie par défaut user.
    let engine = sobria_estimator::engine_for(user_default_method(state));
    let result = sobria_estimator::forecast_yearly(engine.as_ref(), &core_req)
        .map_err(AppError::from)?;

    // Journalisation baseline : on relance l'estimateur baseline pour
    // récupérer un EstimationResult complet (avec bins) à journaliser.
    // Aurait pu être renvoyé par forecast_yearly() mais on garde l'API
    // simple ; le coût est faible (1 Monte-Carlo de plus).
    let params = sobria_estimator::EstimationParams::for_model(&core_req.baseline.model_id)
        .map_err(AppError::from)?;
    let baseline_full = engine
        .estimate(&core_req.baseline, &params)
        .map_err(AppError::from)?;
    let mut ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let entry = ledger.append(&baseline_full).map_err(AppError::from)?;
    let audit_id = entry.id;
    drop(ledger);

    info!(
        baseline_model = %core_req.baseline.model_id,
        scenarios = result.scenarios.len(),
        months = core_req.months,
        audit_id,
        "forecast_yearly_budget: ok"
    );
    Ok(YearlyForecastResultDto::from_result(&result, audit_id))
}

/// Lance une simulation « Et si...? » (M13 / C11).
///
/// Étapes :
/// 1. Validation modèle baseline.
/// 2. Conversion DTO → types Rust internes (avec timestamp baseline).
/// 3. `sobria_estimator::simulate(...)`.
/// 4. Journalisation **du seul baseline** dans le ledger (cf. brief C11 §3).
/// 5. Retourne le DTO complet (baseline avec audit_id, scénarios sans).
pub fn simulate_scenarios(
    req: SimulationRequestDto,
    state: &AppState,
) -> IpcResult<SimulationResultDto> {
    if find_preset(&req.baseline.model_id).is_none() {
        return Err(IpcError::from(AppError::UnknownModel(
            req.baseline.model_id.clone(),
        )));
    }
    let sim_core = req.into_core(Utc::now());
    // Polish G — Simulateur honore la méthodologie par défaut user.
    let engine = sobria_estimator::engine_for(user_default_method(state));
    let result =
        sobria_estimator::simulate(engine.as_ref(), &sim_core).map_err(AppError::from)?;

    // Journalise UNIQUEMENT le baseline (les scénarios sont exploratoires).
    let mut ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let entry = ledger.append(&result.baseline).map_err(AppError::from)?;
    let audit_id = entry.id;
    drop(ledger);

    info!(
        baseline_model = %result.baseline.request.model_id,
        scenarios = result.scenarios.len(),
        forecast = result.forecast.is_some(),
        audit_id,
        "simulate_scenarios: ok"
    );
    Ok(SimulationResultDto::from_result(&result, audit_id))
}

/// Exporte le ledger en NDJSON vers `path`. Retourne le nombre de lignes.
pub fn export_audit_ndjson(path: &std::path::Path, state: &AppState) -> IpcResult<usize> {
    let ledger = state
        .ledger
        .lock()
        .map_err(|e| AppError::Poisoned(format!("ledger: {e}")))?;
    let mut file = std::fs::File::create(path).map_err(AppError::from)?;
    let n = ledger.export_ndjson(&mut file).map_err(AppError::from)?;
    info!(path = %path.display(), lines = n, "audit: export NDJSON");
    Ok(n)
}

// ─────────────────────────────────────────────────────────────────────────────
// préférences utilisateur (C10 — ADR-0010)
// ─────────────────────────────────────────────────────────────────────────────

/// Récupère les préférences utilisateur. Fusionne les valeurs persistées
/// avec les défauts (`AppPreferencesDto::defaults()`).
pub fn get_app_preferences(state: &AppState) -> IpcResult<AppPreferencesDto> {
    let store = state
        .preferences
        .lock()
        .map_err(|e| AppError::Poisoned(format!("preferences: {e}")))?;
    let stored = store.read_all().map_err(IpcError::from)?;
    drop(store);
    let defaults = AppPreferencesDto::defaults();
    Ok(AppPreferencesDto {
        persona: stored.persona.or(defaults.persona),
        enabled_modules: stored.enabled_modules.unwrap_or(defaults.enabled_modules),
        onboarded: stored.onboarded.unwrap_or(defaults.onboarded),
        lang: stored.lang.unwrap_or(defaults.lang),
        default_method: stored.default_method.unwrap_or(defaults.default_method),
        also_show_methods: stored.also_show_methods.unwrap_or(defaults.also_show_methods),
    })
}

/// Persiste les préférences utilisateur après validation stricte.
///
/// Erreurs retournées (toutes en `invalid_request`) :
/// - `lang` n'est pas dans `{"fr", "en"}`,
/// - `enabled_modules` contient des doublons,
/// - `enabled_modules` est vide alors que `onboarded == true` (un utilisateur
///   onboardé doit avoir au moins un module — M1 au strict minimum).
///
/// Note : les variantes invalides de `Persona` / `ModuleId` sont déjà
/// rejetées à la désérialisation par serde — pas besoin de re-valider ici.
pub fn set_app_preferences(prefs: AppPreferencesDto, state: &AppState) -> IpcResult<()> {
    validate_lang(&prefs.lang)?;
    validate_modules(&prefs.enabled_modules, prefs.onboarded)?;

    let stored = StoredPreferences {
        persona: prefs.persona,
        enabled_modules: Some(prefs.enabled_modules),
        onboarded: Some(prefs.onboarded),
        lang: Some(prefs.lang),
        default_method: Some(prefs.default_method),
        also_show_methods: Some(prefs.also_show_methods),
    };

    let mut store = state
        .preferences
        .lock()
        .map_err(|e| AppError::Poisoned(format!("preferences: {e}")))?;
    store.write_all(&stored).map_err(IpcError::from)?;
    info!(
        persona = ?stored.persona,
        onboarded = ?stored.onboarded,
        "préférences : mise à jour"
    );
    Ok(())
}

fn validate_lang(lang: &str) -> IpcResult<()> {
    if lang == "fr" || lang == "en" {
        Ok(())
    } else {
        Err(IpcError::from(AppError::InvalidRequest(format!(
            "lang '{lang}' inconnue, attendu 'fr' ou 'en'"
        ))))
    }
}

fn validate_modules(modules: &[ModuleId], onboarded: bool) -> IpcResult<()> {
    if onboarded && modules.is_empty() {
        return Err(IpcError::from(AppError::InvalidRequest(
            "un utilisateur onboardé doit conserver au moins un module".into(),
        )));
    }
    // Doublons : un module ne peut apparaître qu'une fois.
    let mut seen = std::collections::HashSet::new();
    for m in modules {
        if !seen.insert(*m) {
            return Err(IpcError::from(AppError::InvalidRequest(format!(
                "module {m:?} en doublon dans enabled_modules"
            ))));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> (tempfile::TempDir, AppState) {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::init_in(tmp.path()).unwrap();
        (tmp, state)
    }

    #[test]
    fn meta_info_returns_version() {
        let (_tmp, state) = fresh_state();
        let m = meta_info(&state).unwrap();
        assert!(!m.app_version.is_empty());
        assert_eq!(m.estimator_n, sobria_estimator::DEFAULT_N);
        assert!(m.audit_path.ends_with("audit.sqlite"));
    }

    #[test]
    fn list_models_returns_at_least_8() {
        let models = list_models().unwrap();
        assert!(models.len() >= 8, "got {} models", models.len());
        // Présence de quelques modèles clés du registry C06.
        let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"gpt-4o-mini"));
        assert!(ids.contains(&"claude-3-5-sonnet"));
    }

    #[test]
    fn estimate_unknown_model_returns_unknown_model_code() {
        let (_tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "n-existe-pas".into(),
            tokens_in: 10,
            tokens_out_estimated: 50,
            datacenter_id: None,
            method: None,
        };
        let err = estimate_prompt(req, &state).unwrap_err();
        assert_eq!(err.code, "unknown_model");
    }

    #[test]
    fn estimate_zero_tokens_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 0,
            tokens_out_estimated: 0,
            datacenter_id: None,
            method: None,
        };
        let err = estimate_prompt(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn estimate_happy_path_journalises() {
        let (_tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: None,
        };
        let result = estimate_prompt(req, &state).unwrap();
        assert!(result.audit_id >= 1);
        assert_eq!(result.indicators.len(), 3);
        // CO₂eq attendu strictement positif et fini.
        let co2 = result
            .indicators
            .iter()
            .find(|i| i.indicator == "co2eq")
            .unwrap();
        assert!(co2.p50 > 0.0 && co2.p50.is_finite());
        assert!(co2.p5 <= co2.p50 && co2.p50 <= co2.p95);
    }

    #[test]
    fn verify_audit_is_valid_after_appends() {
        let (_tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 50,
            tokens_out_estimated: 100,
            datacenter_id: None,
            method: None,
        };
        for _ in 0..5 {
            estimate_prompt(req.clone(), &state).unwrap();
        }
        let report = verify_audit(&state).unwrap();
        assert!(report.valid, "{}", report.message);
        assert_eq!(report.total_entries, 5);
    }

    #[test]
    fn list_audit_entries_pagination() {
        let (_tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "claude-3-5-sonnet".into(),
            tokens_in: 10,
            tokens_out_estimated: 30,
            datacenter_id: None,
            method: None,
        };
        for _ in 0..7 {
            estimate_prompt(req.clone(), &state).unwrap();
        }
        let page1 = list_audit_entries(3, 0, &state).unwrap();
        assert_eq!(page1.len(), 3);
        let page2 = list_audit_entries(3, 3, &state).unwrap();
        assert_eq!(page2.len(), 3);
        let page3 = list_audit_entries(3, 6, &state).unwrap();
        assert_eq!(page3.len(), 1);
        assert_eq!(page1[0].model_id, "claude-3-5-sonnet");
        assert!(page1[0].sig_short.len() == 16);
    }

    #[test]
    fn export_audit_ndjson_writes_file() {
        let (tmp, state) = fresh_state();
        let req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 10,
            tokens_out_estimated: 20,
            datacenter_id: None,
            method: None,
        };
        for _ in 0..3 {
            estimate_prompt(req.clone(), &state).unwrap();
        }
        let out = tmp.path().join("export.ndjson");
        let n = export_audit_ndjson(&out, &state).unwrap();
        assert_eq!(n, 3);
        let content = std::fs::read_to_string(&out).unwrap();
        assert_eq!(content.lines().count(), 3);
    }

    // ─────────────────────────────────────────────────────────────────────
    // préférences utilisateur — C10 / ADR-0010
    // ─────────────────────────────────────────────────────────────────────

    use sobria_core::{ModuleId, Persona};

    #[test]
    fn get_preferences_returns_defaults_on_empty_db() {
        let (_tmp, state) = fresh_state();
        let prefs = get_app_preferences(&state).unwrap();
        assert!(prefs.persona.is_none(), "persona doit être None par défaut");
        assert!(!prefs.onboarded, "onboarded false par défaut");
        assert_eq!(prefs.lang, "fr");
        // Le bundle par défaut est celui de pro_tech (cf ADR-0010).
        assert_eq!(
            prefs.enabled_modules,
            Persona::ProTech.default_modules(),
            "bundle par défaut = pro_tech"
        );
    }

    #[test]
    fn set_then_get_preferences_round_trips() {
        let (_tmp, state) = fresh_state();
        let written = AppPreferencesDto {
            persona: Some(Persona::Enterprise),
            enabled_modules: vec![ModuleId::M1, ModuleId::M7, ModuleId::M22],
            onboarded: true,
            lang: "fr".into(),
            ..AppPreferencesDto::defaults()
        };
        set_app_preferences(written.clone(), &state).unwrap();
        let read = get_app_preferences(&state).unwrap();
        assert_eq!(read, written);
    }

    #[test]
    fn set_preferences_rejects_unknown_lang() {
        let (_tmp, state) = fresh_state();
        let prefs = AppPreferencesDto {
            persona: Some(Persona::Student),
            enabled_modules: vec![ModuleId::M1],
            onboarded: true,
            lang: "es".into(), // espagnol pas supporté en v1.0
            ..AppPreferencesDto::defaults()
        };
        let err = set_app_preferences(prefs, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
        assert!(err.message.contains("lang"));
    }

    #[test]
    fn set_preferences_rejects_empty_modules_when_onboarded() {
        let (_tmp, state) = fresh_state();
        let prefs = AppPreferencesDto {
            persona: Some(Persona::Student),
            enabled_modules: vec![],
            onboarded: true,
            lang: "fr".into(),
            ..AppPreferencesDto::defaults()
        };
        let err = set_app_preferences(prefs, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn set_preferences_allows_empty_modules_when_not_onboarded() {
        // Au tout début de l'onboarding, l'utilisateur n'a encore rien coché.
        let (_tmp, state) = fresh_state();
        let prefs = AppPreferencesDto {
            persona: None,
            enabled_modules: vec![],
            onboarded: false,
            lang: "fr".into(),
            ..AppPreferencesDto::defaults()
        };
        assert!(set_app_preferences(prefs, &state).is_ok());
    }

    #[test]
    fn set_preferences_rejects_duplicate_modules() {
        let (_tmp, state) = fresh_state();
        let prefs = AppPreferencesDto {
            persona: Some(Persona::Student),
            enabled_modules: vec![ModuleId::M1, ModuleId::M13, ModuleId::M1],
            onboarded: true,
            lang: "fr".into(),
            ..AppPreferencesDto::defaults()
        };
        let err = set_app_preferences(prefs, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
        assert!(err.message.contains("doublon"));
    }

    #[test]
    fn set_preferences_overwrites_previous() {
        let (_tmp, state) = fresh_state();
        set_app_preferences(
            AppPreferencesDto {
                persona: Some(Persona::Student),
                enabled_modules: vec![ModuleId::M1],
                onboarded: true,
                lang: "fr".into(),
                ..AppPreferencesDto::defaults()
            },
            &state,
        )
        .unwrap();
        set_app_preferences(
            AppPreferencesDto {
                persona: Some(Persona::Researcher),
                enabled_modules: vec![ModuleId::M1, ModuleId::M17, ModuleId::M18],
                onboarded: true,
                lang: "en".into(),
                ..AppPreferencesDto::defaults()
            },
            &state,
        )
        .unwrap();
        let read = get_app_preferences(&state).unwrap();
        assert_eq!(read.persona, Some(Persona::Researcher));
        assert_eq!(read.enabled_modules.len(), 3);
        assert_eq!(read.lang, "en");
    }

    #[test]
    fn default_bundle_for_each_persona_includes_m1() {
        for p in Persona::all() {
            assert!(
                p.default_modules().contains(&ModuleId::M1),
                "M1 manquant dans le bundle {p:?}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // simulation — C11 / M13
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::{
        ForecastConfigDto, ParamOverridesDto, ScenarioDto, SimulationRequestDto,
    };

    fn baseline_dto() -> EstimationRequestDto {
        EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: None,
        }
    }

    #[test]
    fn simulate_baseline_only_journalises_baseline() {
        let (_tmp, state) = fresh_state();
        let req = SimulationRequestDto {
            baseline: baseline_dto(),
            scenarios: vec![],
            forecast: None,
        };
        let res = simulate_scenarios(req, &state).unwrap();
        assert!(res.baseline.audit_id >= 1, "baseline doit être journalisé");
        assert_eq!(res.scenarios.len(), 0);
        assert!(res.forecast.is_none());
    }

    #[test]
    fn simulate_with_scenarios_returns_outcomes_with_deltas() {
        let (_tmp, state) = fresh_state();
        let req = SimulationRequestDto {
            baseline: baseline_dto(),
            scenarios: vec![
                ScenarioDto {
                    label: "PUE bas".into(),
                    overrides: ParamOverridesDto {
                        pue: Some(1.05),
                        ..Default::default()
                    },
                },
                ScenarioDto {
                    label: "PUE haut".into(),
                    overrides: ParamOverridesDto {
                        pue: Some(1.6),
                        ..Default::default()
                    },
                },
            ],
            forecast: None,
        };
        let res = simulate_scenarios(req, &state).unwrap();
        assert_eq!(res.scenarios.len(), 2);
        assert_eq!(res.scenarios[0].label, "PUE bas");
        assert_eq!(res.scenarios[0].result.audit_id, 0, "scenarios non journalisés");
        assert!(res.scenarios[0].delta_co2eq_g.is_finite());
        // PUE 1.05 doit donner un delta négatif (moins que baseline avec PUE
        // uniforme [1.1, 1.4]).
        assert!(
            res.scenarios[0].delta_co2eq_g < 0.0,
            "PUE 1.05 doit baisser CO2eq, got {}",
            res.scenarios[0].delta_co2eq_g
        );
    }

    #[test]
    fn simulate_unknown_baseline_model_returns_unknown_model() {
        let (_tmp, state) = fresh_state();
        let req = SimulationRequestDto {
            baseline: EstimationRequestDto {
                model_id: "ce-modele-existe-pas".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                method: None,
            },
            scenarios: vec![],
            forecast: None,
        };
        let err = simulate_scenarios(req, &state).unwrap_err();
        assert_eq!(err.code, "unknown_model");
    }

    #[test]
    fn simulate_with_forecast_returns_monthly_series() {
        let (_tmp, state) = fresh_state();
        let req = SimulationRequestDto {
            baseline: baseline_dto(),
            scenarios: vec![],
            forecast: Some(ForecastConfigDto {
                months: 12,
                monthly_growth_pct: 5.0,
                base_volume_per_day: 100.0,
            }),
        };
        let res = simulate_scenarios(req, &state).unwrap();
        let f = res.forecast.unwrap();
        assert_eq!(f.months, 12);
        assert_eq!(f.baseline_monthly_co2eq_g.len(), 12);
        assert!(f.baseline_annual_co2eq_g > 0.0);
        // Série géométrique croissante
        for i in 1..f.baseline_monthly_co2eq_g.len() {
            assert!(
                f.baseline_monthly_co2eq_g[i] > f.baseline_monthly_co2eq_g[i - 1],
                "série doit croître à 5%/mois"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // datacenters — C12 / M12
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn list_datacenters_returns_28() {
        let dcs = list_datacenters().unwrap();
        assert_eq!(dcs.len(), 28);
        // Présence de quelques DC clés.
        let ids: Vec<&str> = dcs.iter().map(|d| d.id.as_str()).collect();
        assert!(ids.contains(&"ovh-rbx-roubaix"));
        assert!(ids.contains(&"aws-eu-central-1-frankfurt"));
        assert!(ids.contains(&"gcp-europe-north1-hamina"));
    }

    #[test]
    fn list_datacenters_each_has_coords_and_operator() {
        for dc in list_datacenters().unwrap() {
            assert!(!dc.operator.is_empty(), "{} sans opérateur", dc.id);
            assert!(!dc.country_iso.is_empty(), "{} sans country_iso", dc.id);
            assert!(dc.pue >= 1.0, "{} pue {} trop bas", dc.id, dc.pue);
        }
    }

    #[test]
    fn get_datacenter_detail_known_id_returns_baseline() {
        let (_tmp, state) = fresh_state();
        let detail = get_datacenter_detail("ovh-gra-gravelines", &state).unwrap();
        assert_eq!(detail.id, "ovh-gra-gravelines");
        assert_eq!(detail.country_iso, "FR");
        assert_eq!(detail.hourly_profile_24h.len(), 24);
        // baseline cohérent : > 0 et fini.
        assert!(detail.baseline_co2eq_p50_g.is_finite());
        assert!(detail.baseline_co2eq_p50_g > 0.0);
        assert!(detail.baseline_energy_wh_p50.is_finite());
        assert!(detail.baseline_water_l_p50.is_finite());
        assert!(!detail.sources.is_empty());
    }

    #[test]
    fn get_datacenter_detail_unknown_id_returns_not_found() {
        let (_tmp, state) = fresh_state();
        let err = get_datacenter_detail("does-not-exist", &state).unwrap_err();
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn datacenter_baselines_differ_by_country_mix() {
        // Mix FR (56 g/kWh) doit donner un CO2eq inférieur à mix DE (386 g/kWh)
        // sur le même prompt de référence.
        let (_tmp, state) = fresh_state();
        let fr = get_datacenter_detail("ovh-gra-gravelines", &state).unwrap();
        let de = get_datacenter_detail("aws-eu-central-1-frankfurt", &state).unwrap();
        // L'embodied étant constant entre DC, la différence vient du mix élec.
        // En théorie DE > FR mais l'écart peut être faible si l'embodied
        // domine. On vérifie juste que DE > FR (au moins de quelques %).
        assert!(
            de.baseline_co2eq_p50_g >= fr.baseline_co2eq_p50_g,
            "DE ({}) doit être >= FR ({})",
            de.baseline_co2eq_p50_g,
            fr.baseline_co2eq_p50_g
        );
    }

    #[test]
    fn aggregate_datacenters_returns_13_countries_sorted() {
        let agg = aggregate_datacenters_by_country().unwrap();
        assert_eq!(agg.len(), 13);
        // Trié alphabétique
        for w in agg.windows(2) {
            assert!(w[0].country_iso < w[1].country_iso);
        }
        let fr = agg.iter().find(|c| c.country_iso == "FR").unwrap();
        assert_eq!(fr.datacenter_count, 4);
        assert!((fr.if_electrical_g_per_kwh - 56.0).abs() < 1e-9);
    }

    #[test]
    fn simulate_duplicate_scenario_labels_returns_error() {
        let (_tmp, state) = fresh_state();
        let req = SimulationRequestDto {
            baseline: baseline_dto(),
            scenarios: vec![
                ScenarioDto {
                    label: "même nom".into(),
                    overrides: ParamOverridesDto::default(),
                },
                ScenarioDto {
                    label: "même nom".into(),
                    overrides: ParamOverridesDto::default(),
                },
            ],
            forecast: None,
        };
        let err = simulate_scenarios(req, &state).unwrap_err();
        // Erreur propagée depuis estimator → mappée en estimator_error.
        assert_eq!(err.code, "estimator_error");
    }

    // ─────────────────────────────────────────────────────────────────────
    // territoire_fr — C13 / M20
    // ─────────────────────────────────────────────────────────────────────

    /// Fixture JSON ODRÉ minimale écrite dans `data_root` pour tester le loader.
    fn write_territoire_fixture(state: &AppState) {
        let path = state.data_root.join("territoire_fr.json");
        let json = r#"{
            "_meta": {
                "version": "1.0.0",
                "fetched_at": "2026-05-13T12:00:00+00:00",
                "source_url": "https://odre.opendatasoft.com/api/explore/v2.1/catalog/datasets/consommation-annuelle-par-iris/records",
                "source_sha256": "deadbeef00000000000000000000000000000000000000000000000000000000",
                "license": "Etalab 2.0",
                "notes": ["fixture test"]
            },
            "regions": [
                {"region_iso": "FR-HDF", "name": "Hauts-de-France", "insee_code": "32",
                 "centroid_lat": 50.0, "centroid_lon": 2.7, "nuclear_share_pct": 78.4}
            ],
            "industrial_sites": [
                {"code_iris": "591830001", "commune": "Dunkerque", "commune_code": "59183",
                 "department_code": "59", "department_label": "Nord",
                 "region_insee_code": "32", "region_iso": "FR-HDF",
                 "lat": 51.04, "lon": 2.38,
                 "consumption_mwh_elec": 800000.0, "consumption_mwh_gas_grtgaz": 0.0,
                 "consumption_mwh_gas_terega": 0.0, "consumption_total_mwh": 800000.0,
                 "pdl_count_elec": 12, "pdl_count_gas": 0, "pdl_total": 12, "year": 2022}
            ]
        }"#;
        std::fs::write(&path, json).unwrap();
    }

    fn write_rte_mix_fixture(state: &AppState) {
        let path = state.data_root.join("rte_mix_fr.json");
        let json = r#"{
            "_meta": {
                "version": "1.0.0",
                "fetched_at": "2026-05-13T12:00:00+00:00",
                "source_url": "https://odre.opendatasoft.com/.../eco2mix-national-cons-def",
                "source_sha256": "feedface00000000000000000000000000000000000000000000000000000000",
                "license": "Etalab 2.0",
                "notes": ["fixture test"]
            },
            "mix": {
                "nuclear_twh": 320.0, "hydro_twh": 50.0, "wind_twh": 45.0,
                "solar_twh": 20.0, "gas_twh": 30.0, "coal_twh": 1.0,
                "oil_twh": 1.0, "bioenergies_twh": 5.0, "pumped_twh": 3.0,
                "exchange_net_twh": 50.0, "total_production_twh": 475.0,
                "records_processed": 35040, "year": 2023
            }
        }"#;
        std::fs::write(&path, json).unwrap();
    }

    #[test]
    fn list_industrial_sites_fr_without_data_returns_data_not_ingested() {
        let (_tmp, state) = fresh_state();
        let err = list_industrial_sites_fr(50, 0, &state).unwrap_err();
        assert_eq!(err.code, "data_not_ingested");
        assert!(err.message.contains("sobria-ingest"));
    }

    #[test]
    fn list_industrial_sites_fr_with_fixture_returns_summary() {
        let (_tmp, state) = fresh_state();
        write_territoire_fixture(&state);
        let sites = list_industrial_sites_fr(50, 0, &state).unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].commune, "Dunkerque");
        assert_eq!(sites[0].region_iso, "FR-HDF");
        assert!((sites[0].consumption_total_mwh - 800000.0).abs() < 1e-9);
    }

    #[test]
    fn get_industrial_site_fr_unknown_iris_returns_not_found() {
        let (_tmp, state) = fresh_state();
        write_territoire_fixture(&state);
        let err = get_industrial_site_fr("000000000", &state).unwrap_err();
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn aggregate_industrial_sites_by_region_groups_correctly() {
        let (_tmp, state) = fresh_state();
        write_territoire_fixture(&state);
        let agg = aggregate_industrial_sites_by_region(&state).unwrap();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].region_iso, "FR-HDF");
        assert_eq!(agg[0].site_count, 1);
        assert!((agg[0].nuclear_share_pct - 78.4).abs() < 1e-9);
    }

    #[test]
    fn sankey_fr_data_without_data_returns_data_not_ingested() {
        let (_tmp, state) = fresh_state();
        let err = sankey_fr_data(&state).unwrap_err();
        assert_eq!(err.code, "data_not_ingested");
        assert!(err.message.contains("rte-mix"));
    }

    #[test]
    fn sankey_fr_data_with_fixture_returns_conserved_flows() {
        let (_tmp, state) = fresh_state();
        write_rte_mix_fixture(&state);
        let sankey = sankey_fr_data(&state).unwrap();
        // Σ liens == total production (pas d'import dans la fixture)
        let sum_links: f64 = sankey.links.iter().map(|l| l.value_twh).sum();
        assert!(
            (sum_links - sankey.total_production_twh).abs() < 1e-6,
            "conservation violée"
        );
        assert_eq!(sankey.year, 2023);
        assert!(!sankey.source_url.is_empty());
        assert!(!sankey.source_sha256.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────
    // batch CSV — C21 / M18
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::BatchRequestDto;
    use std::io::Write as _;

    fn write_csv_file(dir: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
        let path = dir.path().join("input.csv");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn batch_file_not_found_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let err = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: "/nonexistent.csv".into(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn batch_happy_path_journalises_n_entries() {
        let (tmp, state) = fresh_state();
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
claude-3-5-sonnet,200,1000,
mistral-medium-3,80,300,
";
        let path = write_csv_file(&tmp, csv);
        let res = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap();
        assert_eq!(res.rows_processed, 3);
        assert_eq!(res.rows_rejected, 0);
        // Ledger doit contenir 3 entrées
        let ledger_len = {
            let l = state.ledger.lock().unwrap();
            l.len().unwrap()
        };
        assert_eq!(ledger_len, 3);
        // Agrégat cohérent
        assert!(res.aggregate.total_co2eq_g_p50 > 0.0);
        assert!(res.aggregate.avg_co2eq_g_p50 > 0.0);
        assert!(res.aggregate.min_co2eq_g_p50 <= res.aggregate.max_co2eq_g_p50);
    }

    #[test]
    fn batch_invalid_model_rejected_but_others_processed() {
        let (tmp, state) = fresh_state();
        // 1 ligne valide + 1 ligne avec modèle inconnu = 50% rejected → OK
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
modele-bidon,50,200,
";
        let path = write_csv_file(&tmp, csv);
        let res = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap();
        assert_eq!(res.rows_processed, 1);
        assert_eq!(res.rows_rejected, 1);
    }

    #[test]
    fn batch_too_many_rejections_returns_invalid_request() {
        let (tmp, state) = fresh_state();
        // 1 valide + 3 invalides = 75% rejected → erreur
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
bidon-1,50,200,
bidon-2,50,200,
bidon-3,50,200,
";
        let path = write_csv_file(&tmp, csv);
        let err = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap_err();
        assert_eq!(err.code, "invalid_request");
        assert!(err.message.contains("rejet"));
    }

    #[test]
    fn batch_groupby_model_returns_sorted_aggregates() {
        let (tmp, state) = fresh_state();
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
claude-3-5-sonnet,200,1000,
gpt-4o-mini,150,600,
";
        let path = write_csv_file(&tmp, csv);
        let res = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap();
        // 2 modèles distincts dans by_model
        assert_eq!(res.by_model.len(), 2);
        // Tri par total_co2eq_p50 desc — le plus gros premier
        for w in res.by_model.windows(2) {
            assert!(w[0].total_co2eq_g_p50 >= w[1].total_co2eq_g_p50);
        }
        // gpt-4o-mini a 2 entrées
        let mini = res
            .by_model
            .iter()
            .find(|m| m.model_id == "gpt-4o-mini")
            .unwrap();
        assert_eq!(mini.count, 2);
    }

    #[test]
    fn batch_with_output_csv_writes_file() {
        let (tmp, state) = fresh_state();
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
";
        let path = write_csv_file(&tmp, csv);
        let out_path = tmp.path().join("results.csv");
        let res = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: Some(out_path.display().to_string()),
            },
            &state,
        )
        .unwrap();
        assert!(out_path.exists());
        let content = std::fs::read_to_string(&out_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 row
        assert!(lines[0].starts_with("row_index"));
        assert!(lines[1].starts_with("1,"));
        assert!(res.output_csv_path.is_some());
    }

    #[test]
    fn batch_audit_ids_consistent_with_journalisation() {
        let (tmp, state) = fresh_state();
        let csv = "\
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
gpt-4o-mini,150,600,
gpt-4o-mini,200,700,
";
        let path = write_csv_file(&tmp, csv);
        let res = run_batch_from_csv(
            BatchRequestDto {
                input_csv_path: path.display().to_string(),
                output_csv_path: None,
            },
            &state,
        )
        .unwrap();
        // first_audit_id < last_audit_id, et écart = N-1
        assert!(res.first_audit_id >= 1);
        assert!(res.last_audit_id >= res.first_audit_id);
        assert_eq!(
            res.last_audit_id - res.first_audit_id,
            2,
            "3 entries → audit_id consécutifs"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // projets + datasheet — C20 / M17
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::{CreateProjectDto, UpdateProjectDto};

    fn create_project_req(name: &str) -> CreateProjectDto {
        CreateProjectDto {
            name: name.into(),
            description: "Test".into(),
            // Période ouverte autour de maintenant
            period_start: (Utc::now() - chrono::Duration::days(30)).to_rfc3339(),
            period_end: (Utc::now() + chrono::Duration::days(1)).to_rfc3339(),
            tags: vec!["test".into(), "etude".into()],
        }
    }

    #[test]
    fn list_projects_empty_by_default() {
        let (_tmp, state) = fresh_state();
        assert!(list_projects(&state).unwrap().is_empty());
    }

    #[test]
    fn create_project_returns_dto_with_id() {
        let (_tmp, state) = fresh_state();
        let p = create_project(create_project_req("Étude A"), &state).unwrap();
        assert!(p.id >= 1);
        assert_eq!(p.name, "Étude A");
        assert_eq!(p.tags.len(), 2);
    }

    #[test]
    fn create_project_rejects_invalid_dates() {
        let (_tmp, state) = fresh_state();
        let mut req = create_project_req("X");
        req.period_start = "not-a-date".into();
        let err = create_project(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn create_project_rejects_empty_name() {
        let (_tmp, state) = fresh_state();
        let mut req = create_project_req("X");
        req.name = "  ".into();
        let err = create_project(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn create_project_rejects_bad_tag_chars() {
        let (_tmp, state) = fresh_state();
        let mut req = create_project_req("X");
        req.tags = vec!["UPPERCASE".into()];
        let err = create_project(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn get_project_unknown_returns_not_found() {
        let (_tmp, state) = fresh_state();
        let err = get_project(999, &state).unwrap_err();
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn update_project_partial_fields() {
        let (_tmp, state) = fresh_state();
        let p = create_project(create_project_req("Original"), &state).unwrap();
        let updated = update_project(
            p.id,
            UpdateProjectDto {
                name: Some("Renommé".into()),
                ..Default::default()
            },
            &state,
        )
        .unwrap();
        assert_eq!(updated.name, "Renommé");
        assert_eq!(updated.description, p.description);
    }

    #[test]
    fn update_project_no_fields_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let p = create_project(create_project_req("X"), &state).unwrap();
        let err = update_project(p.id, UpdateProjectDto::default(), &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn delete_project_idempotent() {
        let (_tmp, state) = fresh_state();
        // Delete sur ID inexistant ne plante pas
        delete_project(999, &state).unwrap();
        let p = create_project(create_project_req("X"), &state).unwrap();
        delete_project(p.id, &state).unwrap();
        assert!(get_project(p.id, &state).is_err());
    }

    #[test]
    fn generate_datasheet_unknown_project_returns_not_found() {
        let (_tmp, state) = fresh_state();
        let err = generate_project_datasheet(999, &state).unwrap_err();
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn generate_datasheet_happy_path_has_seven_gebru_sections() {
        let (_tmp, state) = fresh_state();
        // 1. Crée des estimations qui tomberont dans la période du projet.
        for _ in 0..3 {
            estimate_prompt(
                EstimationRequestDto {
                    model_id: "gpt-4o-mini".into(),
                    tokens_in: 100,
                    tokens_out_estimated: 500,
                    datacenter_id: None,
                    method: None,
                },
                &state,
            )
            .unwrap();
        }
        // 2. Crée le projet.
        let p = create_project(create_project_req("Étude test"), &state).unwrap();
        // 3. Génère le datasheet.
        let ds = generate_project_datasheet(p.id, &state).unwrap();
        // 4. Vérifie les 7 sections Gebru
        let datasheet_node = &ds.jsonld["@graph"][1];
        for key in [
            "sobria:motivation",
            "sobria:composition",
            "sobria:collectionProcess",
            "sobria:preprocessing",
            "sobria:uses",
            "sobria:distribution",
            "sobria:maintenance",
        ] {
            assert!(
                !datasheet_node[key].is_null(),
                "section Gebru manquante : {key}"
            );
        }
        // Composition reflète les 3 estimations
        assert_eq!(ds.composition.total_requests, 3);
        assert!(ds.composition.total_co2eq_g_p50 > 0.0);
        assert_eq!(ds.sha256.len(), 64);
    }

    #[test]
    fn generate_datasheet_empty_period_composition_zero() {
        let (_tmp, state) = fresh_state();
        // Projet sur une période passée vide (pas d'estimations).
        let req = CreateProjectDto {
            name: "Vide".into(),
            description: "".into(),
            period_start: "2020-01-01T00:00:00Z".into(),
            period_end: "2020-12-31T23:59:59Z".into(),
            tags: vec![],
        };
        let p = create_project(req, &state).unwrap();
        let ds = generate_project_datasheet(p.id, &state).unwrap();
        // Composition agrégée vide mais datasheet quand même produit.
        assert_eq!(ds.composition.total_requests, 0);
        assert!(ds.composition.unique_models.is_empty());
    }

    #[test]
    fn datasheet_context_has_4_vocabularies() {
        let (_tmp, state) = fresh_state();
        let p = create_project(create_project_req("X"), &state).unwrap();
        let ds = generate_project_datasheet(p.id, &state).unwrap();
        let ctx = &ds.jsonld["@context"];
        assert!(ctx["schema"].is_string());
        assert!(ctx["prov"].is_string());
        assert!(ctx["dcat"].is_string());
        assert!(ctx["sobria"].is_string());
    }

    // ─────────────────────────────────────────────────────────────────────
    // dashboard + eco-budget — C19 / M15 + M25
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::PersonalGoalDto;

    #[test]
    fn dashboard_unknown_period_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let err = get_dashboard_summary("yesterday", &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn dashboard_empty_ledger_returns_zero_totals() {
        let (_tmp, state) = fresh_state();
        let s = get_dashboard_summary("today", &state).unwrap();
        assert_eq!(s.total_requests, 0);
        assert!(s.top_models.is_empty());
        assert!(s.vs_previous.is_none());
    }

    #[test]
    fn dashboard_after_estimations_aggregates() {
        let (_tmp, state) = fresh_state();
        let est_req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: None,
        };
        for _ in 0..3 {
            estimate_prompt(est_req.clone(), &state).unwrap();
        }
        let s = get_dashboard_summary("today", &state).unwrap();
        assert_eq!(s.total_requests, 3);
        assert_eq!(s.top_models.len(), 1);
        assert_eq!(s.top_models[0].model_id, "gpt-4o-mini");
    }

    #[test]
    fn list_personal_goals_empty_by_default() {
        let (_tmp, state) = fresh_state();
        let goals = list_personal_goals(&state).unwrap();
        assert!(goals.is_empty());
    }

    #[test]
    fn set_then_list_personal_goal() {
        let (_tmp, state) = fresh_state();
        let dto = PersonalGoalDto {
            indicator: "co2eq".into(),
            period: "monthly".into(),
            value_max: 500.0,
            unit: "gCO2eq".into(),
        };
        set_personal_goal(dto, &state).unwrap();
        let goals = list_personal_goals(&state).unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].indicator, "co2eq");
        assert_eq!(goals[0].period, "monthly");
    }

    #[test]
    fn set_personal_goal_rejects_unknown_indicator() {
        let (_tmp, state) = fresh_state();
        let dto = PersonalGoalDto {
            indicator: "bogus".into(),
            period: "monthly".into(),
            value_max: 100.0,
            unit: "gCO2eq".into(),
        };
        let err = set_personal_goal(dto, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn set_personal_goal_rejects_negative_value() {
        let (_tmp, state) = fresh_state();
        let dto = PersonalGoalDto {
            indicator: "energy".into(),
            period: "daily".into(),
            value_max: -10.0,
            unit: "Wh".into(),
        };
        let err = set_personal_goal(dto, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn set_personal_goal_rejects_mismatched_unit() {
        let (_tmp, state) = fresh_state();
        let dto = PersonalGoalDto {
            indicator: "water".into(),
            period: "weekly".into(),
            value_max: 10.0,
            unit: "Wh".into(), // attendu: L
        };
        let err = set_personal_goal(dto, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn delete_personal_goal_is_idempotent() {
        let (_tmp, state) = fresh_state();
        // Suppression d'un goal absent ne plante pas.
        delete_personal_goal("co2eq", "daily", &state).unwrap();
        assert!(list_personal_goals(&state).unwrap().is_empty());
    }

    #[test]
    fn budget_status_no_goals_returns_empty() {
        let (_tmp, state) = fresh_state();
        let st = get_budget_status(&state).unwrap();
        assert!(st.is_empty());
    }

    #[test]
    fn budget_status_with_goal_and_no_usage_is_ok() {
        let (_tmp, state) = fresh_state();
        set_personal_goal(
            PersonalGoalDto {
                indicator: "co2eq".into(),
                period: "monthly".into(),
                value_max: 1000.0,
                unit: "gCO2eq".into(),
            },
            &state,
        )
        .unwrap();
        let st = get_budget_status(&state).unwrap();
        assert_eq!(st.len(), 1);
        assert_eq!(st[0].status, "ok");
        assert!((st[0].current_value).abs() < 1e-9);
        assert!((st[0].consumed_pct).abs() < 1e-9);
        assert!((st[0].remaining - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn budget_status_with_high_usage_warns_or_exceeds() {
        let (_tmp, state) = fresh_state();
        // Goal très bas (0.0001 gCO2eq) → vite dépassé.
        set_personal_goal(
            PersonalGoalDto {
                indicator: "co2eq".into(),
                period: "monthly".into(),
                value_max: 0.0001,
                unit: "gCO2eq".into(),
            },
            &state,
        )
        .unwrap();
        // Quelques estimations
        for _ in 0..3 {
            estimate_prompt(
                EstimationRequestDto {
                    model_id: "gpt-4o-mini".into(),
                    tokens_in: 100,
                    tokens_out_estimated: 500,
                    datacenter_id: None,
                    method: None,
                },
                &state,
            )
            .unwrap();
        }
        let st = get_budget_status(&state).unwrap();
        assert_eq!(st.len(), 1);
        // 0.0001 g de budget vs ~0.006 g de conso → dépassé
        assert_eq!(st[0].status, "exceeded");
        assert!(st[0].consumed_pct > 100.0);
        assert!(st[0].remaining < 0.0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // model detail — C18 / M9
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn get_model_detail_unknown_returns_not_found() {
        let (_tmp, state) = fresh_state();
        let err = get_model_detail("ce-modele-existe-pas", &state).unwrap_err();
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn get_model_detail_known_returns_full_dto() {
        let (_tmp, state) = fresh_state();
        let d = get_model_detail("gpt-4o-mini", &state).unwrap();
        assert_eq!(d.id, "gpt-4o-mini");
        assert!(!d.display_name.is_empty());
        assert!(!d.provider.is_empty());
        assert!(!d.sources.is_empty(), "sources non vides");
        // Triplets ordonnés (P5 ≤ P50 ≤ P95)
        let t = &d.epsilon_prefill_mj_per_token;
        assert!(
            t.p5 <= t.p50 && t.p50 <= t.p95,
            "epsilon_prefill triplet désordonné : {} ≤ {} ≤ {}",
            t.p5,
            t.p50,
            t.p95
        );
        let t = &d.epsilon_decode_mj_per_token;
        assert!(t.p5 <= t.p50 && t.p50 <= t.p95);
        let t = &d.embodied_g_per_request;
        assert!(t.p5 <= t.p50 && t.p50 <= t.p95);
        // Baseline cohérent
        assert!(d.baseline_co2eq_p50_g > 0.0 && d.baseline_co2eq_p50_g.is_finite());
        assert!(d.baseline_co2eq_p5_g <= d.baseline_co2eq_p50_g);
        assert!(d.baseline_co2eq_p50_g <= d.baseline_co2eq_p95_g);
    }

    #[test]
    fn get_model_detail_all_8_models_queryable() {
        let (_tmp, state) = fresh_state();
        let models = list_models().unwrap();
        assert!(models.len() >= 8);
        for m in &models {
            let d = get_model_detail(&m.id, &state).unwrap();
            assert_eq!(d.id, m.id);
        }
    }

    #[test]
    fn get_model_detail_does_not_journal() {
        let (_tmp, state) = fresh_state();
        let before = {
            let l = state.ledger.lock().unwrap();
            l.len().unwrap()
        };
        let _ = get_model_detail("gpt-4o-mini", &state).unwrap();
        let _ = get_model_detail("claude-3-5-sonnet", &state).unwrap();
        let after = {
            let l = state.ledger.lock().unwrap();
            l.len().unwrap()
        };
        assert_eq!(before, after, "get_model_detail ne doit pas journaliser");
    }

    // ─────────────────────────────────────────────────────────────────────
    // benchmark modèles — C17 / M3
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::BenchmarkRequestDto;

    fn bench_req(models: &[&str]) -> BenchmarkRequestDto {
        BenchmarkRequestDto {
            model_ids: models.iter().map(|s| (*s).to_string()).collect(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
        }
    }

    #[test]
    fn benchmark_empty_list_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let err = benchmark_models(bench_req(&[]), &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn benchmark_too_many_models_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let mut models = Vec::new();
        for _ in 0..21 {
            models.push("gpt-4o-mini");
        }
        let req = bench_req(&models);
        let err = benchmark_models(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn benchmark_duplicate_models_rejected() {
        let (_tmp, state) = fresh_state();
        let req = bench_req(&["gpt-4o-mini", "gpt-4o-mini"]);
        let err = benchmark_models(req, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
        assert!(err.message.contains("doublon"));
    }

    #[test]
    fn benchmark_unknown_model_returns_unknown_model() {
        let (_tmp, state) = fresh_state();
        let req = bench_req(&["gpt-4o-mini", "modele-inexistant"]);
        let err = benchmark_models(req, &state).unwrap_err();
        assert_eq!(err.code, "unknown_model");
        assert!(err.message.contains("modele-inexistant"));
    }

    #[test]
    fn benchmark_happy_path_ranks_co2() {
        let (_tmp, state) = fresh_state();
        let req = bench_req(&["gpt-4o-mini", "claude-3-5-sonnet"]);
        let res = benchmark_models(req, &state).unwrap();
        assert_eq!(res.outcomes.len(), 2);
        assert_eq!(res.ranking_by_co2eq_p50.len(), 2);
        // Le ranking #1 doit avoir rank_co2eq = 1
        let winner_id = &res.ranking_by_co2eq_p50[0];
        let winner = res
            .outcomes
            .iter()
            .find(|o| &o.model_id == winner_id)
            .unwrap();
        assert_eq!(winner.rank_co2eq, 1);
        // L'ordre dans `outcomes` suit l'ordre de la requête.
        assert_eq!(res.outcomes[0].model_id, "gpt-4o-mini");
        assert_eq!(res.outcomes[1].model_id, "claude-3-5-sonnet");
    }

    #[test]
    fn benchmark_creates_one_audit_entry_per_model() {
        let (_tmp, state) = fresh_state();
        // Avant : ledger vide.
        let before = {
            let l = state.ledger.lock().unwrap();
            l.len().unwrap()
        };
        assert_eq!(before, 0);
        // 3 modèles → 3 entrées attendues.
        let req = bench_req(&["gpt-4o-mini", "claude-3-5-sonnet", "mistral-medium-3"]);
        let _ = benchmark_models(req, &state).unwrap();
        let after = {
            let l = state.ledger.lock().unwrap();
            l.len().unwrap()
        };
        assert_eq!(after, 3);
    }

    #[test]
    fn benchmark_outcomes_include_calibration_metadata() {
        let (_tmp, state) = fresh_state();
        let req = bench_req(&["gpt-4o-mini", "claude-3-5-sonnet"]);
        let res = benchmark_models(req, &state).unwrap();
        for o in &res.outcomes {
            assert!(!o.display_name.is_empty());
            assert!(!o.provider.is_empty());
            assert!(matches!(
                o.calibration.as_str(),
                "validated" | "indicative" | "extrapolated"
            ));
            assert!(matches!(
                o.openness.as_str(),
                "open" | "open_weights" | "closed"
            ));
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // yearly forecast — C15 / M16
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::{YearlyForecastRequestDto, YearlyScenarioDto};

    fn yearly_request(growth_pcts: &[f64]) -> YearlyForecastRequestDto {
        YearlyForecastRequestDto {
            baseline: EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                method: None,
            },
            scenarios: growth_pcts
                .iter()
                .enumerate()
                .map(|(i, g)| YearlyScenarioDto {
                    label: format!("scenario_{i}"),
                    monthly_growth_pct: *g,
                })
                .collect(),
            months: 12,
            base_volume_per_day: 100.0,
        }
    }

    #[test]
    fn forecast_unknown_model_returns_unknown_model() {
        let (_tmp, state) = fresh_state();
        let mut req = yearly_request(&[0.0]);
        req.baseline.model_id = "ce-modele-existe-pas".into();
        let err = forecast_yearly_budget(req, &state).unwrap_err();
        assert_eq!(err.code, "unknown_model");
    }

    #[test]
    fn forecast_happy_path_journalises_baseline() {
        let (_tmp, state) = fresh_state();
        let req = yearly_request(&[0.0, 5.0, -3.0]);
        let res = forecast_yearly_budget(req, &state).unwrap();
        assert!(res.baseline_audit_id >= 1);
        assert_eq!(res.scenarios.len(), 3);
        // Quantiles baseline cohérents.
        assert!(res.baseline_co2eq_p5_g <= res.baseline_co2eq_p50_g);
        assert!(res.baseline_co2eq_p50_g <= res.baseline_co2eq_p95_g);
    }

    #[test]
    fn forecast_returns_12_monthly_values_per_scenario() {
        let (_tmp, state) = fresh_state();
        let res = forecast_yearly_budget(yearly_request(&[5.0]), &state).unwrap();
        let o = &res.scenarios[0];
        assert_eq!(o.monthly_p5_g.len(), 12);
        assert_eq!(o.monthly_p50_g.len(), 12);
        assert_eq!(o.monthly_p95_g.len(), 12);
        assert_eq!(o.cumulative_p50_g.len(), 12);
    }

    #[test]
    fn forecast_too_many_scenarios_returns_estimator_error() {
        let (_tmp, state) = fresh_state();
        let growths: Vec<f64> = (0..11).map(|i| i as f64).collect();
        let req = yearly_request(&growths);
        let err = forecast_yearly_budget(req, &state).unwrap_err();
        assert_eq!(err.code, "estimator_error");
    }

    #[test]
    fn forecast_invalid_growth_returns_estimator_error() {
        let (_tmp, state) = fresh_state();
        let req = yearly_request(&[200.0]);
        let err = forecast_yearly_budget(req, &state).unwrap_err();
        assert_eq!(err.code, "estimator_error");
    }

    // ─────────────────────────────────────────────────────────────────────
    // CSRD report — C14 / M22
    // ─────────────────────────────────────────────────────────────────────

    use crate::dto::CsrdReportRequestDto;

    #[test]
    fn csrd_report_empty_period_returns_error() {
        let (_tmp, state) = fresh_state();
        let req = CsrdReportRequestDto {
            period_start: "2026-01-01T00:00:00Z".into(),
            period_end: "2026-04-01T00:00:00Z".into(),
            organization_name: "Acme".into(),
            locale: "fr".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let err = export_csrd_report(req, &out_dir, &state).unwrap_err();
        assert_eq!(err.code, "empty_period");
    }

    #[test]
    fn csrd_report_invalid_dates_return_invalid_request() {
        let (_tmp, state) = fresh_state();
        let req = CsrdReportRequestDto {
            period_start: "not-a-date".into(),
            period_end: "2026-04-01T00:00:00Z".into(),
            organization_name: "Acme".into(),
            locale: "fr".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let err = export_csrd_report(req, &out_dir, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn csrd_report_period_inverted_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let req = CsrdReportRequestDto {
            period_start: "2026-04-01T00:00:00Z".into(),
            period_end: "2026-01-01T00:00:00Z".into(),
            organization_name: "Acme".into(),
            locale: "fr".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let err = export_csrd_report(req, &out_dir, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn csrd_report_empty_org_name_rejected() {
        let (_tmp, state) = fresh_state();
        let req = CsrdReportRequestDto {
            period_start: "2026-01-01T00:00:00Z".into(),
            period_end: "2026-04-01T00:00:00Z".into(),
            organization_name: "   ".into(),
            locale: "fr".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let err = export_csrd_report(req, &out_dir, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn csrd_report_unknown_locale_rejected() {
        let (_tmp, state) = fresh_state();
        let req = CsrdReportRequestDto {
            period_start: "2026-01-01T00:00:00Z".into(),
            period_end: "2026-04-01T00:00:00Z".into(),
            organization_name: "Acme".into(),
            locale: "es".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let err = export_csrd_report(req, &out_dir, &state).unwrap_err();
        assert_eq!(err.code, "invalid_request");
    }

    #[test]
    fn csrd_report_happy_path_writes_pdf_and_provo() {
        let (_tmp, state) = fresh_state();
        // 1. Crée des entrées d'audit en passant par estimate_prompt
        let est_req = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: None,
        };
        for _ in 0..3 {
            estimate_prompt(est_req.clone(), &state).unwrap();
        }
        // 2. Génère le rapport pour une période très large couvrant maintenant
        let now = Utc::now();
        let start = now - chrono::Duration::days(1);
        let end = now + chrono::Duration::days(1);
        let req = CsrdReportRequestDto {
            period_start: start.to_rfc3339(),
            period_end: end.to_rfc3339(),
            organization_name: "Acme Corp".into(),
            locale: "fr".into(),
        };
        let out_dir = _tmp.path().join("report-out");
        let result = export_csrd_report(req, &out_dir, &state).unwrap();

        // 3. Validation
        assert_eq!(result.audit_entries_count, 3);
        assert_eq!(result.total_requests, 3);
        assert!(result.total_co2eq_g_p50 > 0.0);
        assert_eq!(result.pdf_sha256.len(), 64);
        let pdf_path = std::path::Path::new(&result.pdf_path);
        let provo_path = std::path::Path::new(&result.provo_path);
        assert!(pdf_path.exists());
        assert!(provo_path.exists());
        // PDF magic bytes
        let pdf_bytes = std::fs::read(pdf_path).unwrap();
        assert!(pdf_bytes.starts_with(b"%PDF-"));
        // PROV-O JSON-LD valide + lien vers le sha256
        let provo_text = std::fs::read_to_string(provo_path).unwrap();
        let provo: serde_json::Value = serde_json::from_str(&provo_text).unwrap();
        assert!(provo["@context"]["prov"].is_string());
        let graph = provo["@graph"].as_array().unwrap();
        assert!(!graph.is_empty());
        let report_node = &graph[0];
        assert_eq!(
            report_node["schema:contentSha256"].as_str().unwrap(),
            result.pdf_sha256
        );
        assert_eq!(report_node["sobria:organizationName"], "Acme Corp");
    }

    #[test]
    fn sankey_fr_data_exposes_real_source_url() {
        let (_tmp, state) = fresh_state();
        write_rte_mix_fixture(&state);
        let sankey = sankey_fr_data(&state).unwrap();
        assert!(sankey.source_url.contains("eco2mix"));
    }

    #[test]
    fn set_preferences_persists_across_state_reinit() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let state = AppState::init_in(tmp.path()).unwrap();
            set_app_preferences(
                AppPreferencesDto {
                    persona: Some(Persona::Enterprise),
                    enabled_modules: vec![ModuleId::M1, ModuleId::M22],
                    onboarded: true,
                    lang: "fr".into(),
                    ..AppPreferencesDto::defaults()
                },
                &state,
            )
            .unwrap();
        }
        // Réouvre l'AppState : le store doit persister.
        let state2 = AppState::init_in(tmp.path()).unwrap();
        let prefs = get_app_preferences(&state2).unwrap();
        assert_eq!(prefs.persona, Some(Persona::Enterprise));
        assert!(prefs.onboarded);
    }
}

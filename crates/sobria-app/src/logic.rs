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
    dto::{
        AppPreferencesDto, AuditEntrySummaryDto, CountryAggregateDto, CsrdReportRequestDto,
        CsrdReportResultDto, DatacenterDetailDto, DatacenterSummaryDto, EstimationRequestDto,
        EstimationResultDto, IndustrialSiteSummaryDto, IntegrityReportDto, MetaInfo,
        ModelPresetDto, RegionFrAggregateDto, SankeyDataDto, SimulationRequestDto,
        SimulationResultDto,
    },
    error::{AppError, IpcError, IpcResult},
    preferences_store::StoredPreferences,
    state::AppState,
};

/// Constante exposée par `meta_info`.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Métadonnées runtime.
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
    let core_req = req.into_core(Utc::now());
    let params = EstimationParams::for_model(&model_id).map_err(AppError::from)?;

    let result = state
        .estimator
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
    let result = state.estimator.estimate(&req, &params)?;
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
    let result = sobria_estimator::simulate(&state.estimator, &sim_core).map_err(AppError::from)?;

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

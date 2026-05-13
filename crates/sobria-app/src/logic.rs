//! Logique métier des commandes IPC, **testable sans Tauri**.
//!
//! Chaque commande `#[tauri::command]` du binaire `sobria-app` se réduit
//! à un appel vers une fonction ici. Cela permet :
//! - des tests unitaires rapides (`cargo test -p sobria-app`),
//! - une réutilisation possible côté CLI plus tard (chantier C10),
//! - une frontière propre `IpcError` ↔ logique interne.

use chrono::Utc;
use sobria_estimator::{available_models, find_preset, EstimationParams};
use tracing::{debug, info};

use crate::{
    dto::{
        AuditEntrySummaryDto, EstimationRequestDto, EstimationResultDto, IntegrityReportDto,
        MetaInfo, ModelPresetDto,
    },
    error::{AppError, IpcError, IpcResult},
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
}

//! Routes admin pour les exports CSRD / PROV-O / CSV (C28.5).
//!
//! Toutes sont en `POST` avec un body JSON `ExportRequest` (cohérent avec
//! les futurs filtres avancés). La fenêtre par défaut : 30 derniers jours.

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use serde::Deserialize;

use crate::exports::{csrd, csv as team_csv, prov_o, ExportRequest};
use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::estimations;

#[derive(Debug, Deserialize, Default)]
pub struct ExportInput {
    /// RFC3339 ; défaut : `now - 30 jours`.
    #[serde(default)]
    pub from: Option<String>,
    /// RFC3339 ; défaut : `now`.
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub entity_name: Option<String>,
    #[serde(default)]
    pub anonymize: bool,
}

impl ExportInput {
    fn resolve(self) -> ApiResult<ExportRequest> {
        let now = Utc::now();
        let from = parse_or(self.from.as_deref(), || now - Duration::days(30))?;
        let to = parse_or(self.to.as_deref(), || now)?;
        if from >= to {
            return Err(ApiError::BadRequest("from doit être < to".into()));
        }
        Ok(ExportRequest {
            from,
            to,
            entity_name: self.entity_name,
            anonymize: self.anonymize,
        })
    }
}

fn parse_or<F: FnOnce() -> chrono::DateTime<Utc>>(
    raw: Option<&str>,
    default_fn: F,
) -> ApiResult<chrono::DateTime<Utc>> {
    match raw {
        None | Some("") => Ok(default_fn()),
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| ApiError::BadRequest(format!("date RFC3339 invalide ({s}): {e}"))),
    }
}

pub async fn handle_csrd(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Json(input): Json<ExportInput>,
) -> ApiResult<Response> {
    let req = input.resolve()?;
    let org = req.entity_name.clone().unwrap_or_else(|| "Équipe".into());
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let rows = estimations::list_for_window(storage.connection(), req.from, req.to)?;
    let artifacts = csrd::build_report(&rows, &org, req.from, req.to)?;

    Ok(pdf_response(
        artifacts.pdf_bytes,
        &org,
        &req.from.format("%Y%m%d").to_string(),
        &req.to.format("%Y%m%d").to_string(),
    ))
}

pub async fn handle_provo(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Json(input): Json<ExportInput>,
) -> ApiResult<Response> {
    let req = input.resolve()?;
    let org = req.entity_name.clone().unwrap_or_else(|| "Équipe".into());
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let rows = estimations::list_for_window(storage.connection(), req.from, req.to)?;
    let v = prov_o::build_team_provo(&rows, &org, req.from, req.to, req.anonymize);
    let bytes = serde_json::to_vec_pretty(&v)
        .map_err(|e| ApiError::InternalMsg(format!("provo serialise: {e}")))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/ld+json")
        .header(
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"sobria-team-prov-{}-to-{}.jsonld\"",
                req.from.format("%Y%m%d"),
                req.to.format("%Y%m%d")
            ),
        )
        .body(Body::from(bytes))
        .map_err(|e| ApiError::InternalMsg(format!("response: {e}")))
}

pub async fn handle_csv(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Json(input): Json<ExportInput>,
) -> ApiResult<Response> {
    let req = input.resolve()?;
    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let rows = estimations::list_for_window(storage.connection(), req.from, req.to)?;
    let bytes = team_csv::build_csv(&rows, req.anonymize)?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"sobria-team-{}-to-{}.csv\"",
                req.from.format("%Y%m%d"),
                req.to.format("%Y%m%d")
            ),
        )
        .body(Body::from(bytes))
        .map_err(|e| ApiError::InternalMsg(format!("response: {e}")))
}

fn pdf_response(bytes: Vec<u8>, org: &str, from_s: &str, to_s: &str) -> Response {
    let slug = org
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"sobria-csrd-{slug}-{from_s}-to-{to_s}.pdf\""),
        )
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

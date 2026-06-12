//! GET /api/v1/admin/analytics?from&to&group_by=day|week|month
//!
//! Retourne 4 sections agrégées sur la fenêtre `[from, to]` :
//! - `series`           : bucketing temporel selon `group_by` (défaut `day`).
//! - `top_models`       : top 10 modèles par gCO₂eq décroissant.
//! - `top_users`        : participants en partage opt-in + agrégat anonyme
//!   (ADR-0015 §3 — plus jamais de classement nominatif par défaut).
//! - `method_breakdown` : afnor_sobria vs ecologits.
//!
//! **k-anonymat (ADR-0015 §2)** : si le nombre d'utilisateurs actifs dans
//! la fenêtre est inférieur à `k = max(3, config.k_anonymity_min)` (défaut
//! 5), les sections sont vides et `k_anonymity.blocked = true` — un agrégat
//! de très petite équipe revient à des données individuelles.
//!
//! Le paramètre `dim` (`user|model|method`) du brief reste accepté mais
//! ignoré en C28.3 — on renvoie tout ce qui peut servir au dashboard.

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::policy::{self, VisibilityPolicy};
use crate::server::auth::middleware::AuthenticatedAdmin;
use crate::server::error::{ApiError, ApiResult};
use crate::server::ServerState;
use crate::storage::analytics::{
    self, GroupBy, MethodBreakdown, ModelTop, ProjectBreakdown, TimeBucket, TopUsersShared,
};

/// Seuil k par défaut si `config.k_anonymity_min` est absent/illisible.
const K_DEFAULT: u32 = 5;
/// Plancher dur : aucune configuration ne peut descendre sous ce k.
const K_FLOOR: u32 = 3;

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    /// RFC3339 ; par défaut : `now - 30 jours`.
    #[serde(default)]
    pub from: Option<String>,
    /// RFC3339 ; par défaut : `now`.
    #[serde(default)]
    pub to: Option<String>,
    /// `day` (défaut) | `week` | `month`.
    #[serde(default)]
    pub group_by: Option<String>,
    /// `user | model | method` — accepté mais ignoré en C28.3.
    #[allow(dead_code)]
    #[serde(default)]
    pub dim: Option<String>,
    /// Top N (défaut : 10, max 100).
    #[serde(default)]
    pub top: Option<u32>,
}

/// État du contrôle k-anonymat joint à chaque réponse (l'UI explique).
#[derive(Debug, Serialize)]
pub struct KAnonymity {
    /// Seuil appliqué (`max(K_FLOOR, config)`).
    pub required: u32,
    /// Utilisateurs actifs distincts dans la fenêtre.
    pub active_users: u64,
    /// `true` → sections vides, agrégats refusés.
    pub blocked: bool,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub from: String,
    pub to: String,
    pub group_by: String,
    /// Politique de visibilité effective (ADR-0016) — l'UI l'affiche.
    pub policy: &'static str,
    pub k_anonymity: KAnonymity,
    pub series: Vec<TimeBucket>,
    pub top_models: Vec<ModelTop>,
    pub top_users: TopUsersShared,
    /// Agrégats par projet (C44) — repli « autres projets » sous k
    /// en modes anonymous/opt_in.
    pub projects: Vec<ProjectBreakdown>,
    pub method_breakdown: Vec<MethodBreakdown>,
}

pub async fn handle(
    State(state): State<ServerState>,
    _admin: AuthenticatedAdmin,
    Query(q): Query<AnalyticsQuery>,
) -> ApiResult<Json<AnalyticsResponse>> {
    let now = Utc::now();
    let from = parse_or_default(q.from.as_deref(), || now - Duration::days(30))?;
    let to = parse_or_default(q.to.as_deref(), || now)?;
    if from >= to {
        return Err(ApiError::BadRequest("from doit être < to".into()));
    }
    let group_by = match q.group_by.as_deref().unwrap_or("day") {
        "day" => GroupBy::Day,
        "week" => GroupBy::Week,
        "month" => GroupBy::Month,
        other => {
            return Err(ApiError::BadRequest(format!(
                "group_by inconnu: {other} (day|week|month)"
            )))
        },
    };
    let top_n = q.top.unwrap_or(10).clamp(1, 100);

    let storage = state
        .storage
        .lock()
        .map_err(|_| ApiError::InternalMsg("storage mutex poisoned".into()))?;
    let conn = storage.connection();

    // ── Politique de visibilité (ADR-0016) puis garde k (ADR-0015 §2). ──
    let pol = policy::load(&storage);
    let k_required = storage
        .get_config("k_anonymity_min")
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(K_DEFAULT)
        .max(K_FLOOR);
    let active_users = analytics::active_user_count(conn, from, to)?;
    // En mode `identified`, le k-anonymat est sans objet (les données
    // individuelles sont visibles par politique d'organisation).
    let blocked = pol != VisibilityPolicy::Identified && active_users < u64::from(k_required);
    let project_k = match pol {
        VisibilityPolicy::Identified => None,
        _ => Some(k_required),
    };

    let (series, top_models, top_users, projects, method_breakdown) = if blocked {
        (
            Vec::new(),
            Vec::new(),
            TopUsersShared::default(),
            Vec::new(),
            Vec::new(),
        )
    } else {
        let top_users = match pol {
            // Anonyme strict : aucune identification, même volontaire —
            // tout le monde est agrégé dans le bloc anonyme.
            VisibilityPolicy::Anonymous => {
                let mut t = analytics::top_users_shared(conn, from, to, top_n)?;
                if !t.identified.is_empty() {
                    for u in t.identified.drain(..) {
                        t.anonymous_users += 1;
                        t.anonymous_count += u.count;
                        t.anonymous_gco2eq_g += u.gco2eq_g;
                    }
                }
                t
            },
            VisibilityPolicy::OptIn => analytics::top_users_shared(conn, from, to, top_n)?,
            VisibilityPolicy::Identified => TopUsersShared {
                identified: analytics::top_users_all(conn, from, to, top_n)?,
                anonymous_users: 0,
                anonymous_count: 0,
                anonymous_gco2eq_g: 0.0,
            },
        };
        (
            analytics::time_buckets(conn, None, from, to, group_by)?,
            analytics::top_models(conn, None, from, to, top_n)?,
            top_users,
            analytics::project_breakdown(conn, from, to, project_k)?,
            analytics::method_breakdown(conn, None, from, to)?,
        )
    };

    Ok(Json(AnalyticsResponse {
        from: from.to_rfc3339(),
        to: to.to_rfc3339(),
        group_by: q.group_by.unwrap_or_else(|| "day".to_string()),
        policy: pol.as_str(),
        k_anonymity: KAnonymity {
            required: k_required,
            active_users,
            blocked,
        },
        series,
        top_models,
        top_users,
        projects,
        method_breakdown,
    }))
}

fn parse_or_default<F: FnOnce() -> DateTime<Utc>>(
    raw: Option<&str>,
    default_fn: F,
) -> Result<DateTime<Utc>, ApiError> {
    match raw {
        None | Some("") => Ok(default_fn()),
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| ApiError::BadRequest(format!("date RFC3339 invalide ({s}): {e}"))),
    }
}

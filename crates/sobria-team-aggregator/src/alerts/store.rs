//! CRUD SQL pour `alert_thresholds` et `alert_triggers` (C29.4).

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::alerts::periods::AlertPeriod;
use crate::error::AggregatorResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertScope {
    User,
    Team,
}

impl AlertScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AlertScope::User => "user",
            AlertScope::Team => "team",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyKind {
    Webhook,
    Email,
    LogOnly,
}

impl NotifyKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            NotifyKind::Webhook => "webhook",
            NotifyKind::Email => "email",
            NotifyKind::LogOnly => "log_only",
        }
    }

    pub fn parse(s: &str) -> AggregatorResult<Self> {
        match s {
            "webhook" => Ok(Self::Webhook),
            "email" => Ok(Self::Email),
            "log_only" => Ok(Self::LogOnly),
            other => Err(crate::error::AggregatorError::Internal(format!(
                "notify_kind inconnu: {other}"
            ))),
        }
    }
}

/// Données pour insérer un nouveau seuil (l'id ULID est généré par l'appelant).
#[derive(Debug, Clone)]
pub struct NewThreshold<'a> {
    pub id: &'a str,
    pub scope: AlertScope,
    pub target_id: Option<&'a str>,
    pub period: AlertPeriod,
    pub gco2eq_max: f64,
    pub notify_kind: NotifyKind,
    pub notify_target: Option<&'a str>,
    pub created_by_admin_id: &'a str,
    pub created_at: DateTime<Utc>,
}

/// Vue d'un seuil pour les routes admin et la logique de check.
#[derive(Debug, Clone, Serialize)]
pub struct Threshold {
    pub id: String,
    pub scope: AlertScope,
    pub target_id: Option<String>,
    pub period: AlertPeriod,
    pub gco2eq_max: f64,
    pub notify_kind: NotifyKind,
    pub notify_target: Option<String>,
    pub created_by_admin_id: String,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

/// Vue d'un trigger pour la table historique.
#[derive(Debug, Clone, Serialize)]
pub struct TriggerRow {
    pub id: String,
    pub threshold_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub observed_gco2eq: f64,
    pub triggered_at: DateTime<Utc>,
    pub notified_at: Option<DateTime<Utc>>,
    pub notify_error: Option<String>,
}

pub fn insert_threshold(conn: &Connection, t: &NewThreshold<'_>) -> AggregatorResult<()> {
    // Garde-fou cohérence scope/target.
    match (t.scope, t.target_id) {
        (AlertScope::User, None) => {
            return Err(crate::error::AggregatorError::Internal(
                "scope=user requiert target_id".into(),
            ));
        },
        (AlertScope::Team, Some(_)) => {
            return Err(crate::error::AggregatorError::Internal(
                "scope=team interdit target_id".into(),
            ));
        },
        _ => {},
    }
    if !t.gco2eq_max.is_finite() || t.gco2eq_max <= 0.0 {
        return Err(crate::error::AggregatorError::Internal(
            "gco2eq_max doit être > 0".into(),
        ));
    }
    if matches!(t.notify_kind, NotifyKind::Webhook | NotifyKind::Email) && t.notify_target.is_none()
    {
        return Err(crate::error::AggregatorError::Internal(
            "notify_kind webhook|email requiert notify_target".into(),
        ));
    }

    conn.execute(
        "INSERT INTO alert_thresholds
            (id, scope, target_id, period, gco2eq_max, notify_kind, notify_target,
             created_by_admin_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            t.id,
            t.scope.as_str(),
            t.target_id,
            t.period.as_str(),
            t.gco2eq_max,
            t.notify_kind.as_str(),
            t.notify_target,
            t.created_by_admin_id,
            t.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Désactive un seuil (soft delete : pose `disabled_at`). Retourne `true`
/// si la ligne existait et était active.
pub fn delete_threshold(conn: &Connection, id: &str, now: DateTime<Utc>) -> AggregatorResult<bool> {
    let affected = conn.execute(
        "UPDATE alert_thresholds SET disabled_at = ?2
         WHERE id = ?1 AND disabled_at IS NULL",
        params![id, now.to_rfc3339()],
    )?;
    Ok(affected > 0)
}

/// Liste les seuils actifs qui s'appliquent à `user_id` (incluant les
/// `scope=team`). Utilisé par le checker.
pub fn list_active_thresholds(
    conn: &Connection,
    user_id: &str,
) -> AggregatorResult<Vec<Threshold>> {
    let mut stmt = conn.prepare(
        "SELECT id, scope, target_id, period, gco2eq_max, notify_kind, notify_target,
                created_by_admin_id, created_at, disabled_at
         FROM alert_thresholds
         WHERE disabled_at IS NULL
           AND (scope = 'team' OR (scope = 'user' AND target_id = ?1))
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![user_id], row_to_threshold)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Liste tous les seuils (actifs ET désactivés) pour le dashboard admin.
pub fn list_thresholds_admin(conn: &Connection) -> AggregatorResult<Vec<Threshold>> {
    let mut stmt = conn.prepare(
        "SELECT id, scope, target_id, period, gco2eq_max, notify_kind, notify_target,
                created_by_admin_id, created_at, disabled_at
         FROM alert_thresholds
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_threshold)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Historique des N derniers triggers (paginé). `from`/`to` filtrent par
/// `triggered_at`.
pub fn list_triggers_admin(
    conn: &Connection,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: u32,
) -> AggregatorResult<Vec<TriggerRow>> {
    let from_s = from
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
    let to_s = to
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| "9999-12-31T23:59:59Z".into());
    let mut stmt = conn.prepare(
        "SELECT id, threshold_id, period_start, period_end, observed_gco2eq,
                triggered_at, notified_at, notify_error
         FROM alert_triggers
         WHERE triggered_at BETWEEN ?1 AND ?2
         ORDER BY triggered_at DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![from_s, to_s, limit], row_to_trigger)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Insère un trigger si aucun n'existe déjà pour `(threshold_id, period_start)`.
/// Retourne `Some(trigger_id)` si insertion réussie, `None` si déjà existant
/// (UNIQUE violé — capturé sans erreur).
pub fn try_insert_trigger(
    conn: &Connection,
    id: &str,
    threshold_id: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    observed_gco2eq: f64,
    triggered_at: DateTime<Utc>,
) -> AggregatorResult<Option<String>> {
    match conn.execute(
        "INSERT INTO alert_triggers
            (id, threshold_id, period_start, period_end, observed_gco2eq, triggered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            threshold_id,
            period_start.to_rfc3339(),
            period_end.to_rfc3339(),
            observed_gco2eq,
            triggered_at.to_rfc3339(),
        ],
    ) {
        Ok(_) => Ok(Some(id.to_string())),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            // Trigger déjà inséré pour cette période — comportement attendu.
            Ok(None)
        },
        Err(e) => Err(e.into()),
    }
}

/// Marque un trigger comme notifié (succès ou échec).
pub fn mark_trigger_notified(
    conn: &Connection,
    trigger_id: &str,
    notified_at: DateTime<Utc>,
    notify_error: Option<&str>,
) -> AggregatorResult<()> {
    conn.execute(
        "UPDATE alert_triggers
         SET notified_at = ?2, notify_error = ?3
         WHERE id = ?1",
        params![trigger_id, notified_at.to_rfc3339(), notify_error],
    )?;
    Ok(())
}

/// Calcule l'observé `gco2eq_p50` pour un user sur une fenêtre.
pub fn observed_gco2eq_user(
    conn: &Connection,
    user_id: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> AggregatorResult<f64> {
    let v: Option<f64> = conn
        .query_row(
            "SELECT COALESCE(SUM(gco2eq_p50), 0.0)
             FROM estimations
             WHERE user_id = ?1 AND ts BETWEEN ?2 AND ?3",
            params![user_id, period_start.to_rfc3339(), period_end.to_rfc3339()],
            |row| row.get::<_, f64>(0),
        )
        .optional()?;
    Ok(v.unwrap_or(0.0))
}

/// Calcule l'observé `gco2eq_p50` pour TOUTE l'équipe sur une fenêtre.
pub fn observed_gco2eq_team(
    conn: &Connection,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> AggregatorResult<f64> {
    let v: Option<f64> = conn
        .query_row(
            "SELECT COALESCE(SUM(gco2eq_p50), 0.0)
             FROM estimations
             WHERE ts BETWEEN ?1 AND ?2",
            params![period_start.to_rfc3339(), period_end.to_rfc3339()],
            |row| row.get::<_, f64>(0),
        )
        .optional()?;
    Ok(v.unwrap_or(0.0))
}

fn row_to_threshold(row: &rusqlite::Row<'_>) -> rusqlite::Result<Threshold> {
    let scope_s: String = row.get(1)?;
    let scope = match scope_s.as_str() {
        "user" => AlertScope::User,
        "team" => AlertScope::Team,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("scope invalide: {scope_s}"),
                )),
            ));
        },
    };
    let period_s: String = row.get(3)?;
    let period: AlertPeriod = period_s.parse().map_err(|e: String| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;
    let notify_kind_s: String = row.get(5)?;
    let notify_kind = match notify_kind_s.as_str() {
        "webhook" => NotifyKind::Webhook,
        "email" => NotifyKind::Email,
        "log_only" => NotifyKind::LogOnly,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("notify_kind invalide: {notify_kind_s}"),
                )),
            ));
        },
    };
    let created_at_s: String = row.get(8)?;
    let created_at = parse_rfc3339_col(8, &created_at_s)?;
    let disabled_at: Option<String> = row.get(9)?;
    let disabled_at = match disabled_at {
        Some(s) => Some(parse_rfc3339_col(9, &s)?),
        None => None,
    };
    Ok(Threshold {
        id: row.get(0)?,
        scope,
        target_id: row.get(2)?,
        period,
        gco2eq_max: row.get(4)?,
        notify_kind,
        notify_target: row.get(6)?,
        created_by_admin_id: row.get(7)?,
        created_at,
        disabled_at,
    })
}

fn row_to_trigger(row: &rusqlite::Row<'_>) -> rusqlite::Result<TriggerRow> {
    let period_start_s: String = row.get(2)?;
    let period_end_s: String = row.get(3)?;
    let triggered_at_s: String = row.get(5)?;
    let notified_at_s: Option<String> = row.get(6)?;
    Ok(TriggerRow {
        id: row.get(0)?,
        threshold_id: row.get(1)?,
        period_start: parse_rfc3339_col(2, &period_start_s)?,
        period_end: parse_rfc3339_col(3, &period_end_s)?,
        observed_gco2eq: row.get(4)?,
        triggered_at: parse_rfc3339_col(5, &triggered_at_s)?,
        notified_at: match notified_at_s {
            Some(s) => Some(parse_rfc3339_col(6, &s)?),
            None => None,
        },
        notify_error: row.get(7)?,
    })
}

fn parse_rfc3339_col(col: usize, s: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn setup() -> Storage {
        let s = Storage::open_in_memory().unwrap();
        s.connection()
            .execute(
                "INSERT INTO admins (id, username, password_hash, created_at)
                 VALUES ('a-1', 'admin', 'h', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        s
    }

    #[test]
    fn insert_team_threshold_roundtrip() {
        let s = setup();
        let now = Utc::now();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-1",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 100.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: now,
            },
        )
        .unwrap();
        let list = list_thresholds_admin(s.connection()).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "t-1");
        assert_eq!(list[0].scope, AlertScope::Team);
        assert!(list[0].target_id.is_none());
        assert!(list[0].disabled_at.is_none());
    }

    #[test]
    fn insert_user_threshold_requires_target() {
        let s = setup();
        let err = insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-2",
                scope: AlertScope::User,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 100.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        );
        assert!(err.is_err());
    }

    #[test]
    fn webhook_kind_requires_target() {
        let s = setup();
        let err = insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-3",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Monthly,
                gco2eq_max: 50.0,
                notify_kind: NotifyKind::Webhook,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        );
        assert!(err.is_err(), "webhook sans URL doit échouer");
    }

    #[test]
    fn delete_threshold_is_soft() {
        let s = setup();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-1",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 100.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        let now = Utc::now();
        assert!(delete_threshold(s.connection(), "t-1", now).unwrap());
        // Re-delete idempotent → false (déjà désactivé).
        assert!(!delete_threshold(s.connection(), "t-1", now).unwrap());
        let list = list_thresholds_admin(s.connection()).unwrap();
        assert!(list[0].disabled_at.is_some());
    }

    #[test]
    fn list_active_filters_disabled_and_scope() {
        let s = setup();
        // Threshold désactivé sur ce user
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-disabled",
                scope: AlertScope::User,
                target_id: Some("u-1"),
                period: AlertPeriod::Daily,
                gco2eq_max: 1.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        delete_threshold(s.connection(), "t-disabled", Utc::now()).unwrap();
        // Threshold actif sur ce user
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-user-1",
                scope: AlertScope::User,
                target_id: Some("u-1"),
                period: AlertPeriod::Daily,
                gco2eq_max: 10.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        // Threshold actif sur un autre user
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-user-2",
                scope: AlertScope::User,
                target_id: Some("u-other"),
                period: AlertPeriod::Daily,
                gco2eq_max: 10.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        // Threshold actif team-wide
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-team",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Monthly,
                gco2eq_max: 100.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        let active = list_active_thresholds(s.connection(), "u-1").unwrap();
        let ids: Vec<&str> = active.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"t-user-1"));
        assert!(ids.contains(&"t-team"));
        assert!(!ids.contains(&"t-user-2"));
        assert!(!ids.contains(&"t-disabled"));
    }

    #[test]
    fn try_insert_trigger_is_idempotent_per_period() {
        let s = setup();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-1",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 50.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: Utc::now(),
            },
        )
        .unwrap();
        let ps = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 5, 16, 0, 0, 0).unwrap();
        let pe = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 5, 16, 23, 59, 59).unwrap();
        let first =
            try_insert_trigger(s.connection(), "tr-1", "t-1", ps, pe, 80.0, Utc::now()).unwrap();
        assert!(first.is_some());
        let second =
            try_insert_trigger(s.connection(), "tr-2", "t-1", ps, pe, 99.0, Utc::now()).unwrap();
        assert!(second.is_none(), "second trigger même période = no-op");
        let triggers = list_triggers_admin(s.connection(), None, None, 10).unwrap();
        assert_eq!(triggers.len(), 1);
    }
}

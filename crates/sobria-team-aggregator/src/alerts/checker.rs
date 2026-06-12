//! Logique de vérification des seuils après chaque estimation insérée.
//!
//! Appelée par `server::api::estimations::handle` (sous le lock SQLite) :
//!
//! 1. Liste les seuils actifs applicables à `user_id` (incluant team).
//! 2. Pour chacun, calcule la consommation observée sur la période courante.
//! 3. Si dépassement et aucun trigger pour cette période → insère un trigger.
//! 4. Retourne la liste des [`AlertEvent`] à notifier (la notification
//!    elle-même est async et tourne en dehors du lock SQLite).

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use ulid::Ulid;

use crate::alerts::periods::period_bounds;
use crate::alerts::store::{
    list_active_thresholds, observed_gco2eq_team, observed_gco2eq_user, try_insert_trigger,
    AlertScope, Threshold,
};
use crate::error::AggregatorResult;

/// Un seuil déclenché, prêt à être notifié.
#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub trigger_id: String,
    pub threshold: Threshold,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub observed_gco2eq: f64,
}

/// Vérifie tous les seuils applicables à `user_id` à l'instant `now`.
///
/// Retourne les [`AlertEvent`] créés (un par seuil dépassé pour la première
/// fois dans sa période). Les seuils déjà déclenchés sur la même période
/// sont silencieusement ignorés (UNIQUE sur `(threshold_id, period_start)`).
pub fn check_thresholds_for_user(
    conn: &Connection,
    user_id: &str,
    now: DateTime<Utc>,
) -> AggregatorResult<Vec<AlertEvent>> {
    let mut events = Vec::new();
    let thresholds = list_active_thresholds(conn, user_id)?;
    for t in thresholds {
        let (period_start, period_end) = period_bounds(t.period, now);
        let observed = match t.scope {
            AlertScope::User => observed_gco2eq_user(
                conn,
                t.target_id.as_deref().unwrap_or(user_id),
                period_start,
                period_end,
            )?,
            AlertScope::Team => observed_gco2eq_team(conn, period_start, period_end)?,
        };
        if observed <= t.gco2eq_max {
            continue;
        }
        let trigger_id = Ulid::new().to_string();
        let inserted = try_insert_trigger(
            conn,
            &trigger_id,
            &t.id,
            period_start,
            period_end,
            observed,
            now,
        )?;
        if let Some(id) = inserted {
            events.push(AlertEvent {
                trigger_id: id,
                threshold: t,
                period_start,
                period_end,
                observed_gco2eq: observed,
            });
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::periods::AlertPeriod;
    use crate::alerts::store::{insert_threshold, NewThreshold, NotifyKind};
    use crate::storage::{estimations, users, Storage};
    use chrono::TimeZone;

    fn setup() -> (Storage, DateTime<Utc>) {
        let s = Storage::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 5, 16, 12, 0, 0).unwrap();
        s.connection()
            .execute(
                "INSERT INTO admins (id, username, password_hash, created_at)
                 VALUES ('a-1', 'admin', 'h', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
        users::insert(s.connection(), "u-1", None, "fp", "h", None, now).unwrap();
        (s, now)
    }

    fn add_estimation(s: &Storage, user_id: &str, gco2eq: f64, ts: DateTime<Utc>) {
        estimations::insert(
            s.connection(),
            &estimations::NewEstimation {
                id: &Ulid::new().to_string(),
                user_id,
                ts,
                method: "afnor_sobria",
                model_id: "m",
                tokens_in: 1,
                tokens_out: 1,
                gco2eq_p50: gco2eq,
                gco2eq_p5: None,
                gco2eq_p95: None,
                water_ml: 0.0,
                energy_wh: 0.0,
                region: None,
                project: None,
                raw_payload_json: "{}",
                received_at: ts,
            },
        )
        .unwrap();
    }

    #[test]
    fn no_threshold_no_event() {
        let (s, now) = setup();
        let evs = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert!(evs.is_empty());
    }

    #[test]
    fn user_threshold_fires_when_exceeded() {
        let (s, now) = setup();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-u",
                scope: AlertScope::User,
                target_id: Some("u-1"),
                period: AlertPeriod::Daily,
                gco2eq_max: 5.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: now,
            },
        )
        .unwrap();
        // 3 estimations à 2g chacune → 6g > 5g
        for _ in 0..3 {
            add_estimation(&s, "u-1", 2.0, now);
        }
        let evs = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert_eq!(evs.len(), 1);
        assert!((evs[0].observed_gco2eq - 6.0).abs() < 1e-9);
        assert_eq!(evs[0].threshold.id, "t-u");
    }

    #[test]
    fn second_check_same_period_does_not_refire() {
        let (s, now) = setup();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-u",
                scope: AlertScope::User,
                target_id: Some("u-1"),
                period: AlertPeriod::Daily,
                gco2eq_max: 5.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: now,
            },
        )
        .unwrap();
        for _ in 0..3 {
            add_estimation(&s, "u-1", 2.0, now);
        }
        let first = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert_eq!(first.len(), 1);
        // Re-check → 0 nouvel event.
        add_estimation(&s, "u-1", 2.0, now);
        let second = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert!(second.is_empty());
    }

    #[test]
    fn team_threshold_uses_team_sum() {
        let (s, now) = setup();
        users::insert(s.connection(), "u-2", None, "fp2", "h", None, now).unwrap();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-team",
                scope: AlertScope::Team,
                target_id: None,
                period: AlertPeriod::Daily,
                gco2eq_max: 10.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: now,
            },
        )
        .unwrap();
        add_estimation(&s, "u-1", 6.0, now);
        add_estimation(&s, "u-2", 6.0, now);
        let evs = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].threshold.scope, AlertScope::Team);
        assert!((evs[0].observed_gco2eq - 12.0).abs() < 1e-9);
    }

    #[test]
    fn threshold_does_not_fire_below_max() {
        let (s, now) = setup();
        insert_threshold(
            s.connection(),
            &NewThreshold {
                id: "t-u",
                scope: AlertScope::User,
                target_id: Some("u-1"),
                period: AlertPeriod::Daily,
                gco2eq_max: 10.0,
                notify_kind: NotifyKind::LogOnly,
                notify_target: None,
                created_by_admin_id: "a-1",
                created_at: now,
            },
        )
        .unwrap();
        add_estimation(&s, "u-1", 5.0, now);
        let evs = check_thresholds_for_user(s.connection(), "u-1", now).unwrap();
        assert!(evs.is_empty());
    }
}

//! Agrégats SQL pour le dashboard admin.
//!
//! Toutes les requêtes utilisent les index `idx_estimations_user_ts`,
//! `idx_estimations_ts` et `idx_estimations_model` posés par le schéma v1.
//! On reste en SQL pur (GROUP BY + COALESCE) : aucune dépendance externe.
//!
//! Granularité temporelle (`GroupBy`) :
//! - `Day`   : `date(ts)`        → "YYYY-MM-DD"
//! - `Week`  : `strftime('%Y-W%W', ts)` → "YYYY-WNN" (semaine ISO simplifiée,
//!   Sunday-based — suffisant pour la viz, on raffine ISO 8601 en C28.4 si besoin).
//! - `Month` : `strftime('%Y-%m', ts)`  → "YYYY-MM"

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AggregatorResult;

/// Bucket temporel (séries pour la viz).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeBucket {
    /// Label du bucket (cf. doc du module pour le format selon `GroupBy`).
    pub bucket: String,
    pub count: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub gco2eq_g: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
}

/// Top modèle (par gCO₂eq cumulé décroissant).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelTop {
    pub model_id: String,
    pub count: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub gco2eq_g: f64,
}

/// Top utilisateur (par gCO₂eq cumulé décroissant). Le dashboard peut
/// anonymiser via un toggle UI ; côté serveur on expose fingerprint +
/// display_name pour permettre les deux vues.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserTop {
    pub user_id: String,
    pub fingerprint: String,
    pub display_name: Option<String>,
    pub count: u64,
    pub gco2eq_g: f64,
}

/// Répartition par méthodologie (AFNOR vs EcoLogits).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodBreakdown {
    pub method: String,
    pub count: u64,
    pub gco2eq_g: f64,
}

/// Granularité temporelle pour les séries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupBy {
    Day,
    Week,
    Month,
}

impl GroupBy {
    fn strftime(self) -> &'static str {
        match self {
            GroupBy::Day => "%Y-%m-%d",
            GroupBy::Week => "%Y-W%W",
            GroupBy::Month => "%Y-%m",
        }
    }
}

/// Séries temporelles agrégées pour `user_id` (`None` = équipe entière) sur
/// la fenêtre `[from, to]`.
pub fn time_buckets(
    conn: &Connection,
    user_id: Option<&str>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    group_by: GroupBy,
) -> AggregatorResult<Vec<TimeBucket>> {
    let bucket_expr = format!("strftime('{}', ts)", group_by.strftime());
    let sql = format!(
        "SELECT {bucket_expr} AS bucket,
                COUNT(*),
                COALESCE(SUM(tokens_in), 0),
                COALESCE(SUM(tokens_out), 0),
                COALESCE(SUM(gco2eq_p50), 0.0),
                COALESCE(SUM(water_ml), 0.0),
                COALESCE(SUM(energy_wh), 0.0)
         FROM estimations
         WHERE ts BETWEEN ?1 AND ?2
           AND (?3 IS NULL OR user_id = ?3)
         GROUP BY bucket
         ORDER BY bucket ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params![from.to_rfc3339(), to.to_rfc3339(), user_id],
        |row| {
            Ok(TimeBucket {
                bucket: row.get(0)?,
                count: row.get::<_, i64>(1)?.max(0) as u64,
                tokens_in: row.get::<_, i64>(2)?.max(0) as u64,
                tokens_out: row.get::<_, i64>(3)?.max(0) as u64,
                gco2eq_g: row.get(4)?,
                water_ml: row.get(5)?,
                energy_wh: row.get(6)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Top N modèles par gCO₂eq décroissant.
pub fn top_models(
    conn: &Connection,
    user_id: Option<&str>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    n: u32,
) -> AggregatorResult<Vec<ModelTop>> {
    let mut stmt = conn.prepare(
        "SELECT model_id,
                COUNT(*),
                COALESCE(SUM(tokens_in), 0),
                COALESCE(SUM(tokens_out), 0),
                COALESCE(SUM(gco2eq_p50), 0.0)
         FROM estimations
         WHERE ts BETWEEN ?1 AND ?2
           AND (?3 IS NULL OR user_id = ?3)
         GROUP BY model_id
         ORDER BY 5 DESC
         LIMIT ?4",
    )?;
    let rows = stmt.query_map(
        params![from.to_rfc3339(), to.to_rfc3339(), user_id, n],
        |row| {
            Ok(ModelTop {
                model_id: row.get(0)?,
                count: row.get::<_, i64>(1)?.max(0) as u64,
                tokens_in: row.get::<_, i64>(2)?.max(0) as u64,
                tokens_out: row.get::<_, i64>(3)?.max(0) as u64,
                gco2eq_g: row.get(4)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Top N utilisateurs par gCO₂eq décroissant (jointure `users` pour
/// l'affichage). Toujours global équipe — un user qui regarde son propre
/// classement passe par `/me/usage`.
pub fn top_users(
    conn: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    n: u32,
) -> AggregatorResult<Vec<UserTop>> {
    let mut stmt = conn.prepare(
        "SELECT u.id, u.fingerprint, u.display_name,
                COUNT(e.id),
                COALESCE(SUM(e.gco2eq_p50), 0.0)
         FROM estimations e
         JOIN users u ON u.id = e.user_id
         WHERE e.ts BETWEEN ?1 AND ?2
         GROUP BY u.id, u.fingerprint, u.display_name
         ORDER BY 5 DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![from.to_rfc3339(), to.to_rfc3339(), n], |row| {
        Ok(UserTop {
            user_id: row.get(0)?,
            fingerprint: row.get(1)?,
            display_name: row.get(2)?,
            count: row.get::<_, i64>(3)?.max(0) as u64,
            gco2eq_g: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Répartition par méthodologie (afnor_sobria vs ecologits) sur la fenêtre.
pub fn method_breakdown(
    conn: &Connection,
    user_id: Option<&str>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> AggregatorResult<Vec<MethodBreakdown>> {
    let mut stmt = conn.prepare(
        "SELECT method,
                COUNT(*),
                COALESCE(SUM(gco2eq_p50), 0.0)
         FROM estimations
         WHERE ts BETWEEN ?1 AND ?2
           AND (?3 IS NULL OR user_id = ?3)
         GROUP BY method
         ORDER BY 3 DESC",
    )?;
    let rows = stmt.query_map(
        params![from.to_rfc3339(), to.to_rfc3339(), user_id],
        |row| {
            Ok(MethodBreakdown {
                method: row.get(0)?,
                count: row.get::<_, i64>(1)?.max(0) as u64,
                gco2eq_g: row.get(2)?,
            })
        },
    )?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{estimations, users, Storage};
    use chrono::TimeZone;

    /// Seed deux users + des estimations dispersées sur 3 jours et 2 modèles.
    fn seed(s: &Storage) {
        users::insert(
            s.connection(),
            "u-1",
            None,
            "fp-1",
            "h",
            Some("Alice"),
            Utc::now(),
        )
        .unwrap();
        users::insert(s.connection(), "u-2", None, "fp-2", "h", None, Utc::now()).unwrap();

        let mk = |id: &str, user: &str, day: u32, model: &str, method: &str, g: f64| {
            let ts = Utc.with_ymd_and_hms(2026, 5, day, 12, 0, 0).unwrap();
            let new = estimations::NewEstimation {
                id,
                user_id: user,
                ts,
                method,
                model_id: model,
                tokens_in: 100,
                tokens_out: 500,
                gco2eq_p50: g,
                gco2eq_p5: None,
                gco2eq_p95: None,
                water_ml: 1.0,
                energy_wh: 0.1,
                region: Some("FR"),
                raw_payload_json: "{}",
                received_at: ts,
            };
            estimations::insert(s.connection(), &new).unwrap();
        };

        // u-1 : 14 mai (llama, afnor 0.2), 14 mai (gpt, afnor 0.3), 15 mai (llama, ecologits 0.4)
        mk("e-1", "u-1", 14, "llama-3-1-70b", "afnor_sobria", 0.2);
        mk("e-2", "u-1", 14, "gpt-4o", "afnor_sobria", 0.3);
        mk("e-3", "u-1", 15, "llama-3-1-70b", "ecologits", 0.4);

        // u-2 : 14 mai (gpt, afnor 0.5), 16 mai (llama, afnor 0.6)
        mk("e-4", "u-2", 14, "gpt-4o", "afnor_sobria", 0.5);
        mk("e-5", "u-2", 16, "llama-3-1-70b", "afnor_sobria", 0.6);
    }

    fn window() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2026, 5, 13, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 5, 17, 0, 0, 0).unwrap(),
        )
    }

    #[test]
    fn time_buckets_groups_by_day_for_whole_team() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let buckets = time_buckets(s.connection(), None, from, to, GroupBy::Day).unwrap();

        // 3 jours présents : 14, 15, 16 mai.
        assert_eq!(buckets.len(), 3);
        let by_day: std::collections::HashMap<_, _> = buckets
            .iter()
            .map(|b| (b.bucket.clone(), b.clone()))
            .collect();
        assert!((by_day["2026-05-14"].gco2eq_g - (0.2 + 0.3 + 0.5)).abs() < 1e-9);
        assert_eq!(by_day["2026-05-14"].count, 3);
        assert_eq!(by_day["2026-05-15"].count, 1);
        assert_eq!(by_day["2026-05-16"].count, 1);
    }

    #[test]
    fn time_buckets_user_scope() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let buckets = time_buckets(s.connection(), Some("u-1"), from, to, GroupBy::Day).unwrap();
        // u-1 a 2 jours d'activité (14 et 15 mai).
        assert_eq!(buckets.len(), 2);
        let total: f64 = buckets.iter().map(|b| b.gco2eq_g).sum();
        assert!((total - 0.9).abs() < 1e-9);
    }

    #[test]
    fn top_models_returns_n_first_descending() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let tops = top_models(s.connection(), None, from, to, 10).unwrap();
        assert_eq!(tops.len(), 2);
        // gpt-4o : 0.3+0.5 = 0.8 ; llama : 0.2+0.4+0.6 = 1.2 → llama 1er.
        assert_eq!(tops[0].model_id, "llama-3-1-70b");
        assert_eq!(tops[1].model_id, "gpt-4o");
        assert!((tops[0].gco2eq_g - 1.2).abs() < 1e-9);
    }

    #[test]
    fn top_users_returns_users_with_display_names() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let tops = top_users(s.connection(), from, to, 10).unwrap();
        assert_eq!(tops.len(), 2);
        // u-2 = 0.5+0.6 = 1.1 ; u-1 = 0.9 → u-2 1er.
        assert_eq!(tops[0].user_id, "u-2");
        assert_eq!(tops[0].display_name, None);
        assert_eq!(tops[1].user_id, "u-1");
        assert_eq!(tops[1].display_name.as_deref(), Some("Alice"));
    }

    #[test]
    fn method_breakdown_separates_afnor_and_ecologits() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let mb = method_breakdown(s.connection(), None, from, to).unwrap();
        assert_eq!(mb.len(), 2);
        // afnor : 0.2+0.3+0.5+0.6 = 1.6 ; ecologits : 0.4 → afnor 1er.
        assert_eq!(mb[0].method, "afnor_sobria");
        assert_eq!(mb[0].count, 4);
        assert!((mb[0].gco2eq_g - 1.6).abs() < 1e-9);
        assert_eq!(mb[1].method, "ecologits");
        assert_eq!(mb[1].count, 1);
    }

    #[test]
    fn time_buckets_groups_by_month() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let buckets = time_buckets(s.connection(), None, from, to, GroupBy::Month).unwrap();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].bucket, "2026-05");
        assert_eq!(buckets[0].count, 5);
    }
}

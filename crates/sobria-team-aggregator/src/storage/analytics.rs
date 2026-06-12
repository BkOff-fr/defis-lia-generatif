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

/// Top utilisateur (par gCO₂eq cumulé décroissant). N'apparaît QUE pour
/// les employés en partage identifié actif (`users.share_identified`,
/// ADR-0015 §3) — l'anonymisation est appliquée côté serveur, jamais
/// déléguée à l'UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserTop {
    pub user_id: String,
    pub fingerprint: String,
    pub display_name: Option<String>,
    pub count: u64,
    pub gco2eq_g: f64,
}

/// Agrégat par projet (C44). `project` NULL → « hors projet ».
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectBreakdown {
    /// Étiquette projet ; `None` = hors projet ; la ligne « autres
    /// projets » (repli k-anonymat) porte `folded = true`.
    pub project: Option<String>,
    pub contributors: u64,
    pub count: u64,
    pub gco2eq_g: f64,
    pub energy_wh: f64,
    pub water_ml: f64,
    pub folded: bool,
}

/// Agrégats par projet sur la fenêtre. Si `k` est fourni (modes
/// anonymous/opt_in, ADR-0016), les projets comptant moins de k
/// contributeurs distincts sont fondus dans une ligne « autres projets »
/// (`folded = true`) — un projet d'une personne est une personne.
pub fn project_breakdown(
    conn: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    k: Option<u32>,
) -> AggregatorResult<Vec<ProjectBreakdown>> {
    let mut stmt = conn.prepare(
        "SELECT project,
                COUNT(DISTINCT user_id),
                COUNT(*),
                COALESCE(SUM(gco2eq_p50), 0.0),
                COALESCE(SUM(energy_wh), 0.0),
                COALESCE(SUM(water_ml), 0.0)
         FROM estimations
         WHERE ts BETWEEN ?1 AND ?2
         GROUP BY project
         ORDER BY SUM(gco2eq_p50) DESC",
    )?;
    let rows: Vec<ProjectBreakdown> = stmt
        .query_map(params![from.to_rfc3339(), to.to_rfc3339()], |row| {
            Ok(ProjectBreakdown {
                project: row.get(0)?,
                contributors: row.get::<_, i64>(1)?.max(0) as u64,
                count: row.get::<_, i64>(2)?.max(0) as u64,
                gco2eq_g: row.get(3)?,
                energy_wh: row.get(4)?,
                water_ml: row.get(5)?,
                folded: false,
            })
        })?
        .collect::<Result<_, _>>()?;

    let Some(k) = k else { return Ok(rows) };
    let k = u64::from(k);
    let mut kept = Vec::new();
    let mut folded = ProjectBreakdown {
        project: None,
        contributors: 0,
        count: 0,
        gco2eq_g: 0.0,
        energy_wh: 0.0,
        water_ml: 0.0,
        folded: true,
    };
    let mut folded_users = std::collections::HashSet::new();
    for r in rows {
        if r.contributors >= k {
            kept.push(r);
        } else {
            // contributors par projet ne s'additionne pas (recouvrements
            // possibles) : on garde le max comme borne basse honnête.
            folded_users.insert(r.project.clone());
            folded.contributors = folded.contributors.max(r.contributors);
            folded.count += r.count;
            folded.gco2eq_g += r.gco2eq_g;
            folded.energy_wh += r.energy_wh;
            folded.water_ml += r.water_ml;
        }
    }
    if folded.count > 0 {
        kept.push(folded);
    }
    Ok(kept)
}

/// Top N users NOMINATIF intégral — réservé à la politique `identified`
/// (ADR-0016) : le handler ne l'appelle jamais dans les autres modes.
pub fn top_users_all(
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
         ORDER BY SUM(e.gco2eq_p50) DESC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(params![from.to_rfc3339(), to.to_rfc3339(), n], |row| {
            Ok(UserTop {
                user_id: row.get(0)?,
                fingerprint: row.get(1)?,
                display_name: row.get(2)?,
                count: row.get::<_, i64>(3)?.max(0) as u64,
                gco2eq_g: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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

/// Nombre d'utilisateurs distincts ACTIFS (≥ 1 estimation) dans la fenêtre.
///
/// Sert de base au contrôle k-anonymat (ADR-0015 §2) : les agrégats équipe
/// ne sont servis que si cette valeur atteint le seuil `k_anonymity_min`.
pub fn active_user_count(
    conn: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> AggregatorResult<u64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT user_id) FROM estimations WHERE ts BETWEEN ?1 AND ?2",
        params![from.to_rfc3339(), to.to_rfc3339()],
        |r| r.get(0),
    )?;
    Ok(n.max(0) as u64)
}

/// Classement « participants » conforme ADR-0015 §3 : seuls les employés
/// en partage identifié actif apparaissent nommément ; tous les autres
/// sont fondus dans une ligne agrégée anonyme.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TopUsersShared {
    /// Participants ayant activé `share_identified`, tri gCO₂eq décroissant.
    pub identified: Vec<UserTop>,
    /// Nombre d'employés actifs NON identifiés (fenêtre).
    pub anonymous_users: u64,
    /// Estimations cumulées de ces employés anonymes.
    pub anonymous_count: u64,
    /// gCO₂eq cumulé de ces employés anonymes.
    pub anonymous_gco2eq_g: f64,
}

/// Top N des participants en partage actif + agrégat anonyme du reste.
/// Toujours global équipe — un user qui regarde son propre usage passe
/// par `/me/usage`.
pub fn top_users_shared(
    conn: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    n: u32,
) -> AggregatorResult<TopUsersShared> {
    let mut stmt = conn.prepare(
        "SELECT u.id, u.fingerprint, u.display_name,
                COUNT(e.id),
                COALESCE(SUM(e.gco2eq_p50), 0.0)
         FROM estimations e
         JOIN users u ON u.id = e.user_id
         WHERE e.ts BETWEEN ?1 AND ?2 AND u.share_identified = 1
         GROUP BY u.id, u.fingerprint, u.display_name
         ORDER BY SUM(e.gco2eq_p50) DESC
         LIMIT ?3",
    )?;
    let identified = stmt
        .query_map(params![from.to_rfc3339(), to.to_rfc3339(), n], |row| {
            Ok(UserTop {
                user_id: row.get(0)?,
                fingerprint: row.get(1)?,
                display_name: row.get(2)?,
                count: row.get::<_, i64>(3)?.max(0) as u64,
                gco2eq_g: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let (anonymous_users, anonymous_count, anonymous_gco2eq_g) = conn.query_row(
        "SELECT COUNT(DISTINCT e.user_id),
                COUNT(e.id),
                COALESCE(SUM(e.gco2eq_p50), 0.0)
         FROM estimations e
         JOIN users u ON u.id = e.user_id
         WHERE e.ts BETWEEN ?1 AND ?2 AND u.share_identified = 0",
        params![from.to_rfc3339(), to.to_rfc3339()],
        |r| {
            Ok((
                r.get::<_, i64>(0)?.max(0) as u64,
                r.get::<_, i64>(1)?.max(0) as u64,
                r.get::<_, f64>(2)?,
            ))
        },
    )?;

    Ok(TopUsersShared {
        identified,
        anonymous_users,
        anonymous_count,
        anonymous_gco2eq_g,
    })
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
                project: None,
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
    fn top_users_shared_hides_everyone_by_default() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        // Personne n'a opté pour le partage → zéro nominatif, tout en agrégat.
        let tops = top_users_shared(s.connection(), from, to, 10).unwrap();
        assert!(tops.identified.is_empty());
        assert_eq!(tops.anonymous_users, 2);
        assert_eq!(tops.anonymous_count, 5);
        assert!((tops.anonymous_gco2eq_g - 2.0).abs() < 1e-9);
    }

    #[test]
    fn top_users_shared_shows_only_opt_in_users() {
        use crate::storage::users::set_share_identified;
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        set_share_identified(s.connection(), "u-1", true).unwrap();
        let tops = top_users_shared(s.connection(), from, to, 10).unwrap();
        // u-1 (Alice, 0.9 g) identifiée ; u-2 reste agrégé anonymement.
        assert_eq!(tops.identified.len(), 1);
        assert_eq!(tops.identified[0].user_id, "u-1");
        assert_eq!(tops.identified[0].display_name.as_deref(), Some("Alice"));
        assert!((tops.identified[0].gco2eq_g - 0.9).abs() < 1e-9);
        assert_eq!(tops.anonymous_users, 1);
        assert_eq!(tops.anonymous_count, 2);
        assert!((tops.anonymous_gco2eq_g - 1.1).abs() < 1e-9);
    }

    #[test]
    fn active_user_count_counts_distinct_users_in_window() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        assert_eq!(active_user_count(s.connection(), from, to).unwrap(), 2);
        // Fenêtre vide → 0.
        let empty_from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let empty_to = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();
        assert_eq!(
            active_user_count(s.connection(), empty_from, empty_to).unwrap(),
            0
        );
    }

    #[test]
    fn project_breakdown_folds_below_k() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        // Tag les estimations : alpha (u-1 + u-2 = 2 contributeurs),
        // solo (u-1 seul), e-5 reste hors projet.
        for (id, p) in [("e-1", "alpha"), ("e-4", "alpha"), ("e-2", "solo"), ("e-3", "solo")] {
            s.connection()
                .execute(
                    "UPDATE estimations SET project = ?1 WHERE id = ?2",
                    rusqlite::params![p, id],
                )
                .unwrap();
        }
        let (from, to) = window();
        // Sans k : 3 lignes (alpha, solo, NULL).
        let all = project_breakdown(s.connection(), from, to, None).unwrap();
        assert_eq!(all.len(), 3);
        // k=2 : alpha (2 contributeurs) reste ; solo (1) et hors-projet (1)
        // sont fondus dans « autres » (folded).
        let gated = project_breakdown(s.connection(), from, to, Some(2)).unwrap();
        assert_eq!(gated.len(), 2);
        let alpha = gated.iter().find(|p| p.project.as_deref() == Some("alpha")).unwrap();
        assert_eq!(alpha.contributors, 2);
        assert!((alpha.gco2eq_g - 0.7).abs() < 1e-9); // e-1 0.2 + e-4 0.5
        let folded = gated.iter().find(|p| p.folded).unwrap();
        assert_eq!(folded.count, 3); // e-2, e-3, e-5
        assert!((folded.gco2eq_g - 1.3).abs() < 1e-9);
    }

    #[test]
    fn top_users_all_is_nominative_regardless_of_sharing() {
        let s = Storage::open_in_memory().unwrap();
        seed(&s);
        let (from, to) = window();
        let tops = top_users_all(s.connection(), from, to, 10).unwrap();
        assert_eq!(tops.len(), 2);
        assert_eq!(tops[0].user_id, "u-2"); // 1.1 g > 0.9 g
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

//! Helpers SQL pour la table `estimations`.
//!
//! Le payload reçu via POST /api/v1/estimations conserve son `raw_payload_json`
//! d'origine (audit + reproductibilité), et les champs typés (méthode, modèle,
//! gCO₂eq, etc.) sont extraits dans des colonnes dédiées pour les analytics.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AggregatorResult;

/// Une estimation typée à insérer.
#[derive(Debug, Clone)]
pub struct NewEstimation<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub ts: DateTime<Utc>,
    pub method: &'a str,
    pub model_id: &'a str,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq_p50: f64,
    pub gco2eq_p5: Option<f64>,
    pub gco2eq_p95: Option<f64>,
    pub water_ml: f64,
    pub energy_wh: f64,
    pub region: Option<&'a str>,
    pub raw_payload_json: &'a str,
    pub received_at: DateTime<Utc>,
}

/// Totaux d'usage d'un user (cards "Mon usage perso" + /me/usage).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageTotals {
    pub count: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub gco2eq_p50_g: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
}

impl Default for UsageTotals {
    fn default() -> Self {
        Self {
            count: 0,
            tokens_in: 0,
            tokens_out: 0,
            gco2eq_p50_g: 0.0,
            water_ml: 0.0,
            energy_wh: 0.0,
        }
    }
}

/// Insère une estimation dans la table.
pub fn insert(conn: &Connection, e: &NewEstimation<'_>) -> AggregatorResult<()> {
    conn.execute(
        "INSERT INTO estimations
            (id, user_id, ts, method, model_id, tokens_in, tokens_out,
             gco2eq_p50, gco2eq_p5, gco2eq_p95, water_ml, energy_wh,
             region, raw_payload_json, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            e.id,
            e.user_id,
            e.ts.to_rfc3339(),
            e.method,
            e.model_id,
            e.tokens_in,
            e.tokens_out,
            e.gco2eq_p50,
            e.gco2eq_p5,
            e.gco2eq_p95,
            e.water_ml,
            e.energy_wh,
            e.region,
            e.raw_payload_json,
            e.received_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Row complet `estimations` (lecture pour exports — joint le user pour
/// avoir fingerprint + display_name disponibles, anonymisables côté handler).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimationRow {
    pub id: String,
    pub user_id: String,
    pub user_fingerprint: String,
    pub user_display_name: Option<String>,
    pub ts: DateTime<Utc>,
    pub method: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq_p50: f64,
    pub gco2eq_p5: Option<f64>,
    pub gco2eq_p95: Option<f64>,
    pub water_ml: f64,
    pub energy_wh: f64,
    pub region: Option<String>,
}

/// Liste les estimations dans la fenêtre `[from, to]` (incluses), avec le
/// fingerprint + display_name du user joint. Utilisé par les exports.
pub fn list_for_window(
    conn: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> AggregatorResult<Vec<EstimationRow>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.user_id, u.fingerprint, u.display_name, e.ts, e.method,
                e.model_id, e.tokens_in, e.tokens_out, e.gco2eq_p50, e.gco2eq_p5,
                e.gco2eq_p95, e.water_ml, e.energy_wh, e.region
         FROM estimations e
         JOIN users u ON u.id = e.user_id
         WHERE e.ts BETWEEN ?1 AND ?2
         ORDER BY e.ts ASC",
    )?;
    let rows = stmt.query_map(params![from.to_rfc3339(), to.to_rfc3339()], |row| {
        let ts_s: String = row.get(4)?;
        let ts = DateTime::parse_from_rfc3339(&ts_s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        Ok(EstimationRow {
            id: row.get(0)?,
            user_id: row.get(1)?,
            user_fingerprint: row.get(2)?,
            user_display_name: row.get(3)?,
            ts,
            method: row.get(5)?,
            model_id: row.get(6)?,
            tokens_in: row.get::<_, i64>(7)?.max(0) as u32,
            tokens_out: row.get::<_, i64>(8)?.max(0) as u32,
            gco2eq_p50: row.get(9)?,
            gco2eq_p5: row.get(10)?,
            gco2eq_p95: row.get(11)?,
            water_ml: row.get(12)?,
            energy_wh: row.get(13)?,
            region: row.get(14)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Totaux pour un utilisateur (toute période confondue).
pub fn totals_for_user(conn: &Connection, user_id: &str) -> AggregatorResult<UsageTotals> {
    let mut totals = UsageTotals::default();
    conn.query_row(
        "SELECT COUNT(*),
                COALESCE(SUM(tokens_in), 0),
                COALESCE(SUM(tokens_out), 0),
                COALESCE(SUM(gco2eq_p50), 0.0),
                COALESCE(SUM(water_ml), 0.0),
                COALESCE(SUM(energy_wh), 0.0)
         FROM estimations WHERE user_id = ?1",
        params![user_id],
        |row| {
            totals.count = row.get::<_, i64>(0)?.max(0) as u64;
            totals.tokens_in = row.get::<_, i64>(1)?.max(0) as u64;
            totals.tokens_out = row.get::<_, i64>(2)?.max(0) as u64;
            totals.gco2eq_p50_g = row.get::<_, f64>(3)?;
            totals.water_ml = row.get::<_, f64>(4)?;
            totals.energy_wh = row.get::<_, f64>(5)?;
            Ok(())
        },
    )?;
    Ok(totals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{users, Storage};

    fn insert_user(s: &Storage) {
        users::insert(s.connection(), "u-1", None, "fp", "h", None, Utc::now()).unwrap();
    }

    fn sample<'a>(id: &'a str, user_id: &'a str, gco2eq: f64) -> NewEstimation<'a> {
        NewEstimation {
            id,
            user_id,
            ts: Utc::now(),
            method: "afnor_sobria",
            model_id: "llama-3-1-70b",
            tokens_in: 120,
            tokens_out: 800,
            gco2eq_p50: gco2eq,
            gco2eq_p5: None,
            gco2eq_p95: None,
            water_ml: 1.5,
            energy_wh: 0.42,
            region: Some("FR"),
            raw_payload_json: "{}",
            received_at: Utc::now(),
        }
    }

    #[test]
    fn insert_and_aggregate_totals() {
        let s = Storage::open_in_memory().unwrap();
        insert_user(&s);
        insert(s.connection(), &sample("e-1", "u-1", 0.4)).unwrap();
        insert(s.connection(), &sample("e-2", "u-1", 0.6)).unwrap();
        let t = totals_for_user(s.connection(), "u-1").unwrap();
        assert_eq!(t.count, 2);
        assert_eq!(t.tokens_in, 240);
        assert_eq!(t.tokens_out, 1600);
        assert!((t.gco2eq_p50_g - 1.0).abs() < 1e-9);
        assert!((t.water_ml - 3.0).abs() < 1e-9);
    }

    #[test]
    fn totals_for_unknown_user_is_zero() {
        let s = Storage::open_in_memory().unwrap();
        let t = totals_for_user(s.connection(), "u-unknown").unwrap();
        assert_eq!(t, UsageTotals::default());
    }
}

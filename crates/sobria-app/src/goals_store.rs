//! Stockage des objectifs personnels (eco-budget, M25 / C19).
//!
//! Table `personal_goals` dans `referentiel.sqlite` (à côté de
//! `app_preferences` introduit en C10).

use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::AppError;

/// Indicateur sur lequel un objectif s'applique.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GoalIndicator {
    Co2Eq,
    Energy,
    Water,
}

impl GoalIndicator {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Co2Eq => "co2eq",
            Self::Energy => "energy",
            Self::Water => "water",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "co2eq" => Some(Self::Co2Eq),
            "energy" => Some(Self::Energy),
            "water" => Some(Self::Water),
            _ => None,
        }
    }

    /// Unité attendue (cohérence indicator ↔ unit).
    #[must_use]
    pub fn expected_unit(self) -> &'static str {
        match self {
            Self::Co2Eq => "gCO2eq",
            Self::Energy => "Wh",
            Self::Water => "L",
        }
    }
}

/// Période d'application d'un objectif.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GoalPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl GoalPeriod {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            _ => None,
        }
    }
}

/// Objectif personnel persisté.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonalGoal {
    pub indicator: GoalIndicator,
    pub period: GoalPeriod,
    pub value_max: f64,
    pub unit: String,
}

impl PersonalGoal {
    /// Valide les invariants : `value_max > 0`, `unit` cohérent.
    pub fn validate(&self) -> Result<(), AppError> {
        if !self.value_max.is_finite() || self.value_max <= 0.0 {
            return Err(AppError::InvalidRequest(format!(
                "value_max doit être strictement positif et fini (got {})",
                self.value_max
            )));
        }
        if self.unit != self.indicator.expected_unit() {
            return Err(AppError::InvalidRequest(format!(
                "unit '{}' incohérent avec indicator '{}' (attendu '{}')",
                self.unit,
                self.indicator.as_str(),
                self.indicator.expected_unit()
            )));
        }
        Ok(())
    }
}

/// Store SQLite des objectifs personnels.
pub struct PersonalGoalsStore {
    conn: Connection,
}

impl PersonalGoalsStore {
    /// Ouvre (ou crée) la table `personal_goals` dans la base donnée.
    /// Utilise la même base `referentiel.sqlite` que `PreferencesStore`.
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS personal_goals (
                indicator   TEXT NOT NULL,
                period      TEXT NOT NULL,
                value_max   REAL NOT NULL,
                unit        TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                PRIMARY KEY (indicator, period)
            );
            ",
        )?;
        info!(path = %path.display(), "goals: ouverture du store");
        Ok(Self { conn })
    }

    /// Liste tous les objectifs, triés par (indicator, period).
    pub fn list_all(&self) -> Result<Vec<PersonalGoal>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT indicator, period, value_max, unit \
             FROM personal_goals ORDER BY indicator, period",
        )?;
        let rows = stmt.query_map([], |row| {
            let indicator: String = row.get(0)?;
            let period: String = row.get(1)?;
            let value_max: f64 = row.get(2)?;
            let unit: String = row.get(3)?;
            Ok((indicator, period, value_max, unit))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (i, p, v, u) = r?;
            let indicator = GoalIndicator::parse(&i).ok_or_else(|| {
                AppError::Internal(format!("indicator '{i}' inconnu en base"))
            })?;
            let period = GoalPeriod::parse(&p).ok_or_else(|| {
                AppError::Internal(format!("period '{p}' inconnue en base"))
            })?;
            out.push(PersonalGoal {
                indicator,
                period,
                value_max: v,
                unit: u,
            });
        }
        Ok(out)
    }

    /// UPSERT d'un objectif (PK = `(indicator, period)`).
    pub fn upsert(&mut self, goal: &PersonalGoal) -> Result<(), AppError> {
        goal.validate()?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO personal_goals (indicator, period, value_max, unit, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(indicator, period) DO UPDATE SET \
               value_max = excluded.value_max, \
               unit = excluded.unit, \
               updated_at = excluded.updated_at",
            params![
                goal.indicator.as_str(),
                goal.period.as_str(),
                goal.value_max,
                goal.unit,
                now,
            ],
        )?;
        debug!(
            indicator = goal.indicator.as_str(),
            period = goal.period.as_str(),
            value_max = goal.value_max,
            "goals: upsert"
        );
        Ok(())
    }

    /// Supprime un objectif. Idempotent (pas d'erreur si absent).
    pub fn delete(
        &mut self,
        indicator: GoalIndicator,
        period: GoalPeriod,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "DELETE FROM personal_goals WHERE indicator = ?1 AND period = ?2",
            params![indicator.as_str(), period.as_str()],
        )?;
        debug!(
            indicator = indicator.as_str(),
            period = period.as_str(),
            "goals: delete"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, PersonalGoalsStore) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("referentiel.sqlite");
        let store = PersonalGoalsStore::open(&path).unwrap();
        (tmp, store)
    }

    #[test]
    fn empty_store_lists_no_goals() {
        let (_tmp, store) = open_temp();
        assert!(store.list_all().unwrap().is_empty());
    }

    #[test]
    fn upsert_then_list_round_trip() {
        let (_tmp, mut store) = open_temp();
        let goal = PersonalGoal {
            indicator: GoalIndicator::Co2Eq,
            period: GoalPeriod::Monthly,
            value_max: 1000.0,
            unit: "gCO2eq".into(),
        };
        store.upsert(&goal).unwrap();
        let list = store.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], goal);
    }

    #[test]
    fn upsert_updates_existing_pk() {
        let (_tmp, mut store) = open_temp();
        let mut g = PersonalGoal {
            indicator: GoalIndicator::Co2Eq,
            period: GoalPeriod::Monthly,
            value_max: 1000.0,
            unit: "gCO2eq".into(),
        };
        store.upsert(&g).unwrap();
        g.value_max = 500.0;
        store.upsert(&g).unwrap();
        let list = store.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert!((list[0].value_max - 500.0).abs() < 1e-9);
    }

    #[test]
    fn validate_rejects_zero_value_max() {
        let g = PersonalGoal {
            indicator: GoalIndicator::Co2Eq,
            period: GoalPeriod::Daily,
            value_max: 0.0,
            unit: "gCO2eq".into(),
        };
        assert!(g.validate().is_err());
    }

    #[test]
    fn validate_rejects_mismatched_unit() {
        let g = PersonalGoal {
            indicator: GoalIndicator::Energy,
            period: GoalPeriod::Daily,
            value_max: 10.0,
            unit: "gCO2eq".into(), // attendu : Wh
        };
        assert!(g.validate().is_err());
    }

    #[test]
    fn delete_existing_goal() {
        let (_tmp, mut store) = open_temp();
        store
            .upsert(&PersonalGoal {
                indicator: GoalIndicator::Water,
                period: GoalPeriod::Weekly,
                value_max: 5.0,
                unit: "L".into(),
            })
            .unwrap();
        store.delete(GoalIndicator::Water, GoalPeriod::Weekly).unwrap();
        assert!(store.list_all().unwrap().is_empty());
    }

    #[test]
    fn delete_missing_goal_is_idempotent() {
        let (_tmp, mut store) = open_temp();
        // Aucun goal présent → delete ne doit pas paniquer.
        store.delete(GoalIndicator::Energy, GoalPeriod::Daily).unwrap();
    }

    #[test]
    fn store_persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("referentiel.sqlite");
        {
            let mut s = PersonalGoalsStore::open(&path).unwrap();
            s.upsert(&PersonalGoal {
                indicator: GoalIndicator::Co2Eq,
                period: GoalPeriod::Daily,
                value_max: 50.0,
                unit: "gCO2eq".into(),
            })
            .unwrap();
        }
        let s2 = PersonalGoalsStore::open(&path).unwrap();
        let list = s2.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert!((list[0].value_max - 50.0).abs() < 1e-9);
    }

    #[test]
    fn multiple_indicators_periods_coexist() {
        let (_tmp, mut store) = open_temp();
        for (i, p) in [
            (GoalIndicator::Co2Eq, GoalPeriod::Daily),
            (GoalIndicator::Co2Eq, GoalPeriod::Monthly),
            (GoalIndicator::Energy, GoalPeriod::Daily),
            (GoalIndicator::Water, GoalPeriod::Weekly),
        ] {
            store
                .upsert(&PersonalGoal {
                    indicator: i,
                    period: p,
                    value_max: 100.0,
                    unit: i.expected_unit().into(),
                })
                .unwrap();
        }
        let list = store.list_all().unwrap();
        assert_eq!(list.len(), 4);
    }
}

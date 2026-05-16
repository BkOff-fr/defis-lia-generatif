//! Calcul des bornes de période d'alerte (daily / weekly / monthly).
//!
//! Toutes les bornes sont en UTC pour rester déterministes côté serveur
//! (l'admin déploie potentiellement dans n'importe quel fuseau). La doc
//! opérationnelle explicite cette convention.
//!
//! - **daily**   : `[YYYY-MM-DD 00:00:00Z, YYYY-MM-DD 23:59:59.999Z]`
//! - **weekly**  : ISO week — lundi 00:00:00Z → dimanche 23:59:59.999Z
//! - **monthly** : 1er du mois 00:00:00Z → dernier jour 23:59:59.999Z

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl AlertPeriod {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AlertPeriod::Daily => "daily",
            AlertPeriod::Weekly => "weekly",
            AlertPeriod::Monthly => "monthly",
        }
    }
}

impl FromStr for AlertPeriod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "daily" => Ok(AlertPeriod::Daily),
            "weekly" => Ok(AlertPeriod::Weekly),
            "monthly" => Ok(AlertPeriod::Monthly),
            other => Err(format!("période inconnue: {other}")),
        }
    }
}

/// Retourne `(start, end)` UTC inclusifs de la période contenant `ts`.
#[must_use]
pub fn period_bounds(period: AlertPeriod, ts: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    match period {
        AlertPeriod::Daily => {
            let date = ts.date_naive();
            let start =
                Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("h00m00s00 valide"));
            let end =
                Utc.from_utc_datetime(&date.and_hms_opt(23, 59, 59).expect("h23m59s59 valide"));
            (start, end)
        },
        AlertPeriod::Weekly => {
            // ISO week : `ts` → lundi de la même semaine, fin = dimanche 23:59:59.
            let date = ts.date_naive();
            let dow = date.weekday().num_days_from_monday() as i64;
            let monday = date - Duration::days(dow);
            let sunday = monday + Duration::days(6);
            let start = Utc.from_utc_datetime(&monday.and_hms_opt(0, 0, 0).expect("monday 00:00"));
            let end = Utc.from_utc_datetime(&sunday.and_hms_opt(23, 59, 59).expect("sunday 23:59"));
            (start, end)
        },
        AlertPeriod::Monthly => {
            let (y, m) = (ts.year(), ts.month());
            let first = NaiveDate::from_ymd_opt(y, m, 1).expect("YYYY-MM-01 valide");
            let last = next_month_first(y, m) - Duration::days(1);
            let start = Utc.from_utc_datetime(&first.and_hms_opt(0, 0, 0).expect("00:00 valide"));
            let end = Utc.from_utc_datetime(&last.and_hms_opt(23, 59, 59).expect("23:59 valide"));
            (start, end)
        },
    }
}

fn next_month_first(year: i32, month: u32) -> NaiveDate {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("year+1 valide")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("mois+1 valide")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Weekday};

    #[test]
    fn from_str_known_periods() {
        assert_eq!("daily".parse::<AlertPeriod>().unwrap(), AlertPeriod::Daily);
        assert_eq!(
            "weekly".parse::<AlertPeriod>().unwrap(),
            AlertPeriod::Weekly
        );
        assert_eq!(
            "monthly".parse::<AlertPeriod>().unwrap(),
            AlertPeriod::Monthly
        );
        assert!("yearly".parse::<AlertPeriod>().is_err());
    }

    #[test]
    fn daily_bounds_clamp_to_utc_day() {
        // 2026-05-16 14:30:00Z → [00:00:00, 23:59:59]
        let ts = Utc.with_ymd_and_hms(2026, 5, 16, 14, 30, 0).unwrap();
        let (s, e) = period_bounds(AlertPeriod::Daily, ts);
        assert_eq!(s.to_rfc3339(), "2026-05-16T00:00:00+00:00");
        assert_eq!(e.to_rfc3339(), "2026-05-16T23:59:59+00:00");
    }

    #[test]
    fn weekly_bounds_iso_monday_to_sunday() {
        // 2026-05-16 = samedi. Lundi de cette semaine = 2026-05-11.
        let ts = Utc.with_ymd_and_hms(2026, 5, 16, 14, 30, 0).unwrap();
        let (s, e) = period_bounds(AlertPeriod::Weekly, ts);
        assert_eq!(s.date_naive().to_string(), "2026-05-11");
        assert_eq!(e.date_naive().to_string(), "2026-05-17");
        assert_eq!(s.weekday(), Weekday::Mon);
        assert_eq!(e.weekday(), Weekday::Sun);
    }

    #[test]
    fn weekly_bounds_when_ts_is_sunday() {
        // 2026-05-17 = dimanche. Lundi = 2026-05-11.
        let ts = Utc.with_ymd_and_hms(2026, 5, 17, 9, 0, 0).unwrap();
        let (s, e) = period_bounds(AlertPeriod::Weekly, ts);
        assert_eq!(s.date_naive().to_string(), "2026-05-11");
        assert_eq!(e.date_naive().to_string(), "2026-05-17");
    }

    #[test]
    fn monthly_bounds_first_to_last_day() {
        let ts = Utc.with_ymd_and_hms(2026, 5, 16, 14, 30, 0).unwrap();
        let (s, e) = period_bounds(AlertPeriod::Monthly, ts);
        assert_eq!(s.to_rfc3339(), "2026-05-01T00:00:00+00:00");
        assert_eq!(e.to_rfc3339(), "2026-05-31T23:59:59+00:00");
    }

    #[test]
    fn monthly_bounds_february_leap() {
        // 2024 = bissextile, février a 29 jours.
        let ts = Utc.with_ymd_and_hms(2024, 2, 15, 0, 0, 0).unwrap();
        let (_, e) = period_bounds(AlertPeriod::Monthly, ts);
        assert_eq!(e.date_naive().to_string(), "2024-02-29");
    }

    #[test]
    fn monthly_bounds_december_wraps_to_january() {
        let ts = Utc.with_ymd_and_hms(2026, 12, 16, 0, 0, 0).unwrap();
        let (_, e) = period_bounds(AlertPeriod::Monthly, ts);
        assert_eq!(e.date_naive().to_string(), "2026-12-31");
    }
}

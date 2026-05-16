//! Sobr.ia — persistance Mode Équipe côté app desktop (C28.6).
//!
//! KV store dans `referentiel.sqlite` (table `team_settings`). Lecture/
//! écriture sérielles via `Mutex<TeamSettingsStore>` dans `AppState`.
//!
//! Clés persistées :
//!
//! - `url`           : URL HTTPS du serveur self-hosted.
//! - `mode`          : `'local' | 'team' | 'both'` (dispatch des estimations).
//! - `user_id`       : ULID du user retourné par /enroll.
//! - `access_token`  : JWT 24h.
//! - `refresh_token` : `<ulid>.<uuid>` (rotation à chaque /refresh).
//! - `fingerprint`   : envoyé à /enroll (déterministe par device).
//! - `enrolled_at`   : ISO timestamp.
//! - `accept_invalid_certs` : `'1'` si l'utilisateur a accepté un cert
//!   auto-signé. Le client reqwest lira cette clé au démarrage.
//! - `last_seen_at`  : C29.1 — ISO timestamp du dernier ping / push réussi.
//! - `estimations_sent` : C29.1 — compteur (string décimal) incrémenté à
//!   chaque `team_push_estimation` ACKé par le serveur.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS team_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

pub const KEY_URL: &str = "url";
pub const KEY_MODE: &str = "mode";
pub const KEY_USER_ID: &str = "user_id";
pub const KEY_ACCESS_TOKEN: &str = "access_token";
pub const KEY_REFRESH_TOKEN: &str = "refresh_token";
pub const KEY_FINGERPRINT: &str = "fingerprint";
pub const KEY_ENROLLED_AT: &str = "enrolled_at";
pub const KEY_ACCEPT_INVALID_CERTS: &str = "accept_invalid_certs";
pub const KEY_LAST_SEEN_AT: &str = "last_seen_at";
pub const KEY_ESTIMATIONS_SENT: &str = "estimations_sent";

/// Mode de dispatch des estimations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamMode {
    #[default]
    Local,
    Team,
    Both,
}

impl TeamMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TeamMode::Local => "local",
            TeamMode::Team => "team",
            TeamMode::Both => "both",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "team" => TeamMode::Team,
            "both" => TeamMode::Both,
            _ => TeamMode::Local,
        }
    }
}

// `TeamMode::Local` est la valeur par défaut canonique (cf. `TeamMode::parse`
// fallback). Le `Default` est dérivé via la convention `#[default]` ci-dessous.

/// Vue d'ensemble du Mode Équipe exposée à l'UI Tauri.
#[derive(Debug, Clone, Serialize)]
pub struct TeamStatus {
    pub enrolled: bool,
    pub url: Option<String>,
    pub user_id: Option<String>,
    pub mode: TeamMode,
    pub fingerprint: Option<String>,
    pub enrolled_at: Option<String>,
    pub accept_invalid_certs: bool,
    /// RFC 3339 du dernier ping/push réussi (C29.1).
    pub last_seen_at: Option<String>,
    /// Nombre d'estimations ACKées par le serveur depuis l'enrôlement (C29.1).
    pub estimations_sent: u32,
}

pub struct TeamSettingsStore {
    conn: Connection,
}

impl TeamSettingsStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("open {}", path.display()))?;
        conn.execute_batch(SCHEMA)
            .context("install team_settings schema")?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM team_settings WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()?)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO team_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM team_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Snapshot complet (utilisé par `team_status` IPC).
    pub fn snapshot(&self) -> Result<TeamStatus> {
        let url = self.get(KEY_URL)?;
        let user_id = self.get(KEY_USER_ID)?;
        let access = self.get(KEY_ACCESS_TOKEN)?;
        let fingerprint = self.get(KEY_FINGERPRINT)?;
        let enrolled_at = self.get(KEY_ENROLLED_AT)?;
        let mode = TeamMode::parse(self.get(KEY_MODE)?.as_deref().unwrap_or("local"));
        let accept_invalid_certs = self
            .get(KEY_ACCEPT_INVALID_CERTS)?
            .as_deref()
            .is_some_and(|v| v == "1");
        let last_seen_at = self.get(KEY_LAST_SEEN_AT)?;
        let estimations_sent = self
            .get(KEY_ESTIMATIONS_SENT)?
            .as_deref()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        Ok(TeamStatus {
            enrolled: access.is_some(),
            url,
            user_id,
            mode,
            fingerprint,
            enrolled_at,
            accept_invalid_certs,
            last_seen_at,
            estimations_sent,
        })
    }

    /// Met à jour `last_seen_at` à l'instant courant (UTC RFC 3339).
    pub fn mark_seen_now(&self) -> Result<()> {
        self.set(KEY_LAST_SEEN_AT, &Utc::now().to_rfc3339())
    }

    /// Incrémente `estimations_sent` (compteur stocké en string décimal).
    /// Réinitialise à `1` si la valeur est absente ou non parsable.
    pub fn increment_estimations_sent(&self) -> Result<u32> {
        let current = self
            .get(KEY_ESTIMATIONS_SENT)?
            .as_deref()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let next = current.saturating_add(1);
        self.set(KEY_ESTIMATIONS_SENT, &next.to_string())?;
        Ok(next)
    }

    /// Efface tokens + user_id + fingerprint + enrolled_at + telemetry locale
    /// (last_seen_at, estimations_sent). Garde url + mode + cert.
    pub fn clear_session(&self) -> Result<()> {
        for k in [
            KEY_ACCESS_TOKEN,
            KEY_REFRESH_TOKEN,
            KEY_USER_ID,
            KEY_FINGERPRINT,
            KEY_ENROLLED_AT,
            KEY_LAST_SEEN_AT,
            KEY_ESTIMATIONS_SENT,
        ] {
            self.delete(k)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_roundtrip() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        assert!(s.get(KEY_URL).unwrap().is_none());
        s.set(KEY_URL, "https://team.exemple.fr:8443").unwrap();
        assert_eq!(
            s.get(KEY_URL).unwrap().as_deref(),
            Some("https://team.exemple.fr:8443")
        );
        s.set(KEY_URL, "https://other.fr:8443").unwrap();
        assert_eq!(
            s.get(KEY_URL).unwrap().as_deref(),
            Some("https://other.fr:8443")
        );
    }

    #[test]
    fn snapshot_default_is_not_enrolled() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        let snap = s.snapshot().unwrap();
        assert!(!snap.enrolled);
        assert_eq!(snap.mode, TeamMode::Local);
        assert!(snap.url.is_none());
        assert!(!snap.accept_invalid_certs);
        assert!(snap.last_seen_at.is_none());
        assert_eq!(snap.estimations_sent, 0);
    }

    #[test]
    fn mark_seen_now_sets_iso_timestamp() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.mark_seen_now().unwrap();
        let snap = s.snapshot().unwrap();
        let ts = snap.last_seen_at.expect("last_seen_at posé");
        // Format RFC 3339 minimal — doit contenir un T et un fuseau.
        assert!(ts.contains('T'), "format RFC 3339 attendu: {ts}");
    }

    #[test]
    fn increment_estimations_sent_starts_at_one_then_monotone() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        assert_eq!(s.increment_estimations_sent().unwrap(), 1);
        assert_eq!(s.increment_estimations_sent().unwrap(), 2);
        assert_eq!(s.increment_estimations_sent().unwrap(), 3);
        let snap = s.snapshot().unwrap();
        assert_eq!(snap.estimations_sent, 3);
    }

    #[test]
    fn increment_recovers_from_corrupt_value() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_ESTIMATIONS_SENT, "garbage").unwrap();
        // Reprise depuis 0 si la valeur est invalide.
        assert_eq!(s.increment_estimations_sent().unwrap(), 1);
    }

    #[test]
    fn snapshot_after_enrollment() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_URL, "https://team.exemple.fr:8443").unwrap();
        s.set(KEY_USER_ID, "01HZZZ001").unwrap();
        s.set(KEY_ACCESS_TOKEN, "jwt-xyz").unwrap();
        s.set(KEY_REFRESH_TOKEN, "id.secret").unwrap();
        s.set(KEY_FINGERPRINT, "app-mac-abc").unwrap();
        s.set(KEY_ENROLLED_AT, "2026-05-16T12:00:00Z").unwrap();
        s.set(KEY_MODE, "both").unwrap();
        s.set(KEY_ACCEPT_INVALID_CERTS, "1").unwrap();
        s.set(KEY_LAST_SEEN_AT, "2026-05-17T08:30:00Z").unwrap();
        s.set(KEY_ESTIMATIONS_SENT, "42").unwrap();

        let snap = s.snapshot().unwrap();
        assert!(snap.enrolled);
        assert_eq!(snap.mode, TeamMode::Both);
        assert_eq!(snap.fingerprint.as_deref(), Some("app-mac-abc"));
        assert!(snap.accept_invalid_certs);
        assert_eq!(snap.last_seen_at.as_deref(), Some("2026-05-17T08:30:00Z"));
        assert_eq!(snap.estimations_sent, 42);
    }

    #[test]
    fn clear_session_keeps_url_and_mode_but_clears_telemetry() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_URL, "https://team.fr:8443").unwrap();
        s.set(KEY_ACCESS_TOKEN, "jwt").unwrap();
        s.set(KEY_REFRESH_TOKEN, "rt").unwrap();
        s.set(KEY_MODE, "team").unwrap();
        s.set(KEY_LAST_SEEN_AT, "2026-05-17T08:30:00Z").unwrap();
        s.set(KEY_ESTIMATIONS_SENT, "42").unwrap();
        s.clear_session().unwrap();

        let snap = s.snapshot().unwrap();
        assert!(!snap.enrolled);
        assert_eq!(snap.url.as_deref(), Some("https://team.fr:8443"));
        assert_eq!(snap.mode, TeamMode::Team);
        assert!(snap.last_seen_at.is_none());
        assert_eq!(snap.estimations_sent, 0);
    }

    #[test]
    fn mode_parse_unknown_falls_back_to_local() {
        assert_eq!(TeamMode::parse("local"), TeamMode::Local);
        assert_eq!(TeamMode::parse("team"), TeamMode::Team);
        assert_eq!(TeamMode::parse("both"), TeamMode::Both);
        assert_eq!(TeamMode::parse("garbage"), TeamMode::Local);
    }
}

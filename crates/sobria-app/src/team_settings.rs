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

use std::path::Path;

use anyhow::{Context, Result};
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
        Ok(TeamStatus {
            enrolled: access.is_some(),
            url,
            user_id,
            mode,
            fingerprint,
            enrolled_at,
            accept_invalid_certs,
        })
    }

    /// Efface tokens + user_id + fingerprint + enrolled_at (garde url + mode + cert).
    pub fn clear_session(&self) -> Result<()> {
        for k in [
            KEY_ACCESS_TOKEN,
            KEY_REFRESH_TOKEN,
            KEY_USER_ID,
            KEY_FINGERPRINT,
            KEY_ENROLLED_AT,
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

        let snap = s.snapshot().unwrap();
        assert!(snap.enrolled);
        assert_eq!(snap.mode, TeamMode::Both);
        assert_eq!(snap.fingerprint.as_deref(), Some("app-mac-abc"));
        assert!(snap.accept_invalid_certs);
    }

    #[test]
    fn clear_session_keeps_url_and_mode() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_URL, "https://team.fr:8443").unwrap();
        s.set(KEY_ACCESS_TOKEN, "jwt").unwrap();
        s.set(KEY_REFRESH_TOKEN, "rt").unwrap();
        s.set(KEY_MODE, "team").unwrap();
        s.clear_session().unwrap();

        let snap = s.snapshot().unwrap();
        assert!(!snap.enrolled);
        assert_eq!(snap.url.as_deref(), Some("https://team.fr:8443"));
        assert_eq!(snap.mode, TeamMode::Team);
    }

    #[test]
    fn mode_parse_unknown_falls_back_to_local() {
        assert_eq!(TeamMode::parse("local"), TeamMode::Local);
        assert_eq!(TeamMode::parse("team"), TeamMode::Team);
        assert_eq!(TeamMode::parse("both"), TeamMode::Both);
        assert_eq!(TeamMode::parse("garbage"), TeamMode::Local);
    }
}

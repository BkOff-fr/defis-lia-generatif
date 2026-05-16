//! Sobr.ia — client REST Mode Équipe côté app desktop (C28.6).
//!
//! Équivalent Rust de `extension/src/lib/team-client.ts`. Wraps `reqwest`
//! avec :
//!
//! - Bearer access token (JWT 24h)
//! - Rotation refresh auto sur 401 (`<ulid>.<uuid>`)
//! - Acceptation explicite des certs auto-signés (opt-in via
//!   `accept_invalid_certs` posé par l'admin au premier enrollment).
//!
//! Voir ADR-0013 Phase 2 et brief C28 §C28.6.
//!
//! ## Mutex + async
//!
//! Le `TeamSettingsStore` vit derrière un `Mutex` dans `AppState`. Pour
//! éviter de garder le verrou en travers d'un `.await` (incompatible avec
//! le pattern `tauri::State` async), [`TeamClient`] possède une **copie
//! locale** des champs nécessaires (URL, tokens, flag cert). Les
//! commandes IPC :
//!
//! 1. Lockent le store, construisent un `ClientConfig` (extraction des
//!    valeurs), droppent le verrou.
//! 2. Font l'appel async.
//! 3. Re-lockent le store pour persister les nouveaux tokens éventuels
//!    (retournés via [`CallOutcome::new_tokens`]).

use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::team_settings::{
    self, TeamSettingsStore, KEY_ACCEPT_INVALID_CERTS, KEY_ACCESS_TOKEN, KEY_ENROLLED_AT,
    KEY_FINGERPRINT, KEY_MODE, KEY_REFRESH_TOKEN, KEY_URL, KEY_USER_ID,
};

#[derive(Debug, Error)]
pub enum TeamClientError {
    #[error("URL serveur équipe non configurée")]
    NoUrl,
    #[error("authentification requise (pas enrôlé ou token expiré)")]
    Unauthorized,
    #[error("requête invalide: {0}")]
    BadRequest(String),
    #[error("erreur HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("erreur transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("erreur stockage: {0}")]
    Storage(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollRequest {
    pub code: String,
    pub password: String,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnrollResponse {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: String,
    pub refresh_expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // champs `role`, `*expires_at` désérialisés pour audit futur
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    access_expires_at: Option<String>,
    #[serde(default)]
    refresh_expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EstimationAck {
    pub id: String,
    pub ack: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
}

/// Snapshot des paramètres lus dans le store, copiés localement pour ne pas
/// devoir garder le `MutexGuard` à travers les `.await`.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub url: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub accept_invalid_certs: bool,
}

impl ClientConfig {
    /// Lit le store et construit une config (URL obligatoire).
    pub fn read(store: &TeamSettingsStore) -> Result<Self, TeamClientError> {
        let url = store
            .get(KEY_URL)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?
            .ok_or(TeamClientError::NoUrl)?;
        let access_token = store
            .get(KEY_ACCESS_TOKEN)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        let refresh_token = store
            .get(KEY_REFRESH_TOKEN)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        let accept_invalid_certs = store
            .get(KEY_ACCEPT_INVALID_CERTS)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?
            .as_deref()
            == Some("1");
        Ok(Self {
            url: url.trim_end_matches('/').to_string(),
            access_token,
            refresh_token,
            accept_invalid_certs,
        })
    }
}

/// Résultat d'un appel API.
///
/// `new_tokens` contient le nouveau couple `(access, refresh)` quand le
/// serveur a émis (via /enroll ou /refresh interne) — à persister dans le
/// store côté caller.
pub struct CallOutcome<T> {
    pub data: T,
    pub new_tokens: Option<(String, String)>,
}

/// Méta-données à persister après un enrollment (en plus des tokens).
pub struct EnrollSideEffects {
    pub user_id: String,
    pub fingerprint: String,
    pub enrolled_at: String,
}

/// Client HTTPS — sans emprunt sur le store, donc déplaçable à travers `.await`.
pub struct TeamClient {
    config: ClientConfig,
    http: Client,
}

impl TeamClient {
    pub fn new(config: ClientConfig) -> Result<Self, TeamClientError> {
        let http = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .user_agent(format!("sobria-app/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { config, http })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.url, path)
    }

    pub async fn ping(&self) -> Result<HealthResponse, TeamClientError> {
        let resp = self.http.get(self.url("/health")).send().await?;
        parse_response(resp).await
    }

    /// POST /api/v1/enroll. Retourne aussi les méta-données à persister.
    pub async fn enroll(
        &self,
        req: EnrollRequest,
    ) -> Result<(EnrollResponse, EnrollSideEffects), TeamClientError> {
        let resp = self
            .http
            .post(self.url("/api/v1/enroll"))
            .json(&req)
            .send()
            .await?;
        let body: EnrollResponse = parse_response(resp).await?;
        let side = EnrollSideEffects {
            user_id: body.user_id.clone(),
            fingerprint: req.fingerprint,
            enrolled_at: Utc::now().to_rfc3339(),
        };
        Ok((body, side))
    }

    /// POST /api/v1/estimations (Bearer requis). Renvoie `(ack, Some(new_tokens))`
    /// si un refresh interne a été effectué.
    pub async fn push_estimation(
        &self,
        payload: &serde_json::Value,
    ) -> Result<CallOutcome<EstimationAck>, TeamClientError> {
        let url = self.url("/api/v1/estimations");
        let token = self
            .config
            .access_token
            .clone()
            .ok_or(TeamClientError::Unauthorized)?;
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(payload)
            .send()
            .await?;

        if resp.status() == StatusCode::UNAUTHORIZED {
            let Some(refresh) = self.config.refresh_token.clone() else {
                return Err(TeamClientError::Unauthorized);
            };
            let refresh_resp = self
                .http
                .post(self.url("/api/v1/refresh"))
                .json(&serde_json::json!({ "refresh_token": refresh }))
                .send()
                .await?;
            if !refresh_resp.status().is_success() {
                return Err(TeamClientError::Unauthorized);
            }
            let new: RefreshResponse = refresh_resp.json().await?;
            let retry = self
                .http
                .post(&url)
                .bearer_auth(&new.access_token)
                .json(payload)
                .send()
                .await?;
            let ack: EstimationAck = parse_response(retry).await?;
            return Ok(CallOutcome {
                data: ack,
                new_tokens: Some((new.access_token, new.refresh_token)),
            });
        }
        let ack: EstimationAck = parse_response(resp).await?;
        Ok(CallOutcome {
            data: ack,
            new_tokens: None,
        })
    }
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, TeamClientError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp.json::<T>().await?);
    }
    let body = resp.text().await.unwrap_or_default();
    if status == StatusCode::UNAUTHORIZED {
        return Err(TeamClientError::Unauthorized);
    }
    if status == StatusCode::BAD_REQUEST {
        return Err(TeamClientError::BadRequest(body));
    }
    Err(TeamClientError::Http {
        status: status.as_u16(),
        body,
    })
}

/// Helpers de persistance — appelés par les IPC après un appel async.
pub mod persist {
    use super::EnrollSideEffects;
    use super::{
        TeamClientError, TeamSettingsStore, KEY_ACCESS_TOKEN, KEY_ENROLLED_AT, KEY_FINGERPRINT,
        KEY_MODE, KEY_REFRESH_TOKEN, KEY_USER_ID,
    };

    /// Pose les tokens dans le store (utilisé après /enroll ou /refresh).
    pub fn save_tokens(
        store: &TeamSettingsStore,
        access: &str,
        refresh: &str,
    ) -> Result<(), TeamClientError> {
        store
            .set(KEY_ACCESS_TOKEN, access)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        store
            .set(KEY_REFRESH_TOKEN, refresh)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Pose les méta-données post-enrollment + bascule le mode à `both` si
    /// le store était sur `local` (cohérent avec l'extension C28.6).
    pub fn save_enroll_side_effects(
        store: &TeamSettingsStore,
        side: &EnrollSideEffects,
    ) -> Result<(), TeamClientError> {
        for (k, v) in [
            (KEY_USER_ID, side.user_id.as_str()),
            (KEY_FINGERPRINT, side.fingerprint.as_str()),
            (KEY_ENROLLED_AT, side.enrolled_at.as_str()),
        ] {
            store
                .set(k, v)
                .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        }
        let current_mode = store
            .get(KEY_MODE)
            .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        if current_mode.as_deref().unwrap_or("local") == "local" {
            store
                .set(KEY_MODE, "both")
                .map_err(|e| TeamClientError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}

/// Wrapper TeamClientError → IpcError pour la couche Tauri.
#[must_use]
pub fn map_team_err(e: TeamClientError) -> crate::error::IpcError {
    let code = match &e {
        TeamClientError::Unauthorized => "unauthorized",
        TeamClientError::NoUrl => "no_url",
        TeamClientError::BadRequest(_) => "bad_request",
        TeamClientError::Http { .. } => "http_error",
        TeamClientError::Transport(_) => "transport",
        TeamClientError::Storage(_) => "storage",
    };
    crate::error::IpcError::new(code, e.to_string())
}

/// Init du store et purge complète (utilisé par les IPC `team_logout` ou
/// les tests d'intégration).
pub fn logout_local(store: &TeamSettingsStore) -> Result<()> {
    store.clear_session().context("clear_session")?;
    store.set(KEY_MODE, "local").context("reset mode")?;
    Ok(())
}

/// `true` si le snapshot indique qu'il faut dispatcher vers le serveur
/// équipe (mode != local + enrolled).
#[must_use]
pub fn should_dispatch_team(snapshot: &team_settings::TeamStatus) -> bool {
    matches!(
        snapshot.mode,
        team_settings::TeamMode::Team | team_settings::TeamMode::Both
    ) && snapshot.enrolled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::team_settings::{TeamMode, TeamSettingsStore};

    #[test]
    fn client_config_read_returns_no_url_when_unset() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        let err = ClientConfig::read(&s).unwrap_err();
        assert!(matches!(err, TeamClientError::NoUrl));
    }

    #[test]
    fn client_config_read_with_full_state() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_URL, "https://team.exemple.fr:8443").unwrap();
        s.set(KEY_ACCESS_TOKEN, "jwt").unwrap();
        s.set(KEY_REFRESH_TOKEN, "rt").unwrap();
        s.set(KEY_ACCEPT_INVALID_CERTS, "1").unwrap();
        let cfg = ClientConfig::read(&s).unwrap();
        assert_eq!(cfg.url, "https://team.exemple.fr:8443");
        assert_eq!(cfg.access_token.as_deref(), Some("jwt"));
        assert!(cfg.accept_invalid_certs);
    }

    #[test]
    fn client_construct_with_accept_invalid_flag() {
        let cfg = ClientConfig {
            url: "https://team.exemple.fr:8443".into(),
            access_token: None,
            refresh_token: None,
            accept_invalid_certs: true,
        };
        let c = TeamClient::new(cfg).unwrap();
        assert_eq!(c.url("/health"), "https://team.exemple.fr:8443/health");
    }

    #[test]
    fn logout_local_clears_session_and_resets_mode() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_URL, "https://team.exemple.fr:8443").unwrap();
        s.set(KEY_ACCESS_TOKEN, "jwt").unwrap();
        s.set(KEY_REFRESH_TOKEN, "rt").unwrap();
        s.set(KEY_USER_ID, "uid").unwrap();
        s.set(KEY_MODE, "team").unwrap();

        logout_local(&s).unwrap();
        let snap = s.snapshot().unwrap();
        assert!(!snap.enrolled);
        assert_eq!(snap.url.as_deref(), Some("https://team.exemple.fr:8443"));
        assert_eq!(snap.mode, TeamMode::Local);
    }

    #[test]
    fn should_dispatch_logic() {
        let mut snap = team_settings::TeamStatus {
            enrolled: false,
            url: None,
            user_id: None,
            mode: TeamMode::Local,
            fingerprint: None,
            enrolled_at: None,
            accept_invalid_certs: false,
            last_seen_at: None,
            estimations_sent: 0,
        };
        assert!(!should_dispatch_team(&snap));
        snap.mode = TeamMode::Team;
        assert!(!should_dispatch_team(&snap), "team mais non enrollé");
        snap.enrolled = true;
        assert!(should_dispatch_team(&snap));
        snap.mode = TeamMode::Both;
        assert!(should_dispatch_team(&snap));
        snap.mode = TeamMode::Local;
        assert!(!should_dispatch_team(&snap), "local même enrollé");
    }

    #[test]
    fn save_tokens_persists() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        persist::save_tokens(&s, "new-jwt", "new-rt").unwrap();
        assert_eq!(s.get(KEY_ACCESS_TOKEN).unwrap().as_deref(), Some("new-jwt"));
        assert_eq!(s.get(KEY_REFRESH_TOKEN).unwrap().as_deref(), Some("new-rt"));
    }

    #[test]
    fn save_enroll_side_effects_switches_local_to_both() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        let side = EnrollSideEffects {
            user_id: "u-1".into(),
            fingerprint: "app-mac-abc".into(),
            enrolled_at: "2026-05-16T12:00:00Z".into(),
        };
        persist::save_enroll_side_effects(&s, &side).unwrap();
        assert_eq!(s.get(KEY_USER_ID).unwrap().as_deref(), Some("u-1"));
        assert_eq!(s.get(KEY_MODE).unwrap().as_deref(), Some("both"));
    }

    #[test]
    fn save_enroll_side_effects_keeps_team_mode_if_already_set() {
        let s = TeamSettingsStore::open_in_memory().unwrap();
        s.set(KEY_MODE, "team").unwrap();
        let side = EnrollSideEffects {
            user_id: "u-1".into(),
            fingerprint: "fp".into(),
            enrolled_at: "2026-05-16T12:00:00Z".into(),
        };
        persist::save_enroll_side_effects(&s, &side).unwrap();
        assert_eq!(s.get(KEY_MODE).unwrap().as_deref(), Some("team"));
    }
}

//! Socket server : reçoit les requêtes du `sobria-bridge` via socket Unix
//! ou named pipe Windows, et les route vers la logique pairing / ingestion
//! existante (patch C27 v0.6.0).
//!
//! ## Protocole
//!
//! Length-prefixed JSON (uint32 LE + JSON UTF-8), idem stdio bridge :
//!   - Bridge envoie `sobria_bridge::BridgeRequest` ;
//!   - App répond `sobria_bridge::BridgeResponse`.
//!
//! ## Fallback offline
//!
//! Si l'app n'est pas lancée quand l'extension envoie une estimation, le
//! bridge écrit dans le spool fichier (`~/.sobria/spool/incoming.jsonl`).
//! L'app continue de drainer le spool toutes les 5 s en parallèle (cf.
//! `logic::drain_extension_spool`) pour récupérer les écritures offline.
//!
//! ## Sécurité
//!
//! Le socket Unix est `bind`é sur un chemin local (`$XDG_RUNTIME_DIR` ou
//! `/tmp`) et hérite des permissions du fs. Le named pipe Windows est
//! créé via `ServerOptions` (HKCU scope par défaut). Pas de port réseau,
//! pas d'authentification réseau — la sécurité repose sur l'OS.
//!
//! Le secret est validé dans `dispatch_request` (Argon2id PHC) avant
//! toute insertion d'évènement.

use anyhow::{Context, Result};
use sobria_bridge::{BridgeRequest, BridgeResponse};
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    extension_store::ExtensionEventInput,
    pairing::{verify_code, PairingError, PairingSecret},
    state::AppState,
};

/// Limite haute de payload (1 MB) — mirroir de la limite côté bridge.
const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

/// Démarre le serveur de socket / pipe. Boucle infinie : retourne `Err`
/// uniquement si le bind initial échoue ; les erreurs par connexion sont
/// logguées et ignorées.
///
/// Reçoit le `AppHandle` Tauri pour pouvoir résoudre `AppState` à chaque
/// dispatch — évite la nécessité de wrapper `AppState` dans un `Arc` côté
/// Tauri manage.
pub async fn run(app_handle: AppHandle) -> Result<()> {
    let path = sobria_bridge::default_socket_path();
    #[cfg(unix)]
    {
        run_unix(app_handle, &path).await
    }
    #[cfg(windows)]
    {
        run_windows(app_handle, &path).await
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (app_handle, path);
        anyhow::bail!("plateforme non supportée pour le socket forward")
    }
}

#[cfg(unix)]
async fn run_unix(app_handle: AppHandle, path: &std::path::Path) -> Result<()> {
    use tokio::net::UnixListener;
    // Nettoie un éventuel résidu d'un précédent crash. `bind` échouerait
    // sinon avec EADDRINUSE.
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let listener =
        UnixListener::bind(path).with_context(|| format!("bind unix socket {}", path.display()))?;
    tracing::info!(path = %path.display(), "bridge_server: en écoute (Unix)");
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_stream(stream, &app_handle).await {
                        tracing::warn!(error = %e, "bridge_server: erreur connexion");
                    }
                });
            },
            Err(e) => {
                tracing::warn!(error = %e, "bridge_server: accept failed");
            },
        }
    }
}

#[cfg(windows)]
async fn run_windows(app_handle: AppHandle, path: &std::path::Path) -> Result<()> {
    use tokio::net::windows::named_pipe::ServerOptions;
    let pipe_name = path.to_str().context("pipe path non-UTF-8")?.to_owned();
    tracing::info!(pipe = %pipe_name, "bridge_server: en écoute (Windows pipe)");
    // Premier instance — `first_pipe_instance(true)` bloque les binds
    // concurrents au même nom.
    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
        .with_context(|| format!("create named pipe {pipe_name}"))?;
    loop {
        server.connect().await.context("named pipe connect")?;
        // Prépare la prochaine instance AVANT de spawn — la spec Win32
        // exige qu'une nouvelle instance soit prête pour le client suivant.
        let next = ServerOptions::new()
            .create(&pipe_name)
            .context("named pipe next instance")?;
        let active = std::mem::replace(&mut server, next);
        let app_handle = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_stream(active, &app_handle).await {
                tracing::warn!(error = %e, "bridge_server: erreur connexion (pipe)");
            }
        });
    }
}

/// Lit une requête length-prefixed sur `stream`, dispatche, écrit la réponse.
async fn handle_stream<S>(mut stream: S, app_handle: &AppHandle) -> Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .context("read request length")?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_PAYLOAD_BYTES {
        anyhow::bail!("request > {MAX_PAYLOAD_BYTES} bytes: {len}");
    }
    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .await
        .context("read request payload")?;
    let req: BridgeRequest = serde_json::from_slice(&payload).context("decode BridgeRequest")?;
    // Tauri manage directement `AppState` (cf. main.rs) — `state.inner()`
    // donne `&AppState`.
    let state = app_handle.state::<AppState>();
    let resp = dispatch_request(state.inner(), req);
    let bytes = serde_json::to_vec(&resp).context("encode BridgeResponse")?;
    let resp_len = u32::try_from(bytes.len()).context("response > u32")?;
    stream.write_all(&resp_len.to_le_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

/// Pure-logic dispatcher : aucune I/O. Testable en isolation.
///
/// - `Ping` → `{ pong: true }`.
/// - `Pair { code }` → consomme `pending_code` via `verify_code`, génère
///   un `PairingSecret`, persiste, retourne `secret` + `pairing_id`.
/// - `Estimate { secret, payload }` → vérifie le secret (Argon2id) contre
///   `device_pairings`, insère un `ExtensionEventInput`.
/// - `Revoke { secret }` → trouve le pairing actif matchant le secret,
///   le révoque.
pub fn dispatch_request(state: &AppState, req: BridgeRequest) -> BridgeResponse {
    match req {
        BridgeRequest::Ping { req_id } => BridgeResponse {
            req_id,
            ok: true,
            error: None,
            pong: Some(true),
            secret: None,
            pairing_id: None,
            fingerprint: None,
        },
        BridgeRequest::Pair { req_id, code } => handle_pair(state, &req_id, &code),
        BridgeRequest::Estimate {
            req_id,
            secret,
            payload,
        } => handle_estimate(state, &req_id, &secret, &payload),
        BridgeRequest::Revoke { req_id, secret } => handle_revoke(state, &req_id, &secret),
    }
}

/// Identifiant fingerprint synthétique côté app (le bridge ne le passe pas
/// dans la requête en v0.6.0 — patch C27 v0.7+ pourra l'enrichir avec un
/// ID par browser détecté côté extension).
fn bridge_fingerprint() -> &'static str {
    "sobria-bridge-native-messaging"
}

fn handle_pair(state: &AppState, req_id: &str, code: &str) -> BridgeResponse {
    let now = chrono::Utc::now();
    let mut guard = match state.pending_code.lock() {
        Ok(g) => g,
        Err(e) => return BridgeResponse::err(req_id, format!("state lock: {e}")),
    };
    let Some(pending) = guard.as_ref() else {
        return BridgeResponse::err(
            req_id,
            "aucun code de pairing en attente — génère-en un depuis Sobr.ia /parametres",
        );
    };
    if let Err(e) = verify_code(pending, code, now) {
        let msg = match e {
            PairingError::Malformed => "code à 6 chiffres requis",
            PairingError::InvalidCode => "code invalide ou expiré",
        };
        return BridgeResponse::err(req_id, msg);
    }
    let secret = PairingSecret::new();
    let fingerprint = bridge_fingerprint();
    let store = match state.extension_store.lock() {
        Ok(s) => s,
        Err(e) => return BridgeResponse::err(req_id, format!("store lock: {e}")),
    };
    let pairing_id = match store.record_pairing(fingerprint, &secret) {
        Ok(id) => id,
        Err(e) => return BridgeResponse::err(req_id, format!("record pairing: {e}")),
    };
    // Consomme le code (single-use).
    *guard = None;
    tracing::info!(%pairing_id, "socket_server: pairing validé via bridge");
    BridgeResponse {
        req_id: req_id.into(),
        ok: true,
        error: None,
        pong: None,
        secret: Some(secret.secret_hex),
        pairing_id: Some(pairing_id),
        fingerprint: Some(fingerprint.into()),
    }
}

fn handle_estimate(
    state: &AppState,
    req_id: &str,
    secret: &str,
    payload: &serde_json::Value,
) -> BridgeResponse {
    let store = match state.extension_store.lock() {
        Ok(s) => s,
        Err(e) => return BridgeResponse::err(req_id, format!("store lock: {e}")),
    };
    let fingerprint = bridge_fingerprint();
    let pairing_id = match store.verify_secret(fingerprint, secret) {
        Ok(Some(id)) => id,
        Ok(None) => return BridgeResponse::err(req_id, "secret invalide ou pairing révoqué"),
        Err(e) => return BridgeResponse::err(req_id, format!("verify_secret: {e}")),
    };
    let Some(estimate) = payload.get("estimate") else {
        return BridgeResponse::err(req_id, "payload sans champ `estimate`");
    };
    let input = ExtensionEventInput {
        pairing_id: pairing_id.clone(),
        ts: chrono::Utc::now(),
        method: estimate
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("afnor_sobria")
            .to_string(),
        model_id: estimate
            .get("modelId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        tokens_in: estimate
            .get("tokensIn")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        tokens_out: estimate
            .get("tokensOut")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        gco2eq_p50: estimate
            .get("gco2eq")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        water_ml: estimate
            .get("waterMl")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        energy_wh: estimate
            .get("energyWh")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        raw_payload_json: payload.to_string(),
    };
    if let Err(e) = store.record_event(&input) {
        return BridgeResponse::err(req_id, format!("record_event: {e}"));
    }
    BridgeResponse {
        req_id: req_id.into(),
        ok: true,
        error: None,
        pong: None,
        secret: None,
        pairing_id: Some(pairing_id),
        fingerprint: None,
    }
}

fn handle_revoke(state: &AppState, req_id: &str, secret: &str) -> BridgeResponse {
    let store = match state.extension_store.lock() {
        Ok(s) => s,
        Err(e) => return BridgeResponse::err(req_id, format!("store lock: {e}")),
    };
    let fingerprint = bridge_fingerprint();
    let pairing_id = match store.verify_secret(fingerprint, secret) {
        Ok(Some(id)) => id,
        Ok(None) => return BridgeResponse::err(req_id, "secret invalide — déjà révoqué ?"),
        Err(e) => return BridgeResponse::err(req_id, format!("verify_secret: {e}")),
    };
    if let Err(e) = store.revoke_pairing(&pairing_id) {
        return BridgeResponse::err(req_id, format!("revoke: {e}"));
    }
    tracing::info!(%pairing_id, "socket_server: pairing révoqué via bridge");
    BridgeResponse {
        req_id: req_id.into(),
        ok: true,
        error: None,
        pong: None,
        secret: None,
        pairing_id: Some(pairing_id),
        fingerprint: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use sobria_bridge::BridgeRequest;

    fn fresh_state() -> (tempfile::TempDir, AppState) {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::init_in(tmp.path()).unwrap();
        (tmp, state)
    }

    #[test]
    fn ping_returns_pong() {
        let (_tmp, state) = fresh_state();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Ping {
                req_id: "r1".into(),
            },
        );
        assert!(resp.ok);
        assert_eq!(resp.pong, Some(true));
        assert!(resp.error.is_none());
    }

    #[test]
    fn pair_without_pending_code_returns_error() {
        let (_tmp, state) = fresh_state();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r1".into(),
                code: "123456".into(),
            },
        );
        assert!(!resp.ok);
        assert!(resp.error.unwrap().contains("aucun code"));
    }

    #[test]
    fn pair_with_valid_code_returns_secret_and_pairing_id() {
        let (_tmp, state) = fresh_state();
        // Génère un code.
        let dto = crate::logic::regenerate_pairing_code(&state).unwrap();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r1".into(),
                code: dto.code,
            },
        );
        assert!(resp.ok, "got error: {:?}", resp.error);
        assert!(resp.secret.as_deref().unwrap_or("").len() == 64);
        assert!(resp.pairing_id.is_some());
        assert_eq!(resp.fingerprint.as_deref(), Some(bridge_fingerprint()));
    }

    #[test]
    fn pair_with_wrong_code_returns_invalid_error() {
        let (_tmp, state) = fresh_state();
        let _ = crate::logic::regenerate_pairing_code(&state).unwrap();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r1".into(),
                code: "000000".into(),
            },
        );
        assert!(!resp.ok);
        let err = resp.error.unwrap();
        // Soit malformé (chiffres non-valides) soit invalide (mauvais code) :
        // selon les chances de collision, "000000" peut accidentellement matcher.
        // On accepte les deux variantes.
        assert!(
            err.contains("invalide") || err.contains("expiré") || err.contains("chiffres"),
            "got: {err}"
        );
    }

    #[test]
    fn estimate_with_wrong_secret_returns_error() {
        let (_tmp, state) = fresh_state();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Estimate {
                req_id: "r1".into(),
                secret: "0".repeat(64),
                payload: serde_json::json!({ "estimate": {} }),
            },
        );
        assert!(!resp.ok);
        assert!(resp.error.unwrap().contains("secret invalide"));
    }

    #[test]
    fn estimate_with_valid_secret_records_event() {
        let (_tmp, state) = fresh_state();
        // Pair pour obtenir un secret.
        let dto = crate::logic::regenerate_pairing_code(&state).unwrap();
        let pair_resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r-pair".into(),
                code: dto.code,
            },
        );
        let secret = pair_resp.secret.unwrap();

        // Maintenant Estimate.
        let resp = dispatch_request(
            &state,
            BridgeRequest::Estimate {
                req_id: "r-est".into(),
                secret,
                payload: serde_json::json!({
                    "estimate": {
                        "method": "afnor_sobria",
                        "modelId": "gpt-4o-mini",
                        "tokensIn": 50,
                        "tokensOut": 200,
                        "gco2eq": 0.42,
                        "waterMl": 1.8,
                        "energyWh": 0.12
                    }
                }),
            },
        );
        assert!(resp.ok, "got error: {:?}", resp.error);

        // Vérifie l'évènement persisté.
        let store = state.extension_store.lock().unwrap();
        let events = store.list_events(10, 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].model_id, "gpt-4o-mini");
        assert_eq!(events[0].tokens_in, 50);
    }

    #[test]
    fn revoke_with_valid_secret_revokes_pairing() {
        let (_tmp, state) = fresh_state();
        let dto = crate::logic::regenerate_pairing_code(&state).unwrap();
        let pair_resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r-pair".into(),
                code: dto.code,
            },
        );
        let secret = pair_resp.secret.unwrap();

        let revoke_resp = dispatch_request(
            &state,
            BridgeRequest::Revoke {
                req_id: "r-rev".into(),
                secret: secret.clone(),
            },
        );
        assert!(revoke_resp.ok, "got error: {:?}", revoke_resp.error);

        // Second revoke avec le même secret → ne doit plus matcher (révoqué).
        let resp2 = dispatch_request(
            &state,
            BridgeRequest::Revoke {
                req_id: "r-rev2".into(),
                secret,
            },
        );
        assert!(!resp2.ok);
    }

    #[test]
    fn estimate_without_estimate_field_returns_error() {
        let (_tmp, state) = fresh_state();
        let dto = crate::logic::regenerate_pairing_code(&state).unwrap();
        let pair_resp = dispatch_request(
            &state,
            BridgeRequest::Pair {
                req_id: "r-pair".into(),
                code: dto.code,
            },
        );
        let secret = pair_resp.secret.unwrap();
        let resp = dispatch_request(
            &state,
            BridgeRequest::Estimate {
                req_id: "r-est".into(),
                secret,
                payload: serde_json::json!({ "other": "thing" }),
            },
        );
        assert!(!resp.ok);
        assert!(resp.error.unwrap().contains("estimate"));
    }
}

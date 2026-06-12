//! `sobria-bridge` — pont Native Messaging extension ↔ app Sobr.ia desktop (C27.5).
//!
//! Voir `main.rs` pour la boucle stdin/stdout du binaire. Cette `lib`
//! expose le protocole + le spool pour permettre tests et intégration
//! côté `sobria-app` (C27.5.b/c/d).

use std::io::{Read, Write};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Nom du binaire dans le manifest natif WebExtensions.
pub const NATIVE_HOST_NAME: &str = "com.sobria.bridge";

/// Taille max du spool fichier avant rotation (10 MB).
pub const SPOOL_MAX_BYTES: u64 = 10 * 1024 * 1024;

/// Requête envoyée par l'extension (discriminée par `type`).
///
/// `Serialize` est ajoutée pour le patch v0.6.0 — le bridge sérialise la
/// requête vers le socket app desktop ; les tests round-trip exercent les
/// deux sens.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeRequest {
    Ping {
        #[serde(rename = "reqId")]
        req_id: String,
    },
    Pair {
        #[serde(rename = "reqId")]
        req_id: String,
        code: String,
    },
    Estimate {
        #[serde(rename = "reqId")]
        req_id: String,
        secret: String,
        payload: Value,
    },
    Revoke {
        #[serde(rename = "reqId")]
        req_id: String,
        secret: String,
    },
}

impl BridgeRequest {
    pub fn req_id(&self) -> &str {
        match self {
            BridgeRequest::Ping { req_id }
            | BridgeRequest::Pair { req_id, .. }
            | BridgeRequest::Estimate { req_id, .. }
            | BridgeRequest::Revoke { req_id, .. } => req_id,
        }
    }
}

/// Réponse renvoyée à l'extension.
///
/// `Deserialize` est ajoutée pour le patch v0.6.0 — le bridge décode la
/// réponse renvoyée par le socket app desktop avant de la repasser à
/// l'extension par stdout.
#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeResponse {
    #[serde(rename = "reqId")]
    pub req_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pong: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(rename = "pairingId", skip_serializing_if = "Option::is_none")]
    pub pairing_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

impl BridgeResponse {
    pub fn ok(req_id: impl Into<String>) -> Self {
        Self {
            req_id: req_id.into(),
            ok: true,
            error: None,
            pong: None,
            secret: None,
            pairing_id: None,
            fingerprint: None,
        }
    }

    pub fn err(req_id: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            req_id: req_id.into(),
            ok: false,
            error: Some(msg.into()),
            pong: None,
            secret: None,
            pairing_id: None,
            fingerprint: None,
        }
    }
}

/// Lit un message length-prefixed (uint32 LE + JSON bytes) depuis `r`.
/// Retourne `Ok(None)` quand le pipe est fermé (EOF).
pub fn read_message<R: Read>(r: &mut R) -> Result<Option<BridgeRequest>> {
    let mut len_buf = [0u8; 4];
    if let Err(e) = r.read_exact(&mut len_buf) {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            return Ok(None);
        }
        return Err(e.into());
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > 1024 * 1024 {
        anyhow::bail!("message length exceeds 1 MB: {len}");
    }
    let mut payload = vec![0u8; len];
    r.read_exact(&mut payload)
        .context("read native messaging payload")?;
    let req: BridgeRequest =
        serde_json::from_slice(&payload).with_context(|| format!("decode JSON ({len} bytes)"))?;
    Ok(Some(req))
}

/// Écrit un message length-prefixed sur `w`.
pub fn write_message<W: Write>(w: &mut W, resp: &BridgeResponse) -> Result<()> {
    let bytes = serde_json::to_vec(resp)?;
    let len = u32::try_from(bytes.len()).context("response exceeds u32")?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&bytes)?;
    w.flush()?;
    Ok(())
}

/// Chemin du spool fichier (`~/.sobria/spool/incoming.jsonl`).
pub fn spool_path() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().context("home_dir introuvable")?;
    Ok(home.join(".sobria").join("spool").join("incoming.jsonl"))
}

/// Append une ligne JSON dans le spool, avec rotation auto à 10 MB.
pub fn append_to_spool(payload: &Value) -> Result<()> {
    append_to_spool_at(&spool_path()?, payload, SPOOL_MAX_BYTES)
}

/// Variante testable : écrit dans un chemin explicite avec seuil de rotation
/// paramétrable. Utilisé par les tests pour ne pas écrire dans `$HOME`.
pub fn append_to_spool_at(path: &std::path::Path, payload: &Value, max_bytes: u64) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > max_bytes {
            let bak = path.with_extension("jsonl.bak");
            let _ = std::fs::remove_file(&bak);
            std::fs::rename(path, &bak)?;
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(payload)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Chemin par défaut du socket / named pipe partagé bridge ↔ app (patch
/// C27 v0.6.0).
///
///   - **Unix** : `$XDG_RUNTIME_DIR/sobria-bridge.sock` si défini, sinon
///     `/tmp/sobria-bridge.sock`.
///   - **Windows** : `\\.\pipe\sobria-bridge`.
///
/// Les permissions du fs (Unix) ou la portée HKCU (named pipe) suffisent
/// à scoper à l'utilisateur courant — pas de port réseau, pas d'auth réseau.
pub fn default_socket_path() -> std::path::PathBuf {
    #[cfg(unix)]
    {
        if let Some(rt) = std::env::var_os("XDG_RUNTIME_DIR") {
            return std::path::PathBuf::from(rt).join("sobria-bridge.sock");
        }
        std::path::PathBuf::from("/tmp/sobria-bridge.sock")
    }
    #[cfg(windows)]
    {
        std::path::PathBuf::from(r"\\.\pipe\sobria-bridge")
    }
    #[cfg(not(any(unix, windows)))]
    {
        std::path::PathBuf::from("sobria-bridge.sock")
    }
}

/// Timeout court (2 s) pour les forwards synchrones : si l'app desktop ne
/// répond pas en 2 s, on retombe sur le spool fichier. Sur Windows on
/// s'appuie sur le timeout natif du `OpenOptions` sur le pipe.
#[cfg(unix)]
const FORWARD_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2_000);

/// Tente de forwarder une requête au serveur de socket de l'app desktop.
///
/// Protocole identique au stdio bridge : `u32 little-endian` de longueur +
/// payload JSON UTF-8. La réponse est lue dans le même format.
///
/// Retourne `Err` si :
///   - le socket / pipe n'existe pas (app non lancée) ;
///   - la connexion timeout (2 s) ;
///   - la réponse est malformée.
///
/// Le bridge utilise ce retour pour décider de retomber sur `append_to_spool`.
pub fn try_forward_to(path: &std::path::Path, req: &BridgeRequest) -> Result<BridgeResponse> {
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        let mut stream =
            UnixStream::connect(path).with_context(|| format!("connect {}", path.display()))?;
        stream.set_read_timeout(Some(FORWARD_TIMEOUT))?;
        stream.set_write_timeout(Some(FORWARD_TIMEOUT))?;
        forward_over(&mut stream, req)
    }
    #[cfg(windows)]
    {
        let path_str = path.to_str().context("pipe path non-UTF-8")?;
        let mut stream = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_str)
            .with_context(|| format!("open pipe {path_str}"))?;
        forward_over(&mut stream, req)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (path, req);
        anyhow::bail!("plateforme non supportée pour try_forward_to")
    }
}

fn forward_over<S: Read + Write>(stream: &mut S, req: &BridgeRequest) -> Result<BridgeResponse> {
    let bytes = serde_json::to_vec(req).context("encode BridgeRequest")?;
    let len = u32::try_from(bytes.len()).context("request > u32")?;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .context("read response length")?;
    let resp_len = u32::from_le_bytes(len_buf) as usize;
    if resp_len > 1024 * 1024 {
        anyhow::bail!("response too large: {resp_len}");
    }
    let mut payload = vec![0u8; resp_len];
    stream
        .read_exact(&mut payload)
        .context("read response payload")?;
    let resp: BridgeResponse = serde_json::from_slice(&payload).context("decode BridgeResponse")?;
    Ok(resp)
}

/// Hash court (8 hex chars d'un FNV-like) — éviter de logger le secret en clair.
pub fn short_hash(s: &str) -> String {
    let mut acc: u64 = 0;
    for &b in s.as_bytes() {
        acc = acc.wrapping_mul(31).wrapping_add(u64::from(b));
    }
    format!("{:016x}", acc)
}

/// Traite une requête. v0.6.0 :
///   - Estimate → spool fichier (drainé par l'app C27.5.d).
///   - Pair / Revoke → erreur tant que l'app desktop n'est pas joignable
///     (C27.5.b/c côté Tauri).
///   - Ping → toujours OK pour signaler que le bridge est installé.
pub fn handle_request(req: BridgeRequest) -> BridgeResponse {
    match req {
        BridgeRequest::Ping { req_id } => BridgeResponse {
            req_id,
            ok: true,
            pong: Some(true),
            error: None,
            secret: None,
            pairing_id: None,
            fingerprint: None,
        },
        BridgeRequest::Pair { req_id, .. } => BridgeResponse::err(
            req_id,
            "Pairing nécessite l'app Sobr.ia desktop ≥ 0.6.0 lancée — \
             non disponible (C27.5.b/c).",
        ),
        BridgeRequest::Estimate {
            req_id,
            secret,
            payload,
        } => {
            let envelope = serde_json::json!({
                "secret_hash": short_hash(&secret),
                "payload": payload,
                "received_at": chrono::Utc::now().to_rfc3339(),
            });
            match append_to_spool(&envelope) {
                Ok(()) => BridgeResponse::ok(req_id),
                Err(e) => BridgeResponse::err(req_id, format!("spool write: {e}")),
            }
        },
        BridgeRequest::Revoke { req_id, .. } => BridgeResponse::err(
            req_id,
            "Revoke nécessite l'app Sobr.ia desktop joignable — \
             effacement local côté extension uniquement.",
        ),
    }
}

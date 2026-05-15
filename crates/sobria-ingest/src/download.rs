//! Téléchargement HTTP streaming avec vérification SHA-256 et retries.
//!
//! # Garanties
//!
//! - Le hash est vérifié **avant** de renommer le fichier `.partial` en final.
//! - Sur erreur transiente (timeout, connexion coupée, 5xx), retry exponentiel.
//! - Si `expected_sha256` est fourni et qu'un fichier existe déjà avec le bon
//!   hash, aucune requête HTTP n'est émise ([`DownloadStatus::CachedHit`]).
//!
//! # Non-couvertures (v1.0)
//!
//! - Pas de reprise via `Range` (ajouté ultérieurement si nécessaire).
//! - Pas de support multi-connexions parallèles.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{
    error::{IngestError, IngestResult},
    hash,
};

/// Nombre maximum de tentatives sur erreurs transientes.
const DEFAULT_MAX_RETRIES: u32 = 3;
/// Délai de base du backoff exponentiel (ms).
const RETRY_BASE_MS: u64 = 500;
/// Timeout de la requête HTTP (par défaut).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Headers HTTP conservés dans le manifest pour traçabilité.
const TRACKED_HEADERS: &[&str] = &["etag", "last-modified", "content-type", "content-length"];

/// Résultat d'un téléchargement.
#[derive(Debug, Clone)]
pub struct DownloadOutcome {
    /// Taille finale du fichier en octets.
    pub bytes: u64,
    /// SHA-256 hexadécimal sur 64 caractères minuscules.
    pub sha256: String,
    /// Statut du téléchargement.
    pub status: DownloadStatus,
    /// Headers HTTP utiles capturés (etag, last-modified, content-*).
    pub headers: BTreeMap<String, String>,
}

/// Statut d'un téléchargement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Téléchargement effectif (réseau utilisé).
    Downloaded,
    /// Fichier déjà présent avec le hash attendu — aucune requête HTTP.
    CachedHit,
}

/// Téléchargeur réutilisable.
///
/// Conserve un `reqwest::Client` (pool de connexions, TLS).
pub struct Downloader {
    client: reqwest::Client,
    max_retries: u32,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    /// Crée un téléchargeur avec les valeurs par défaut.
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(concat!("sobria-ingest/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("client reqwest construit avec valeurs par défaut valides");
        Self {
            client,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Crée un téléchargeur avec un client `reqwest` custom (tests, proxy…).
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Définit le nombre max de tentatives.
    #[must_use]
    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Télécharge `url` vers `dest`, vérifie le hash, et renvoie un [`DownloadOutcome`].
    ///
    /// Si `expected_sha256` est fourni et que `dest` existe déjà avec ce hash,
    /// aucun appel réseau n'est émis.
    pub async fn fetch_to_file(
        &self,
        url: &str,
        dest: &Path,
        expected_sha256: Option<&str>,
    ) -> IngestResult<DownloadOutcome> {
        // 1. Cached hit possible ?
        if let Some(expected) = expected_sha256 {
            if tokio::fs::try_exists(dest).await? && hash::verify_file(dest, expected).await? {
                let meta = tokio::fs::metadata(dest).await?;
                tracing::info!(%url, ?dest, "cached hit (hash OK), pas de requête réseau");
                return Ok(DownloadOutcome {
                    bytes: meta.len(),
                    sha256: expected.to_ascii_lowercase(),
                    status: DownloadStatus::CachedHit,
                    headers: BTreeMap::new(),
                });
            }
        }

        // 2. Boucle de retry sur erreurs transientes.
        let partial = partial_path(dest);
        let mut attempt: u32 = 0;
        loop {
            match self.download_once(url, &partial).await {
                Ok(headers) => {
                    // 3. Vérification du hash avant le renommage final.
                    let computed = hash::sha256_file(&partial).await?;
                    if let Some(exp) = expected_sha256 {
                        if !computed.eq_ignore_ascii_case(exp) {
                            // Hash invalide : on détruit le partiel pour éviter
                            // de re-servir un fichier corrompu en cached hit.
                            let _ = tokio::fs::remove_file(&partial).await;
                            return Err(IngestError::HashMismatch {
                                expected: exp.to_string(),
                                actual: computed,
                            });
                        }
                    }
                    let bytes = tokio::fs::metadata(&partial).await?.len();
                    if let Some(parent) = dest.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    tokio::fs::rename(&partial, dest).await?;
                    tracing::info!(%url, ?dest, bytes, "téléchargement terminé");
                    return Ok(DownloadOutcome {
                        bytes,
                        sha256: computed,
                        status: DownloadStatus::Downloaded,
                        headers,
                    });
                },
                Err(e) if is_transient(&e) && attempt < self.max_retries => {
                    attempt += 1;
                    let delay = Duration::from_millis(RETRY_BASE_MS * (1u64 << attempt));
                    tracing::warn!(%url, attempt, ?delay, error = %e, "retry transient");
                    tokio::time::sleep(delay).await;
                },
                Err(e) => {
                    let _ = tokio::fs::remove_file(&partial).await;
                    return Err(e);
                },
            }
        }
    }

    /// Une tentative unique : ouvre la connexion, stream vers le partiel,
    /// retourne les headers capturés. Le hash sera vérifié par l'appelant.
    async fn download_once(
        &self,
        url: &str,
        partial: &Path,
    ) -> IngestResult<BTreeMap<String, String>> {
        if let Some(parent) = partial.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        // On écrase systématiquement le partiel — pas de resume v1.0.
        let mut file = tokio::fs::File::create(partial).await?;

        // `error_for_status()` convertit un statut 4xx/5xx en erreur,
        // automatiquement remontée en `IngestError::Http` via `From`.
        let response = self.client.get(url).send().await?.error_for_status()?;
        let captured = capture_headers(response.headers());

        let mut stream = response.bytes_stream();
        let mut total = 0u64;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes).await?;
            total += bytes.len() as u64;
        }
        file.flush().await?;
        tracing::debug!(%url, total_bytes = total, "stream terminé");
        Ok(captured)
    }
}

/// Détermine si une erreur mérite un retry.
fn is_transient(err: &IngestError) -> bool {
    match err {
        IngestError::Http(e) => {
            e.is_timeout() || e.is_connect() || matches!(e.status(), Some(s) if s.is_server_error())
        },
        // Une erreur d'I/O fugace peut bénéficier d'un retry.
        IngestError::Io(_) => true,
        _ => false,
    }
}

/// Construit le chemin du fichier partiel : `dest` + suffixe `.partial`.
fn partial_path(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(".partial");
    PathBuf::from(s)
}

/// Extrait les headers utiles du Response.
fn capture_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for name in TRACKED_HEADERS {
        if let Some(v) = headers.get(*name) {
            if let Ok(s) = v.to_str() {
                out.insert((*name).to_string(), s.to_string());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const HELLO: &[u8] = b"hello world";
    /// SHA-256 de "hello world".
    const HELLO_SHA256: &str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

    #[tokio::test]
    async fn downloads_and_verifies_hash() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/file"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(HELLO))
            .mount(&server)
            .await;
        let url = format!("{}/file", server.uri());

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");

        let dl = Downloader::new();
        let out = dl
            .fetch_to_file(&url, &dest, Some(HELLO_SHA256))
            .await
            .unwrap();
        assert_eq!(out.bytes, HELLO.len() as u64);
        assert_eq!(out.sha256, HELLO_SHA256);
        assert_eq!(out.status, DownloadStatus::Downloaded);
        let contents = tokio::fs::read(&dest).await.unwrap();
        assert_eq!(contents, HELLO);
    }

    #[tokio::test]
    async fn hash_mismatch_yields_error_and_no_dest_file() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(HELLO))
            .mount(&server)
            .await;
        let url = format!("{}/file", server.uri());

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");

        let dl = Downloader::new();
        let wrong = "0".repeat(64);
        let res = dl.fetch_to_file(&url, &dest, Some(&wrong)).await;
        assert!(matches!(res, Err(IngestError::HashMismatch { .. })));
        assert!(!tokio::fs::try_exists(&dest).await.unwrap());
    }

    #[tokio::test]
    async fn cached_hit_avoids_network() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");
        tokio::fs::write(&dest, HELLO).await.unwrap();

        // Pas de mock serveur configuré : si la requête part, ça doit échouer.
        let dl = Downloader::new();
        let out = dl
            .fetch_to_file(
                "http://127.0.0.1:1/should-not-be-called",
                &dest,
                Some(HELLO_SHA256),
            )
            .await
            .unwrap();
        assert_eq!(out.status, DownloadStatus::CachedHit);
        assert_eq!(out.bytes, HELLO.len() as u64);
    }

    #[tokio::test]
    async fn retries_on_5xx_then_succeeds() {
        let server = MockServer::start().await;
        // Up_to(1) puis un second mock pour la suite — pattern wiremock.
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(HELLO))
            .mount(&server)
            .await;
        let url = format!("{}/file", server.uri());

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");

        let dl = Downloader::new().with_max_retries(3);
        let out = dl
            .fetch_to_file(&url, &dest, Some(HELLO_SHA256))
            .await
            .unwrap();
        assert_eq!(out.bytes, HELLO.len() as u64);
    }

    #[tokio::test]
    async fn no_retry_on_4xx() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let url = format!("{}/file", server.uri());

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");

        let dl = Downloader::new();
        let res = dl.fetch_to_file(&url, &dest, None).await;
        assert!(matches!(res, Err(IngestError::Http(_))));
        assert!(!tokio::fs::try_exists(&dest).await.unwrap());
    }

    #[tokio::test]
    async fn captures_tracked_headers() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("ETag", "\"abc123\"")
                    .insert_header("Content-Type", "application/octet-stream")
                    .set_body_bytes(HELLO),
            )
            .mount(&server)
            .await;
        let url = format!("{}/file", server.uri());

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.bin");

        let dl = Downloader::new();
        let out = dl.fetch_to_file(&url, &dest, None).await.unwrap();
        assert_eq!(
            out.headers.get("etag").map(String::as_str),
            Some("\"abc123\"")
        );
        assert_eq!(
            out.headers.get("content-type").map(String::as_str),
            Some("application/octet-stream"),
        );
    }

    #[test]
    fn partial_path_appends_suffix() {
        let p = partial_path(Path::new("/tmp/foo.parquet"));
        assert_eq!(p, Path::new("/tmp/foo.parquet.partial"));
    }
}

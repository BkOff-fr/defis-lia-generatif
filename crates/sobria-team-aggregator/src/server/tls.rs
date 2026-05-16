//! Construction d'une [`rustls::ServerConfig`] à partir des PEM `cert.pem`
//! + `key.pem` générés par la commande `init`.
//!
//! Provider crypto : `ring` (installé une seule fois via [`ensure_crypto_provider`]).
//! Pas de support TLS 1.2 désactivé : on garde 1.2 + 1.3 pour compat extension /
//! reverse-proxy.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::Once;

use rustls::ServerConfig;

use crate::error::{AggregatorError, AggregatorResult};

static INSTALL_CRYPTO_PROVIDER: Once = Once::new();

/// Installe le provider `ring` comme provider par défaut de `rustls` —
/// idempotent (utilise `std::sync::Once`).
pub fn ensure_crypto_provider() {
    INSTALL_CRYPTO_PROVIDER.call_once(|| {
        // `install_default` ne peut être appelé qu'une fois par process,
        // sinon il renvoie Err. `Once` empêche le double appel.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Charge cert + clé depuis les PEM et construit une `ServerConfig` rustls.
pub fn load_server_config(cert_path: &Path, key_path: &Path) -> AggregatorResult<ServerConfig> {
    ensure_crypto_provider();

    let cert_chain = {
        let mut reader = BufReader::new(File::open(cert_path)?);
        rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AggregatorError::Tls(format!("parse {}: {e}", cert_path.display())))?
    };
    if cert_chain.is_empty() {
        return Err(AggregatorError::Tls(format!(
            "aucun certificat trouvé dans {}",
            cert_path.display()
        )));
    }

    let private_key = {
        let mut reader = BufReader::new(File::open(key_path)?);
        rustls_pemfile::private_key(&mut reader)?
            .ok_or_else(|| AggregatorError::Tls(format!("clé absente de {}", key_path.display())))?
    };

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .map_err(|e| AggregatorError::Tls(format!("ServerConfig: {e}")))?;

    Ok(config)
}

/// Variante prête à brancher dans `axum_server::tls_rustls::RustlsConfig`.
pub fn load_server_config_arc(
    cert_path: &Path,
    key_path: &Path,
) -> AggregatorResult<Arc<ServerConfig>> {
    Ok(Arc::new(load_server_config(cert_path, key_path)?))
}

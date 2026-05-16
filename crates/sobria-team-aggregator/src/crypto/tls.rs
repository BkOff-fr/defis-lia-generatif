//! Génération d'un certificat TLS auto-signé via `rcgen` (backend ring).
//!
//! Le CN par défaut est `localhost`, et les SANs incluent `localhost`,
//! `127.0.0.1`, `::1` + le hostname OS. Validité 10 ans (cf. brief C28.1).
//! Pas d'OpenSSL : `rcgen = { default-features = false, features = ["crypto", "ring", "pem"] }`.

use std::fs;
use std::net::IpAddr;
use std::path::Path;

use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};

use crate::error::{AggregatorError, AggregatorResult};

/// PEM bundle d'un certificat auto-signé.
#[derive(Debug)]
pub struct GeneratedCert {
    pub cert_pem: String,
    pub key_pem: String,
}

/// Génère un cert auto-signé pour `subject_alt_names` (le premier est aussi
/// utilisé comme CN). Validité 10 ans, ECDSA-P256.
pub fn generate_self_signed(
    subject_alt_names: &[String],
    hostname: Option<&str>,
) -> AggregatorResult<GeneratedCert> {
    let mut sans: Vec<SanType> = subject_alt_names
        .iter()
        .filter_map(|s| parse_san(s))
        .collect();

    if let Some(h) = hostname {
        if !subject_alt_names.iter().any(|s| s == h) {
            if let Some(san) = parse_san(h) {
                sans.push(san);
            }
        }
    }

    if sans.is_empty() {
        return Err(AggregatorError::Tls(
            "au moins un SAN requis pour générer un cert".into(),
        ));
    }

    let cn = hostname.unwrap_or("localhost").to_string();
    let mut params = CertificateParams::new(Vec::<String>::new())
        .map_err(|e| AggregatorError::Tls(format!("rcgen params: {e}")))?;
    params.subject_alt_names = sans;
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, cn);
    dn.push(DnType::OrganizationName, "Sobr.ia Team Aggregator");
    params.distinguished_name = dn;

    // Validité 10 ans.
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(365 * 10);

    let key_pair =
        KeyPair::generate().map_err(|e| AggregatorError::Tls(format!("keypair: {e}")))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| AggregatorError::Tls(format!("self_signed: {e}")))?;

    Ok(GeneratedCert {
        cert_pem: cert.pem(),
        key_pem: key_pair.serialize_pem(),
    })
}

/// Écrit le bundle PEM sur disque (cert.pem + key.pem).
///
/// Sur les systèmes Unix, on tente un `chmod 600` sur la clé privée. Sur
/// Windows, on s'en remet aux ACLs par défaut du data dir (documenté dans
/// `docs/operations/team-aggregator.md`).
pub fn write_cert_files(
    cert_path: &Path,
    key_path: &Path,
    bundle: &GeneratedCert,
) -> AggregatorResult<()> {
    fs::write(cert_path, &bundle.cert_pem)?;
    fs::write(key_path, &bundle.key_pem)?;
    restrict_key_permissions(key_path)?;
    Ok(())
}

/// Empreinte SHA-256 hex du certificat PEM (à des fins d'affichage / pinning).
/// Calculé sur le PEM tel quel — c'est l'empreinte "transport" usuelle.
#[must_use]
pub fn cert_fingerprint_sha256(pem: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(pem.as_bytes());
    format!("{:x}", h.finalize())
}

/// Résultat d'une rotation TLS (C29.3).
#[derive(Debug)]
pub struct RegenOutcome {
    /// Chemin du backup de l'ancien cert (`cert.pem.bak.<unix_ts>`).
    pub cert_backup_path: std::path::PathBuf,
    /// Chemin du backup de l'ancienne clé (`key.pem.bak.<unix_ts>`).
    pub key_backup_path: std::path::PathBuf,
    /// Empreinte SHA-256 hex du nouveau cert PEM.
    pub new_fingerprint: String,
}

/// Régénère `cert.pem` + `key.pem` à partir du data dir actuel.
///
/// 1. Sauvegarde l'ancien cert et l'ancienne clé sous `*.bak.<unix_ts>`.
/// 2. Génère un nouveau cert auto-signé (mêmes SANs `localhost`/`127.0.0.1`/
///    `::1` + hostname OS), CN identique (`hostname` ou `localhost`).
/// 3. Écrit les nouveaux fichiers via [`write_cert_files`].
/// 4. Retourne l'empreinte SHA-256 du nouveau cert (à communiquer aux clients).
pub fn regen_self_signed(cert_path: &Path, key_path: &Path) -> AggregatorResult<RegenOutcome> {
    if !cert_path.exists() || !key_path.exists() {
        return Err(AggregatorError::Tls(format!(
            "cert ou clé absents — exécuter `init` d'abord ({} / {})",
            cert_path.display(),
            key_path.display(),
        )));
    }

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let cert_backup = cert_path.with_extension(format!("pem.bak.{ts}"));
    let key_backup = key_path.with_extension(format!("pem.bak.{ts}"));
    fs::rename(cert_path, &cert_backup)?;
    fs::rename(key_path, &key_backup)?;

    let hostname = hostname::get()
        .ok()
        .and_then(|os| os.into_string().ok())
        .filter(|s| !s.is_empty());
    let sans = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    let bundle = generate_self_signed(&sans, hostname.as_deref())?;
    write_cert_files(cert_path, key_path, &bundle)?;
    let fp = cert_fingerprint_sha256(&bundle.cert_pem);

    Ok(RegenOutcome {
        cert_backup_path: cert_backup,
        key_backup_path: key_backup,
        new_fingerprint: fp,
    })
}

#[cfg(unix)]
fn restrict_key_permissions(key_path: &Path) -> AggregatorResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(key_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(key_path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_key_permissions(_key_path: &Path) -> AggregatorResult<()> {
    Ok(())
}

fn parse_san(input: &str) -> Option<SanType> {
    if let Ok(ip) = input.parse::<IpAddr>() {
        return Some(SanType::IpAddress(ip));
    }
    let ia5 = rcgen::Ia5String::try_from(input.to_string()).ok()?;
    Some(SanType::DnsName(ia5))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generates_cert_with_localhost_and_127_0_0_1() {
        let bundle =
            generate_self_signed(&["localhost".into(), "127.0.0.1".into()], Some("localhost"))
                .unwrap();
        assert!(bundle.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(bundle.key_pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn rejects_when_no_sans_provided() {
        let err = generate_self_signed(&[], None).unwrap_err();
        assert!(matches!(err, AggregatorError::Tls(_)));
    }

    #[test]
    fn write_cert_files_creates_both_files() {
        let dir = tempdir().unwrap();
        let cert = dir.path().join("cert.pem");
        let key = dir.path().join("key.pem");
        let bundle =
            generate_self_signed(&["localhost".into(), "127.0.0.1".into()], Some("localhost"))
                .unwrap();
        write_cert_files(&cert, &key, &bundle).unwrap();
        assert!(cert.exists());
        assert!(key.exists());
        let cert_text = std::fs::read_to_string(&cert).unwrap();
        assert!(cert_text.contains("BEGIN CERTIFICATE"));
    }

    #[test]
    fn fingerprint_is_64_hex() {
        let pem = "-----BEGIN CERTIFICATE-----\nfake\n-----END CERTIFICATE-----\n";
        let fp = cert_fingerprint_sha256(pem);
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn regen_self_signed_backs_up_and_replaces() {
        let dir = tempdir().unwrap();
        let cert = dir.path().join("cert.pem");
        let key = dir.path().join("key.pem");
        let bundle1 =
            generate_self_signed(&["localhost".into(), "127.0.0.1".into()], Some("localhost"))
                .unwrap();
        write_cert_files(&cert, &key, &bundle1).unwrap();
        let cert_v1 = std::fs::read_to_string(&cert).unwrap();

        let out = regen_self_signed(&cert, &key).unwrap();
        assert!(out.cert_backup_path.exists(), "backup cert manquant");
        assert!(out.key_backup_path.exists(), "backup key manquant");
        // Backups contiennent l'ancien contenu
        let cert_bak = std::fs::read_to_string(&out.cert_backup_path).unwrap();
        assert_eq!(cert_bak, cert_v1);
        // Le nouveau cert est différent et a une empreinte unique
        let cert_v2 = std::fs::read_to_string(&cert).unwrap();
        assert_ne!(cert_v1, cert_v2);
        assert_eq!(out.new_fingerprint.len(), 64);
        assert_eq!(out.new_fingerprint, cert_fingerprint_sha256(&cert_v2));
    }

    #[test]
    fn regen_self_signed_fails_when_files_absent() {
        let dir = tempdir().unwrap();
        let cert = dir.path().join("cert.pem");
        let key = dir.path().join("key.pem");
        let err = regen_self_signed(&cert, &key);
        assert!(err.is_err());
    }
}

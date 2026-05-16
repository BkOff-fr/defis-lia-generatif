//! Commande `sobria-team-aggregator init` — prépare un data dir vierge :
//!
//! 1. SQLite `team.sqlite` + migration v1.
//! 2. JWT signing key (32 octets OS RNG, hex) persistée dans `config`.
//! 3. Certificat TLS auto-signé (rcgen + ring) + clé privée (chmod 600 sur Unix).
//! 4. Admin initial (Argon2id PHC, username paramétrable).
//!
//! Idempotente uniquement via `--force` : refuse de réinitialiser un data dir
//! déjà peuplé pour ne pas effacer des données équipe par erreur.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::config::DataPaths;
use crate::crypto::{password, secret, tls};
use crate::error::AggregatorError;
use crate::storage::{admins, schema, Storage};

/// Clés du KV `config` utilisées par `init`.
pub const CFG_JWT_SIGNING_KEY: &str = "jwt_signing_key";
pub const CFG_SCHEMA_INSTALLED_AT: &str = "schema_installed_at";
pub const CFG_SCHEMA_VERSION: &str = "schema_version";

/// Résultat retourné à la CLI pour affichage post-init.
#[derive(Debug)]
pub struct InitOutcome {
    pub admin_id: String,
    pub admin_username: String,
    pub cert_path: std::path::PathBuf,
    pub key_path: std::path::PathBuf,
    pub db_path: std::path::PathBuf,
    pub hostname: Option<String>,
}

/// Exécute l'init complète.
///
/// `admin_password` est consommé en clair une seule fois pour calculer
/// le PHC Argon2id ; il ne traîne pas en mémoire au-delà.
pub fn run(
    paths: &DataPaths,
    admin_username: &str,
    admin_password: &str,
    force: bool,
) -> Result<InitOutcome> {
    if paths.already_initialized() && !force {
        return Err(
            AggregatorError::AlreadyInitialized(paths.as_path().display().to_string()).into(),
        );
    }

    paths.ensure_dir().context("ensure data dir")?;

    // 1. DB + schema v1
    let storage = Storage::open(&paths.db()).context("open team.sqlite")?;

    // 2. JWT signing key + métadonnées
    let jwt_key = secret::jwt_signing_key();
    storage.set_config(CFG_JWT_SIGNING_KEY, &jwt_key)?;
    storage.set_config(CFG_SCHEMA_INSTALLED_AT, &Utc::now().to_rfc3339())?;
    storage.set_config(CFG_SCHEMA_VERSION, &schema::SCHEMA_VERSION.to_string())?;

    // 3. Certificat auto-signé (SANs : localhost, 127.0.0.1, ::1, hostname OS)
    let hostname = hostname::get()
        .ok()
        .and_then(|os| os.into_string().ok())
        .filter(|s| !s.is_empty());
    let sans = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    let bundle = tls::generate_self_signed(&sans, hostname.as_deref())
        .context("generate self-signed cert")?;
    tls::write_cert_files(&paths.cert(), &paths.key(), &bundle)
        .context("write cert.pem / key.pem")?;

    // 4. Admin initial
    let admin_id = Ulid::new().to_string();
    let admin_hash = password::hash_password(admin_password).context("hash admin password")?;
    admins::insert(
        storage.connection(),
        &admin_id,
        admin_username,
        &admin_hash,
        Utc::now(),
    )
    .context("insert admin")?;

    Ok(InitOutcome {
        admin_id,
        admin_username: admin_username.to_string(),
        cert_path: paths.cert(),
        key_path: paths.key(),
        db_path: paths.db(),
        hostname,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_populates_data_dir() {
        let dir = tempdir().unwrap();
        let paths = DataPaths::new(dir.path());
        let outcome = run(&paths, "admin", "init-password", false).unwrap();

        assert!(paths.db().exists(), "team.sqlite manquant");
        assert!(paths.cert().exists(), "cert.pem manquant");
        assert!(paths.key().exists(), "key.pem manquant");
        assert_eq!(outcome.admin_username, "admin");
        assert!(!outcome.admin_id.is_empty());

        // L'admin est bien dans la base et le password vérifie.
        let storage = Storage::open(&paths.db()).unwrap();
        let row = admins::find_by_username(storage.connection(), "admin")
            .unwrap()
            .unwrap();
        assert!(password::verify_password(
            &row.password_hash,
            "init-password"
        ));
        assert!(!password::verify_password(&row.password_hash, "wrong"));

        // JWT signing key est posée.
        let jwt = storage.get_config(CFG_JWT_SIGNING_KEY).unwrap().unwrap();
        assert_eq!(jwt.len(), 64);
    }

    #[test]
    fn init_refuses_existing_data_dir_without_force() {
        let dir = tempdir().unwrap();
        let paths = DataPaths::new(dir.path());
        run(&paths, "admin", "p1", false).unwrap();
        let err = run(&paths, "admin", "p2", false).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("data dir déjà initialisé"), "msg = {msg}");
    }
}

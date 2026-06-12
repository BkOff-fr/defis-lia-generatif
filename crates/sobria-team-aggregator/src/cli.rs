//! Définition `clap` du binaire `sobria-team-aggregator`.
//!
//! Sous-commandes :
//!
//! - `init`              (C28.1) — prépare un data dir.
//! - `serve [--regen-cert]` (C28.1 + C29.3) — lance le serveur HTTPS,
//!   avec option de rotation du cert auto-signé avant bind.
//! - `code`              (C28.2) — crée / liste / révoque des enrollment codes.
//! - `admin {list, reset-password}` (C29.2) — gestion des comptes admin.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::commands;
use crate::config::{DataPaths, DEFAULT_BIND, DEFAULT_DATA_DIR, DEFAULT_PORT};

/// CLI racine `sobria-team-aggregator`.
#[derive(Debug, Parser)]
#[command(name = "sobria-team-aggregator", version, about, long_about = None)]
pub struct Cli {
    /// Data dir contenant `team.sqlite`, `cert.pem` et `key.pem`.
    #[arg(long, global = true, default_value = DEFAULT_DATA_DIR)]
    pub data_dir: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialise un nouveau data dir (DB + TLS cert + admin initial).
    Init {
        /// Nom de l'admin initial.
        #[arg(long, default_value = "admin")]
        admin_username: String,

        /// Mot de passe admin. Si absent : prompt interactif (sans écho).
        #[arg(long, env = "SOBRIA_TEAM_ADMIN_PASSWORD")]
        admin_password: Option<String>,

        /// Réinitialise un data dir existant (efface l'init précédent).
        #[arg(long)]
        force: bool,
    },

    /// Lance le serveur HTTPS.
    Serve {
        /// Adresse de bind (ex : `0.0.0.0`, `127.0.0.1`).
        #[arg(long, default_value = DEFAULT_BIND)]
        bind: String,

        /// Port HTTPS.
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,

        /// Régénère le certificat TLS auto-signé avant de démarrer (C29.3).
        ///
        /// L'ancien cert/key est sauvegardé en `*.pem.bak.<unix_ts>`. Les
        /// clients qui avaient accepté l'ancien fingerprint devront accepter
        /// le nouveau.
        #[arg(long)]
        regen_cert: bool,
    },

    /// Gère les enrollment codes distribués aux employés.
    Code {
        #[command(subcommand)]
        action: CodeAction,
    },

    /// Gère les comptes administrateurs (reset-password / list).
    Admin {
        #[command(subcommand)]
        action: AdminAction,
    },

    /// Lit/écrit la configuration runtime (k-anonymat, rétention — ADR-0015).
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Liste les clés runtime autorisées (valeur effective, défaut, plancher).
    List,
    /// Lit la valeur effective d'une clé.
    Get {
        /// Nom de la clé (cf. `config list`).
        key: String,
    },
    /// Écrit une clé (entier validé contre le plancher, ou politique).
    Set {
        /// Nom de la clé (cf. `config list`).
        key: String,
        /// Nouvelle valeur (entier, ou `anonymous|opt_in|identified`).
        value: String,
        /// Attestation OBLIGATOIRE pour `visibility_policy identified` :
        /// déclare que le CSE (L2312-38) et les salariés (L1222-4) ont
        /// été informés — cf. ADR-0016.
        #[arg(long)]
        attest: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdminAction {
    /// Liste tous les admins du data dir.
    List,
    /// Réinitialise le mot de passe d'un admin (Argon2id + révocation tokens).
    ResetPassword {
        /// Username de l'admin à modifier.
        username: String,
        /// Nouveau mot de passe. Si absent : prompt interactif.
        #[arg(long, env = "SOBRIA_TEAM_NEW_PASSWORD")]
        password: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum CodeAction {
    /// Crée N nouveaux codes 12 chiffres (affichés en clair UNE seule fois).
    Create {
        /// Nombre de codes à créer.
        count: u32,
        /// TTL des codes en jours (défaut : 7).
        #[arg(long, default_value_t = 7)]
        ttl_days: i64,
        /// Username admin créateur (défaut : `admin`).
        #[arg(long, default_value = "admin")]
        admin: String,
    },
    /// Liste tous les codes (id, créé par, état).
    List,
    /// Révoque un code par id (ULID).
    Revoke {
        /// Identifiant du code (ULID).
        id: String,
    },
}

/// Entrée du binaire : parse les args et dispatche.
pub fn run() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let paths = DataPaths::new(cli.data_dir);

    match cli.command {
        Command::Init {
            admin_username,
            admin_password,
            force,
        } => run_init(&paths, &admin_username, admin_password, force),
        Command::Serve {
            bind,
            port,
            regen_cert,
        } => run_serve(&paths, &bind, port, regen_cert),
        Command::Code { action } => run_code(&paths, action),
        Command::Admin { action } => run_admin(&paths, action),
        Command::Config { action } => run_config(&paths, action),
    }
}

fn run_config(paths: &DataPaths, action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::List => {
            println!(
                "{:<18} {:>10} {:>8} {:>9}  description",
                "clé", "effectif", "défaut", "plancher"
            );
            for spec in commands::config::RUNTIME_KEYS {
                let value = commands::config::get(paths, spec.key)?;
                println!(
                    "{:<18} {:>10} {:>8} {:>9}  {}",
                    spec.key, value, spec.default, spec.floor, spec.description
                );
            }
            let (pol, attest) = commands::config::get_policy(paths)?;
            println!(
                "{:<18} {:>10} {:>8} {:>9}  Politique de visibilité (ADR-0016){}",
                "visibility_policy",
                pol,
                "opt_in",
                "—",
                attest
                    .map(|a| format!(" — attestation du {}", a.set_at))
                    .unwrap_or_default()
            );
            Ok(())
        },
        ConfigAction::Get { key } => {
            if key == commands::config::POLICY_KEY_NAME {
                let (pol, _) = commands::config::get_policy(paths)?;
                println!("{pol}");
            } else {
                println!("{}", commands::config::get(paths, &key)?);
            }
            Ok(())
        },
        ConfigAction::Set { key, value, attest } => {
            if key == commands::config::POLICY_KEY_NAME {
                let pol = commands::config::set_policy(paths, &value, attest.as_deref())?;
                println!("visibility_policy = {pol}");
            } else {
                let v = commands::config::set(paths, &key, &value)?;
                println!("{key} = {v}");
            }
            Ok(())
        },
    }
}

fn run_admin(paths: &DataPaths, action: AdminAction) -> Result<()> {
    match action {
        AdminAction::List => {
            let admins = commands::admin::list_admins(paths)?;
            if admins.is_empty() {
                println!("Aucun admin en base.");
                return Ok(());
            }
            println!(
                "{:<28} {:<22} {:<22} last_login_at",
                "id", "username", "created_at"
            );
            for a in admins {
                let last = a
                    .last_login_at
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "—".into());
                println!(
                    "{:<28} {:<22} {:<22} {}",
                    a.id,
                    a.username,
                    a.created_at.format("%Y-%m-%d %H:%M"),
                    last,
                );
            }
            Ok(())
        },
        AdminAction::ResetPassword { username, password } => {
            let new_pw = match password {
                Some(p) => p,
                None => prompt_password().context("lire le nouveau password")?,
            };
            let out = commands::admin::reset_password(paths, &username, &new_pw)?;
            println!(
                "Mot de passe de {} réinitialisé. {} token(s) révoqué(s).",
                out.username, out.revoked_tokens
            );
            Ok(())
        },
    }
}

fn run_code(paths: &DataPaths, action: CodeAction) -> Result<()> {
    match action {
        CodeAction::Create {
            count,
            ttl_days,
            admin,
        } => {
            let codes = commands::code::create_batch(paths, count, ttl_days, &admin)?;
            println!(
                "\n{} code(s) enrôlement généré(s) (admin={admin}, TTL={ttl_days}j) :\n",
                codes.len()
            );
            for (i, c) in codes.iter().enumerate() {
                println!(
                    "  {:>3}. {}  (id: {}, expire le {})",
                    i + 1,
                    c.code,
                    c.id,
                    c.expires_at.format("%Y-%m-%d %H:%M UTC"),
                );
            }
            println!(
                "\n⚠️  Conservez ces codes — ils ne seront PLUS affichés (hash Argon2id en base)."
            );
            println!("    Distribuez-les à vos employés par un canal sûr.");
            Ok(())
        },
        CodeAction::List => {
            let rows = commands::code::list_all(paths)?;
            if rows.is_empty() {
                println!("Aucun enrollment code en base.");
                return Ok(());
            }
            println!("{:<28} {:<22} {:<20} créé par", "id", "expires_at", "état");
            let now = chrono::Utc::now();
            for r in &rows {
                let etat = if r.revoked_at.is_some() {
                    "révoqué"
                } else if r.used_at.is_some() {
                    "utilisé"
                } else if r.expires_at <= now {
                    "expiré"
                } else {
                    "actif"
                };
                println!(
                    "{:<28} {:<22} {:<20} {}",
                    r.id,
                    r.expires_at.format("%Y-%m-%d %H:%M"),
                    etat,
                    r.created_by,
                );
            }
            Ok(())
        },
        CodeAction::Revoke { id } => {
            let ok = commands::code::revoke(paths, &id)?;
            if ok {
                println!("Code {id} révoqué.");
            } else {
                println!("Code {id} introuvable ou déjà révoqué.");
            }
            Ok(())
        },
    }
}

fn run_init(
    paths: &DataPaths,
    admin_username: &str,
    admin_password: Option<String>,
    force: bool,
) -> Result<()> {
    let password = match admin_password {
        Some(p) => p,
        None => prompt_password().context("lire le password admin")?,
    };
    if password.len() < 8 {
        anyhow::bail!("le mot de passe admin doit faire au moins 8 caractères");
    }

    let outcome = commands::init::run(paths, admin_username, &password, force)?;
    print_init_summary(&outcome);
    Ok(())
}

fn run_serve(paths: &DataPaths, bind: &str, port: u16, regen_cert: bool) -> Result<()> {
    if regen_cert {
        let out = crate::crypto::tls::regen_self_signed(&paths.cert(), &paths.key())
            .context("régénérer le certificat TLS")?;
        println!(
            "Nouveau certificat TLS généré.\n  ancien cert -> {}\n  ancienne clé -> {}\n  empreinte SHA-256 : {}\n",
            out.cert_backup_path.display(),
            out.key_backup_path.display(),
            out.new_fingerprint
        );
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("démarrer le runtime tokio")?;
    rt.block_on(commands::serve::run(paths, bind, port))
}

fn prompt_password() -> Result<String> {
    let pw = rpassword::prompt_password("Mot de passe admin : ")?;
    let confirm = rpassword::prompt_password("Confirmer            : ")?;
    if pw != confirm {
        anyhow::bail!("les mots de passe ne correspondent pas");
    }
    Ok(pw)
}

fn print_init_summary(outcome: &commands::init::InitOutcome) {
    println!("\nsobria-team-aggregator — data dir initialisé.\n");
    println!(
        "  Admin   : {} (id: {})",
        outcome.admin_username, outcome.admin_id
    );
    println!("  DB      : {}", outcome.db_path.display());
    println!("  Cert    : {}", outcome.cert_path.display());
    println!("  Key     : {}", outcome.key_path.display());
    println!();
    println!("URLs d'accès (HTTPS) :");
    println!("  https://localhost:{DEFAULT_PORT}/");
    if let Some(h) = &outcome.hostname {
        println!("  https://{h}:{DEFAULT_PORT}/");
    }
    println!();
    println!("Le certificat est auto-signé : votre navigateur affichera un avertissement");
    println!("au premier accès — c'est attendu. Voir docs/operations/team-aggregator.md.");
    println!();
    println!("Lancer le serveur : sobria-team-aggregator serve");
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("sobria_team_aggregator=info,tower_http=info"));
    let _ = fmt().with_env_filter(filter).try_init();
}

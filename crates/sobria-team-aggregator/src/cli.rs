//! Définition `clap` du binaire `sobria-team-aggregator`.
//!
//! Pour C28.1 on expose deux sous-commandes : `init` (préparer un data dir)
//! et `serve` (lancer le serveur HTTPS). Les commandes `code`, `user` et
//! `admin` arrivent avec C28.2 / C28.3.

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
        Command::Serve { bind, port } => run_serve(&paths, &bind, port),
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

fn run_serve(paths: &DataPaths, bind: &str, port: u16) -> Result<()> {
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

//! Binaire CLI `sobria-ingest` — voir CLAUDE.md §10 et ADR-0009.
//!
//! Sous-commandes :
//! - `pipeline run [--source <id>] [--incremental]` — pipeline complet
//! - `copper [--all|--source <id>]` — couche brute uniquement
//! - `silver [--all|--source <id>]` — promotion Copper → Silver
//! - `gold` — construction du Gold final
//! - `validate` — vérifie l'intégrité du lineage

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "sobria-ingest", version, about = "Pipeline médaillon Sobr.ia")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Pipeline complet Copper → Silver → Gold.
    Pipeline(PipelineArgs),
    /// Ingestion brute uniquement.
    Copper(SourceArgs),
    /// Promotion Copper → Silver.
    Silver(SourceArgs),
    /// Construction du Gold final.
    Gold,
    /// Vérification d'intégrité.
    Validate,
}

#[derive(clap::Args, Debug)]
struct PipelineArgs {
    /// Sous-commande effective.
    #[command(subcommand)]
    sub: PipelineSubcommand,
}

#[derive(Subcommand, Debug)]
enum PipelineSubcommand {
    /// Exécute le pipeline.
    Run(PipelineRunArgs),
}

#[derive(clap::Args, Debug)]
struct PipelineRunArgs {
    /// Limiter à une source (id).
    #[arg(long)]
    source: Option<String>,
    /// Mode incrémental — ne ré-ingère que ce qui a changé.
    #[arg(long)]
    incremental: bool,
}

#[derive(clap::Args, Debug)]
struct SourceArgs {
    /// Toutes les sources.
    #[arg(long, conflicts_with = "source")]
    all: bool,
    /// Une source en particulier (id).
    #[arg(long)]
    source: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match cli.command {
        Command::Pipeline(args) => match args.sub {
            PipelineSubcommand::Run(run) => {
                tracing::info!(?run, "lancement pipeline (stub)");
                // TODO(sobria-002): câbler LayerRegistry::default().run_full_pipeline()
                Ok(())
            },
        },
        Command::Copper(args) => {
            tracing::info!(?args, "ingestion copper (stub)");
            Ok(())
        },
        Command::Silver(args) => {
            tracing::info!(?args, "promotion silver (stub)");
            Ok(())
        },
        Command::Gold => {
            tracing::info!("construction gold (stub)");
            Ok(())
        },
        Command::Validate => {
            tracing::info!("validation (stub)");
            Ok(())
        },
    }
}

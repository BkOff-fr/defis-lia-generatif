//! Binaire CLI `sobria-ingest` — voir CLAUDE.md §10 et ADR-0009.
//!
//! Sous-commandes :
//! - `pipeline run [--source <id>] [--incremental]` — pipeline complet
//! - `copper [--all|--source <id>]` — couche brute uniquement
//! - `silver [--all|--source <id>]` — promotion Copper → Silver
//! - `gold` — construction du Gold final
//! - `validate` — vérifie l'intégrité du lineage
//! - `fetch <target>` — pull d'un dataset public ciblé (auto-download)

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use sobria_ingest::sources::territoire_fr;

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
    /// Téléchargement automatique d'un dataset public (Etalab 2.0).
    Fetch(FetchArgs),
    /// Recherche un dataset dans le catalogue ODRÉ par mot-clé.
    Discover(DiscoverArgs),
}

#[derive(clap::Args, Debug)]
struct DiscoverArgs {
    /// Mot-clé(s) à chercher (full-text sur titre/description/tags).
    keyword: String,
    /// Nombre de résultats max (≤ 50).
    #[arg(long, default_value_t = 20)]
    limit: u32,
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

#[derive(clap::Args, Debug)]
struct FetchArgs {
    /// Dataset cible.
    #[arg(value_enum)]
    target: FetchTarget,
    /// Nombre maximum d'enregistrements (pour les datasets paginés).
    #[arg(long, default_value_t = 200)]
    limit: u32,
    /// Année (pour les datasets annuels comme `rte-mix`).
    #[arg(long, default_value_t = 2023)]
    year: u32,
    /// Chemin de sortie alternatif (défaut : `crates/sobria-geoloc/data/`).
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FetchTarget {
    /// Top sites industriels FR — ODRÉ consommation IRIS.
    TerritoireFr,
    /// Mix électrique national annuel — RTE eco2mix.
    RteMix,
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
        Command::Fetch(args) => run_fetch(args).await,
        Command::Discover(args) => run_discover(args).await,
    }
}

async fn run_discover(args: DiscoverArgs) -> Result<()> {
    let matches = territoire_fr::discover_datasets(&args.keyword, args.limit).await?;
    if matches.is_empty() {
        println!("Aucun dataset trouvé pour '{}'", args.keyword);
        return Ok(());
    }
    println!(
        "{} dataset(s) trouvé(s) pour '{}' :\n",
        matches.len(),
        args.keyword
    );
    for (i, m) in matches.iter().enumerate() {
        println!("{:>2}. {}", i + 1, m.title);
        println!("    slug      : {}", m.dataset_id);
        if let Some(p) = &m.publisher {
            println!("    publisher : {p}");
        }
        if let Some(d) = &m.description {
            println!("    desc      : {d}");
        }
        println!("    explore   : {}", m.explore_url);
        println!();
    }
    Ok(())
}

async fn run_fetch(args: FetchArgs) -> Result<()> {
    let default_data_dir = std::env::current_dir()?
        .join("crates")
        .join("sobria-geoloc")
        .join("data");
    match args.target {
        FetchTarget::TerritoireFr => {
            let out = args.out.unwrap_or_else(|| default_data_dir.join("territoire_fr.json"));
            tracing::info!(limit = args.limit, ?out, "fetch territoire-fr");
            let artifact = territoire_fr::fetch_industrial_sites(args.limit).await?;
            territoire_fr::write_artifact_json(&artifact, &out)?;
            tracing::info!(
                count = artifact.industrial_sites.len(),
                path = %out.display(),
                "territoire-fr écrit"
            );
        },
        FetchTarget::RteMix => {
            let out = args.out.unwrap_or_else(|| default_data_dir.join("rte_mix_fr.json"));
            tracing::info!(year = args.year, ?out, "fetch rte-mix");
            let artifact = territoire_fr::fetch_rte_mix(args.year).await?;
            territoire_fr::write_artifact_json(&artifact, &out)?;
            tracing::info!(
                total_twh = artifact.mix.total_production_twh,
                path = %out.display(),
                "rte-mix écrit"
            );
        },
    }
    Ok(())
}

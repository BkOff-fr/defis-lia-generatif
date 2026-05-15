//! Binaire CLI `sobria-ingest` — voir CLAUDE.md §10 et ADR-0009.
//!
//! Sous-commandes :
//! - `pipeline run [--source <id>] [--incremental]` — pipeline complet
//! - `copper [--all|--source <id>]` — couche brute uniquement
//! - `silver [--all|--source <id>]` — promotion Copper → Silver
//! - `gold` — construction du Gold final
//! - `validate` — vérifie l'intégrité du lineage (recalcule SHA-256 des
//!   manifests Copper)
//! - `fetch <target>` — pull d'un dataset public ciblé (auto-download
//!   utilisé par `sobria-geoloc`, indépendant du pipeline médaillon)
//!
//! ## Variables d'environnement honorées
//!
//! - `SOBRIA_DATA_ROOT` — racine des données (défaut `./data`).
//! - `SOBRIA_SEED` — seed Monte-Carlo (défaut 42).

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use sobria_ingest::{
    cli::{build_context, filter_registry, rehydrate_copper},
    sources::territoire_fr,
    LayerRegistry, PipelineReport, StepResult,
};

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
    /// Vérification d'intégrité (recalcul SHA-256 des manifests Copper).
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Pipeline(args) => match args.sub {
            PipelineSubcommand::Run(run) => run_pipeline(run).await,
        },
        Command::Copper(args) => run_copper(args).await,
        Command::Silver(args) => run_silver(args).await,
        Command::Gold => run_gold().await,
        Command::Validate => run_validate().await,
        Command::Fetch(args) => run_fetch(args).await,
        Command::Discover(args) => run_discover(args).await,
    }
}

async fn run_pipeline(args: PipelineRunArgs) -> Result<()> {
    let ctx = build_context(args.incremental)?;
    let registry = filter_registry(args.source.as_deref())?;
    tracing::info!(
        sources = registry.len(),
        data_root = %ctx.data_root.display(),
        incremental = ctx.incremental,
        "pipeline médaillon : démarrage"
    );
    let report = registry.run_full_pipeline(&ctx).await?;
    print_pipeline_report(&report);
    Ok(())
}

async fn run_copper(args: SourceArgs) -> Result<()> {
    let ctx = build_context(false)?;
    let registry = filter_registry(args.source.as_deref())?;
    let copper = registry.run_copper(&ctx).await;
    print_step_results("copper", &copper);
    if copper.iter().any(|r| r.result.is_err()) {
        return Err(anyhow!("copper: au moins une source a échoué"));
    }
    Ok(())
}

async fn run_silver(args: SourceArgs) -> Result<()> {
    let ctx = build_context(false)?;
    let registry = filter_registry(args.source.as_deref())?;
    // C26.2 : Silver repart d'un Copper déjà persisté sur disque
    // (`data/copper/<source>/<latest>/manifest.json`) plutôt que de
    // ré-ingérer depuis les sources amont. La validation `verify_files`
    // garantit l'intégrité de bout en bout.
    let copper = rehydrate_copper(&ctx, &registry).await;
    print_step_results("copper (réhydraté)", &copper);
    if copper.iter().any(|r| r.result.is_err()) {
        return Err(anyhow!(
            "silver: au moins une source n'a pas de snapshot Copper exploitable. \
             Lancez d'abord `cargo run -p sobria-ingest -- copper --all` ou `pipeline run`."
        ));
    }
    let silver = registry.run_silver(&ctx, &copper).await;
    print_step_results("silver", &silver);
    if silver.iter().any(|r| r.result.is_err()) {
        return Err(anyhow!("silver: au moins une source a échoué"));
    }
    Ok(())
}

async fn run_gold() -> Result<()> {
    let ctx = build_context(false)?;
    let registry = LayerRegistry::standard();
    let report = registry.run_full_pipeline(&ctx).await?;
    print_pipeline_report(&report);
    if report.gold_artifacts.is_none() {
        return Err(anyhow!("gold: assemblage échoué (cf. logs ci-dessus)"));
    }
    Ok(())
}

async fn run_validate() -> Result<()> {
    use sobria_ingest::manifest::CopperManifest;

    let ctx = build_context(false)?;
    let copper_root = ctx.data_root.join("copper");
    if !copper_root.exists() {
        println!(
            "Aucun snapshot Copper trouvé sous {}",
            copper_root.display()
        );
        return Ok(());
    }

    let mut total = 0usize;
    let mut ko = 0usize;
    for source_entry in std::fs::read_dir(&copper_root)? {
        let source_dir = source_entry?.path();
        if !source_dir.is_dir() {
            continue;
        }
        for snapshot_entry in std::fs::read_dir(&source_dir)? {
            let snapshot_dir = snapshot_entry?.path();
            if !snapshot_dir.is_dir() {
                continue;
            }
            let manifest_path = snapshot_dir.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            total += 1;
            match CopperManifest::load(&manifest_path).await {
                Ok(manifest) => match manifest.verify_files(&snapshot_dir).await {
                    Ok(()) => println!(
                        "OK   {}  (files: {})",
                        manifest_path.display(),
                        manifest.files.len()
                    ),
                    Err(e) => {
                        ko += 1;
                        println!("KO   {}  ←  {}", manifest_path.display(), e);
                    },
                },
                Err(e) => {
                    ko += 1;
                    println!("KO   {}  ←  load: {}", manifest_path.display(), e);
                },
            }
        }
    }

    println!("\n{} manifest(s) inspecté(s), {} KO.", total, ko);
    if ko > 0 {
        return Err(anyhow!("validate: {ko} manifest(s) corrompu(s)"));
    }
    Ok(())
}

fn print_step_results<T>(stage: &str, results: &[StepResult<T>]) {
    let ok = results.iter().filter(|r| r.is_ok()).count();
    let ko = results.len() - ok;
    println!("\n[{stage}] {} OK, {} KO", ok, ko);
    for r in results {
        match &r.result {
            Ok(_) => println!("  ok   {}", r.source_id),
            Err(e) => println!("  ko   {}  ←  {}", r.source_id, e),
        }
    }
}

fn print_pipeline_report(report: &PipelineReport) {
    print_step_results("copper", &report.copper);
    print_step_results("silver", &report.silver);
    print_step_results("gold", &report.gold_contributions);

    println!("\n[lineage]");
    println!(
        "  sources contributing : {}",
        report.gold_lineage.source_ids().join(", ")
    );
    println!(
        "  silver entities      : {}",
        report.gold_lineage.silver_inputs.len()
    );
    println!(
        "  gold artifacts       : {}",
        report.gold_lineage.gold_artifacts.join(", ")
    );

    println!("\n[résumé]");
    println!(
        "  durée                : {} ms",
        (report.finished_at - report.started_at).num_milliseconds()
    );
    println!(
        "  succès complet       : {} source(s)",
        report.fully_successful_count()
    );
    let failed = report.failed_sources();
    if !failed.is_empty() {
        println!("  échecs               : {}", failed.join(", "));
    }
    if let Some(artifacts) = &report.gold_artifacts {
        println!(
            "  referentiel.sqlite   : {}",
            artifacts.referentiel_sqlite.display()
        );
        println!(
            "  analytics.parquet    : {}",
            artifacts.analytics_parquet.display()
        );
        println!(
            "  datasheet.jsonld     : {}",
            artifacts.datasheet_jsonld.display()
        );
        println!(
            "  MANIFEST.sha256      : {}",
            artifacts.manifest_sha256.display()
        );
    } else {
        println!("  ⚠ gold: assemblage non produit");
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
            let out = args
                .out
                .unwrap_or_else(|| default_data_dir.join("territoire_fr.json"));
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
            let out = args
                .out
                .unwrap_or_else(|| default_data_dir.join("rte_mix_fr.json"));
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

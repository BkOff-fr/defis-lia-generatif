//! Entry point du binaire `sobria-team-aggregator`.

fn main() -> anyhow::Result<()> {
    sobria_team_aggregator::cli::run()
}

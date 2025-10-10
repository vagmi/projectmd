mod backend;
mod cli;
mod commands;
mod parser;
mod sync;
mod types;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { dry_run } => {
            let token = cli.github_token
                .or_else(|| std::env::var("GITHUB_TOKEN").ok())
                .context("GitHub token is required. Set GITHUB_TOKEN env var or use --github-token")?;

            commands::sync(&cli.project_file, &token, dry_run).await?;
        }

        Commands::Status { verbose } => {
            commands::status(&cli.project_file, cli.github_token.as_deref(), verbose).await?;
        }

        Commands::Init { backend, repo } => {
            commands::init(&backend, &repo).await?;
        }
    }

    Ok(())
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "projectmd")]
#[command(about = "A plain text LLM-friendly project management system", long_about = None)]
pub struct Cli {
    /// Path to the project.md file
    #[arg(short, long, default_value = "project.md")]
    pub project_file: PathBuf,

    /// GitHub personal access token (can be set via GITHUB_TOKEN env var)
    #[arg(long)]
    pub github_token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sync tasks with the backend (create/update issues)
    Sync {
        /// Dry run - show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Show the status of all tasks
    Status {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Initialize a new project.md file
    Init {
        /// Backend to use (github)
        #[arg(short, long, default_value = "github")]
        backend: String,

        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,
    },
}

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod parser;
mod sops;
mod sync;

use sync::{check_files, sync_files};

#[derive(Parser)]
#[command(name = "shell-sops")]
#[command(about = "Sync secrets from shell commands to SOPS encrypted files")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sync {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    Check {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let files = match &cli.command {
        Commands::Sync { files } | Commands::Check { files } => {
            for file in files {
                if !file.exists() {
                    return Err(anyhow!("File not found: {}", file.display()));
                }
            }
            files
        }
    };

    match cli.command {
        Commands::Sync { .. } => sync_files(files)?,
        Commands::Check { .. } => check_files(files)?,
    }

    Ok(())
}

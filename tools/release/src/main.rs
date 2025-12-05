use anyhow::Result;
use clap::{Parser, Subcommand};

mod prepare;
mod publish;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prepare a release (bump version, update changelog, create PR)
    Prepare {
        /// Version to release (e.g., v0.1.14)
        version: String,
    },
    /// Publish a release (tag, publish to crates.io/npm, update homebrew)
    Publish {
        /// Version to publish (e.g., v0.1.14)
        version: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prepare { version } => prepare::run(version),
        Commands::Publish { version } => publish::run(version),
    }
}

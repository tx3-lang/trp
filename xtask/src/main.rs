mod gen;
mod mapper;
mod openrpc;
mod resolver;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks for TRP", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate language bindings from the OpenRPC specification
    Gen(gen::GenArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gen(args) => gen::run(args)?,
    }

    Ok(())
}

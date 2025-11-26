//! CoreProver CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber;

mod commands;
mod config;

#[derive(Parser)]
#[command(name = "coreprover")]
#[command(about = "CoreProver CLI - Escrow management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Escrow management commands
    Escrow {
        #[command(subcommand)]
        command: commands::escrow::EscrowCommands,
    },
    /// Monitor blockchain events
    Monitor {
        #[command(flatten)]
        args: commands::monitor::MonitorArgs,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Escrow { command } => {
            commands::escrow::handle_command(command).await?;
        }
        Commands::Monitor { args } => {
            commands::monitor::handle_monitor(args).await?;
        }
    }
    
    Ok(())
}
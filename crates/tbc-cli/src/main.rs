//! CoreProver CLI
//!
//! Command-line interface for CoreProve/TBC management.
//!
//! Features:
//! - Escrow management
//! - Event monitoring
//! - Remote TBC administration (SSH-like secure access)

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber;

mod commands;
mod config;

#[derive(Parser)]
#[command(name = "coreprover")]
#[command(about = "CoreProver CLI - TBC & Escrow management tool", long_about = None)]
#[command(version)]
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
    
    /// Remote TBC administration
    #[command(alias = "ssh")]
    Remote {
        #[command(flatten)]
        args: commands::remote::RemoteArgs,
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
        Commands::Remote { args } => {
            commands::remote::handle_remote(args).await?;
        }
    }
    
    Ok(())
}
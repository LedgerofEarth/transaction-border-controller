//! CoreProver Service Entry Point

use anyhow::Result;
use coreprover_service::{Config, create_router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "coreprover_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_file("config/default.toml")
        .unwrap_or_else(|_| {
            tracing::warn!("Using default configuration");
            default_config()
        });

    tracing::info!("Starting CoreProver Service");
    tracing::info!("Server: {}:{}", config.server.host, config.server.port);

    // Create router
    let app = create_router();

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

fn default_config() -> Config {
    Config {
        server: coreprover_service::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
        },
        database: coreprover_service::DatabaseConfig {
            url: "postgres://postgres:postgres@localhost/coreprover".to_string(),
            max_connections: 10,
        },
        redis: coreprover_service::RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
        },
        blockchain: coreprover_service::BlockchainConfig {
            rpc_url: "http://localhost:8545".to_string(),
            contract_address: "0x0000000000000000000000000000000000000000".to_string(),
            chain_id: 31337,
        },
    }
}
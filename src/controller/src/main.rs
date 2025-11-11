mod config;
mod controller;
mod tgp;
mod prover_client;
mod x402_adapter;
mod telemetry;

use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing / logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Transaction Border Controller...");

    // Load configuration and initialize controller
    let cfg = config::ControllerConfig::default();
    let _ctrl = controller::Controller::new(cfg.clone());

    // Basic health-check endpoint
    let app = Router::new().route("/healthz", get(|| async { "ok" }));

    // Parse listen address into a concrete SocketAddr
    let addr: SocketAddr = cfg.listen_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Controller listening on {}", listener.local_addr()?);

    // Start Axum server (Axum 0.7+ API)
    axum::serve(listener, app).await?;

    Ok(())
}
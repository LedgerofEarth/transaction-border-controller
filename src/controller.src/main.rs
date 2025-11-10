mod config;
mod controller;
mod tgp;
mod prover_client;
mod x402_adapter;
mod telemetry;

use anyhow::Result;
use axum::{routing::get, Router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Transaction Border Controller...");

    let cfg = config::ControllerConfig::default();
    let _ctrl = controller::Controller::new(cfg);

    let app = Router::new().route("/healthz", get(|| async { "ok" }));

    let addr = "0.0.0.0:8080".parse()?;
    tracing::info!("Controller listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
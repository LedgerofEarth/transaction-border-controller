mod app_state;
mod config;
mod rpc_adapter;
mod routes;
mod health;
mod errors;

use axum::Server;
use tower_http::cors::{CorsLayer, Any};
use routes::build_routes;

use crate::{
    config::GatewayConfig,
    rpc_adapter::RpcAdapter,
    app_state::AppState,
};

#[tokio::main]
async fn main() {
    println!("🚀 Starting TBC Node…");

    // ------------------------------------------------------
    // Load config + adapters
    // ------------------------------------------------------
    let cfg = GatewayConfig::load();
    let rpc = RpcAdapter::new(cfg.rpc_url.clone());

    let state = AppState::new(cfg.clone(), rpc);

    // ------------------------------------------------------
    // Build Axum router
    // ------------------------------------------------------
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(cfg.allow_origin.parse().unwrap());

    let app = build_routes(state).layer(cors);

    println!("📡 Listening on {}", cfg.listen_addr);

    // ------------------------------------------------------
    // Run node
    // ------------------------------------------------------
    Server::bind(&cfg.listen_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
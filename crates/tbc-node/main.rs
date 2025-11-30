mod app_state;
mod config;
mod rpc_adapters;
mod routers;
mod health;
mod errors;

use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use routers::build_routes;

use crate::{
    config::GatewayConfig,
    rpc_adapters::RpcAdapter,
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
        .allow_origin(cfg.allow_origin.parse::<axum::http::HeaderValue>().unwrap());

    let app = build_routes(state).layer(cors);

    println!("📡 Listening on {}", cfg.listen_addr);

    // ------------------------------------------------------
    // Run node (axum 0.7 style)
    // ------------------------------------------------------
    let listener = TcpListener::bind(&cfg.listen_addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
mod app_state;
mod config;
mod rpc_adapters;
mod routers;
mod health;
mod errors;
mod admin;

use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use routers::build_routes;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::GatewayConfig,
    rpc_adapters::RpcAdapter,
    app_state::AppState,
    admin::routes::{AdminState, build_admin_routes},
};

#[tokio::main]
async fn main() {
    // ------------------------------------------------------
    // Load config from environment
    // ------------------------------------------------------
    let cfg = GatewayConfig::load();
    
    // ------------------------------------------------------
    // Initialize structured logging
    // ------------------------------------------------------
    let log_level = match cfg.log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("tbc_node={},tbc_gateway={}", log_level, log_level).into())
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
    
    // ------------------------------------------------------
    // Startup banner
    // ------------------------------------------------------
    println!();
    println!("==================================================");
    println!("  TBC - Transaction Border Controller");
    println!("  TGP-00 v3.2 Security Gateway");
    println!("==================================================");
    println!();
    
    cfg.print_summary();
    println!();

    // ------------------------------------------------------
    // Initialize adapters
    // ------------------------------------------------------
    let rpc = RpcAdapter::new(cfg.rpc_url.clone());
    let state = AppState::new(cfg.clone(), rpc);

    // ------------------------------------------------------
    // Initialize admin state
    // ------------------------------------------------------
    let admin_state = Arc::new(AdminState::new(cfg.clone()));
    
    // Check for admin keys
    let admin_count = admin_state.auth.key_store().list_admins().len();
    if admin_count > 0 {
        println!("Admin API: {} authorized keys loaded", admin_count);
    } else {
        println!("Admin API: No admin keys configured (set TBC_ADMIN_KEYS)");
    }

    // ------------------------------------------------------
    // Build Axum router with CORS
    // ------------------------------------------------------
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(cfg.allow_origin.parse::<axum::http::HeaderValue>().unwrap());

    // Merge TGP routes with admin routes
    let tgp_routes = build_routes(state);
    let admin_routes = build_admin_routes(admin_state);
    
    let app = tgp_routes
        .merge(admin_routes)
        .layer(cors);

    // ------------------------------------------------------
    // Bind and serve
    // ------------------------------------------------------
    tracing::info!(
        listen_addr = %cfg.listen_addr,
        tbc_id = ?cfg.tbc_id,
        chain_id = cfg.chain_id,
        admin_keys = admin_count,
        "TBC Gateway starting"
    );
    
    println!();
    println!("Listening on http://{}", cfg.listen_addr);
    println!("  POST /tgp        - TGP messages (HTTP)");
    println!("  GET  /tgp/ws     - TGP messages (WebSocket)");
    println!("  GET  /health     - Health check");
    println!("  POST /admin/exec - Admin commands (authenticated)");
    println!("  GET  /admin/health - Admin health (public)");
    println!();

    let listener = TcpListener::bind(&cfg.listen_addr).await.expect("Failed to bind to address");
    
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server error");
}

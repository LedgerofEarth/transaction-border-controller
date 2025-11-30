use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub listen_addr: String,
    pub rpc_url: String,
    pub chain_id: u64,

    /// SETTLE monitoring interval (in ms)
    pub settle_poll_interval_ms: u64,

    /// Allow CORS for extension
    pub allow_origin: String,
    
    /// TBC instance identifier (for logging/audit)
    pub tbc_id: Option<String>,
    
    /// WebSocket path (default: /tgp/ws)
    pub ws_path: String,
    
    /// Log level (default: info)
    pub log_level: String,
}

impl GatewayConfig {
    /// Load configuration from environment variables with sensible defaults.
    /// 
    /// Environment Variables:
    /// - TBC_LISTEN_ADDR: Server bind address (default: 0.0.0.0:8080)
    /// - TBC_RPC_URL: Blockchain RPC endpoint (default: https://rpc.pulsechain.com)
    /// - TBC_CHAIN_ID: Chain ID (default: 369 for PulseChain)
    /// - TBC_SETTLE_POLL_MS: Settlement poll interval in ms (default: 1000)
    /// - TBC_ALLOW_ORIGIN: CORS allowed origin (default: *)
    /// - TBC_ID: Instance identifier for logging (default: tbc-primary)
    /// - TBC_WS_PATH: WebSocket path (default: /tgp/ws)
    /// - TBC_LOG_LEVEL: Log level (default: info)
    /// - PORT: Alternative port binding (for Railway/Heroku compatibility)
    pub fn load() -> Self {
        // Support PORT env var for Railway/Heroku/Fly.io
        let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
        let default_listen = format!("0.0.0.0:{}", port);
        
        Self {
            listen_addr: env::var("TBC_LISTEN_ADDR")
                .unwrap_or(default_listen),
            
            rpc_url: env::var("TBC_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.pulsechain.com".into()),
            
            chain_id: env::var("TBC_CHAIN_ID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(369),
            
            settle_poll_interval_ms: env::var("TBC_SETTLE_POLL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            
            allow_origin: env::var("TBC_ALLOW_ORIGIN")
                .unwrap_or_else(|_| "*".into()),
            
            tbc_id: Some(env::var("TBC_ID")
                .unwrap_or_else(|_| "tbc-primary".into())),
            
            ws_path: env::var("TBC_WS_PATH")
                .unwrap_or_else(|_| "/tgp/ws".into()),
            
            log_level: env::var("TBC_LOG_LEVEL")
                .unwrap_or_else(|_| "info".into()),
        }
    }
    
    /// Print configuration summary (safe - no secrets)
    pub fn print_summary(&self) {
        println!("┌────────────────────────────────────────┐");
        println!("│     TBC Gateway Configuration          │");
        println!("├────────────────────────────────────────┤");
        println!("│ Listen:    {:<27}│", self.listen_addr);
        println!("│ Chain ID:  {:<27}│", self.chain_id);
        println!("│ TBC ID:    {:<27}│", self.tbc_id.as_deref().unwrap_or("default"));
        println!("│ WS Path:   {:<27}│", self.ws_path);
        println!("│ Log Level: {:<27}│", self.log_level);
        println!("│ CORS:      {:<27}│", self.allow_origin);
        println!("└────────────────────────────────────────┘");
    }
}
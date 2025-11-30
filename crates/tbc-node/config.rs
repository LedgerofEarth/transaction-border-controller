use serde::{Serialize, Deserialize};

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
}

impl GatewayConfig {
    pub fn load() -> Self {
        // In future: load from TOML/YAML file or env
        Self {
            listen_addr: "0.0.0.0:8080".into(),
            rpc_url: "https://rpc.pulsechain.com".into(),
            chain_id: 369,
            settle_poll_interval_ms: 1000,
            allow_origin: "*".into(),
            tbc_id: Some("tbc-primary".into()),
        }
    }
}
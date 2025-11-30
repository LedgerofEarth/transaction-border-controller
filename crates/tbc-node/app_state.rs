use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<GatewayConfig>,
    pub rpc: Arc<RpcAdapter>,
}

use crate::config::GatewayConfig;
use crate::rpc_adapters::RpcAdapter;

impl AppState {
    pub fn new(cfg: GatewayConfig, rpc: RpcAdapter) -> Self {
        Self {
            cfg: Arc::new(cfg),
            rpc: Arc::new(rpc),
        }
    }
}
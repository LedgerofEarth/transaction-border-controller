use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::config::GatewayConfig;
use crate::rpc_adapters::RpcAdapter;
use crate::admin::routes::AdminState;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<GatewayConfig>,
    pub rpc: Arc<RpcAdapter>,
    pub admin: Arc<AdminState>,
}

impl AppState {
    pub fn new(cfg: GatewayConfig, rpc: RpcAdapter) -> Self {
        let admin = AdminState::new(cfg.clone());
        Self {
            cfg: Arc::new(cfg),
            rpc: Arc::new(rpc),
            admin: Arc::new(admin),
        }
    }
}
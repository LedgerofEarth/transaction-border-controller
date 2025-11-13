//! Gateway trait and implementation

use async_trait::async_trait;
use anyhow::Result;

/// Core gateway trait for TBC protocol
#[async_trait]
pub trait Gateway {
/// Route an order through the gateway
async fn route_order(&self, order_id: &str) -> Result<String>;

/// Get gateway status
async fn status(&self) -> Result<GatewayStatus>;

}

/// Gateway status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayStatus {
pub online: bool,
pub active_orders: usize,
pub version: String,
}

impl Default for GatewayStatus {
fn default() -> Self {
Self {
online: true,
active_orders: 0,
version: "0.1.0".to_string(),
}
}
}

impl GatewayStatus {
/// Create a new gateway status
pub fn new(online: bool, active_orders: usize) -> Self {
Self {
online,
active_orders,
version: "0.1.0".to_string(),
}
}

/// Check if gateway is healthy
pub fn is_healthy(&self) -> bool {
    self.online
}

}
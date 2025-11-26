//! CLI configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub rpc_url: String,
    pub contract_address: String,
    pub private_key: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            contract_address: "0x0000000000000000000000000000000000000000".to_string(),
            private_key: None,
        }
    }
}
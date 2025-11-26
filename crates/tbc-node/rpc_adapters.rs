use anyhow::Result;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct RpcAdapter {
    pub rpc_url: String,
}

impl RpcAdapter {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self { rpc_url: rpc_url.into() }
    }

    pub async fn eth_call(&self, to: &str, data: &str) -> Result<String> {
        // Future: use reqwest or ethers-rs
        // Stub for now
        Ok("0xdeadbeef".into())
    }

    pub async fn get_tx_receipt(&self, tx_hash: &str) -> Result<Option<serde_json::Value>> {
        Ok(None) // stub
    }
}
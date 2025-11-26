use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}
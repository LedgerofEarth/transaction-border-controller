//! CoreProver Settlement Service

pub mod api;
pub mod settlement;
pub mod workers;
pub mod profiles;
pub mod engine;
pub mod types;

pub use api::routes::create_router;

/// Service configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub blockchain: BlockchainConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub contract_address: String,
    pub chain_id: u64,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
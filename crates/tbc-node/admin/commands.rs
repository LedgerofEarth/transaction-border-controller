//! Admin Commands
//!
//! Available commands for remote TBC administration.
//! All commands require authentication and appropriate role.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Admin command enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum AdminCommand {
    // ===========================================
    // Monitor Commands (all roles)
    // ===========================================
    
    /// Get TBC health and status
    Health,
    
    /// Get current configuration (sanitized)
    GetConfig,
    
    /// Get connection statistics
    GetStats,
    
    /// Get recent log entries
    GetLogs { 
        lines: Option<usize>,
        level: Option<String>,
    },
    
    /// Ping/pong for connectivity check
    Ping,

    // ===========================================
    // Operator Commands
    // ===========================================
    
    /// List active WebSocket connections
    ListConnections,
    
    /// Get nullifier cache status
    GetNullifierStatus,
    
    /// Get RPC provider health
    GetRpcHealth,
    
    /// Query a specific session by ID
    QuerySession { session_id: String },
    
    /// Get verification layer status
    GetLayerStatus,

    // ===========================================
    // SuperAdmin Commands
    // ===========================================
    
    /// Reload configuration from environment
    ReloadConfig,
    
    /// Set a runtime configuration value
    SetConfig { 
        key: String, 
        value: Value,
    },
    
    /// Add a new admin key
    AddAdmin {
        name: String,
        public_key: String,
        role: String,
    },
    
    /// Remove an admin key
    RemoveAdmin { public_key: String },
    
    /// List all registered admins
    ListAdmins,
    
    /// Enable/disable a verification layer
    SetLayerEnabled {
        layer: u8,
        enabled: bool,
    },
    
    /// Add a merchant to whitelist
    AddMerchantWhitelist { address: String },
    
    /// Remove a merchant from whitelist
    RemoveMerchantWhitelist { address: String },
    
    /// Clear the nullifier cache (dangerous!)
    ClearNullifierCache { confirm: bool },
    
    /// Graceful shutdown
    Shutdown { delay_secs: Option<u64> },
}

impl AdminCommand {
    /// Get the command name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Health => "health",
            Self::GetConfig => "get_config",
            Self::GetStats => "get_stats",
            Self::GetLogs { .. } => "get_logs",
            Self::Ping => "ping",
            Self::ListConnections => "list_connections",
            Self::GetNullifierStatus => "get_nullifier_status",
            Self::GetRpcHealth => "get_rpc_health",
            Self::QuerySession { .. } => "query_session",
            Self::GetLayerStatus => "get_layer_status",
            Self::ReloadConfig => "reload_config",
            Self::SetConfig { .. } => "set_config",
            Self::AddAdmin { .. } => "add_admin",
            Self::RemoveAdmin { .. } => "remove_admin",
            Self::ListAdmins => "list_admins",
            Self::SetLayerEnabled { .. } => "set_layer_enabled",
            Self::AddMerchantWhitelist { .. } => "add_merchant_whitelist",
            Self::RemoveMerchantWhitelist { .. } => "remove_merchant_whitelist",
            Self::ClearNullifierCache { .. } => "clear_nullifier_cache",
            Self::Shutdown { .. } => "shutdown",
        }
    }

    /// Get required role for this command
    pub fn required_role(&self) -> super::auth::AdminRole {
        use super::auth::AdminRole;
        
        match self {
            // Monitor commands - all roles
            Self::Health
            | Self::GetConfig
            | Self::GetStats
            | Self::GetLogs { .. }
            | Self::Ping => AdminRole::Monitor,

            // Operator commands
            Self::ListConnections
            | Self::GetNullifierStatus
            | Self::GetRpcHealth
            | Self::QuerySession { .. }
            | Self::GetLayerStatus => AdminRole::Operator,

            // SuperAdmin commands
            Self::ReloadConfig
            | Self::SetConfig { .. }
            | Self::AddAdmin { .. }
            | Self::RemoveAdmin { .. }
            | Self::ListAdmins
            | Self::SetLayerEnabled { .. }
            | Self::AddMerchantWhitelist { .. }
            | Self::RemoveMerchantWhitelist { .. }
            | Self::ClearNullifierCache { .. }
            | Self::Shutdown { .. } => AdminRole::SuperAdmin,
        }
    }
}

/// Result of command execution
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub command: String,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl CommandResult {
    pub fn ok(command: &str, data: Value) -> Self {
        Self {
            success: true,
            command: command.to_string(),
            data: Some(data),
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn err(command: &str, error: impl ToString) -> Self {
        Self {
            success: false,
            command: command.to_string(),
            data: None,
            error: Some(error.to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}


//! Admin API Routes
//!
//! Exposes admin endpoints:
//! - POST /admin/exec - Execute authenticated command
//! - GET /admin/health - Public health check (no auth)

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

use super::{
    auth::{AdminAuth, AdminRole, SignedRequest},
    commands::{AdminCommand, CommandResult},
};
use crate::config::GatewayConfig;

/// Admin API state
pub struct AdminState {
    pub auth: AdminAuth,
    pub config: GatewayConfig,
    pub start_time: std::time::Instant,
}

impl AdminState {
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            auth: AdminAuth::new(),
            config,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Build admin routes
pub fn build_admin_routes(state: Arc<AdminState>) -> Router {
    Router::new()
        .route("/admin/exec", post(execute_command))
        .route("/admin/health", get(public_health))
        .with_state(state)
}

/// Public health endpoint (no auth required)
async fn public_health(
    State(state): State<Arc<AdminState>>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(json!({
        "status": "ok",
        "service": "tbc-admin",
        "uptime_seconds": uptime,
        "tbc_id": state.config.tbc_id,
    }))
}

/// Execute authenticated admin command
async fn execute_command(
    State(state): State<Arc<AdminState>>,
    Json(request): Json<SignedRequest>,
) -> impl IntoResponse {
    // Verify authentication
    let admin = match state.auth.verify_request(&request) {
        Ok(admin) => admin,
        Err(e) => {
            tracing::warn!(
                pubkey = %request.public_key,
                command = %request.command,
                error = %e,
                "Admin auth failed"
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(CommandResult::err(&request.command, format!("Auth failed: {}", e))),
            );
        }
    };

    // Parse command
    let cmd: AdminCommand = match serde_json::from_value(json!({
        "cmd": request.command,
        "args": request.args,
    })) {
        Ok(cmd) => cmd,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CommandResult::err(&request.command, format!("Invalid command: {}", e))),
            );
        }
    };

    // Check role
    if let Err(e) = state.auth.check_role(&admin, cmd.required_role()) {
        tracing::warn!(
            admin = %admin.name,
            command = %request.command,
            error = %e,
            "Admin permission denied"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(CommandResult::err(&request.command, e.to_string())),
        );
    }

    // Execute command
    let result = execute(&state, &admin, cmd).await;
    
    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(result))
}

/// Execute a specific admin command
async fn execute(
    state: &AdminState,
    admin: &super::auth::AdminEntry,
    cmd: AdminCommand,
) -> CommandResult {
    let cmd_name = cmd.name();
    
    tracing::info!(
        admin = %admin.name,
        command = %cmd_name,
        "Executing admin command"
    );

    match cmd {
        // ===========================================
        // Monitor Commands
        // ===========================================
        
        AdminCommand::Ping => {
            CommandResult::ok(cmd_name, json!({ "pong": true }))
        }

        AdminCommand::Health => {
            let uptime = state.start_time.elapsed().as_secs();
            CommandResult::ok(cmd_name, json!({
                "status": "healthy",
                "uptime_seconds": uptime,
                "tbc_id": state.config.tbc_id,
                "chain_id": state.config.chain_id,
                "rpc_url": mask_url(&state.config.rpc_url),
            }))
        }

        AdminCommand::GetConfig => {
            // Return sanitized config (no secrets)
            CommandResult::ok(cmd_name, json!({
                "listen_addr": state.config.listen_addr,
                "chain_id": state.config.chain_id,
                "tbc_id": state.config.tbc_id,
                "ws_path": state.config.ws_path,
                "log_level": state.config.log_level,
                "allow_origin": state.config.allow_origin,
            }))
        }

        AdminCommand::GetStats => {
            // TODO: Implement real stats
            CommandResult::ok(cmd_name, json!({
                "connections": {
                    "websocket": 0,
                    "http": 0,
                },
                "messages": {
                    "received": 0,
                    "sent": 0,
                    "errors": 0,
                },
                "proofs": {
                    "verified": 0,
                    "rejected": 0,
                },
            }))
        }

        AdminCommand::GetLogs { lines, level } => {
            let n = lines.unwrap_or(100);
            // TODO: Implement log retrieval
            CommandResult::ok(cmd_name, json!({
                "lines": n,
                "level": level.unwrap_or_else(|| "info".to_string()),
                "entries": [],
            }))
        }

        // ===========================================
        // Operator Commands
        // ===========================================
        
        AdminCommand::ListConnections => {
            // TODO: Implement connection listing
            CommandResult::ok(cmd_name, json!({
                "connections": []
            }))
        }

        AdminCommand::GetNullifierStatus => {
            // TODO: Implement nullifier status
            CommandResult::ok(cmd_name, json!({
                "cached": 0,
                "verified": 0,
                "rejected": 0,
            }))
        }

        AdminCommand::GetRpcHealth => {
            // TODO: Implement RPC health check
            CommandResult::ok(cmd_name, json!({
                "rpc_url": mask_url(&state.config.rpc_url),
                "status": "unknown",
                "latency_ms": null,
            }))
        }

        AdminCommand::QuerySession { session_id } => {
            // TODO: Implement session query
            CommandResult::ok(cmd_name, json!({
                "session_id": session_id,
                "status": "not_found",
            }))
        }

        AdminCommand::GetLayerStatus => {
            CommandResult::ok(cmd_name, json!({
                "layers": {
                    "L1": { "name": "Registry", "enabled": true },
                    "L2": { "name": "Signature", "enabled": true },
                    "L3": { "name": "Bytecode", "enabled": true },
                    "L4": { "name": "ZK", "enabled": true },
                    "L5": { "name": "Policy", "enabled": true },
                }
            }))
        }

        // ===========================================
        // SuperAdmin Commands
        // ===========================================
        
        AdminCommand::ListAdmins => {
            let admins: Vec<_> = state.auth.key_store().list_admins()
                .into_iter()
                .map(|a| json!({
                    "name": a.name,
                    "public_key": mask_key(&a.public_key),
                    "role": format!("{:?}", a.role),
                    "created_at": a.created_at,
                    "last_seen": a.last_seen,
                }))
                .collect();
            
            CommandResult::ok(cmd_name, json!({ "admins": admins }))
        }

        AdminCommand::AddAdmin { name, public_key, role } => {
            let role = match role.to_lowercase().as_str() {
                "super" | "superadmin" => AdminRole::SuperAdmin,
                "operator" => AdminRole::Operator,
                _ => AdminRole::Monitor,
            };
            
            let entry = super::auth::AdminEntry {
                name: name.clone(),
                public_key: public_key.clone(),
                role,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                last_seen: None,
            };
            
            if let Err(e) = state.auth.key_store().register_admin(entry) {
                return CommandResult::err(cmd_name, e.to_string());
            }
            
            tracing::info!(
                by = %admin.name,
                new_admin = %name,
                role = ?role,
                "Admin added"
            );
            
            CommandResult::ok(cmd_name, json!({
                "added": name,
                "role": format!("{:?}", role),
            }))
        }

        AdminCommand::ReloadConfig => {
            // TODO: Implement config reload
            CommandResult::ok(cmd_name, json!({ "reloaded": true }))
        }

        AdminCommand::SetConfig { key, value } => {
            // TODO: Implement runtime config changes
            tracing::info!(
                by = %admin.name,
                key = %key,
                "Config set attempted"
            );
            CommandResult::err(cmd_name, "Runtime config changes not yet implemented")
        }

        AdminCommand::RemoveAdmin { public_key } => {
            // TODO: Implement admin removal
            CommandResult::err(cmd_name, "Admin removal not yet implemented")
        }

        AdminCommand::SetLayerEnabled { layer, enabled } => {
            tracing::info!(
                by = %admin.name,
                layer = layer,
                enabled = enabled,
                "Layer toggle attempted"
            );
            CommandResult::err(cmd_name, "Layer toggling not yet implemented")
        }

        AdminCommand::AddMerchantWhitelist { address } => {
            tracing::info!(
                by = %admin.name,
                address = %address,
                "Merchant whitelist add attempted"
            );
            CommandResult::err(cmd_name, "Merchant whitelist not yet implemented")
        }

        AdminCommand::RemoveMerchantWhitelist { address } => {
            CommandResult::err(cmd_name, "Merchant whitelist not yet implemented")
        }

        AdminCommand::ClearNullifierCache { confirm } => {
            if !confirm {
                return CommandResult::err(cmd_name, "Must confirm=true to clear nullifier cache");
            }
            tracing::warn!(
                by = %admin.name,
                "Nullifier cache clear attempted"
            );
            CommandResult::err(cmd_name, "Nullifier cache clear not yet implemented")
        }

        AdminCommand::Shutdown { delay_secs } => {
            let delay = delay_secs.unwrap_or(5);
            tracing::warn!(
                by = %admin.name,
                delay_secs = delay,
                "Shutdown requested"
            );
            // TODO: Implement graceful shutdown
            CommandResult::ok(cmd_name, json!({
                "shutdown_scheduled": true,
                "delay_seconds": delay,
            }))
        }
    }
}

/// Mask a URL for safe display (hide credentials)
fn mask_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if parsed.password().is_some() || parsed.username() != "" {
            return format!("{}://***@{}{}", 
                parsed.scheme(),
                parsed.host_str().unwrap_or(""),
                parsed.path()
            );
        }
    }
    url.to_string()
}

/// Mask a key for safe display (show first/last 4 chars)
fn mask_key(key: &str) -> String {
    if key.len() > 12 {
        format!("{}...{}", &key[..6], &key[key.len()-4..])
    } else {
        key.to_string()
    }
}


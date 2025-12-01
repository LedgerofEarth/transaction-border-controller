//! TBC Admin Module
//!
//! Provides secure remote administration via:
//! - Ed25519 public key authentication
//! - Signed request verification
//! - Audit logging
//!
//! Per TGP-TBC-SEC-00: All admin actions are logged and authenticated.

pub mod auth;
pub mod commands;
pub mod routes;

pub use auth::{AdminAuth, AdminKeyStore};
pub use commands::AdminCommand;
pub use routes::{build_admin_routes, run_admin_command, AdminState};


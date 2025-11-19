//! TBC Gateway – Production Interface (Option C)
//!
//! This module defines the production-facing TBC Gateway trait,
//! which routes TGP messages, enforces policy, drives the state machine,
//! and produces decisions compatible with:
//!
//! - TGP-00 (Core Protocol)
//! - TGP-SEC-00 (Security & Enforcement)
//! - MCP-AUTO-PAY-00 (Delegated Session-Key Automation)
//!
//! The Gateway is the *entry point* for:
//! - CoreProve extension clients
//! - MCP agents
//! - API integrations
//!
//! It receives raw JSON messages, parses them into TGP structs,
//! validates them, applies policy, drives session state transitions,
//! and produces Offer / Accept / Settle / Error responses.

use async_trait::async_trait;
use anyhow::Result;

use crate::tgp::{
    messages::{Query, Offer, Accept, Settle, ErrorMessage},
    decision::{TGPValidationResult, TBCDecision},
    state::{TGPSession, TGPState},
};

/// High-level routing result returned by all gateway operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayResponse<T> {
    /// The decision produced (Approve, Reject, EscrowRequired, etc.)
    pub decision: TBCDecision,

    /// Optional session state summary
    pub session: Option<TGPSession>,

    /// The message produced by the gateway (OFFER, ERROR, SETTLE, etc.)
    pub message: Option<T>,
}

/// Core gateway interface for TGP routing
#[async_trait]
pub trait Gateway {
    // -------------------------------------------------------------------------
    // Legacy API (kept for compatibility)
    // -------------------------------------------------------------------------

    /// Dummy order routing (legacy)
    async fn route_order(&self, order_id: &str) -> Result<String>;

    /// Get gateway status
    async fn status(&self) -> Result<GatewayStatus>;

    // -------------------------------------------------------------------------
    // TGP-00 Message Routing API (NEW)
    // -------------------------------------------------------------------------

    /// Handle inbound QUERY → produce OFFER or ERROR
    async fn handle_query(&self, msg: Query) -> Result<GatewayResponse<Offer>>;

    /// Handle inbound ACCEPT → advance session → return SETTLE or ERROR
    async fn handle_accept(&self, msg: Accept) -> Result<GatewayResponse<Settle>>;

    /// Handle inbound SETTLE (BuyerNotify or Indexer) → finalize session
    async fn handle_settle(&self, msg: Settle) -> Result<GatewayResponse<()>>;

    /// Handle inbound ERROR → transition session → return acknowledgment
    async fn handle_error(&self, msg: ErrorMessage) -> Result<GatewayResponse<()>>;
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
            version: crate::VERSION.to_string(),
        }
    }
}
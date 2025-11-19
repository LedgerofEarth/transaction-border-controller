//! # TGP Router Module (mod.rs)
//!
//! This directory follows SIP RFC3261 structure:
//!   • `inbound.rs`  – Transaction-layer inbound dispatcher
//!   • `mod.rs`      – Re-exports + public interface
//!
//! The router is intentionally separated from:
//!   • codec_tx.rs  → transport parsing/encoding/replay
//!   • handlers/    → pure functional message handlers
//!   • logging.rs   → structured JSON/ANSI logging
//!   • state.rs     → TGP session state machine
//!
//! This mirrors SIP design where the Transaction Layer is isolated from:
//!   • Message Parsing (Chapter 7)
//!   • Transport (Chapter 18)
//!
//! The TBC architecture preserves these layers to ensure:
//!   • correctness
//!   • replay protection
//!   • minimal mutation surface
//!   • secure boundaries for MCP auto-routing
//!
//! Controller code should use:
//!
//! ```rust
//! use tbc_gateway::router::InboundRouter;
//! ```

mod inbound;

pub use inbound::{
    TGPInboundRouter,
    InboundRouter,
};
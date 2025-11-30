//! Transaction Border Controller (TBC) Gateway
//!
//! This crate implements the gateway-side runtime of the TBC ecosystem,
//! responsible for:
//!   • receiving inbound TGP-00 messages
//!   • parsing & classification (via tbc-core + codec_tx.rs)
//!   • policy enforcement via handlers
//!   • session lifecycle management
//!   • structured logging
//!   • emitting outbound protocol-compliant TGP messages
//!
//! Architectural alignment:
//!   • Mirrors SIP-RFC3261 decomposition (transport ↔ parsing ↔ routing)
//!   • All serialization lives in `codec_tx.rs`
//!   • All protocol grammar lives in `tbc-core/tgp/protocol.rs`
//!   • The router is a pure control-plane state engine
//!
//! This crate does *not* contain business logic, settlement logic,
//! or economic-layer responsibilities -- those live in handlers and
//! the CoreProver / Layer-8 settlement stack.

pub mod router;        // Inbound TGP routing engine
pub mod handlers;      // Handler layer implementations (QUERY/OFFER/SETTLE/ERROR)
pub mod logging;       // Structured + colorized TGP logs
// pub mod store;      // TODO: Session storage implementations (not yet created)
// pub mod workers;    // TODO: depends on SessionStore trait
// pub mod ws;         // TODO: WebSocket module (files in wrong location)
//pub mod codec_tx;      // NEW -- parsing, classification, metadata construction
// pub mod error;         // Future: gateway-specific errors (optional)

// Re-exports for convenience
pub use router::{InboundRouter, TGPInboundRouter};
// pub use store::InMemorySessionStore;  // TODO: depends on store module
// pub use workers::{run_cleanup_worker, CleanupConfig};  // TODO: depends on workers module

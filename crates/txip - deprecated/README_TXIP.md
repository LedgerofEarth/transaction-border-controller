# TxIP Implementation for Transaction Border Controller

**Version:** 0.2-draft  
**Date:** November 14, 2025  
**Status:** Ready for Integration

---

## Overview

This directory contains the complete Rust implementation of TxIP-00 (Transaction Interchange Protocol), the transport and envelope layer for TGP (Transaction Gateway Protocol) messages.

## Files Included

### Documentation
- `TxIP-00.md` - Complete protocol specification
- `TXIP_RUST_ANALYSIS.md` - Detailed code analysis and integration guide
- `README_TXIP.md` - This file

### Core Implementation (6 modules)
1. `txip_types.rs` - Type definitions and serialization (387 lines)
2. `txip_session.rs` - Session management (282 lines)
3. `txip_http_handler.rs` - HTTP endpoint handler (325 lines)
4. `txip_websocket_handler.rs` - WebSocket handler (366 lines)
5. `txip_mod.rs` - Module root and public API (159 lines)
6. `txip_server_example.rs` - Example server (254 lines)

**Total:** 1,773 lines of clean ASCII Rust code

---

## Quick Start

### 1. Copy Files to Your Project

```bash
# Create TxIP module directory
mkdir -p crates/tbc-gateway/src/txip

# Copy module files
cp txip_types.rs crates/tbc-gateway/src/txip/types.rs
cp txip_session.rs crates/tbc-gateway/src/txip/session.rs
cp txip_http_handler.rs crates/tbc-gateway/src/txip/http_handler.rs
cp txip_websocket_handler.rs crates/tbc-gateway/src/txip/websocket_handler.rs
cp txip_mod.rs crates/tbc-gateway/src/txip/mod.rs

# Copy example
cp txip_server_example.rs crates/tbc-gateway/examples/txip_server.rs
```

### 2. Add Dependencies

Add to `crates/tbc-gateway/Cargo.toml`:

```toml
[dependencies]
axum = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
tracing = "0.1"

[dev-dependencies]
tower = "0.4"
tracing-subscriber = "0.3"
```

### 3. Update Module Exports

In `crates/tbc-gateway/src/lib.rs`:

```rust
pub mod txip;
```

### 4. Run Tests

```bash
cd crates/tbc-gateway
cargo test --lib txip
```

### 5. Run Example Server

```bash
cargo run --example txip_server
```

Server will start on `http://127.0.0.1:3000`

---

## API Usage

### Creating a TxIP Envelope

```rust
use tbc_gateway::txip::prelude::*;
use serde_json::json;

// Create TGP message envelope
let envelope = TxipEnvelope::tgp(
    "msg-123".to_string(),
    "sess-456".to_string(),
    Direction::ClientToTbc,
    Role::BuyerAgent,
    TgpPhase::Query,
    json!({
        "phase": "QUERY",
        "id": "q-123",
        "from": "buyer://alice",
        "to": "seller://pizza",
        "asset": "USDC",
        "amount": "30000000"
    }),
);

// Validate
envelope.validate()?;

// Serialize
let json = serde_json::to_string(&envelope)?;
```

### Setting Up HTTP Handler

```rust
use axum::{routing::post, Router};
use tbc_gateway::txip::{HttpHandlerState, handle_txip_message, SessionManager};
use std::sync::Arc;

let session_manager = Arc::new(SessionManager::new(Default::default()));

let state = Arc::new(HttpHandlerState {
    session_manager,
    tbc_id: "tbc://my-tbc".to_string(),
});

let app = Router::new()
    .route("/txip/v0/messages", post(handle_txip_message))
    .with_state(state);
```

### Setting Up WebSocket Handler

```rust
use axum::{routing::get, Router};
use tbc_gateway::txip::{WebSocketHandlerState, handle_websocket_upgrade};

let ws_state = Arc::new(WebSocketHandlerState {
    session_manager,
    tbc_id: "tbc://my-tbc".to_string(),
});

let app = Router::new()
    .route("/txip/v0/ws", get(handle_websocket_upgrade))
    .with_state(ws_state);
```

---

## Testing

### Send HELLO via HTTP

```bash
curl -X POST http://localhost:3000/txip/v0/messages \
  -H "Content-Type: application/json" \
  -d '{
    "txip_version": "0.2",
    "msg_id": "550e8400-e29b-41d4-a716-446655440000",
    "session_id": "sess-123",
    "direction": "CLIENT_TO_TBC",
    "role": "BUYER_AGENT",
    "timestamp": 1731600000,
    "message_type": "CONTROL",
    "tgp_phase": "NONE",
    "tgp_type": null,
    "payload": {
      "control_type": "HELLO",
      "agent_id": "buyer://alice",
      "supported_tgp_versions": ["2.0"],
      "supported_transports": ["HTTP"],
      "supported_chains": ["pulse-mainnet"],
      "supported_assets": ["USDC"],
      "features": {
        "zk_discount_proofs": true,
        "receipt_ownership_proofs": true,
        "late_discount_support": true
      },
      "auth": {
        "scheme": "NONE",
        "token": null
      }
    }
  }'
```

### Connect via WebSocket

```bash
# Using wscat
wscat -c ws://localhost:3000/txip/v0/ws

# Then send HELLO message (same JSON as above)
```

---

## Integration Points

### TODO: Connect TGP Router

In `txip_http_handler.rs` and `txip_websocket_handler.rs`, replace:

```rust
// TODO: Forward to TGP routing layer
tracing::info!("Received TGP message: ...");
```

With:

```rust
// Forward to your TGP routing layer
tgp_router::route_message(&envelope, &tgp_payload.tgp).await?;
```

### TODO: Add Authentication

Implement auth validation in HELLO handler:

```rust
fn validate_auth(auth: &AuthInfo) -> Result<(), String> {
    match auth.scheme {
        AuthScheme::BearerJwt => validate_jwt(auth.token.as_ref().unwrap()),
        AuthScheme::ApiKey => validate_api_key(auth.token.as_ref().unwrap()),
        // ...
    }
}
```

---

## Production Checklist

Before deploying to production:

- [ ] Enable TLS/SSL on endpoints
- [ ] Implement authentication/authorization
- [ ] Add rate limiting per session/IP
- [ ] Configure request timeouts
- [ ] Set message size limits
- [ ] Add metrics and monitoring
- [ ] Implement log aggregation
- [ ] Set up health checks
- [ ] Configure graceful shutdown
- [ ] Add DoS protection
- [ ] Review session timeout values
- [ ] Implement connection pooling
- [ ] Add performance benchmarks

---

## Architecture

```
┌─────────────────────────────────────────┐
│   Client (Agent/Wallet/Service)         │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴──────────┐
        │                    │
    HTTP POST            WebSocket
        │                    │
┌───────▼────────────────────▼───────────┐
│         TxIP Layer (This Module)       │
│  ┌────────────────────────────────┐   │
│  │  Session Manager               │   │
│  │  - Capability negotiation      │   │
│  │  - Idempotency tracking        │   │
│  │  - Timeout management          │   │
│  └────────────────────────────────┘   │
│  ┌────────────────────────────────┐   │
│  │  Message Router                │   │
│  │  - CONTROL → Session ops       │   │
│  │  - TGP → Forward to TGP layer  │   │
│  │  - ERROR → Log and respond     │   │
│  └────────────────────────────────┘   │
└────────────────┬───────────────────────┘
                 │
                 │ TGP Messages
                 │
┌────────────────▼───────────────────────┐
│         TGP Layer (Your Code)          │
│  - L8: Economic routing                │
│  - L9: Identity routing                │
│  - L10: Policy routing                 │
└────────────────────────────────────────┘
```

---

## Support

For questions or issues:
1. Review `TxIP-00.md` specification
2. Check `TXIP_RUST_ANALYSIS.md` for detailed code analysis
3. Examine example in `txip_server_example.rs`
4. Review unit tests in each module

---

## License

Part of the Transaction Border Controller project.

---

**Created:** November 14, 2025  
**Last Updated:** November 14, 2025  
**Status:** Ready for integration

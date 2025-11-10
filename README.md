# Transaction Border Controller (TBC) Monorepo

**Organization:** Ledger of Earth  
**Version:** 0.1-draft  
**Status:** Active Development  

—

## Overview

This monorepo houses both the **protocol specifications** and the **reference implementation**
for the **Transaction Border Controller (TBC)** — a Layer-8 appliance that routes and enforces
policy for cross-ledger settlements using the **Transaction Gateway Protocol (TGP-00)** and
related standards.

The TBC functions as the **economic control plane** for blockchain systems, analogous to a
Session Border Controller in telecom architecture.

—

## Directory Layout

| Path | Purpose |
|——|-———|
| `specs/` | Normative specifications (TxIP-00, TGP-00, X402-EXT, appendices). |
| `src/controller/` | Rust implementation of the Transaction Border Controller appliance. |
| `src/coreprover/` | CoreProver library for escrow and proof-of-settlement receipts. |
| `tests/` | Integration and simulation harnesses. |
| `docs/` | Architecture, system topology, roadmap, and context for AI agents. |
| `.anthropic/` | AI agent configuration for Claude MCP and related tools. |

—

## Build Instructions

```bash
# Build everything
cargo build

# Run the controller (development mode)
cargo run —package controller

# Run tests
cargo test —workspace
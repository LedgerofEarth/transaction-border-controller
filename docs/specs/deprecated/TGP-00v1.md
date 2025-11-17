# TGP-00: Transaction Gateway Protocol

## Abstract

The Transaction Gateway Protocol (**TGP-00**) defines a metadata signaling layer (**Layer 8**) that enables compliant, cross-boundary transaction routing in blockchain-based networks. It facilitates **trust-aware session coordination** between wallets, gateways, and AI agents operating across jurisdictions, identity systems, and regulatory zones.

TGP operates at Layer 8 — the economic layer — alongside the blockchain’s ledgers and distributed databases. It interacts directly with RPC endpoints or flattened ledger data to inform economic-layer routing and compliance decisions. It sits beneath identity (Layer 9) and policy (Layer 10) systems.

TGP supports both **direct settlement paths** (e.g. via x402) and **non-custodial swap settlement** through **CoreProver escrow contracts**. These escrow contracts facilitate safe exchange of value for value (e.g. tokens for tokens) or value for verifiable output (e.g. digital receipts, download links, or external delivery confirmation). The `zk_profile` field in TGP.QUERY indicates the Buyer’s preference for ZK involvement, while `zk_required` in TGP.OFFER reflects the Controller’s policy decision. CoreProver escrow can function with or without ZK proofs, using onchain acknowledgments or offchain signatures as settlement triggers.

All accepted sessions result in emission of a **Transaction Detail Record (TDR)**, enabling traceable, auditable, and policy-compliant transaction flows without revealing sensitive user data. TGP is designed for compatibility with **AI-driven agents**, **cross-chain smart contracts**, and **federated compliance registries**, and serves as a foundational component of the emerging Layer 8–10 trust stack.

——

## Implementation Status

**Current Milestone:** M0 Complete (Repo Bootstrap + Health Endpoint) ✅  
**Next Milestone:** M1 (TGP Message Parsing & Basic Routing) 🔄  
**Last Updated:** November 12, 2025

### Quick Status

- ✅ **Smart Contracts**: CoreProverEscrow and ReceiptVault fully implemented in Solidity 0.8.20
- ✅ **Workspace Structure**: 7 Rust crates with proper dependency management
- ✅ **Payment Profiles**: Three reference implementations (pizza delivery, digital goods, physical goods)
- 🔄 **TGP Messages**: Specification complete, implementation in progress (M1)
- 🔄 **State Machine**: Enum defined, transition logic pending (M1)
- ❌ **x402 Integration**: Planned for M3
- ❌ **TDR/SSO**: Planned for M2-M3

### Repository Structure

```
transaction-border-controller/
├── crates/
│   ├── tbc-core/              # Core gateway protocol (TGP messages)
│   ├── tbc-gateway/           # TGP router and agent coordination
│   ├── coreprover-bridge/     # Rust ↔ Solidity bindings
│   ├── coreprover-service/    # Settlement service + REST API
│   ├── coreprover-contracts/  # CoreProver escrow contracts (Foundry)
│   ├── coreprover-zk/         # ZK circuits (Circom) + provers
│   ├── coreprover-cli/        # Operator CLI
│   └── coreprover-sdk/        # Developer SDK
├── docs/                      # Complete specification & guides
├── docker/                    # Deployment configurations
└── tests/                     # Integration test suite
```

### Implementation Completeness Matrix

|Component             |Spec Status|Code Status      |Location                                              |Milestone|
|-———————|————|——————|——————————————————|———|
|**Core Messages**     |✅ Defined  |🔄 Partial        |`crates/tbc-core/src/protocol.rs`                     |M1       |
|**QUERY Message**     |✅ Defined  |❌ Not Implemented|`crates/tbc-core/src/tgp/messages.rs`                 |M1       |
|**OFFER Message**     |✅ Defined  |❌ Not Implemented|`crates/tbc-core/src/tgp/messages.rs`                 |M1       |
|**SETTLE Message**    |✅ Defined  |❌ Not Implemented|`crates/tbc-core/src/tgp/messages.rs`                 |M1       |
|**ERROR Message**     |✅ Defined  |❌ Not Implemented|`crates/tbc-core/src/tgp/messages.rs`                 |M1       |
|**State Machine**     |✅ Defined  |🔄 Partial        |`crates/tbc-core/src/protocol.rs`                     |M1       |
|**CoreProver Escrow** |✅ Defined  |✅ Implemented    |`crates/coreprover-contracts/src/CoreProverEscrow.sol`|M0 ✅     |
|**Receipt Vault**     |✅ Defined  |✅ Implemented    |`crates/coreprover-contracts/src/ReceiptVault.sol`    |M0 ✅     |
|**ZK Circuits**       |✅ Defined  |🔄 Placeholder    |`crates/coreprover-zk/circuits/ownership.circom`      |M2       |
|**Gateway/Router**    |✅ Defined  |🔄 Stub           |`crates/tbc-gateway/src/router.rs`                    |M1       |
|**Payment Profiles**  |✅ Defined  |✅ Implemented    |`crates/coreprover-service/src/profiles/templates.rs` |M0 ✅     |
|**x402 Integration**  |✅ Defined  |❌ Not Implemented|-                                                     |M3       |
|**TDR Emission**      |✅ Defined  |❌ Not Implemented|-                                                     |M2       |
|**SSO Storage**       |✅ Defined  |❌ Not Implemented|-                                                     |M2       |
|**Economic Envelope** |✅ Defined  |❌ Not Implemented|-                                                     |M1       |
|**Attribute Registry**|✅ Defined  |❌ Not Implemented|-                                                     |M3       |

——

## Table of Contents

- Abstract
- Implementation Status
- 1. Introduction
  - 1.1 Where TGP Runs
  - 1.2 Relationship to x402
  - 1.3 Design Principles
  - 1.4 Settlement Profiles
- 1. Architecture
  - 2.1 Network Topology
  - 2.2 Message Types
  - 2.3 Controller States
- 1. Message Types and Semantics
  - 3.1 QUERY Message
  - 3.2 OFFER Message
  - 3.3 SETTLE Message
  - 3.4 ERROR Message
  - 3.5 ZkProfile Enumeration
  - 3.6 EconomicEnvelope Structure
  - 3.7 SettleSource Enumeration
  - 3.8 Message Encoding
  - 3.9 Implementation Checklist
- 1. State Machine
- 1. Security Considerations
- 1. Attribute Registry
- 1. x402 Integration
- 1. Example Flows
  - 8.1 Profile A: Simple Payment via x402
  - 8.2 Profile B: Escrow Settlement via CoreProver
  - 8.3 Profile C: Pizza Delivery with Timed Release
  - 8.4 Profile D: Physical Goods with Counter-Escrow
  - 8.5 Development Roadmap Alignment
- 1. Future Extensions
- 1. References
- 1. The 10-Layer Trust Stack
- 1. TGP Info Block (TIB)
- 1. State Summary Objects (SSO)
- 1. Receipts & TDR Triplet
- 1. Policy Expression Language (PEL-0.1)
- 1. Prover Abstraction & Settlement Middleware
- Appendices A-M

——

## 1. Introduction

### 1.1 Where TGP Runs

TGP operates at the edges of transaction domains, enforcing trust-zone policies before economic settlement is permitted. It runs on gateways that may interact directly with RPC interfaces or flattened ledger data to determine settlement eligibility.

### 1.2 Relationship to x402

TGP can operate as a control-plane overlay atop x402 sessions or independently as a settlement coordination protocol. When integrated with x402:

- x402 provides the payment endpoint advertisement (HTTP 402 status)
- TGP provides policy validation and settlement path selection
- Controllers can validate or override x402-advertised contracts
- Economic envelope constraints ensure fee predictability

### 1.3 Design Principles

1. **Non-Custodial**: Controllers coordinate but never hold funds
1. **Policy-Driven**: Trust decisions based on domain attributes
1. **Auditable**: All sessions produce Transaction Detail Records
1. **Privacy-Preserving**: ZK proofs enable compliance without disclosure
1. **Multi-Chain**: Chain-agnostic settlement coordination

### 1.4 Settlement Profiles

TGP supports two primary settlement architectures, each with distinct trust models and message flows:

#### **Profile A: Direct x402 Settlement (Seller-Advertised, Optional Controller)**

In this profile, the Seller advertises escrow/CoreProver support directly in the HTTP 402 response headers (e.g., `X-Escrow-Contract: 0x742d35...`). The Buyer **may or may not** consult a TGP Controller.

##### **A1: Without Controller (Pure x402)**

The Buyer sees the 402 headers, decides to proceed directly, and submits a signed transaction to the Seller’s advertised payment endpoint:

```
┌─────────────────────────────────────────────┐
│          CONTROL PLANE (HTTP)               │
│                                             │
│  Buyer ──402──> Seller                      │
│        <──402 headers with X-Escrow-──      │
│                                             │
└─────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────┐
│          SETTLEMENT PLANE (L8)              │
│                                             │
│  Buyer ───signed tx──> CoreProver Contract  │
│                        (or direct payment)  │
└─────────────────────────────────────────────┘
```

**TGP Messages:** None (pure x402 flow).

**Implementation Status:** ❌ Not Implemented (M3 target)

**Tradeoff:** Buyer must trust the Seller or accept risk. If the Seller provided a malicious CoreProver address or disappears after payment, funds may be lost.

##### **A2: With Controller Validation (Hybrid)**

The Buyer receives a 402 with escrow metadata but consults a TGP Controller for policy validation before proceeding:

```
┌─────────────────────────────────────────────┐
│          CONTROL PLANE                      │
│                                             │
│  Buyer ──402──> Seller                      │
│        <──402 headers──                     │
│         │                                   │
│         ├──TGP.QUERY──> Controller          │
│         <──TGP.OFFER──                      │
│                                             │
└─────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────┐
│          SETTLEMENT PLANE (L8)              │
│                                             │
│  Buyer ───signed tx──> CoreProver Contract  │
│                        (validated address)  │
│         │                                   │
│         └──TGP.SETTLE──> Controller         │
│                                             │
└─────────────────────────────────────────────┘
```

**TGP Message Flow:**

A. **Buyer → Controller:** `TGP.QUERY`

- Includes `escrow_from_402: true`
- Includes `escrow_contract_from_402: “0x742d35...”`
- Buyer sets `zk_profile: OPTIONAL` or `REQUIRED`

B. **Controller → Buyer:** `TGP.OFFER`

- Validates the CoreProver contract against policy
- Returns `coreprover_contract: “0x742d35...”` (same or substituted)
- Returns `session_id: “sess-xyz”` for tracking
- Sets `zk_required: true/false` based on policy

C. **Buyer → CoreProver:** Submits Layer-8 transaction with `session_id`

D. **Buyer → Controller:** `TGP.SETTLE`

- Reports `success: true`, `layer8_tx: “0x9f2d...”`, `session_id: “sess-xyz”`
- Or Controller’s watcher auto-detects settlement

**Implementation Status:** 🔄 M1-M3 (messages in M1, x402 parsing in M3)

**Benefit:** Controller can block malicious contracts, enforce compliance, and provide telemetry without custody.

#### **Profile B: Controller-Mediated Escrow Settlement**

In this profile, the Buyer **always** consults the Controller before settlement. The Controller selects or provisions a CoreProver contract and session, potentially independent of any 402 metadata.

```
┌─────────────────────────────────────────────┐
│          CONTROL PLANE (TGP)                │
│                                             │
│  Buyer ───TGP.QUERY──> Controller           │
│        <──TGP.OFFER──                       │
│         (with session_id + contract)        │
│                                             │
└─────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────┐
│          SETTLEMENT PLANE (L8)              │
│                                             │
│  Buyer ───signed tx──> CoreProver Contract  │
│         (using Controller’s session_id)     │
│         │              │                    │
│         │              └──ack──> Seller     │
│         │                                   │
│         └──TGP.SETTLE──> Controller         │
│                                             │
└─────────────────────────────────────────────┘
```

**TGP Message Flow:**

A. **Buyer → Controller:** `TGP.QUERY`

- May include `escrow_from_402: false` (no 402 headers)
- Or `escrow_from_402: true` but Controller overrides with policy-selected contract
- Buyer sets `zk_profile: REQUIRED` to demand escrow

B. **Controller → Buyer:** `TGP.OFFER`

- Returns `coreprover_contract: “0xPolicyApproved...”`
- Returns `session_id: “sess-abc123”` for onchain routing
- Sets `zk_required: true` (enforced by policy)
- Includes `economic_envelope` with fee caps and expiry

C. **Buyer → CoreProver:** Submits Layer-8 transaction

- Includes `session_id` in transaction metadata or calldata
- Funds escrowed until Seller acknowledges or provides ZK proof

D. **Seller → CoreProver:** Acknowledges delivery (onchain or offchain signature)

- CoreProver releases funds upon valid acknowledgment

E. **Controller or Buyer → Controller:** `TGP.SETTLE`

- `source: “controller-watcher”` if Controller’s indexer detected settlement
- `source: “buyer-notify”` if Buyer explicitly reports
- `success: true/false` based on escrow outcome

**Implementation Status:** ✅ Smart contracts implemented (M0), 🔄 TGP integration in progress (M1-M2)

**Benefits:**

- Non-custodial fairness: Funds only release on delivery/proof
- Controller enforces compliance without touching funds
- Refund path if Seller never acknowledges
- Suitable for untrusted counterparties or regulated environments

#### **Settlement Profile Comparison**

|Aspect             |Profile A (Direct/Hybrid)          |Profile B (Controller-Mediated)    |
|-——————|————————————|————————————|
|**402 Required?**  |Yes (Seller advertises)            |No (Controller provisions)         |
|**Controller Role**|Optional validator                 |Required coordinator               |
|**Trust Model**    |Buyer trusts Seller or 402 metadata|Buyer trusts Controller policy     |
|**TGP Messages**   |Optional (0 or 2-3 messages)       |Required (3+ messages)             |
|**Use Case**       |Low-friction payments, trusted APIs|High-value, untrusted, or regulated|
|**Failure Risk**   |Buyer loses funds if Seller cheats |Funds escrowed, refundable         |
|**Implementation** |M3 (x402 dependent)                |M1-M2 (current focus)              |

——

## 2. Architecture

### 2.1 Network Topology

TGP is designed to operate across trust domains, enabling value-routing and policy negotiation between distinct agents, networks, and protocols. The topology includes both human participants and machine agents that mediate trust and compliance across domain boundaries.

#### TGP Topology Component Definitions

- **Buyer**: The economic initiator of a transaction. Typically originates a QUERY or ACCEPT message, provides payment, and expects delivery of a good, service, or receipt.
- **Buyer Agent**: An AI, browser extension, TBC instance, or delegated actor representing the buyer. It may handle escrow initiation, proof validation, or fulfillment verification.
- **Seller**: The economic recipient of value in exchange for delivering a product or fulfilling a service. Often responsible for confirming receipt or responding to policy-bound delivery.
- **Seller Agent**: An automated or delegated component that performs fulfillment validation, delivery tracking, or escrow interaction on behalf of the seller.
- **Gateway**: A TGP-aware process that resides at the trust boundary of a domain. It interprets TGP messages, enforces policy constraints, and facilitates routing and session handoff. In many deployments, it also acts as a facilitator or a TBC.
- **Transaction Border Controller (TBC)**: A hardened Gateway that adds rate-limiting, session logging, compliance enforcement, and protocol translation. It serves as the institutional or carrier-grade version of a Gateway.
- **Facilitator**: In x402-based flows, the facilitator acts as the payment intermediary. It may hold value temporarily or coordinate settlement between the buyer and seller without direct custody of goods. In TGP, the Gateway often serves this role.
- **Prover (Escrow Middleware)**: The TGP settlement controller. It verifies mutual acknowledgment of fulfillment before releasing escrowed funds or receipts. This component may operate as a smart contract with off-chain hooks, generating proof-of-receipt or compliance attestations. In ZK-enabled deployments, it may also validate zero-knowledge fulfillment proofs.
- **Attribute Registry**: A service or index that maps domain metadata (such as jurisdiction, compliance policies, or ledger characteristics) into policy tags or session constraints. Gateways use registries for trust evaluation and route decisions.
- **x402 Service**: A Layer 7 payment endpoint compatible with Coinbase’s x402 protocol. It receives TGP metadata, advertises price and terms, and interacts with the Gateway as part of session establishment. Optionally integrated directly into the Gateway.

——

### 2.2 Message Types

TGP defines the following message types for inter-gateway signaling:

- `QUERY`: Initiates a capability or path query
- `OFFER`: Suggests a viable route or settlement method
- `SETTLE`: Reports settlement completion
- `ERROR`: Notifies of protocol or transaction failure

These messages may be encapsulated in x402-compatible payloads or used independently across custom transport layers.

#### Implementation Status

**Current State** (`crates/tbc-core/src/protocol.rs` - M0):

```rust
// Basic enum - does not align with TGP-00 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    OrderCreate,
    OrderRoute,
    OrderUpdate,
    OrderComplete,
}
```

**Status:** ❌ Not aligned with TGP-00 specification. Basic enum exists but needs replacement.

**Target Implementation** (`crates/tbc-core/src/tgp/messages.rs` - M1):

```rust
//! TGP message types per TGP-00 specification

use serde::{Deserialize, Serialize};

/// TGP message discriminator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = “phase”)]
pub enum TGPMessage {
    #[serde(rename = “QUERY”)]
    Query(QueryMessage),
    #[serde(rename = “OFFER”)]
    Offer(OfferMessage),
    #[serde(rename = “SETTLE”)]
    Settle(SettleMessage),
    #[serde(rename = “ERROR”)]
    Error(ErrorMessage),
}

/// QUERY message structure (§3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub asset: String,
    pub amount: u64,
    pub escrow_from_402: bool,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub escrow_contract_from_402: Option<String>,
    pub zk_profile: ZkProfile,
}

/// OFFER message structure (§3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferMessage {
    pub id: String,
    pub query_id: String,
    pub asset: String,
    pub amount: u64,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub coreprover_contract: Option<String>,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub session_id: Option<String>,
    pub zk_required: bool,
    pub economic_envelope: EconomicEnvelope,
}

/// SETTLE message structure (§3.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleMessage {
    pub id: String,
    pub query_or_offer_id: String,
    pub success: bool,
    pub source: SettleSource,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub layer8_tx: Option<String>,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub session_id: Option<String>,
}

/// ERROR message structure (§3.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub id: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub correlation_id: Option<String>,
}

impl TGPMessage {
    /// Get the message ID
    pub fn id(&self) -> &str {
        match self {
            TGPMessage::Query(m) => &m.id,
            TGPMessage::Offer(m) => &m.id,
            TGPMessage::Settle(m) => &m.id,
            TGPMessage::Error(m) => &m.id,
        }
    }
    
    /// Validate message structure
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TGPMessage::Query(m) => m.validate(),
            TGPMessage::Offer(m) => m.validate(),
            TGPMessage::Settle(m) => m.validate(),
            TGPMessage::Error(m) => m.validate(),
        }
    }
}

impl QueryMessage {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() { return Err(“id required”.to_string()); }
        if self.from.is_empty() { return Err(“from required”.to_string()); }
        if self.to.is_empty() { return Err(“to required”.to_string()); }
        if self.asset.is_empty() { return Err(“asset required”.to_string()); }
        if self.amount == 0 { return Err(“amount must be > 0”.to_string()); }
        Ok(())
    }
}

// Similar validation for OfferMessage, SettleMessage, ErrorMessage...
```

**M1 Implementation Tasks:**

- [ ] Create `crates/tbc-core/src/tgp/` module
- [ ] Implement all message types with proper serde annotations
- [ ] Add validation methods for each message type
- [ ] Write JSON parsing tests (valid and invalid cases)
- [ ] Add serialization round-trip tests
- [ ] Document field requirements and constraints

——

### 2.3 Controller States

Each TGP session progresses through well-defined states. Gateways use timers and failure handling logic to resolve unresponsive or malformed messages, and may re-initiate under retry policy.

#### Implementation Status

**Current State Enum** (`crates/tbc-core/src/protocol.rs` - M0):

```rust
// Basic state enum - does not match TGP specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState {
    Initialized,
    Routing,
    Processing,
    Completed,
    Failed,
}
```

**Status:** ❌ Not aligned with TGP-00 state machine.

**Target State Machine** (`crates/tbc-core/src/tgp/state.rs` - M1):

```rust
//! TGP state machine per TGP-00 §4

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// TGP session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TGPState {
    Idle,
    QuerySent,
    OfferReceived,
    AcceptSent,
    Finalizing,
    Settled,
    Errored,
}

/// TGP session with state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TGPSession {
    pub session_id: String,
    pub state: TGPState,
    pub query_id: Option<String>,
    pub offer_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub timeout_at: Option<u64>,
}

impl TGPSession {
    /// Create a new session
    pub fn new(session_id: String) -> Self {
        let now = current_timestamp();
        Self {
            session_id,
            state: TGPState::Idle,
            query_id: None,
            offer_id: None,
            created_at: now,
            updated_at: now,
            timeout_at: None,
        }
    }
    
    /// Transition to a new state with validation
    pub fn transition(&mut self, new_state: TGPState) -> Result<(), String> {
        // Validate state transition
        if !self.is_valid_transition(new_state) {
            return Err(format!(
                “Invalid transition from {:?} to {:?}”,
                self.state, new_state
            ));
        }
        
        self.state = new_state;
        self.updated_at = current_timestamp();
        Ok(())
    }
    
    /// Check if transition is valid
    fn is_valid_transition(&self, new_state: TGPState) -> bool {
        use TGPState::*;
        match (self.state, new_state) {
            (Idle, QuerySent) => true,
            (QuerySent, OfferReceived) => true,
            (QuerySent, Errored) => true,
            (OfferReceived, AcceptSent) => true,
            (OfferReceived, Errored) => true,
            (AcceptSent, Finalizing) => true,
            (AcceptSent, Errored) => true,
            (Finalizing, Settled) => true,
            (Finalizing, Errored) => true,
            (_, Errored) => true, // Can error from any state
            _ => false,
        }
    }
    
    /// Check if session has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_at {
            current_timestamp() > timeout
        } else {
            false
        }
    }
    
    /// Set timeout deadline
    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_at = Some(current_timestamp() + seconds);
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
```

**State Transition Rules:**

|From State     |To State       |Trigger                          |
|—————|—————|———————————|
|`Idle`         |`QuerySent`    |QUERY message sent               |
|`QuerySent`    |`OfferReceived`|OFFER message received           |
|`QuerySent`    |`Errored`      |Timeout or rejection             |
|`OfferReceived`|`AcceptSent`   |Buyer accepts offer              |
|`OfferReceived`|`Errored`      |Buyer rejects offer              |
|`AcceptSent`   |`Finalizing`   |Settlement initiated (Layer-8 tx)|
|`AcceptSent`   |`Errored`      |Settlement failed                |
|`Finalizing`   |`Settled`      |SETTLE confirmation received     |
|`Finalizing`   |`Errored`      |Settlement timeout               |
|Any            |`Errored`      |ERROR message or critical failure|

**M1 Implementation Tasks:**

- [ ] Implement `TGPState` enum with all states
- [ ] Implement `TGPSession` with state transition validation
- [ ] Add timeout handling logic
- [ ] Implement session persistence (database schema - M2)
- [ ] Add state machine unit tests
- [ ] Document all valid transitions

——

## 3. Message Types and Semantics

TGP-00 Stage-1 defines four primary message types for Layer-8 control plane signaling: **QUERY**, **OFFER**, **SETTLE**, and **ERROR**. All messages are JSON-encoded with a `phase` discriminator field and share a common structure for correlation and traceability.

### 3.1 QUERY Message

Sent by a Buyer (or Buyer Agent) to a Controller/Gateway to request routing advice and settlement options. Typically initiated after receiving an HTTP 402 response with Layer-8 metadata.

#### Fields

|Field                     |Type     |Required|Description                                                                 |
|—————————|———|———|-—————————————————————————|
|`phase`                   |string   |✓       |Fixed value: `”QUERY”`                                                      |
|`id`                      |string   |✓       |Unique identifier for this query (client-generated)                         |
|`from`                    |string   |✓       |Buyer identifier (e.g., `buyer://alice`, wallet address, or agent URI)      |
|`to`                      |string   |✓       |Seller identifier (e.g., `seller://bob`, service endpoint)                  |
|`asset`                   |string   |✓       |Asset denomination (e.g., `”USDC”`, `”ETH”`, token symbol)                  |
|`amount`                  |u64      |✓       |Amount in smallest unit (e.g., wei, lamports, base units)                   |
|`escrow_from_402`         |boolean  |✓       |Whether the 402 response explicitly advertised CoreProver/escrow support    |
|`escrow_contract_from_402`|string   |optional|CoreProver contract address from 402 `X-Escrow-Contract` header (if present)|
|`zk_profile`              |ZkProfile|✓       |Buyer’s preference for ZK/CoreProver involvement (see §3.5)                 |

#### Example

```json
{
  “phase”: “QUERY”,
  “id”: “q-abc123”,
  “from”: “buyer://alice.wallet”,
  “to”: “seller://store.example”,
  “asset”: “USDC”,
  “amount”: 1000000,
  “escrow_from_402”: true,
  “escrow_contract_from_402”: “0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb”,
  “zk_profile”: “REQUIRED”
}
```

#### Validation Rules

- `id` must be unique per session
- `from` and `to` must be valid identifiers (format TBD)
- `asset` must be a recognized token symbol
- `amount` must be greater than zero
- `escrow_contract_from_402` must be a valid Ethereum address if present
- `zk_profile` must be one of: `”NONE”`, `”OPTIONAL”`, `”REQUIRED”`

——

### 3.2 OFFER Message

Sent by a Controller/Gateway in response to a QUERY. Contains routing recommendations, settlement parameters, and economic envelope constraints.

#### Fields

|Field                |Type            |Required|Description                                                 |
|———————|-—————|———|————————————————————|
|`phase`              |string          |✓       |Fixed value: `”OFFER”`                                      |
|`id`                 |string          |✓       |Unique identifier for this offer (controller-generated)     |
|`query_id`           |string          |✓       |Correlation ID linking back to the originating QUERY        |
|`asset`              |string          |✓       |Asset denomination (echoed from QUERY)                      |
|`amount`             |u64             |✓       |Amount in smallest unit (echoed from QUERY)                 |
|`coreprover_contract`|string          |optional|CoreProver escrow contract address (if escrow path selected)|
|`session_id`         |string          |optional|Unique session identifier for CoreProver onchain routing    |
|`zk_required`        |boolean         |✓       |Whether ZK/CoreProver is required under Controller policy   |
|`economic_envelope`  |EconomicEnvelope|✓       |Fee limits and validity constraints (see §3.6)              |

#### Example

```json
{
  “phase”: “OFFER”,
  “id”: “offer-abc123”,
  “query_id”: “q-abc123”,
  “asset”: “USDC”,
  “amount”: 1000000,
  “coreprover_contract”: “0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb”,
  “session_id”: “sess-abc123”,
  “zk_required”: true,
  “economic_envelope”: {
    “max_fees_bps”: 50,
    “expiry”: “2025-11-10T23:59:59Z”
  }
}
```

#### Validation Rules

- `id` must be unique
- `query_id` must reference a valid QUERY message
- `coreprover_contract` must be a valid address if present
- `session_id` should be unique and traceable to CoreProver contract
- `economic_envelope.max_fees_bps` must be between 0 and 10000 (100%)
- `economic_envelope.expiry` must be in RFC3339 format and in the future

——

### 3.3 SETTLE Message

Sent to notify the Controller that settlement has occurred. May be sent by the Buyer, an external indexer, or synthesized by the Controller’s own watcher infrastructure.

#### Fields

|Field              |Type        |Required|Description                                                    |
|-——————|————|———|—————————————————————|
|`phase`            |string      |✓       |Fixed value: `”SETTLE”`                                        |
|`id`               |string      |✓       |Unique identifier for this settlement report                   |
|`query_or_offer_id`|string      |✓       |Correlation ID (references original QUERY or OFFER)            |
|`success`          |boolean     |✓       |Whether settlement completed successfully                      |
|`source`           |SettleSource|✓       |Who reported this settlement (see §3.7)                        |
|`layer8_tx`        |string      |optional|Layer-8 transaction hash (e.g., CoreProver tx, blockchain txid)|
|`session_id`       |string      |optional|Session ID used with CoreProver (if applicable)                |

#### Example

```json
{
  “phase”: “SETTLE”,
  “id”: “settle-abc123”,
  “query_or_offer_id”: “offer-abc123”,
  “success”: true,
  “source”: “buyer-notify”,
  “layer8_tx”: “0x9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e”,
  “session_id”: “sess-abc123”
}
```

#### Validation Rules

- `id` must be unique
- `query_or_offer_id` must reference a valid QUERY or OFFER
- `source` must be one of: `”buyer-notify”`, `”controller-watcher”`, `”coreprover-indexer”`
- `layer8_tx` must be a valid transaction hash if present
- `session_id` should match the OFFER’s session_id if present

——

### 3.4 ERROR Message

Signals a protocol-level failure or policy violation during QUERY/OFFER/SETTLE processing.

#### Fields

|Field           |Type  |Required|Description                                                                |
|-—————|——|———|—————————————————————————|
|`phase`         |string|✓       |Fixed value: `”ERROR”`                                                     |
|`id`            |string|✓       |Unique identifier for this error report                                    |
|`code`          |string|✓       |Machine-readable error code (e.g., `”POLICY_VIOLATION”`, `”INVALID_ASSET”`)|
|`message`       |string|✓       |Human-readable error description                                           |
|`correlation_id`|string|optional|ID of the message that triggered this error (QUERY/OFFER/SETTLE)           |

#### Example

```json
{
  “phase”: “ERROR”,
  “id”: “err-abc123”,
  “code”: “UNSUPPORTED_ASSET”,
  “message”: “Asset DOGE not supported in this jurisdiction”,
  “correlation_id”: “q-abc123”
}
```

#### Standard Error Codes

|Code                  |Meaning                               |
|-———————|—————————————|
|`INVALID_QUERY`       |QUERY message failed validation       |
|`UNSUPPORTED_ASSET`   |Asset not supported by Controller     |
|`POLICY_VIOLATION`    |Request violates domain policy        |
|`CONTRACT_BLACKLISTED`|CoreProver contract is blacklisted    |
|`INSUFFICIENT_FUNDS`  |Buyer has insufficient balance        |
|`TIMEOUT`             |Session or operation timed out        |
|`SETTLEMENT_FAILED`   |Layer-8 transaction failed            |
|`INVALID_STATE`       |Operation not allowed in current state|

——

### 3.5 ZkProfile Enumeration

Indicates the Buyer’s preference for zero-knowledge proof and CoreProver escrow involvement:

|Value     |Meaning                                                                   |
|-———|—————————————————————————|
|`NONE`    |Buyer does not want CoreProver escrow (direct x402 preferred)             |
|`OPTIONAL`|Buyer is willing to use CoreProver if Controller recommends it            |
|`REQUIRED`|Buyer demands CoreProver escrow (e.g., for high-value or untrusted seller)|

**Serialization:** Uppercase string values (`”NONE”`, `”OPTIONAL”`, `”REQUIRED”`)

**Rust Implementation:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkProfile {
    #[serde(rename = “NONE”)]
    None,
    #[serde(rename = “OPTIONAL”)]
    Optional,
    #[serde(rename = “REQUIRED”)]
    Required,
}
```

——

### 3.6 EconomicEnvelope Structure

Encodes economic constraints for an OFFER:

#### Fields

|Field         |Type  |Required|Description                                                     |
|—————|——|———|-—————————————————————|
|`max_fees_bps`|u32   |✓       |Maximum acceptable total fees in basis points (e.g., 50 = 0.50%)|
|`expiry`      |string|optional|RFC3339 timestamp after which the offer is invalid              |

#### Example

```json
{
  “max_fees_bps”: 50,
  “expiry”: “2025-11-10T23:59:59Z”
}
```

**Rust Implementation:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicEnvelope {
    pub max_fees_bps: u32,
    #[serde(skip_serializing_if = “Option::is_none”)]
    pub expiry: Option<String>,
}

impl EconomicEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_fees_bps > 10000 {
            return Err(“max_fees_bps cannot exceed 10000 (100%)”.to_string());
        }
        
        if let Some(expiry) = &self.expiry {
            // Validate RFC3339 format
            chrono::DateTime::parse_from_rfc3339(expiry)
                .map_err(|_| “Invalid expiry format”.to_string())?;
        }
        
        Ok(())
    }
}
```

**Future Extensions:**

- `slippage_tolerance_bps`: For swap-based settlements
- `multi_asset_swaps`: Enable multiple asset exchanges
- `regulatory_constraints`: Policy flags for compliance
- `sla_commitments`: Service-level agreements

**Implementation Status:** ❌ Not yet implemented (M1 target)

——

### 3.7 SettleSource Enumeration

Indicates who is notifying the Controller about settlement:

|Value               |Meaning                                                             |
|———————|———————————————————————|
|`buyer-notify`      |Buyer (or Buyer Agent) directly reporting settlement                |
|`controller-watcher`|Controller’s own CoreProver indexer/watcher observed the transaction|
|`coreprover-indexer`|External third-party CoreProver indexer sent notification           |

**Serialization:** Kebab-case string values (`”buyer-notify”`, `”controller-watcher”`, `”coreprover-indexer”`)

**Rust Implementation:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = “kebab-case”)]
pub enum SettleSource {
    BuyerNotify,
    ControllerWatcher,
    CoreproverIndexer,
}
```

——

### 3.8 Message Encoding

Stage-1 TGP messages are encoded as **UTF-8 JSON**. The `phase` field acts as a discriminator for message type.

#### Parsing Rules

- All messages MUST contain a `phase` field
- All messages MUST contain an `id` field
- Unknown fields SHOULD be ignored (forward compatibility)
- Parsers MAY reject messages with invalid or missing required fields

#### Transport

TGP messages can be transmitted over:

- HTTP POST (JSON body)
- WebSocket frames
- Message queues (RabbitMQ, Kafka)
- gRPC streams (with JSON-to-protobuf mapping)

#### Future Encoding Options

Future stages may support:

- Binary encoding (CBOR, Protocol Buffers)
- Embedding in x402 HTTP headers (`X-TGP-Query`, `X-TGP-Offer`)
- WebSocket or gRPC transports for real-time negotiation
- Compression and batching for high-throughput scenarios

——

### 3.9 Implementation Checklist (M1 Priority)

#### Phase 1: Core Message Types (Week 1)

- [ ] Create `crates/tbc-core/src/tgp/messages.rs`
- [ ] Define `QueryMessage` struct with all fields from §3.1
- [ ] Define `OfferMessage` struct with all fields from §3.2
- [ ] Define `SettleMessage` struct with all fields from §3.3
- [ ] Define `ErrorMessage` struct with all fields from §3.4
- [ ] Implement `ZkProfile` enum with serde annotations
- [ ] Implement `EconomicEnvelope` struct
- [ ] Implement `SettleSource` enum
- [ ] Add JSON parsing tests for all message types
- [ ] Add validation functions for required fields
- [ ] Add serialization round-trip tests

#### Phase 2: State Machine (Week 1)

- [ ] Create `crates/tbc-core/src/tgp/state.rs`
- [ ] Implement `TGPSession` struct
- [ ] Implement state transition validation
- [ ] Add timeout handling
- [ ] Add state persistence (database schema in M2)
- [ ] Implement session recovery logic
- [ ] Add state machine unit tests
- [ ] Document all valid transitions

#### Phase 3: Gateway Integration (Week 2)

- [ ] Create `crates/tbc-gateway/src/tgp/handler.rs`
- [ ] Update `Router` to handle TGP messages
- [ ] Implement QUERY message handling
- [ ] Implement OFFER message generation
- [ ] Implement SETTLE message processing
- [ ] Add ERROR message generation
- [ ] Integrate with CoreProver bridge
- [ ] Add end-to-end flow tests

#### Phase 4: REST API Endpoints (Week 2)

- [ ] `POST /tgp/query` - Accept QUERY messages
- [ ] `GET /tgp/offer/:query_id` - Retrieve OFFER
- [ ] `POST /tgp/settle` - Report settlement
- [ ] `GET /tgp/session/:id` - Get session status
- [ ] Add OpenAPI/Swagger documentation
- [ ] Add rate limiting
- [ ] Add authentication (API keys)

#### Files to Create/Update

**New Files:**

```
crates/tbc-core/src/tgp/
├── messages.rs       # QueryMessage, OfferMessage, etc.
├── state.rs          # TGPSession, state machine
├── validation.rs     # Message validation logic
└── types.rs          # ZkProfile, EconomicEnvelope, etc.

crates/tbc-gateway/src/tgp/
├── handler.rs        # TGP message handlers
├── router.rs         # Route selection logic
└── session.rs        # Session management

crates/coreprover-service/src/tgp/
├── api.rs            # REST API endpoints
└── handlers.rs       # HTTP request handlers

tests/tgp/
├── message_tests.rs  # Message parsing tests
├── state_tests.rs    # State machine tests
└── integration_tests.rs  # End-to-end TGP flows
```

**Updated Files:**

```
crates/tbc-core/src/lib.rs              # Export tgp module
crates/tbc-core/src/protocol.rs         # Deprecate old MessageType
crates/tbc-gateway/src/lib.rs           # Export tgp handlers
crates/coreprover-service/src/api/routes.rs  # Add TGP routes
```

——

## 4. State Machine

Each TGP session progresses through well-defined states. The state machine ensures protocol correctness and prevents invalid transitions.

### State Diagram

```
    ┌──────┐
    │ Idle │
    └───┬──┘
        │ QUERY sent
        ▼
 ┌──────────────┐
 │ QuerySent    │
 └──┬───────────┘
    │ OFFER received
    │
    ▼
 ┌──────────────────┐
 │ OfferReceived    │
 └──┬───────────────┘
    │ ACCEPT sent
    │
    ▼
 ┌──────────────┐
 │ AcceptSent   │
 └──┬───────────┘
    │ Settlement initiated
    │
    ▼
 ┌──────────────┐
 │ Finalizing   │
 └──┬───────────┘
    │ SETTLE received
    │
    ▼
 ┌──────────┐
 │ Settled  │
 └──────────┘

   Any state ──ERROR──> Errored
```

### State Descriptions

- **Idle**: No active session, ready to initiate
- **QuerySent**: QUERY message sent, waiting for OFFER
- **OfferReceived**: OFFER received, Buyer decides whether to accept
- **AcceptSent**: Buyer accepted OFFER, settlement in progress
- **Finalizing**: Layer-8 transaction submitted, waiting for confirmation
- **Settled**: Settlement confirmed, TDR emitted
- **Errored**: Terminal error state, may retry from Idle

### Timeout Policies

|State        |Timeout   |Action                                 |
|-————|-———|—————————————|
|QuerySent    |30 seconds|Transition to Errored, emit ERROR      |
|OfferReceived|5 minutes |Transition to Errored (offer expired)  |
|Finalizing   |10 minutes|Transition to Errored, check for refund|

### Implementation

See §2.3 for complete Rust implementation with validation logic.

——

## 5. Security Considerations

TGP does not mandate encryption but recommends:

- **Transport Security**: Use of TLS or equivalent secure transport for all message exchanges
- **Message Signing**: Digital signatures using domain keys (ECDSA or EdDSA)
- **Replay Protection**: Include timestamps and nonces in messages
- **Policy Validation**: Gateways must validate OFFER contracts against blacklists
- **Rate Limiting**: Protect against DoS attacks (both Controller and Gateway)
- **ZK Proofs**: Optional ZK proofs for policy compliance without disclosure
- **Audit Logging**: All TGP sessions logged for compliance and dispute resolution

### Critical Validations

Controllers must:

1. Verify `escrow_contract_from_402` addresses against known-good contract lists
1. Reject offers with excessive fees (`max_fees_bps` > policy limit)
1. Enforce timeout policies to prevent resource exhaustion
1. Validate `layer8_tx` hashes against blockchain state before emitting TDR

Gateways must:

1. Prevent settlement spoofing by verifying `source` authenticity
1. Rate-limit QUERY and SETTLE messages per source
1. Sanitize all user-provided fields (SQL injection, XSS prevention)

——

## 6. Attribute Registry

Gateways may maintain or consult an **Attribute Registry** for:

- **Policy Domains**: Jurisdiction-specific compliance tags (e.g., `US-OFAC`, `EU-GDPR`)
- **Chain Metadata**: Ledger characteristics (finality time, gas costs, bridge availability)
- **SLA Commitments**: Availability guarantees, settlement speed promises
- **x402 Capabilities**: Min/max payment amounts, supported assets
- **Contract Whitelists**: Approved CoreProver contract addresses per jurisdiction

**Implementation Status:** ❌ Not yet implemented (M3 target)

**Planned Structure:**

```rust
pub struct AttributeRegistry {
    policies: HashMap<String, PolicyDomain>,
    contracts: HashMap<String, ContractMetadata>,
    chains: HashMap<u64, ChainMetadata>,
}

pub struct PolicyDomain {
    pub jurisdiction: String,
    pub compliance_tags: Vec<String>,
    pub allowed_assets: Vec<String>,
    pub max_fee_bps: u32,
}

pub struct ContractMetadata {
    pub address: String,
    pub chain_id: u64,
    pub audit_status: AuditStatus,
    pub deployed_at: u64,
}
```

——

## 7. x402 Integration

TGP can operate as a control-plane overlay atop x402 sessions or independently as a settlement coordination protocol.

### x402 Header Mapping

|x402 Header        |TGP Field                 |Description                   |
|-——————|—————————|——————————|
|`X-Escrow-Contract`|`escrow_contract_from_402`|CoreProver contract address   |
|`X-Payment-Asset`  |`asset`                   |Token symbol (USDC, ETH, etc.)|
|`X-Payment-Amount` |`amount`                  |Price in smallest unit        |
|`X-Session-ID`     |`session_id`              |Optional pre-assigned session |

### Integration Flow (Profile A2)

1. Buyer requests resource, receives HTTP 402
1. Buyer parses `X-Escrow-Contract` from 402 headers
1. Buyer sends TGP.QUERY with `escrow_from_402: true`
1. Controller validates contract, responds with TGP.OFFER
1. Buyer submits Layer-8 transaction to validated contract
1. Buyer or Controller sends TGP.SETTLE notification

**Implementation Status:** ❌ Not yet implemented (M3 target)

**Required Components:**

- x402 header parser (`crates/tbc-gateway/src/x402/parser.rs`)
- Contract validation service (`crates/coreprover-service/src/validation/contract.rs`)
- x402 client library integration (Coinbase SDK)

——

## 8. Example Flows

### 8.1 Profile A: Simple Payment via x402

**Implementation Status:** ❌ Not Implemented (x402 integration pending, M3)

**Planned Flow:**

1. Buyer → HTTP GET → Seller (resource request)
1. Seller → HTTP 402 → Buyer (with `X-Escrow-Contract` header)
1. Buyer → TGP.QUERY → Gateway (with `escrow_from_402: true`)
1. Gateway validates contract address
1. Gateway → TGP.OFFER → Buyer (validated contract or substitution)
1. Buyer → CoreProver contract (Layer-8 transaction)
1. Buyer → TGP.SETTLE → Gateway (success notification)
1. Gateway → TDR emission

——

### 8.2 Profile B: Escrow Settlement via CoreProver

**Implementation Status:** 🔄 Partially Implemented (contracts done, TGP integration in progress)

#### Working Contract Flow (M0)

From `crates/coreprover-contracts/test/CoreProverEscrow.t.sol`:

```solidity
function testBothCommitted() public {
    bytes32 orderId = keccak256(“order1”);
    
    // 1. Buyer creates escrow
    vm.prank(buyer);
    escrow.createEscrow{value: 0.1 ether}(
        orderId,
        seller,
        3600,      // 1 hour commitment window
        86400,     // 24 hour claim window
        false,     // No timed release
        0          // No timed release delay
    );
    
    // 2. Seller commits with legal signature
    bytes memory signature = createMockSignature();
    vm.prank(seller);
    escrow.sellerCommitSignature(
        orderId,
        signature,
        “Pizza Hut Franchise #4521”,
        “LICENSE-123456”,
        keccak256(“terms-and-conditions”)
    );
    
    // 3. Verify both committed state
    (,,,, uint8 state,,,,,) = escrow.escrows(orderId);
    assertEq(state, 3); // BOTH_COMMITTED
    
    // 4. Seller claims payment
    vm.prank(seller);
    uint256 receiptId = escrow.sellerClaimPayment(orderId);
    assertGt(receiptId, 0);
    
    // 5. Verify seller received funds
    assertEq(seller.balance, initialBalance + 0.1 ether);
}
```

#### Planned TGP Integration (M1)

```rust
// Target: crates/tbc-gateway/src/tgp/handler.rs
async fn handle_query(query: QueryMessage) -> Result<OfferMessage> {
    // 1. Validate query
    query.validate()?;
    
    // 2. Select CoreProver contract based on policy
    let contract_addr = policy_registry
        .select_coreprover(&query.asset, &query.to)?;
    
    // 3. Generate session ID
    let session_id = format!(“sess-{}”, uuid::Uuid::new_v4());
    
    // 4. Determine ZK requirements
    let zk_required = match query.zk_profile {
        ZkProfile::Required => true,
        ZkProfile::Optional => policy_registry.requires_zk(&query.to),
        ZkProfile::None => false,
    };
    
    // 5. Create economic envelope
    let economic_envelope = EconomicEnvelope {
        max_fees_bps: 50,  // 0.50% max fees
        expiry: Some(calculate_expiry(3600)), // 1 hour
    };
    
    // 6. Build offer
    let offer = OfferMessage {
        id: format!(“offer-{}”, uuid::Uuid::new_v4()),
        query_id: query.id.clone(),
        asset: query.asset,
        amount: query.amount,
        coreprover_contract: Some(contract_addr),
        session_id: Some(session_id.clone()),
        zk_required,
        economic_envelope,
    };
    
    // 7. Store session state
    let session = TGPSession {
        session_id: session_id.clone(),
        state: TGPState::OfferReceived,
        query_id: Some(query.id),
        offer_id: Some(offer.id.clone()),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        timeout_at: Some(current_timestamp() + 3600),
    };
    session_store.save(session).await?;
    
    Ok(offer)
}
```

——

### 8.3 Profile C: Pizza Delivery with Timed Release

**Implementation Status:** ✅ Smart contract implemented, 🔄 TGP integration in progress

#### Contract Flow (Working in M0)

```solidity
function testTimedRelease() public {
    bytes32 orderId = keccak256(“pizza-order”);
    
    // 1. Buyer creates escrow with timed release
    vm.prank(buyer);
    escrow.createEscrow{value: 0.03 ether}(
        orderId,
        seller,
        1800,   // 30 min commitment window
        7200,   // 2 hour claim window
        true,   // Enable timed release
        3600    // 1 hour auto-release delay
    );
    
    // 2. Seller commits with signature (no counter-escrow)
    bytes memory signature = createMockSignature();
    vm.prank(seller);
    escrow.sellerCommitSignature(
        orderId,
        signature,
        “Pizza Hut #4521”,
        “LICENSE-789”,
        keccak256(“order-details”)
    );
    
    // 3. [Pizza delivered off-chain - 25 minutes later]
    
    // 4. Fast forward past timed release
    vm.warp(block.timestamp + 5401); // > 1.5 hours total
    
    // 5. Anyone can trigger timed release
    escrow.triggerTimedRelease(orderId);
    
    // 6. Verify seller received payment automatically
    assertEq(seller.balance, initialBalance + 0.03 ether);
}
```

#### Payment Profile (M0)

From `crates/coreprover-service/src/profiles/templates.rs`:

```rust
pub fn pizza_delivery_profile() -> PaymentProfile {
    PaymentProfile {
        required_commitment_type: SellerCommitmentType::LegalSignature,
        counter_escrow_amount: 0,
        commitment_window: 1800,  // 30 minutes
        claim_window: 3600,       // 1 hour
        fulfillment_type: FulfillmentType::Service,
        requires_tracking: false,
        allows_timed_release: true,
        timed_release_delay: 3600,  // 1 hour auto-release
        payment_token: “USDC”.to_string(),
        price_in_usd: 25,
        accepts_multiple_assets: false,
    }
}
```

——

### 8.4 Profile D: Physical Goods with Counter-Escrow

**Implementation Status:** ✅ Smart contract implemented, 🔄 TGP integration in progress

#### Contract Flow (Working in M0)

```solidity
function testPhysicalGoodsCounterEscrow() public {
    bytes32 orderId = keccak256(“electronics-order”);
    
    // 1. Buyer creates escrow
    vm.prank(buyer);
    escrow.createEscrow{value: 1 ether}(
        orderId,
        seller,
        86400,   // 24 hour commitment window
        604800,  // 7 day claim window
        false,   // No timed release
        0
    );
    
    // 2. Seller commits with matching counter-escrow
    vm.prank(seller);
    escrow.sellerCommitEscrow{value: 1 ether}(orderId);
    
    // 3. Verify both committed
    (,,,, uint8 state,,,,,) = escrow.escrows(orderId);
    assertEq(state, 3); // BOTH_COMMITTED
    
    // 4. [Tracking submitted and goods delivered off-chain]
    
    // 5. Buyer claims counter-escrow back
    vm.prank(buyer);
    uint256 claimedAmount = escrow.buyerClaimCounterEscrow(orderId);
    assertEq(claimedAmount, 1 ether);
    
    // 6. Seller claims payment
    vm.prank(seller);
    escrow.sellerClaimPayment(orderId);
    
    // 7. Verify both parties got their funds
    assertEq(buyer.balance, initialBuyerBalance); // Counter-escrow returned
    assertEq(seller.balance, initialSellerBalance + 1 ether); // Payment received
}
```

#### Payment Profile (M0)

```rust
pub fn physical_goods_profile(price: u64) -> PaymentProfile {
    PaymentProfile {
        required_commitment_type: SellerCommitmentType::CounterEscrow,
        counter_escrow_amount: price as u128,  // Matches buyer payment
        commitment_window: 86400,    // 24 hours
        claim_window: 604800,        // 7 days
        fulfillment_type: FulfillmentType::Shipping,
        requires_tracking: true,
        allows_timed_release: false,
        timed_release_delay: 0,
        payment_token: “USDC”.to_string(),
        price_in_usd: price,
        accepts_multiple_assets: false,
    }
}
```

——

### 8.5 Development Roadmap Alignment

#### M0: Repo Bootstrap + Health Endpoint ✅

**Status:** Complete  
**Deliverables:**

- ✅ Workspace configuration (`Cargo.toml`)
- ✅ All 7 crates scaffolded
- ✅ REST API health endpoint (`/health`)
- ✅ Docker configurations
- ✅ CI/CD workflows
- ✅ CoreProverEscrow.sol fully implemented and tested
- ✅ ReceiptVault.sol fully implemented and tested
- ✅ Payment profile templates (pizza, digital, physical)
- ✅ Basic documentation structure

#### M1: TGP Message Parsing & Basic Routing 🔄

**Status:** In Progress (Weeks 1-2)  
**Target Date:** November 26, 2025

**Deliverables:**

- [ ] Complete TGP message types (QUERY, OFFER, SETTLE, ERROR)
- [ ] State machine implementation with transition validation
- [ ] Basic gateway routing logic
- [ ] CoreProver contract integration via Rust bridge
- [ ] Message validation tests (100+ test cases)
- [ ] Integration test suite (3 scenarios: pizza, digital, physical)
- [ ] REST API endpoints (`/tgp/query`, `/tgp/offer/:id`, `/tgp/settle`)

**Critical Path:**

1. **Days 1-3**: Implement message types in `tbc-core`

- Define all structs with serde annotations
- Add validation methods
- Write unit tests

1. **Days 4-5**: Implement state machine

- `TGPSession` struct with transition logic
- Timeout handling
- State persistence layer

1. **Days 6-8**: Gateway integration

- QUERY handler
- OFFER generator
- SETTLE processor
- Error handling

1. **Days 9-10**: Integration tests

- Pizza delivery flow
- Digital goods flow
- Physical goods flow

#### M2: CoreProver Escrow & Proof Logic 🔄

**Status:** Smart contracts complete, Rust integration in progress  
**Target Date:** December 10, 2025

**Deliverables:**

- ✅ CoreProverEscrow.sol deployed and tested
- ✅ ReceiptVault.sol deployed and tested
- [ ] Rust bindings generation (via ethers-rs abigen)
- [ ] Event listener implementation
- [ ] TDR (Transaction Detail Record) emission
- [ ] SSO (State Summary Objects) storage
- [ ] ZK circuit production implementation
- [ ] Database schema for sessions, TDRs, SSOs

**Dependencies:**

- M1 message types (for TDR structure)
- PostgreSQL schema design
- Event indexer infrastructure

**Database Schema (Draft):**

```sql
— TGP Sessions
CREATE TABLE tgp_sessions (
    session_id VARCHAR(64) PRIMARY KEY,
    state VARCHAR(20) NOT NULL,
    query_id VARCHAR(64),
    offer_id VARCHAR(64),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    timeout_at BIGINT
);

— Transaction Detail Records
CREATE TABLE tdr_triplets (
    id SERIAL PRIMARY KEY,
    session_id VARCHAR(64) NOT NULL,
    control_plane_receipt JSONB NOT NULL,
    settlement_receipt JSONB NOT NULL,
    application_receipt JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

— State Summary Objects
CREATE TABLE state_summaries (
    session_id VARCHAR(64) PRIMARY KEY,
    state VARCHAR(20) NOT NULL,
    snapshot JSONB NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW()
);
```

#### M3: Multi-Chain Routing & x402 Integration 📅

**Status:** Planned (Weeks 5-6)  
**Target Date:** December 24, 2025

**Deliverables:**

- [ ] x402 header parsing (`X-Escrow-Contract`, etc.)
- [ ] x402 payment endpoint integration
- [ ] Base chain support (in addition to PulseChain)
- [ ] Cross-chain message passing
- [ ] Attribute registry implementation
- [ ] Settlement profile A1/A2 implementation
- [ ] Policy expression language (PEL-0.1) parser

**Blockers:**

- M1 message types must be stable
- M2 CoreProver bridge must be functional
- Coinbase x402 SDK integration

#### M4: Production-Grade Appliance 📅

**Status:** Planned (Weeks 7-8)  
**Target Date:** January 7, 2026

**Deliverables:**

- [ ] Security audit (external firm)
- [ ] Performance optimization (1000+ TPS target)
- [ ] Monitoring & alerting (Prometheus + Grafana)
- [ ] Complete API documentation (OpenAPI 3.0)
- [ ] Deployment runbooks
- [ ] Load testing results
- [ ] Incident response playbook
- [ ] Rate limiting and DDoS protection
- [ ] Multi-region deployment guide

——

## 9. Future Extensions

TGP is designed to accommodate:

- **Multi-Hop Settlement**: Route through multiple gateways for cross-domain transactions
- **Pseudonymous Agents**: AI-driven negotiation with privacy preservation
- **Localized Compliance**: Jurisdiction-specific overlays without forking protocol
- **ZK Audit Trails**: Zero-knowledge proofs of compliance for regulatory review
- **Dispute Resolution**: Arbitration hooks for contested settlements
- **Multi-Asset Swaps**: Atomic exchange of multiple tokens via CoreProver
- **Streaming Payments**: Micropayments with periodic settlement
- **DAO Governance**: Decentralized governance of attribute registries and policies

——

## 10. References

- [x402 Protocol](https://github.com/coinbase/x402) - HTTP 402 payment protocol
- [TxIP-00 Spec](https://github.com/LedgerofEarth/txip) - Transaction identification protocol
- [CoreProver Contracts](https://github.com/LedgerofEarth/coreprove) - Dual-commitment escrow
- [PEL-0.1 Spec](#15-policy-expression-language-pel-01) - Policy expression language
- [EIP-4337](https://eips.ethereum.org/EIPS/eip-4337) - Account abstraction
- [ZK-SNARKs](https://z.cash/technology/zksnarks/) - Zero-knowledge proofs

——

## 11. The 10-Layer Trust Stack (Informative)

TGP operates within a broader trust architecture:

```
Layer 10: Policy (Regulatory, Legal Frameworks)
         ↓
Layer 9 : Identity (Agent, Org, Wallet Reputation)
         ↓
Layer 8 : Economic (Ledger State, TGP Messages) ← TGP OPERATES HERE
         ↓
Layer 7 : Application (Service-Specific Logic, x402)
         ↓
Layer 6 : Presentation (Encoding, Formatting)
         ↓
Layer 5 : Session (TGP/x402 Negotiation State)
         ↓
Layer 4 : Transport (QUIC, TCP, WebSocket)
         ↓
Layer 3 : Network (IP Addressing, Routing)
         ↓
Layer 2 : Data Link (MAC, Ethernet, Wi-Fi)
         ↓
Layer 1 : Physical (Wires, Waves, Silicon)
```

TGP bridges Layers 7-9 by providing economic settlement coordination with identity and policy awareness.

——

## 12. TGP Info Block (TIB)

The **TGP Info Block (TIB)** encodes Layer 8-10 context in a compact, machine-readable format. TIBs are included in QUERY and OFFER messages to communicate trust metadata.

### Structure

```rust
pub struct TIB {
    // Layer 8: Economic
    pub chain_id: u64,
    pub ledger_state_hash: String,
    pub block_number: u64,
    
    // Layer 9: Identity
    pub agent_id: String,
    pub domain_id: String,
    pub wallet_type: String,
    pub reputation_score: Option<u32>,
    
    // Layer 10: Policy
    pub policy_hash: String,
    pub compliance_tags: Vec<String>,
    pub jurisdiction: String,
}
```

### Example

```json
{
  “chain_id”: 369,
  “ledger_state_hash”: “0xabc123...”,
  “block_number”: 12345678,
  “agent_id”: “agent://alice-ai”,
  “domain_id”: “example.com”,
  “wallet_type”: “gnosis-safe”,
  “reputation_score”: 95,
  “policy_hash”: “0xdef456...”,
  “compliance_tags”: [“KYC”, “OFAC”, “GDPR”],
  “jurisdiction”: “US”
}
```

**Implementation Status:** ❌ Not yet implemented (M3 target)

——

## 13. State Summary Objects (SSO)

**State Summary Objects (SSOs)** provide a snapshot of the current state of a TGP session at any point in its lifecycle. They enable session rehydration, distributed state reconciliation, and audit trail reconstruction without requiring full message replay.

### Purpose

SSOs serve three primary functions:

1. **Session Recovery**: Allow gateways or agents to resume interrupted sessions by reconstructing the last known valid state
1. **Distributed Coordination**: Enable multi-gateway architectures where session state must be shared or synchronized across trust boundaries
1. **Audit and Compliance**: Provide compact, verifiable checkpoints for regulatory review or dispute resolution

### Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshotObject {
    pub session_id: String,
    pub state: TGPState,
    pub query_id: Option<String>,
    pub offer_id: Option<String>,
    pub timestamp: u64,
    pub participants: Participants,
    pub settlement_path: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participants {
    pub buyer: String,
    pub seller: String,
    pub controller: String,
}
```

### Example

```json
{
  “session_id”: “sess-abc123”,
  “state”: “Finalizing”,
  “query_id”: “q-abc123”,
  “offer_id”: “offer-abc123”,
  “timestamp”: 1699737600,
  “participants”: {
    “buyer”: “buyer://alice.wallet”,
    “seller”: “seller://store.example”,
    “controller”: “controller://gateway.tbc”
  },
  “settlement_path”: “coreprover”,
  “checksum”: “sha256:9f2d8e7c3b1a5f4e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e”
}
```

### Usage

SSOs are typically emitted by Gateways or Controllers at key transitions:

- QUERY received → SSO with `state: “QuerySent”`
- OFFER sent → SSO with `state: “OfferReceived”`
- Settlement detected → SSO with `state: “Settled”`

SSOs may be stored in:

- Append-only logs
- Distributed ledgers (IPFS, Arweave)
- PostgreSQL with versioning
- Redis for fast recovery

**Implementation Status:** ❌ Not yet implemented (M2 target)

**Database Schema:**

```sql
CREATE TABLE state_summaries (
    session_id VARCHAR(64) PRIMARY KEY,
    state VARCHAR(20) NOT NULL,
    snapshot JSONB NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW(),
    version INT NOT NULL DEFAULT 1
);

CREATE INDEX idx_state_summaries_timestamp ON state_summaries(timestamp);
CREATE INDEX idx_state_summaries_state ON state_summaries(state);
```

——

## 14. Receipts & TDR Triplet (Informative)

Every completed TGP session produces a **Transaction Detail Record (TDR)** triplet consisting of three layered receipts that together provide cryptographic proof of transaction completion, policy compliance, and application-level fulfillment.

### The TDR Triplet

1. **Control-Plane Receipt**: Records the TGP negotiation and session metadata (QUERY, OFFER, settlement confirmation). Anchored by the Gateway or Controller.
1. **Settlement Receipt**: Provides cryptographic proof of on-chain or off-chain value transfer. May be a blockchain transaction hash, x402 payment confirmation, or CoreProver escrow release signature.
1. **Application Receipt**: Captures fulfillment-specific evidence (download link delivered, shipping confirmation, ZK proof of delivery). Generated by the Seller or Seller Agent.

### Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDRTriplet {
    pub session_id: String,
    pub control_plane_receipt: ControlPlaneReceipt,
    pub settlement_receipt: SettlementReceipt,
    pub application_receipt: ApplicationReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneReceipt {
    pub query_id: String,
    pub offer_id: String,
    pub settle_id: String,
    pub timestamp: u64,
    pub gateway_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementReceipt {
    pub layer8_tx: String,
    pub block_number: u64,
    pub confirmation_count: u32,
    pub prover_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationReceipt {
    pub fulfillment_proof: String,
    pub delivery_timestamp: u64,
    pub seller_signature: String,
    pub metadata_uri: Option<String>,
}
```

### Example

```json
{
  “session_id”: “sess-abc123”,
  “control_plane_receipt”: {
    “query_id”: “q-abc123”,
    “offer_id”: “offer-abc123”,
    “settle_id”: “settle-abc123”,
    “timestamp”: 1699737600,
    “gateway_signature”: “0xsig...”
  },
  “settlement_receipt”: {
    “layer8_tx”: “0x9f2d8e7c...”,
    “block_number”: 12345678,
    “confirmation_count”: 12,
    “prover_signature”: “0xsig...”
  },
  “application_receipt”: {
    “fulfillment_proof”: “tracking:USPS-123456789”,
    “delivery_timestamp”: 1699740000,
    “seller_signature”: “0xsig...”,
    “metadata_uri”: “ipfs://Qm...”
  }
}
```

### Usage

TDR triplets are:

- Stored in Gateway audit logs
- Submitted to compliance registries
- Anchored on-chain for immutable proof
- Used in dispute scenarios to demonstrate:
  - What was agreed (control-plane)
  - What was paid (settlement)
  - What was delivered (application)

**Implementation Status:** ❌ Not yet implemented (M2 target)

**Database Schema:**

```sql
CREATE TABLE tdr_triplets (
    id SERIAL PRIMARY KEY,
    session_id VARCHAR(64) NOT NULL UNIQUE,
    control_plane_receipt JSONB NOT NULL,
    settlement_receipt JSONB NOT NULL,
    application_receipt JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    anchored BOOLEAN DEFAULT FALSE,
    anchor_tx VARCHAR(66)
);

CREATE INDEX idx_tdr_session ON tdr_triplets(session_id);
CREATE INDEX idx_tdr_created ON tdr_triplets(created_at);
```

——

## 15. Policy Expression Language (PEL-0.1)

**PEL-0.1** is a structured JSON format for describing compliance policies, regulatory constraints, and trust requirements in TGP sessions.

### Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyExpression {
    pub version: String,  // “PEL-0.1”
    pub jurisdiction: String,
    pub requires: Vec<String>,
    pub exemptions: Vec<String>,
    pub constraints: Constraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub max_amount_usd: Option<u64>,
    pub min_amount_usd: Option<u64>,
    pub allowed_assets: Vec<String>,
    pub blocked_assets: Vec<String>,
    pub delivery_promise: Option<String>,
    pub zk_required: bool,
}
```

### Example

```json
{
  “version”: “PEL-0.1”,
  “jurisdiction”: “US”,
  “requires”: [“KYC”, “OFAC”],
  “exemptions”: [“NFT under $500”, “Charity donations”],
  “constraints”: {
    “max_amount_usd”: 10000,
    “min_amount_usd”: 1,
    “allowed_assets”: [“USDC”, “ETH”, “WBTC”],
    “blocked_assets”: [“USDT”],
    “delivery_promise”: “72h”,
    “zk_required”: false
  }
}
```

### Usage

Controllers evaluate incoming QUERY messages against PEL policies:

```rust
pub fn evaluate_policy(
    query: &QueryMessage,
    policy: &PolicyExpression
) -> Result<bool, String> {
    // Check asset allowed
    if !policy.constraints.allowed_assets.contains(&query.asset) {
        return Err(format!(“Asset {} not allowed”, query.asset));
    }
    
    // Check amount constraints
    let amount_usd = convert_to_usd(&query.asset, query.amount)?;
    if let Some(max) = policy.constraints.max_amount_usd {
        if amount_usd > max {
            return Err(format!(“Amount exceeds max {}”, max));
        }
    }
    
    // Check ZK requirements
    if policy.constraints.zk_required && query.zk_profile == ZkProfile::None {
        return Err(“ZK proof required by policy”.to_string());
    }
    
    Ok(true)
}
```

**Implementation Status:** ❌ Not yet implemented (M3 target)

——

## 16. Prover Abstraction & Settlement Middleware (Informative)

TGP does not mandate a specific implementation of the **CoreProver** escrow system. Instead, it defines an abstract interface that settlement middleware must satisfy to enable non-custodial, trust-minimized exchange of value.

### What is CoreProver?

CoreProver refers to any settlement middleware component—typically a smart contract with off-chain coordination hooks—that:

1. **Escrows funds or assets** submitted by the Buyer
1. **Waits for proof of fulfillment** from the Seller (on-chain acknowledgment, off-chain signature, or ZK proof)
1. **Releases funds or refunds** based on fulfillment status and timeout policies
1. **Generates receipts** for the settlement outcome

### Interface Requirements

A TGP-compliant CoreProver implementation MUST support:

- **Session binding**: Accept a `session_id` from TGP.OFFER to correlate on-chain activity with control-plane negotiation
- **Timeout handling**: Enforce deadlines for fulfillment acknowledgment and trigger refunds on expiry
- **Receipt generation**: Emit settlement receipts compatible with the TDR triplet structure (§14)

Implementations MAY optionally support:

- Zero-knowledge proof verification (for privacy-preserving fulfillment)
- Multi-party escrow (e.g., buyer + seller + arbiter)
- Cross-chain settlement via bridges or atomic swaps

### Reference Implementation (M0)

The current implementation in `crates/coreprover-contracts/src/CoreProverEscrow.sol` provides:

```solidity
interface ICoreProverEscrow {
    function createEscrow(
        bytes32 orderId,
        address seller,
        uint256 commitmentWindow,
        uint256 claimWindow,
        bool allowsTimedRelease,
        uint256 timedReleaseDelay
    ) external payable returns (bytes32);
    
    function sellerCommitEscrow(bytes32 orderId) external payable;
    
    function sellerCommitSignature(
        bytes32 orderId,
        bytes memory signature,
        string memory businessName,
        string memory businessLicense,
        bytes32 documentHash
    ) external;
    
    function sellerClaimPayment(bytes32 orderId) external returns (uint256);
    
    function buyerClaimCounterEscrow(bytes32 orderId) external returns (uint256);
    
    function triggerTimedRelease(bytes32 orderId) external;
    
    function refundBuyerTimeout(bytes32 orderId) external;
}
```

### Deployment Flexibility

CoreProver middleware can be deployed as:

- **EVM Smart Contract**: Ethereum, Base, Arbitrum, PulseChain (current implementation)
- **Solana Program**: With off-chain indexer coordination
- **Federated Service**: With cryptographic attestations
- **Threshold-Signature Custodian**: With policy enforcement

TGP Gateways and Controllers validate CoreProver contracts against policy registries but do not constrain their internal implementation. This abstraction enables innovation in settlement mechanisms while maintaining interoperability at the TGP message layer.

——

## Appendices

### Appendix A: TAI – Transaction Area Identifier

`TGP-Appendix-A-TAI.md`

Defines the schema for representing and matching Transaction Areas in gateway policy lookups.

### Appendix B: CoreProver Reference

`TGP-Appendix-CoreProver-Reference-E.md`

Describes the CoreProver escrow settlement topology used as an alternative to x402.

**Implementation Status:** ✅ Complete (see §16 and `crates/coreprover-contracts/`)

### Appendix C: ZKB-01 – Zero Knowledge Buyer Proof

`ZKB-01-ZK-Buyer-Proof.md`

Formal circuit for proving buyer control of a receipt address without revealing wallet.

**Implementation Status:** 🔄 Placeholder circuit (see `crates/coreprover-zk/circuits/ownership.circom`)

### Appendix D: ZKS-01 – Zero Knowledge Seller Proof

`ZKS-01-ZK-Seller-Proof.md`

Formal circuit for proving seller ownership of delivery address or escrow destination.

### Appendix E: ZKA – ZK Aggregator Registry

`TGP-Appendix-ZK-Aggregator-Reference-Appendix.md`

Defines the structure for aggregators who register zk proof verifiers.

### Appendix F: ZKB – Buyer ZK Reference Notes

`TGP-Appendix-ZK-Buyer-Reference-Appendix-F.md`

Practical reference materials and constraints used in ZKB-01 implementation.

### Appendix G: ZKS – Seller ZK Reference Notes

`TGP-Appendix-ZK-Seller-Reference-Appendix-G.md`

Reference implementation and assumptions used in ZKS-01.

### Appendix H: Combined Buyer & Seller Reference

`TGP-Appendix-ZK-Buyer-and-Seller-Reference-Appendix.md`

Joint appendix summarizing both ZKB and ZKS systems with schema links.

### Appendix I: ZKR – ZK Receipts and Anchor Proofs

`TGP-Appendix-ZK-Recipts-Reference-Appendix.md`

Describes the receipt system, anchoring ZK proof of fulfillment or delivery.

### Appendix J: Terminology

**Key Terms:**

- **TA (Transaction Area)**: Geographic or regulatory zone
- **TZ (Trust Zone)**: Domain with unified policy enforcement
- **TDR (Transaction Detail Record)**: Complete audit trail triplet
- **TIB (TGP Info Block)**: Layer 8-10 context metadata
- **PEL (Policy Expression Language)**: Structured policy format
- **SSO (State Summary Object)**: Session state snapshot

### Appendix K: Revision History

- **v0.1-draft** (November 12, 2025): Fully aligned to canonical 10-layer trust stack and updated settlement architectures. Added implementation status tracking.
- **v0.0-initial**: Early drafts treating TGP as Layer 8.5 (superseded)

### Appendix L: Deprecation Note

Supersedes early drafts treating TGP as Layer 8.5 or solely dependent on x402 for finality.

——

## Appendix M: Implementation Reference

### Crate-to-Specification Mapping

|TGP Component         |Specification Section|Implementation Location                               |Status       |Milestone|
|-———————|———————|——————————————————|-————|———|
|**Message Types**     |§3.1-3.4             |`crates/tbc-core/src/tgp/messages.rs`                 |🔄 In Progress|M1       |
|**State Machine**     |§4                   |`crates/tbc-core/src/tgp/state.rs`                    |🔄 In Progress|M1       |
|**Gateway/Router**    |§2.1                 |`crates/tbc-gateway/src/tgp/handler.rs`               |🔄 In Progress|M1       |
|**CoreProver Escrow** |§1.4, §16, Appendix B|`crates/coreprover-contracts/src/CoreProverEscrow.sol`|✅ Complete   |M0       |
|**Receipt Vault**     |§14                  |`crates/coreprover-contracts/src/ReceiptVault.sol`    |✅ Complete   |M0       |
|**ZK Circuits**       |Appendix C           |`crates/coreprover-zk/circuits/ownership.circom`      |🔄 Placeholder|M2       |
|**Payment Profiles**  |§1.4                 |`crates/coreprover-service/src/profiles/templates.rs` |✅ Complete   |M0       |
|**Economic Envelope** |§3.6                 |`crates/tbc-core/src/tgp/types.rs`                    |❌ Not Started|M1       |
|**TDR Emission**      |§14                  |`crates/coreprover-service/src/tdr/emitter.rs`        |❌ Not Started|M2       |
|**SSO Storage**       |§13                  |`crates/coreprover-service/src/sso/storage.rs`        |❌ Not Started|M2       |
|**x402 Integration**  |§7                   |`crates/tbc-gateway/src/x402/`                        |❌ Not Started|M3       |
|**Attribute Registry**|§6                   |`crates/tbc-core/src/policy/registry.rs`              |❌ Not Started|M3       |
|**Policy Expression** |§15                  |`crates/tbc-core/src/policy/pel.rs`                   |❌ Not Started|M3       |
|**TIB Encoding**      |§12                  |`crates/tbc-core/src/tgp/tib.rs`                      |❌ Not Started|M2-M3    |

### Smart Contract Addresses

**Testnet Deployments:**

```
Network: PulseChain Testnet v4
CoreProverEscrow: [Pending M1 Deployment]
ReceiptVault:     [Pending M1 Deployment]

Network: Base Sepolia
CoreProverEscrow: [Planned for M3]
ReceiptVault:     [Planned for M3]
```

**Mainnet Deployments:**

```
Network: PulseChain Mainnet
CoreProverEscrow: [Planned for M4]
ReceiptVault:     [Planned for M4]

Network: Base Mainnet
CoreProverEscrow: [Planned for M4]
ReceiptVault:     [Planned for M4]
```

### Running Tests

```bash
# All Rust unit tests
cargo test —workspace

# Specific crate tests
cargo test -p tbc-core
cargo test -p tbc-gateway
cargo test -p coreprover-service

# Solidity contract tests
cd crates/coreprover-contracts
forge test -vvv

# Integration tests (requires running services)
./scripts/setup-dev.sh
cargo test —test integration_tests

# TGP message tests (M1+)
cargo test -p tbc-core —test tgp_message_tests

# State machine tests (M1+)
cargo test -p tbc-core —test tgp_state_tests

# End-to-end flow tests (M1+)
cargo test —test test_pizza_delivery_flow
cargo test —test test_digital_goods_flow
cargo test —test test_physical_goods_flow
```

### Developer Quick Start

```bash
# 1. Clone repository
git clone <repo-url>
cd transaction-border-controller

# 2. Install dependencies
rustup update
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 3. Build workspace
cargo build —workspace

# 4. Build contracts
cd crates/coreprover-contracts
forge build

# 5. Start local blockchain
anvil —port 8545 &

# 6. Deploy contracts
forge script script/Deploy.s.sol \
  —rpc-url http://localhost:8545 \
  —broadcast

# 7. Start services
cd ../..
docker-compose -f docker/docker-compose.dev.yml up -d

# 8. Start CoreProver service
cargo run -p coreprover-service

# 9. Test TGP flow (M1+)
curl -X POST http://localhost:3000/tgp/query \
  -H “Content-Type: application/json” \
  -d ‘{
    “phase”: “QUERY”,
    “id”: “test-query-1”,
    “from”: “buyer://alice”,
    “to”: “seller://bob”,
    “asset”: “USDC”,
    “amount”: 1000000,
    “escrow_from_402”: false,
    “zk_profile”: “OPTIONAL”
  }’
```

### OpenAPI Specification

**Status:** ❌ Not yet generated (M1 target)

**Planned Endpoints:**

```yaml
openapi: 3.0.3
info:
  title: TGP API
  version: 0.1.0
  description: Transaction Gateway Protocol REST API

paths:
  /tgp/query:
    post:
      summary: Submit TGP QUERY
      requestBody:
        content:
          application/json:
            schema:
              $ref: ‘#/components/schemas/QueryMessage’
      responses:
        ‘200’:
          description: OFFER returned
          content:
            application/json:
              schema:
                $ref: ‘#/components/schemas/OfferMessage’
        ‘400’:
          description: Invalid QUERY
          content:
            application/json:
              schema:
                $ref: ‘#/components/schemas/ErrorMessage’

  /tgp/settle:
    post:
      summary: Report settlement
      requestBody:
        content:
          application/json:
            schema:
              $ref: ‘#/components/schemas/SettleMessage’
      responses:
        ‘200’:
          description: Settlement acknowledged
```

——

## Document Control

**Version:** 0.1-draft  
**Last Updated:** November 12, 2025  
**Status:** Living Specification  
**Implementation Milestone:** M0 Complete, M1 In Progress

**Authors:**

- TBC Core Team
- CoreProver Contributors

**License:** MIT OR Apache-2.0

**Repository:** <https://github.com/yourusername/transaction-border-controller>

**Feedback:** Submit issues or pull requests to the repository

——

**END OF SPECIFICATION**
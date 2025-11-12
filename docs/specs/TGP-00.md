# TGP-00: Transaction Gateway Protocol

## Abstract

The Transaction Gateway Protocol (**TGP-00**) defines a metadata signaling layer (**Layer 8**) that enables compliant, cross-boundary transaction routing in blockchain-based networks. It facilitates **trust-aware session coordination** between wallets, gateways, and AI agents operating across jurisdictions, identity systems, and regulatory zones.

TGP operates at Layer 8 — the economic layer — alongside the blockchain’s ledgers and distributed databases. It interacts directly with RPC endpoints or flattened ledger data to inform economic-layer routing and compliance decisions. It sits beneath identity (Layer 9) and policy (Layer 10) systems.

TGP supports both **direct settlement paths** (e.g. via x402) and **non-custodial swap settlement** through **CoreProver escrow contracts**. These escrow contracts facilitate safe exchange of value for value (e.g. tokens for tokens) or value for verifiable output (e.g. digital receipts, download links, or external delivery confirmation). The `zk_profile` field in TGP.QUERY indicates the Buyer’s preference for ZK involvement, while `zk_required` in TGP.OFFER reflects the Controller’s policy decision. CoreProver escrow can function with or without ZK proofs, using onchain acknowledgments or offchain signatures as settlement triggers.

All accepted sessions result in emission of a **Transaction Detail Record (TDR)**, enabling traceable, auditable, and policy-compliant transaction flows without revealing sensitive user data. TGP is designed for compatibility with **AI-driven agents**, **cross-chain smart contracts**, and **federated compliance registries**, and serves as a foundational component of the emerging Layer 8–10 trust stack.

## Table of Contents

- Abstract
- 1. Introduction
  - 1.1 Where TGP Runs
  - 1.2 Relationship to x402
  - 1.3 Design Principles
- 2 Architecture
  - 2.1 Network Topology
  - 2.2 Message Types
  - 2.3 Controller States
  - 2.4 Settlement Profiles
- 3. Message Types and Semantics
  - 3.1 QUERY Message
  - 3.2 OFFER Message
  - 3.3 SETTLE Message
  - 3.4 ERROR Message
  - 3.5 ZkProfile Enumeration
  - 3.6 EconomicEnvelope Structure
  - 3.7 SettleSource Enumeration
  - 3.8 Message Encoding
- 4. State Machine
- 5. Security Considerations
- 6. Attribute Registry
- 7. x402 Integration
- 8. Example Flows
- 9. Future Extensions
- 10. References
- 11. The 10-Layer Trust Stack (Informative)
- 12. TGP Info Block (TIB)
- 13. Policy Expression Language (PEL-0.1)
- 14. State Summary Objects (SSO)
- 15. Receipts & TDR Triplet (Informative)
- 16. Prover Abstraction & Settlement Middleware (Informative)
- Appendices A-I

——

## 1. Introduction

### 1.1 Where TGP Runs

TGP operates at the edges of transaction domains, enforcing trust-zone policies before economic settlement is permitted. It runs on gateways that may interact directly with RPC interfaces or flattened ledger data to determine settlement eligibility.


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

## 2.2 Message Types

TGP defines the following message types for inter-gateway signaling:

- `QUERY`: Initiates a capability or path query
- `OFFER`: Suggests a viable route or settlement method
- `ACCEPT`: Confirms a proposed route or agreement
- `FINAL`: Signals readiness for finalization
- `RECEIPT`: Confirms successful delivery or transfer
- `REJECT`: Denies or aborts the proposed action
- `ERROR`: Notifies of protocol or transaction failure

These messages may be encapsulated in x402-compatible payloads or used independently across custom transport layers.

## 2.3 Controller States

Each TGP session progresses through well-defined states:

1. `Idle`
2. `QuerySent`
3. `OfferReceived`
4. `AcceptSent`
5. `Finalizing`
6. `Settled`
7. `Errored`

Transaction Controllers use timers and failure handling logic to resolve unresponsive or malformed messages, and may re-initiate under retry policy.


——

### 1.4 Settlement Profiles

TGP supports two primary settlement architectures, each with distinct trust models and message flows:

——

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

**Tradeoff:** Buyer must trust the Seller or accept risk. If the Seller provided a malicious CoreProver address or disappears after payment, funds may be lost.

——

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

**Benefit:** Controller can block malicious contracts, enforce compliance, and provide telemetry without custody.

——

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

**Benefits:**

- Non-custodial fairness: Funds only release on delivery/proof
- Controller enforces compliance without touching funds
- Refund path if Seller never acknowledges
- Suitable for untrusted counterparties or regulated environments

——

#### **Settlement Profile Comparison**

|Aspect             |Profile A (Direct/Hybrid)          |Profile B (Controller-Mediated)    |
|-——————|————————————|————————————|
|**402 Required?**  |Yes (Seller advertises)            |No (Controller provisions)         |
|**Controller Role**|Optional validator                 |Required coordinator               |
|**Trust Model**    |Buyer trusts Seller or 402 metadata|Buyer trusts Controller policy     |
|**TGP Messages**   |Optional (0 or 2-3 messages)       |Required (3+ messages)             |
|**Use Case**       |Low-friction payments, trusted APIs|High-value, untrusted, or regulated|
|**Failure Risk**   |Buyer loses funds if Seller cheats |Funds escrowed, refundable         |

——

## 2. Message Types and Semantics

TGP-00 Stage-1 defines three primary message types for Layer-8 control plane signaling: **QUERY**, **OFFER**, and **SETTLE**. Additionally, an **ERROR** message type handles exceptional conditions.

All messages are JSON-encoded with a `phase` discriminator field and share a common structure for correlation and traceability.

——

### 2.1 QUERY Message

Sent by a Buyer (or Buyer Agent) to a Controller/Gateway to request routing advice and settlement options. Typically initiated after receiving an HTTP 402 response with Layer-8 metadata.

#### **Fields**

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
|`zk_profile`              |ZkProfile|✓       |Buyer’s preference for ZK/CoreProver involvement (see §2.5)                 |

#### **Example**

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

——

### 2.2 OFFER Message

Sent by a Controller/Gateway in response to a QUERY. Contains routing recommendations, settlement parameters, and economic envelope constraints.

#### **Fields**

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
|`economic_envelope`  |EconomicEnvelope|✓       |Fee limits and validity constraints (see §2.6)              |

#### **Example**

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

——

### 2.3 SETTLE Message

Sent to notify the Controller that settlement has occurred. May be sent by the Buyer, an external indexer, or synthesized by the Controller’s own watcher infrastructure.

#### **Fields**

|Field              |Type        |Required|Description                                                    |
|-——————|————|———|—————————————————————|
|`phase`            |string      |✓       |Fixed value: `”SETTLE”`                                        |
|`id`               |string      |✓       |Unique identifier for this settlement report                   |
|`query_or_offer_id`|string      |✓       |Correlation ID (references original QUERY or OFFER)            |
|`success`          |boolean     |✓       |Whether settlement completed successfully                      |
|`source`           |SettleSource|✓       |Who reported this settlement (see §2.7)                        |
|`layer8_tx`        |string      |optional|Layer-8 transaction hash (e.g., CoreProver tx, blockchain txid)|
|`session_id`       |string      |optional|Session ID used with CoreProver (if applicable)                |

#### **Example**

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

——

### 2.4 ERROR Message

Signals a protocol-level failure or policy violation during QUERY/OFFER/SETTLE processing.

#### **Fields**

|Field           |Type  |Required|Description                                                                |
|-—————|——|———|—————————————————————————|
|`phase`         |string|✓       |Fixed value: `”ERROR”`                                                     |
|`id`            |string|✓       |Unique identifier for this error report                                    |
|`code`          |string|✓       |Machine-readable error code (e.g., `”POLICY_VIOLATION”`, `”INVALID_ASSET”`)|
|`message`       |string|✓       |Human-readable error description                                           |
|`correlation_id`|string|optional|ID of the message that triggered this error (QUERY/OFFER/SETTLE)           |

#### **Example**

```json
{
  “phase”: “ERROR”,
  “id”: “err-abc123”,
  “code”: “UNSUPPORTED_ASSET”,
  “message”: “Asset DOGE not supported in this jurisdiction”,
  “correlation_id”: “q-abc123”
}
```

——

### 2.5 ZkProfile Enumeration

Indicates the Buyer’s preference for zero-knowledge proof and CoreProver escrow involvement:

|Value     |Meaning                                                                   |
|-———|—————————————————————————|
|`NONE`    |Buyer does not want CoreProver escrow (direct x402 preferred)             |
|`OPTIONAL`|Buyer is willing to use CoreProver if Controller recommends it            |
|`REQUIRED`|Buyer demands CoreProver escrow (e.g., for high-value or untrusted seller)|

**Serialization:** Uppercase string values (`”NONE”`, `”OPTIONAL”`, `”REQUIRED”`)

——

### 2.6 EconomicEnvelope Structure

Encodes economic constraints for an OFFER:

#### **Fields**

|Field         |Type  |Required|Description                                                     |
|—————|——|———|-—————————————————————|
|`max_fees_bps`|u32   |✓       |Maximum acceptable total fees in basis points (e.g., 50 = 0.50%)|
|`expiry`      |string|optional|RFC3339 timestamp after which the offer is invalid              |

#### **Example**

```json
{
  “max_fees_bps”: 50,
  “expiry”: “2025-11-10T23:59:59Z”
}
```

**Future extensions** may include: slippage tolerance, multi-asset swaps, regulatory constraint flags, and SLA commitments.

——

### 2.7 SettleSource Enumeration

Indicates who is notifying the Controller about settlement:

|Value               |Meaning                                                             |
|———————|———————————————————————|
|`buyer-notify`      |Buyer (or Buyer Agent) directly reporting settlement                |
|`controller-watcher`|Controller’s own CoreProver indexer/watcher observed the transaction|
|`coreprover-indexer`|External third-party CoreProver indexer sent notification           |

**Serialization:** Kebab-case string values (`”buyer-notify”`, `”controller-watcher”`, `”coreprover-indexer”`)

——

### 2.8 Message Encoding

Stage-1 TGP messages are encoded as **UTF-8 JSON**. The `phase` field acts as a discriminator for message type.

#### **Parsing Rules**

- All messages MUST contain a `phase` field
- All messages MUST contain an `id` field
- Unknown fields SHOULD be ignored (forward compatibility)
- Parsers MAY reject messages with invalid or missing required fields

#### **Future Encoding Options**

Future stages may support:

- Binary encoding (CBOR, Protocol Buffers)
- Embedding in x402 HTTP headers (`X-TGP-Query`, `X-TGP-Offer`)
- WebSocket or gRPC transports for real-time negotiation
- Compression and batching for high-throughput scenarios

——

## 3. State Machine

Each TGP session progresses through well-defined states:

1. `Idle`
1. `QuerySent`
1. `OfferReceived`
1. `AcceptSent`
1. `Finalizing`
1. `Settled`
1. `Errored`

Gateways use timers and failure handling logic to resolve unresponsive or malformed messages, and may re-initiate under retry policy.

——

## 4. Security Considerations

TGP does not mandate encryption but recommends:

- Use of TLS or equivalent secure transport
- Signing of messages using domain keys
- Optional ZK proofs for policy compliance
- Logging of TDRs for auditability and compliance

Gateways must validate offers to ensure no settlement spoofing or value redirection occurs.

——

## 5. Attribute Registry

Gateways may maintain or consult an Attribute Registry for:

- Policy domains and compliance levels
- Regional legal flags or chain jurisdiction
- SLA commitments or availability guarantees
- x402 capability declarations (e.g. min/max price)

——

## 6. x402 Integration

TGP can operate as a control-plane overlay atop x402 sessions.

- x402 payment endpoints may embed TGP route attributes
- x402 Facilitators can implement gateway logic
- Dual-path offers (e.g. x402 and escrow) are supported

This allows for enhanced trust negotiation over existing payment paths.

——
## 7. Example Flows

### A. Simple Payment via x402

1. Buyer → QUERY → Gateway
1. Gateway → OFFER (x402)
1. Buyer → ACCEPT
1. x402 settles payment
1. Gateway → RECEIPT

### B. Escrow Settlement via CoreProver

1. Buyer → QUERY
1. Gateway → OFFER (escrow path)
1. Buyer → ACCEPT + deposit to escrow
1. Seller → ACK + fulfill item
1. CoreProver → RECEIPT to both

——

## 8. Future Extensions

TGP is designed to accommodate:

- Multi-hop settlement routing
- Pseudonymous agent negotiation
- Localized compliance overlays
- ZK audit trails and dispute resolution hooks

——

## 9. References

- [x402 Protocol](https://github.com/coinbase/x402)
- [TxIP-00 Spec](https://github.com/LedgerofEarth/txip)
- [CoreProver Contracts](https://github.com/LedgerofEarth/coreprove)
- [PEP-0.1 Policy Expression Language]

——

## 10. The 10-Layer Trust Stack (Informative)

```
Layer 10: Policy (Regulatory, Legal)
Layer 9 : Identity (Agent, Org, Wallet reputation)
Layer 8 : Economic (Ledger, On-chain state)
Layer 7 : Application (Service-specific logic)
Layer 6 : Presentation (Encoding, Formatting)
Layer 5 : Session (TGP/x402 negotiation state)
Layer 4 : Transport (QUIC, TCP, etc.)
Layer 3 : Network (IP addressing)
Layer 2 : Data Link (MAC, carrier media)
Layer 1 : Physical (Wires, Waves, Silicon)
```

——

## 11. TGP Info Block (TIB)

The TIB encodes L8–L10 context:

- `chain_id`, `ledger_state` (L8)
- `agent_id`, `domain_id`, `wallet_type` (L9)
- `policy_hash`, `compliance_tags` (L10)

——

## 12. Policy Expression Language (PEL-0.1)

A structured format to describe compliance:

```json
{
  “jurisdiction”: “US”,
  “requires”: [“KYC”, “OFAC”],
  “exemptions”: [“NFT under $500”],
  “delivery_promise”: “72h”
}
```

——

## 13. State Summary Objects (SSO)

State Summary Objects (SSOs) provide a snapshot of the current state of a TGP session at any point in its lifecycle. They enable session rehydration, distributed state reconciliation, and audit trail reconstruction without requiring full message replay.

### Purpose

SSOs serve three primary functions:

1. **Session Recovery**: Allow gateways or agents to resume interrupted sessions by reconstructing the last known valid state
1. **Distributed Coordination**: Enable multi-gateway architectures where session state must be shared or synchronized across trust boundaries
1. **Audit and Compliance**: Provide compact, verifiable checkpoints for regulatory review or dispute resolution

### Structure

An SSO captures essential session metadata and progression status:

|Field            |Type  |Description                                          |
|——————|——|——————————————————|
|`session_id`     |string|Unique identifier for the TGP session                |
|`state`          |string|Current state (e.g., `QuerySent`, `OfferReceived`)   |
|`query_id`       |string|ID of the originating QUERY message                  |
|`offer_id`       |string|ID of the accepted OFFER (if applicable)             |
|`timestamp`      |string|ISO 8601 timestamp of last state transition          |
|`participants`   |object|Buyer, Seller, and Controller identifiers            |
|`settlement_path`|string|Chosen settlement method (`x402`, `coreprover`, etc.)|
|`checksum`       |string|Hash of all prior messages for integrity validation  |

SSOs are typically emitted by Gateways or Controllers at key transitions (QUERY received, OFFER sent, settlement detected) and may be stored in append-only logs or distributed ledgers for tamper-evident auditability.

——

## 14. Receipts & TDR Triplet (Informative)

Every completed TGP session produces a **Transaction Detail Record (TDR)** triplet consisting of three layered receipts that together provide cryptographic proof of transaction completion, policy compliance, and application-level fulfillment.

### The TDR Triplet

1. **Control-Plane Receipt**: Records the TGP negotiation and session metadata (QUERY, OFFER, settlement confirmation). Anchored by the Gateway or Controller.
1. **Settlement Receipt**: Provides cryptographic proof of on-chain or off-chain value transfer. May be a blockchain transaction hash, x402 payment confirmation, or CoreProver escrow release signature.
1. **Application Receipt**: Captures fulfillment-specific evidence (download link delivered, shipping confirmation, ZK proof of delivery). Generated by the Seller or Seller Agent.

### Structure

Each receipt in the triplet includes:

- `session_id`: Links all three receipts to the originating TGP session
- `receipt_hash`: Cryptographic digest of receipt contents
- `timestamp`: ISO 8601 timestamp of receipt generation
- `signature`: Digital signature from the issuing party (Gateway, Prover, or Seller)

The triplet enables multi-layer verification: auditors can validate policy compliance (control-plane), financial settlement (settlement receipt), and actual delivery (application receipt) independently while maintaining correlation through the shared `session_id`.

### Usage

TDR triplets are stored in Gateway audit logs, submitted to compliance registries, or anchored on-chain for immutable proof. In dispute scenarios, any party can present their copy of the triplet to demonstrate what was agreed, what was paid, and what was delivered.

——

## 15. Prover Abstraction & Settlement Middleware (Informative)

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

### Deployment Flexibility

CoreProver middleware can be deployed as:

- An EVM smart contract (Ethereum, Base, Arbitrum)
- A Solana program with off-chain indexer coordination
- A federated service with cryptographic attestations
- A threshold-signature-based custodian with policy enforcement

TGP Gateways and Controllers validate CoreProver contracts against policy registries but do not constrain their internal implementation. This abstraction enables innovation in settlement mechanisms while maintaining interoperability at the TGP message layer.

——

## Appendices

### Appendix A: TAI – Transaction Area Identifier

`TGP-Appendix-A-TAI.md`  
Defines the schema for representing and matching Transaction Areas in gateway policy lookups.

### Appendix B: CoreProver Reference

`TGP-Appendix-CoreProver-Reference-E.md`  
Describes the CoreProver escrow settlement topology used as an alternative to x402.

### Appendix C: ZKB-01 – Zero Knowledge Buyer Proof

`ZKB-01-ZK-Buyer-Proof.md`  
Formal circuit for proving buyer control of a receipt address without revealing wallet.

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

Key terms used throughout the spec: TA, TZ, TDR, TIB, PEL, SSO, etc.

### Appendix K: Revision History

v0.1-draft — Fully aligned to canonical 10-layer trust stack and updated settlement architectures.

### Appendix L: Deprecation Note

Supersedes early drafts treating TGP as Layer 8.5 or solely dependent on x402 for finality.​​​​​​​​​​​​​​​​
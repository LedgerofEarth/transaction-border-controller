# TGP-00: Transaction Gateway Protocol Specification

**Version:** 3.1  
**Date:** November 18, 2025  
**Status:** Final  
**Authors:** CoreProver Development Team

——

## Abstract

The Transaction Gateway Protocol (TGP) is a Layer 8 economic-control plane signaling protocol for privacy-preserving peer-to-peer commerce. TGP-00 defines the core message flows, security boundaries, and settlement semantics that enable trustless escrow transactions across heterogeneous blockchain networks.

**Key Innovation:** TGP separates signaling (Layer 8) from settlement (blockchain execution) and places the Transaction Border Controller (TBC) as a security gateway that validates merchant payment profiles before any consumer funds move.

**Scope:** This specification defines the TGP core protocol. Extensions (TGP-CP-EXT-00), management layer (TGP-MGMT-00), and settlement engine details (TGP-ENGINE-00) are defined in separate specifications.

This specification describes:

1. **Protocol Architecture** - System actors, boundaries, and interaction patterns
1. **Payment Initiation** - HTTP 402 and Direct Pay consumer flows
1. **TBC Security Model** - Multi-layered verification before funds move
1. **Message Types** - QUERY, Economic Envelope, SETTLE schemas
1. **Payment Profiles** - Merchant-defined immutable escrow configurations
1. **Engine Overview** - High-level settlement lifecycle (detailed in TGP-ENGINE-00)

**Design Principles:**

- **Privacy-First:** Consumer wallet addresses never exposed to merchant until transaction
- **Non-Custodial:** TBC authorizes but never holds funds
- **Fail-Closed:** Any security check failure prevents transaction
- **Dual-Commitment:** Both parties commit before claims unlock (see TGP-ENGINE-00)
- **Seller-Driven Settlement:** Prevents buyer holdout attacks (see TGP-ENGINE-00)

——

## 1. Architecture Overview

### 1.1 System Actors

TGP defines four primary actors in the payment lifecycle:

1. **Consumer** - Initiates payments via browser extension (TGP-CP-EXT-00)
1. **Merchant** - Deploys payment profiles and fulfills orders
1. **Transaction Border Controller (TBC)** - Consumer-side security gateway
1. **Contract Brokerage Service (CBS)** - Merchant-side contract deployment system

**Critical Separation:** CBS and TBC are strictly separated systems with no cross-interface communication. CBS is merchant-facing only; TBC is consumer-facing only.

### 1.2 Interaction Patterns

TGP supports two payment initiation patterns:

1. **HTTP 402-Triggered:** Merchant website returns HTTP 402; extension intercepts and initiates payment
1. **Direct Pay:** Consumer opens extension, scans QR or enters merchant URL, initiates payment

Both patterns converge on the same TBC verification flow before funds move.

### 1.3 System Responsibilities Matrix

|Component    |Deployment Phase                     |Payment Phase                      |Settlement Phase                   |
|-————|-————————————|————————————|————————————|
|**CBS**      |Deploys contracts, generates profiles|No participation                   |No participation                   |
|**TBC**      |No participation                     |Verifies contracts independently   |No participation                   |
|**Extension**|No participation                     |Constructs QUERY, receives Envelope|Monitors events                    |
|**Wallet**   |No participation                     |Signs transaction (user approval)  |No participation                   |
|**Contract** |Receives deployment                  |Receives transaction               |Executes escrow (see TGP-ENGINE-00)|

**Normative Rule:** CBS MUST NOT have programmatic access to TBC verification endpoints. TBC MUST independently verify all contracts regardless of CBS deployment metadata.

### 1.4 Component Topology

```
┌─────────────────────────────────────────────────────────────────┐
│                        CONSUMER SIDE                             │
│                                                                  │
│  ┌──────────────┐         ┌─────────────────────────────────┐  │
│  │   Browser    │         │  CoreProver Browser Extension   │  │
│  │              │◄────────┤  (implements TGP-CP-EXT-00)     │  │
│  │ Merchant Site│         │                                 │  │
│  └──────┬───────┘         │  • Policy Selection             │  │
│         │                 │  • TGP Signaling                │  │
│         │ HTTP 402        │  • QR Scanner                   │  │
│         │ (with profile)  │  • Direct Pay UI                │  │
│         └─────────────────┤                                 │  │
│                           └─────────┬───────────────────────┘  │
│                                     │ TGP Query                │
│                                     │ (with policy metadata)   │
└─────────────────────────────────────┼──────────────────────────┘
                                      │
                                      ▼
                    ┌─────────────────────────────────┐
                    │ Transaction Border Controller   │
                    │          (TBC)                  │
                    │                                 │
                    │  SECURITY FIREWALL (Non-Custodial)
                    │  ┌────────────────────────────┐ │
                    │  │ Layer 1: Registry Check    │ │
                    │  │ Layer 2: Signature Verify  │ │
                    │  │ Layer 3: Contract Verify   │ │
                    │  │ Layer 4: ZK Attestation    │ │
                    │  │ Layer 5: Policy Engine     │ │
                    │  └────────────────────────────┘ │
                    │                                 │
                    │  Returns: Economic Envelope     │
                    │  (verified contract address +   │
                    │   payment parameters)           │
                    └────────┬────────────────────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
                ▼                         ▼
    ┌───────────────────┐      ┌──────────────────┐
    │  Consumer Wallet  │      │  Merchant Portal │
    │  (MetaMask, etc.) │      │  (via CBS)       │
    │                   │      │                  │
    │  Standard Web3    │      │  • Profile Setup │
    │  Transaction      │      │  • QR Generation │
    │  Approval         │      │  • Receipts      │
    └─────────┬─────────┘      └──────────────────┘
              │
              │ User Approves Transaction
              │
              ▼
    ┌─────────────────────────────────────┐
    │      Blockchain (EVM)               │
    │                                     │
    │  ┌──────────────────────────────┐  │
    │  │ Payment Profile Contract     │  │
    │  │ (Deployed by CBS)            │  │
    │  │                              │  │
    │  │ • Adminless                  │  │
    │  │ • Immutable                  │  │
    │  │ • CoreProver Engine          │  │
    │  │ • Dual-Commitment Escrow     │  │
    │  └──────────────────────────────┘  │
    └─────────────────────────────────────┘
```

**Pre-Deployment (CBS Domain):**

```
Merchant → CBS → Blockchain
         (deploys contract)
         
CBS Functions:
• Authentication
• Profile validation
• Contract deployment
• Registry update
```

**Post-Deployment (TBC Domain):**

```
Consumer → TBC → Envelope → Wallet → Blockchain
         (verifies)        (approves)  (executes)
         
TBC Functions:
• Independent verification
• Security firewall
• NO trust in CBS metadata
```

**Architectural Invariant:** CBS and TBC operate in separate trust domains with no shared authentication, no shared endpoints, and no cross-system RPC calls.

### 1.5 Key Architectural Boundaries

**Contract Brokerage Service (CBS):**

- **Purpose:** Merchant-side contract lifecycle management
- **Scope:** Pre-deployment ONLY - NOT involved in consumer payment flows
- **Function:** Deploys adminless payment profile smart contracts to blockchain
- **Access:** Merchant authentication required
- **Isolation:** NO consumer-facing interface, NO participation in TBC verification, NO settlement execution

**Transaction Border Controller (TBC):**

- **Purpose:** Consumer-side security gateway with no merchant-facing interface
- **Scope:** All consumer payment flows pass through TBC
- **Function:** Independent multi-layered verification before authorizing fund transfer
- **Architecture:** Non-custodial (never holds funds), authoritative (controls flow)
- **Output:** Economic Envelope with verified contract address
- **Verification:** MUST independently verify contracts; CBS metadata is advisory only

**CoreProver Browser Extension:**

- **Purpose:** Consumer agent implementing TGP-CP-EXT-00
- **Function:**
  - Intercepts HTTP 402 responses
  - Provides Direct Pay interface
  - Generates TGP signaling metadata
  - Submits queries to TBC
  - Hands transaction to user’s wallet
- **Relationship:** Client-side only, wallet-agnostic

**Payment Profile (Smart Contract):**

- **Purpose:** Merchant’s escrow and settlement configuration
- **Deployment:** Via CBS during merchant onboarding
- **Properties:** Adminless, immutable, revenue-supported
- **Contains:** CoreProver Engine implementing dual-commitment escrow

### 1.6 TGP as Layer 8 Protocol

TGP operates at the **economic-control plane** (Layer 8) above traditional network layers:

**Layer 8 (Economic Signaling):**

- Policy selection and routing
- Payment profile resolution
- Fee and pricing negotiation
- Discount application
- Economic envelope construction

**Layer 9 (Identity):**

- Pseudonymous buyer/seller resolution
- ZK-based identity proofs
- Reputation aggregation
- Receipt ownership verification

**Layer 10 (Policy & Compliance):**

- Withdrawal lock enforcement
- State machine validation
- Time window monitoring
- Jurisdictional rules
- Risk assessment

——

## 2. Payment Initiation

### 2.1 Overview

TGP defines payment initiation as the signaling process from consumer intent to TBC verification. Two initiation patterns are supported, both converging on identical TBC verification semantics.

**Shared Payment Initiation Semantics:**

Regardless of initiation pattern, all payments follow:

1. Consumer intent expressed (via HTTP 402 intercept OR Direct Pay)
1. Extension constructs TGP QUERY message
1. Extension sends QUERY to TBC endpoint
1. TBC performs onion verification (Section 3)
1. TBC returns Economic Envelope (success) OR Error (denial)
1. Extension hands verified transaction to wallet
1. User approves in wallet (standard Web3 flow)
1. Transaction submitted to blockchain

**Initiation Pattern Comparison:**

|Aspect               |HTTP 402-Triggered  |Direct Pay                |
|———————|———————|—————————|
|**Initiator**        |Merchant website    |Consumer                  |
|**Trigger**          |HTTP 402 response   |Extension UI action       |
|**Amount**           |Merchant-specified  |Consumer-specified        |
|**Context**          |Shopping cart/order |Direct payment/tip        |
|**Profile Discovery**|HTTP headers        |QR scan or manual URL     |
|**Use Cases**        |E-commerce, services|Tips, donations, in-person|

### 2.2 Initiation Pattern 1: HTTP 402-Triggered Payment

**Trigger:** Merchant website returns HTTP 402 Payment Required

**Flow:**

```
1. Consumer browses merchant website (e.g., shop.example.com)
   │
2. Consumer adds items to cart, clicks “Checkout”
   │
3. Merchant server returns HTTP 402 Payment Required with headers:
   │
   HTTP/1.1 402 Payment Required
   TGP-Payment-Profile: profile-uuid-or-url
   TGP-Amount: 30000000
   TGP-Asset: USDC
   TGP-TBC-Endpoint: https://tbc.coreprove.com/query
   TGP-Merchant-ID: merchant-12345
   TGP-Order-Details: {“items”: [...], “description”: “...”}
   │
4. CoreProver Browser Extension intercepts HTTP 402
   │
5. Extension presents payment UI:
   ├─ Parsed order details
   ├─ Amount and asset
   ├─ Policy selection (user’s payment profiles)
   └─ Confirm/Cancel buttons
   │
6. User selects policy, clicks Confirm
   │
7. Extension constructs TGP QUERY message:
   {
     “phase”: “QUERY”,
     “from”: “buyer://pseudonym”,
     “to”: “seller://merchant-12345”,
     “asset”: “USDC”,
     “amount”: 30000000,
     “profile_reference”: “profile-uuid-or-url”,
     “tbc_endpoint”: “https://tbc.coreprove.com/query”,
     “metadata”: {...}
   }
   │
8. Extension sends QUERY to TBC endpoint
   │
9. TBC performs “onion” security verification (see Section 3)
   │
10. TBC responds with Economic Envelope:
    {
      “verified_contract_address”: “0x742d35Cc6634...”,
      “amount”: 30000000,
      “asset”: “0x... (USDC address)”,
      “gas_estimate”: {...},
      “policy_terms”: {...},
      “expires_at”: “2025-11-18T12:00:00Z”
    }
    OR
    {
      “error”: “MERCHANT_DISABLED”,
      “reason”: “Merchant payment profile is disabled”,
      “user_message”: “This merchant is temporarily unavailable”
    }
    │
11. Extension receives Economic Envelope
    │
12. Extension constructs blockchain transaction:
    {
      to: “0x742d35Cc6634...”,  // verified contract address
      value: 0,
      data: encodeEscrowCreation(amount, asset, orderDetails)
    }
    │
13. Extension passes transaction to user’s wallet (MetaMask, etc.)
    │
14. User sees standard wallet prompt:
    ├─ “Send 30 USDC to 0x742d35...”
    ├─ Gas estimate
    └─ Approve / Reject
    │
15. User approves → Transaction submitted to blockchain
    │
16. Payment Profile Contract receives transaction
    │
17. CoreProver Engine creates escrow (BUYER_COMMITTED state)
    │
18. Merchant receives notification, fulfills order
```

**Key Points:**

- Extension never holds keys or funds
- TBC never touches funds (non-custodial)
- TBC provides authorization before wallet interaction
- Standard Web3 UX maintained for user
- Merchant receives funds only after dual-commitment

### 2.3 Initiation Pattern 2: Direct Pay (Consumer-Initiated)

**Trigger:** User opens extension and selects “Direct Pay”

**Flow:**

```
1. User opens CoreProver Browser Extension
   │
2. User clicks “Direct Pay” mode
   │
3. Extension displays Direct Pay form:
   ├─ Amount input field
   ├─ Asset selector (USDC, WETH, etc.)
   ├─ “Scan QR” button
   └─ “Enter Merchant URL” field
   │
4. User enters amount (e.g., “30 USDC”)
   │
5. User obtains merchant identifier via ONE of:
   │
   OPTION A: Scan QR Code
   ├─ User clicks “Scan QR”
   ├─ Camera activates
   ├─ User scans merchant QR code (contains payment URL)
   └─ Extension extracts: merchant payment URL + profile reference
   │
   OPTION B: Manual Entry
   ├─ User types or pastes merchant payment URL
   └─ Example: “pay.merchant.com/profile/abc123”
   │
6. Extension validates format, displays parsed merchant info
   │
7. User confirms payment
   │
8. Extension constructs TGP QUERY message:
   {
     “phase”: “QUERY”,
     “from”: “buyer://pseudonym”,
     “to”: “seller://merchant-from-url”,
     “asset”: “USDC”,
     “amount”: 30000000,  // user-entered
     “profile_reference”: “extracted-from-qr-or-url”,
     “direct_pay”: true,
     “metadata”: {
       “initiated_by”: “consumer”,
       “order_description”: “Direct payment”
     }
   }
   │
9-18. [Same as HTTP 402 flow from step 8 onward]
```

**Key Differences from HTTP 402:**

- Consumer initiates, not merchant
- Amount set by consumer (not predetermined)
- No cart or order context from merchant site
- Merchant payment URL required (QR or manual)
- Useful for: tips, donations, in-person payments, service payments

### 2.4 QR Code Format

Merchant QR codes encode payment URLs in standard format:

```
Format: https://pay.merchant.com/profile/{profile_id}?m={merchant_id}

Example QR Content:
https://pay.pizzahut.com/profile/store-4521?m=pizzahut-4521

Parsed Fields:
- Domain: pay.pizzahut.com
- Profile ID: store-4521
- Merchant ID: pizzahut-4521
- TBC Endpoint: Resolved via domain or registry
```

Extension behavior on scan:

1. Extract payment URL
1. Resolve TBC endpoint (domain-based or registry lookup)
1. Construct QUERY with profile reference
1. Proceed with standard flow

——

## 3. TBC Security Model: “Onion” Verification

The Transaction Border Controller acts as a **transaction firewall**, not a router. It enforces progressively stronger security checks before authorizing any consumer funds to move.

**Core Principle:** No consumer transaction proceeds to a merchant contract address without TBC validation that the contract is authentic, enabled, and safe.

**Layer Taxonomy:**

- **REQUIRED Layers:** Layers 1, 2, 3, 5 (MUST be evaluated for all transactions)
- **OPTIONAL Layer:** Layer 4 (ZK attestation - configurable per policy)

**Normative Sequencing Rule:** Layers MUST be evaluated sequentially (1 → 2 → 3 → 4 → 5). No layer may be skipped. First failure terminates evaluation and returns error.

**Fail-Closed Behavior:** At every layer, all checks pass → continue to next layer; any check fails or returns ambiguous data → return ERROR/CANCEL immediately.

### 3.1 Layer Evaluation Semantics

**Normative Summary:**

1. Layers are evaluated in strict sequence: 1 → 2 → 3 → (4 optional) → 5
1. Each layer implements fail-closed logic: pass → continue; fail → reject immediately
1. Ambiguous or partial results MUST be treated as failures
1. TBC NEVER forwards unverified transactions
1. Economic Envelope generation requires all REQUIRED layers to pass

### 3.2 Security Layers (Onion Model)

#### Layer 1: Merchant Registry / Enable Flag (REQUIRED)

**Purpose:** Ensure merchant profile is currently active and authorized; fail-closed on any registry unavailability.

**Process:**

1. TBC queries Merchant Registry (CBS backend read-only API) for profile status
1. Registry returns: `{enabled: true/false, status: “active”|”suspended”|”disabled”}`
1. **Rejection Cases:**
- `enabled = false` → REJECT
- `status ≠ “active”` → REJECT
- Registry query fails (timeout, network error, 5xx) → REJECT (fail-closed)
- Registry returns partial/malformed response → REJECT

**Error Response:**

```json
{
  “error”: “MERCHANT_DISABLED”,
  “reason”: “Merchant payment profile is currently disabled”,
  “code”: “TBC_L1_REGISTRY_FAIL”,
  “user_message”: “This merchant is temporarily unavailable. Please try again later.”
}
```

**Prevents:**

- Payments to suspended/terminated merchants
- Use of revoked payment profiles
- Bypass of merchant compliance requirements

#### Layer 2: Merchant-Verifiable Signature on Profile (REQUIRED)

**Purpose:** Verify authentic merchant identity via cryptographic signature; merchant-signed descriptors are advisory only, not authoritative for contract verification.

**Process:**

1. TBC retrieves payment profile descriptor (on-chain or off-chain)
1. Profile contains:
   
   ```json
   {
     “profile_id”: “uuid”,
     “merchant_id”: “merchant-12345”,
     “contract_address”: “0x742d35...”,
     “chain_id”: 1,
     “asset”: “USDC”,
     “policy_params”: {...},
     “signature”: “0x...”,  // Signed by merchant’s authorized key
     “signed_at”: “2025-11-01T00:00:00Z”
   }
   ```
1. TBC verifies signature using merchant’s registered public key
1. **Rejection Cases:**
- Signature verification fails → REJECT
- Signature expired (age > configured threshold) → REJECT
- Merchant public key not found in registry → REJECT
- Profile descriptor malformed/incomplete → REJECT

**Clarification:** This signature proves merchant intent to deploy a profile, but TBC MUST NOT trust the contract_address field. Layer 3 independently verifies the actual on-chain code.

**Error Response:**

```json
{
  “error”: “INVALID_SIGNATURE”,
  “reason”: “Payment profile signature verification failed”,
  “code”: “TBC_L2_SIGNATURE_FAIL”,
  “user_message”: “Unable to verify merchant authenticity. Transaction cancelled for your safety.”
}
```

**Prevents:**

- Phishing attacks with fake merchant addresses
- Man-in-the-middle address substitution
- Unauthorized profile modifications

#### Layer 3: On-Chain Contract Verification (REQUIRED)

**Purpose:** Confirm contract bytecode matches audited CoreProver template and state is valid; enforce deterministic multi-RPC consistency.

**Process:**

1. TBC performs read-only queries against target blockchain using quorum RPC
1. Verification steps:
   
   **A. Code Hash Verification (Canonical Algorithm):**
   
   ```
   expected_hash = keccak256(audited_coreprover_bytecode_v{VERSION})
   actual_hash = keccak256(eth_getCode(contract_address))
   
   MUST: actual_hash == expected_hash
   MUST: VERSION matches profile.engine_version
   ```
   
   **B. State Validation:**
   
   ```
   Contract state checks (all MUST pass):
   - Contract not paused
   - Contract not self-destructed
   - Contract in valid operational phase
   - No admin keys present (adminless verification)
   - Escrow parameters match profile descriptor
   ```
   
   **C. Chain ID Alignment:**
   
   ```
   declared_chain = profile.chain_id
   actual_chain = block.chainid (from RPC)
   MUST: declared_chain == actual_chain
   ```
   
   **D. Token Address Validation:**
   
   ```
   declared_asset = profile.asset_address
   contract_asset = contract.getAsset()
   MUST: declared_asset == contract_asset
   ```
1. **Quorum RPC Checks (Deterministic Consistency):**
- Query N independent RPC endpoints (N ≥ 3 recommended)
- Require M-of-N consensus (M ≥ 2, typically M = ⌈2N/3⌉)
- Compare: bytecode hash, state values, chain ID, block number
- **Rejection Cases:**
  - RPC responses disagree on bytecode hash → REJECT (potential tampering)
  - RPC responses disagree on state → REJECT
  - Fewer than M successful responses → REJECT
  - Any RPC returns error for critical queries → REJECT

**Error Response:**

```json
{
  “error”: “CONTRACT_VERIFICATION_FAILED”,
  “reason”: “Contract bytecode does not match audited CoreProver template”,
  “code”: “TBC_L3_CODE_MISMATCH”,
  “user_message”: “Security verification failed. Transaction cancelled for your protection.”
}
```

**Prevents:**

- Payments to non-CoreProver contracts
- Contract upgrade tricks (immutability verification)
- RPC tampering attacks
- Wrong chain/token attacks
- Ambiguous/inconsistent RPC data

**Implementation Note:** Detailed RPC quorum algorithms, timeout handling, and retry logic are specified in TGP-TBC-SEC-00.

#### Layer 4: Optional Light Client / ZK Attestation (OPTIONAL)

**Purpose:** Provide cryptographic assurance of contract validity without RPC trust; configurable per policy or transaction value threshold.

**Process:**

1. TBC requests ZK proof that target contract satisfies invariants:
   
   ```
   Proof claims:
   - Contract bytecode hash = expected_hash
   - Contract state = {not paused, no admin keys, ...}
   - Registry enabled flag = true
   - All checks performed on valid chain state
   ```
1. TBC verifies ZK proof locally (no trust in external RPC)
1. **Rejection Cases:**
- Proof verification fails → REJECT
- Proof not available when required by policy → REJECT
- Proof expired (timestamp too old) → REJECT

**Configuration:**

- **OPTIONAL** for most transactions
- **REQUIRED** when: transaction value exceeds threshold, jurisdiction requires enhanced verification, merchant policy mandates, or user opts-in
- Configurable threshold (e.g., require ZK proof for payments >$10,000)

**Error Response:**

```json
{
  “error”: “ZK_ATTESTATION_REQUIRED”,
  “reason”: “Transaction value exceeds threshold for standard verification”,
  “code”: “TBC_L4_ZK_REQUIRED”,
  “user_message”: “This transaction requires enhanced verification. Please try again.”
}
```

**Prevents:**

- RPC collusion attacks
- State visibility manipulation
- Ultra-high-value transaction risks

**Implementation Note:** ZK circuit specifications and proof formats are defined in TGP-02.

#### Layer 5: Policy / Risk Engine (REQUIRED)

**Purpose:** Apply Layer 9/10 policy rules, compliance checks, and risk assessment; all transactions MUST pass policy validation.

**Purpose:** Apply Layer 9/10 policy rules and compliance checks

**Process:**

1. TBC evaluates transaction against policy engine:
   
   **Chain Whitelist:**
   
   ```
   allowed_chains = [1, 137, 42161]  // Ethereum, Polygon, Arbitrum
   if transaction.chain_id not in allowed_chains → FAIL
   ```
   
   **Asset Whitelist:**
   
   ```
   allowed_assets = [“USDC”, “WETH”, “DAI”]
   if transaction.asset not in allowed_assets → FAIL
   ```
   
   **Value Limits:**
   
   ```
   max_transaction = 100000_000000  // $100,000 USDC
   if transaction.amount > max_transaction → FAIL
   ```
   
   **Sanctions / Compliance:**
   
   ```
   if merchant_id in sanctions_list → FAIL
   if buyer_jurisdiction in restricted_regions → FAIL
   ```
   
   **Rate Limiting:**
   
   ```
   if buyer_transaction_count_today > limit → FAIL
   ```
1. If any policy rule fails → CANCEL transaction with specific reason

**Error Response:**

```json
{
  “error”: “POLICY_VIOLATION”,
  “reason”: “Transaction exceeds daily limit for your account”,
  “code”: “TBC_L5_RATE_LIMIT”,
  “user_message”: “Daily transaction limit reached. Please try again tomorrow.”
}
```

**Prevents:**

- Unauthorized chain usage
- Unsupported asset types
- Excessive transaction values
- Sanctioned entity payments
- Compliance violations

**Implementation Note:** Policy rule configuration and risk scoring algorithms are specified in TGP-TBC-SEC-00.

### 3.3 Normative Verification Sequencing

**Mandatory Sequence:**

1. Layer 1 (Registry) → 2 (Signature) → 3 (Contract) → 4 (ZK, if required) → 5 (Policy)
1. Layers MUST NOT be evaluated in parallel (dependencies exist)
1. Layers MUST NOT be skipped (even if previous layer provides partial information)
1. First layer failure MUST terminate evaluation immediately

**Fail-Closed Semantics:**

- **Pass:** All required checks in layer pass → Proceed to next layer
- **Fail:** Any check fails → Return error, do not evaluate remaining layers
- **Ambiguous:** Partial data, timeout, unexpected response → Treat as fail

**Result:**

- All required layers pass → Generate Economic Envelope (Section 4)
- Any required layer fails → Generate Error Response (Section 4)
- Optional layer (Layer 4) fails when not required → Log warning, continue

——

## 4. Economic Envelope & Messaging

### 4.1 Overview

The **Economic Envelope** is TBC’s response to a consumer QUERY, containing verified contract address and payment parameters. It is pre-authorization metadata only, not a transaction record.

**Normative Rule:** The Economic Envelope is authoritative for the consumer payment flow. Extension MUST use envelope parameters for transaction construction. Envelope parameters MUST NOT be modified by extension before wallet handoff.

**Scope:** This section defines TGP core message types. Serialization formats (JSON, JWT, encoding) and transport details are specified in TGP-CP-EXT-00. Wallet handoff is outside TGP’s scope (standard Web3 transaction flow).

### 4.2 TGP Message Type Registry

TGP-00 defines the following core message types:

|Message Type         |Direction         |Purpose                       |Section|
|———————|——————|——————————|-——|
|**QUERY**            |Extension → TBC   |Payment initiation request    |4.3    |
|**Economic Envelope**|TBC → Extension   |Verified payment authorization|4.4    |
|**Error Response**   |TBC → Extension   |Verification failure details  |4.5    |
|**SETTLE**           |Engine → Extension|Settlement confirmation       |4.6    |

**Note:** Management messages (OFFER, session coordination) are defined in TGP-MGMT-00. Blockchain event messages are defined in TGP-ENGINE-00.

### 4.3 QUERY Message Schema

**Purpose:** Consumer initiates transaction discovery via TBC

**Direction:** Extension → TBC

**Field Taxonomy:**

**REQUIRED Fields (Base Message):**

```json
{
  “tgp_version”: “3.1”,
  “phase”: “QUERY”,
  “id”: “q-{uuid}”,
  “from”: “buyer://{pseudonym}”,
  “to”: “seller://{merchant_id}”,
  “asset”: “{token_symbol}”,
  “amount”: “{integer_wei}”,
  “profile_reference”: “{profile_uuid_or_url}”,
  “tbc_endpoint”: “{tbc_url}”
}
```

**OPTIONAL Fields:**

```json
{
  “direct_pay”: false,
  “zk_profile”: {
    “circuit”: “receipt_ownership_v1”,
    “proof”: “{proof_bytes}”,
    “public_inputs”: {...}
  },
  “metadata”: {
    “product_id”: “optional”,
    “quantity”: “optional”,
    “order_description”: “optional”,
    “initiated_from”: “http_402 | direct_pay”
  }
}
```

**Domain Separation:**

- **Profile Domain:** `profile_reference`, `asset`, `to` (merchant-related)
- **Transaction Domain:** `amount`, `from`, `id` (consumer-related)
- **Verification Domain:** `tbc_endpoint`, `tgp_version` (protocol-related)

**Field Definitions:**

- `tgp_version` - Protocol version (MUST be “3.1” for this spec)
- `phase` - Message type (MUST be “QUERY”)
- `id` - Unique query identifier (UUID format recommended)
- `from` - Buyer pseudonymous identifier (privacy-preserving)
- `to` - Merchant identifier
- `asset` - Payment token symbol (USDC, WETH, etc.)
- `amount` - Payment amount in smallest unit (wei for ETH, 1e6 for USDC)
- `profile_reference` - Merchant payment profile ID or URL
- `tbc_endpoint` - TBC service URL for this query
- `direct_pay` - Boolean indicating consumer-initiated payment (vs HTTP 402)
- `zk_profile` - Optional zero-knowledge proof (for discount redemption, see Appendix B)
- `metadata` - Application-specific data (NOT used in TBC verification)

### 4.4 Economic Envelope Schema (Success Response)

**Purpose:** TBC returns verified payment parameters after successful onion verification

**Direction:** TBC → Extension

**REQUIRED Fields:**

**REQUIRED Fields:**

```json
{
  “status”: “APPROVED”,
  “envelope”: {
    “verified_contract_address”: “0x742d35Cc6634C0532925a3b844Bc454e4438f44e”,
    “chain_id”: 1,
    “asset_address”: “0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48”,
    “amount”: 30000000,
    “session_id”: “session-uuid-12345”,
    “expires_at”: “2025-11-18T12:15:00Z”,
    “tbc_signature”: “0x...”
  }
}
```

**OPTIONAL Fields:**

```json
{
  “envelope”: {
    “asset_symbol”: “USDC”,
    “gas_estimate”: {
      “gas_limit”: 250000,
      “max_fee_per_gas”: 50000000000,
      “max_priority_fee_per_gas”: 2000000000
    },
    “policy_terms”: {
      “acceptance_window_seconds”: 1800,
      “fulfillment_window_seconds”: 3600,
      “claim_window_seconds”: 3600,
      “enables_late_discount”: true
    }
  },
  “verification_summary”: {
    “layer1_registry”: “PASS”,
    “layer2_signature”: “PASS”,
    “layer3_contract”: “PASS”,
    “layer4_zk”: “NOT_REQUIRED”,
    “layer5_policy”: “PASS”
  }
}
```

**Domain Separation:**

- **Verification Domain:** `verified_contract_address`, `chain_id`, `tbc_signature`, `verification_summary`
- **Transaction Domain:** `amount`, `asset_address`, `session_id`, `expires_at`
- **Profile Domain:** `policy_terms` (escrow timing parameters)
- **Advisory Domain:** `gas_estimate`, `asset_symbol` (NOT authoritative, consumer may override)

**Field Definitions:**

- `status` - MUST be “APPROVED” for success response
- `verified_contract_address` - Contract address passing all TBC verification layers
- `chain_id` - Blockchain chain ID (Ethereum mainnet = 1)
- `asset_address` - ERC-20 token contract address (or 0x0 for native asset)
- `amount` - Payment amount (echoed from QUERY for confirmation)
- `session_id` - Unique session identifier for this payment flow
- `expires_at` - Envelope expiration timestamp (ISO 8601) - envelope invalid after this time
- `tbc_signature` - TBC’s cryptographic signature over envelope (for non-repudiation)
- `gas_estimate` - Optional gas parameters (advisory only, extension may override)
- `policy_terms` - Escrow timing from payment profile (see Section 5)
- `verification_summary` - Per-layer verification results (for transparency)

**Consumer Safety Requirements:**

Extension MUST perform the following checks before using envelope:

1. Verify `expires_at` is in future (envelope not expired)
1. Verify `tbc_signature` is valid (using TBC’s published public key)
1. Verify `amount` matches user intent
1. Detect anomalies: if `chain_id` changes from query, warn user
1. If `gas_estimate` seems unreasonable (>10x expected), warn user

**Normative:** CBS does NOT participate in envelope formation. TBC generates envelope independently based on verification results.

### 4.5 Error Response Schema

**Purpose:** TBC returns structured error when verification fails

**Direction:** TBC → Extension

**REQUIRED Fields:**

```json
{
  “status”: “DENIED”,
  “error”: “CONTRACT_VERIFICATION_FAILED”,
  “code”: “TBC_L3_CODE_MISMATCH”,
  “layer_failed”: 3,
  “timestamp”: “2025-11-18T12:00:00Z”
}
```

**OPTIONAL Fields:**

```json
{
  “reason”: “Contract bytecode does not match audited CoreProver template”,
  “user_message”: “Security verification failed. Transaction cancelled for your protection.”,
  “support_reference”: “ref-abc123”,
  “retry_allowed”: false
}
```

**Field Definitions:**

- `status` - MUST be “DENIED” for error response
- `error` - Machine-readable error type (see error type registry below)
- `code` - Specific error code (format: TBC_L{layer}_{TYPE})
- `layer_failed` - Which verification layer failed (1-5)
- `timestamp` - Error timestamp (ISO 8601)
- `reason` - Human-readable technical reason (for logging)
- `user_message` - User-friendly message (extension displays this)
- `support_reference` - Support ticket reference for debugging
- `retry_allowed` - Boolean indicating if retry is sensible (false for permanent errors)

**Error Type Registry:**

|Error Type                    |Layer|Retry Allowed|Meaning                              |
|——————————|——|-————|-————————————|
|`MERCHANT_DISABLED`           |1    |No           |Profile disabled in registry         |
|`REGISTRY_UNAVAILABLE`        |1    |Yes          |Registry query failed                |
|`INVALID_SIGNATURE`           |2    |No           |Profile signature verification failed|
|`CONTRACT_VERIFICATION_FAILED`|3    |No           |Bytecode mismatch                    |
|`RPC_INCONSISTENCY`           |3    |Yes          |RPC endpoints disagree               |
|`ZK_ATTESTATION_REQUIRED`     |4    |No           |High-value transaction needs ZK proof|
|`POLICY_VIOLATION`            |5    |Maybe        |Depends on specific policy rule      |

**Extension Behavior:**

- If `retry_allowed = true`: Display retry button
- If `retry_allowed = false`: Display “Contact Support” with `support_reference`
- Always display `user_message` (never display `reason` to user)
- Log full error response for debugging

### 4.6 SETTLE Message Schema

**Purpose:** Confirms transaction completion and settlement

**Direction:** CoreProver Engine / Watcher → Extension / Merchant

**Schema:**

```json
{
  “phase”: “SETTLE”,
  “id”: “settle-{uuid}”,
  “query_id”: “{query_id}”,
  “session_id”: “{session_uuid}”,
  “success”: true,
  “source”: “coreprover-engine | watcher”,
  “blockchain_tx”: “{transaction_hash}”,
  “escrow_state”: “SELLER_CLAIMED”,
  “fulfillment_metadata”: {
    “on_time”: true,
    “late_fulfilled”: false,
    “discount_pct”: 0,
    “discount_expiration”: null,
    “fulfillment_timestamp”: “{unix_timestamp}”,
    “settlement_timestamp”: “{unix_timestamp}”,
    “receipt_id”: “{nft_token_id}”,
    “buyer_withdrawal_locked”: false,
    “next_discount_available”: false
  }
}
```

**Field Definitions:**

- `success` - Whether settlement succeeded
- `source` - Origin of settlement notification
- `blockchain_tx` - On-chain transaction hash
- `escrow_state` - Final state from state machine
- `fulfillment_metadata` - Delivery and discount details

### 4.4 EVENT Messages

**Purpose:** Broadcast state transitions from smart contracts

**Direction:** Blockchain → TGP Watchers → Subscribers

Event types are defined in detail in Section 2.4 of the original spec (unchanged).

Key events:

- `tgp.escrow.created`
- `tgp.seller.accepted` (LOCKS buyer withdrawal)
- `tgp.seller.fulfilled`
- `tgp.fulfillment.expired` (UNLOCKS buyer withdrawal)
- `tgp.seller.latefulfilled` (RE-LOCKS buyer withdrawal, issues discount)
- `tgp.seller.claimed`
- `tgp.receipt.minted`

——

## 5. Payment Profiles

### 5.1 Definition

A **Payment Profile** is a merchant-authored, CBS-deployed, TBC-verified immutable rule-set with no owner/admin roles. It is a smart contract configuration that specifies escrow parameters, timing windows, and settlement policies.

**Normative Rule:** Payment Profiles are immutable post-deployment. TBC MUST independently verify contract bytecode; TBC MUST NOT trust CBS deployment metadata as authoritative for security decisions.

**Terminology:** “Settlement Profiles” (v2.0 and earlier) have been renamed to “Payment Profiles” across all CoreProver documentation.

### 5.2 Minimal Canonical Schema

**Purpose:** Define the minimal required fields for a valid payment profile

**REQUIRED Fields:**

```json
{
  “profile_id”: “uuid”,
  “merchant_id”: “string”,
  “contract_address”: “0x...”,
  “chain_id”: 1,
  “asset_address”: “0x...”,
  “engine_version”: “v0.3”,
  “deployed_at”: “ISO8601 timestamp”
}
```

**OPTIONAL Fields (Policy Parameters):**

```json
{
  “timing_config”: {
    “acceptance_window_seconds”: 1800,
    “fulfillment_window_seconds”: 3600,
    “claim_window_seconds”: 3600
  },
  “commitment_type”: “LEGAL_SIGNATURE | COUNTER_ESCROW”,
  “fulfillment_type”: “DIGITAL | SHIPPING | SERVICE”,
  “enables_late_discount”: true
}
```

**Normative Rules:**

1. `contract_address` MUST reference an immutable, adminless smart contract
1. `engine_version` MUST match TBC’s expected CoreProver Engine version
1. TBC verification (Layer 3) MUST independently confirm bytecode matches `engine_version`
1. Profile metadata is advisory; on-chain contract state is authoritative

**TBC Rejection Criteria:**

TBC MUST reject a profile if:

- `contract_address` bytecode does not match expected hash for `engine_version`
- Contract has admin/owner functions (violates adminless requirement)
- Contract is upgradeable (violates immutability requirement)
- `chain_id` does not match actual contract chain
- `asset_address` does not match contract’s configured asset

### 5.3 Contract Lifecycle Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   PRE-DEPLOYMENT                         │
│                   (CBS Domain)                           │
│                                                          │
│  Merchant → CBS Portal → Profile Creation               │
│                       ↓                                  │
│               CBS validates parameters                   │
│                       ↓                                  │
│               CBS deploys contract to blockchain         │
│                       ↓                                  │
│               CBS registers profile in registry          │
│                                                          │
└─────────────────────────────────────────────────────────┘
                          ↓
            [Contract now exists on-chain]
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   POST-DEPLOYMENT                        │
│                   (TBC Domain)                           │
│                                                          │
│  Consumer → Extension → TBC Query                        │
│                       ↓                                  │
│               TBC independently verifies:                │
│               • Bytecode hash (Layer 3)                  │
│               • Registry status (Layer 1)                │
│               • Signature (Layer 2)                      │
│               • Policy compliance (Layer 5)              │
│                       ↓                                  │
│               TBC returns Economic Envelope              │
│                       ↓                                  │
│  Extension → Wallet → User Approves                      │
│                       ↓                                  │
│  Transaction → Blockchain → Contract Executes            │
│                       ↓                                  │
│               [See TGP-ENGINE-00 for settlement]         │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**Separation of Concerns:**

- **CBS:** Deploys contracts (pre-deployment only)
- **TBC:** Verifies contracts (runtime only)
- **Contract:** Executes settlement (see TGP-ENGINE-00)
- **No overlap:** CBS does not participate in TBC verification; TBC does not participate in CBS deployment

### 5.4 Profile Deployment via CBS

Payment Profiles are deployed by the Contract Brokerage Service during merchant onboarding. This is a one-time deployment process; profiles cannot be modified post-deployment.

**Merchant Portal Flow:**

1. Merchant authenticates to Merchant Portal (merchant.coreprove.com)
1. Merchant configures profile parameters (assets, timing, policies)
1. Merchant submits deployment request to CBS
1. CBS validates parameters and merchant authorization
1. CBS deploys adminless, immutable smart contract to blockchain
1. CBS registers profile in Merchant Registry (enable flag = true)
1. CBS returns profile ID and payment URL to merchant
1. Merchant integrates URL into QR codes / HTTP 402 headers

**Contract Properties (Enforced by CBS):**

- **Adminless:** No owner, admin, or privileged functions
- **Immutable:** Cannot be upgraded or modified
- **Revenue-Supported:** Merchant pays deployment fee
- **Audited:** Bytecode matches known-good template

**CBS Does NOT:**

- Participate in consumer payment flows
- Have access to TBC verification endpoints
- Modify contracts post-deployment
- Execute or monitor settlement (see TGP-ENGINE-00)

### 5.5 Profile URLs

Payment Profiles are addressable via standard URLs for QR codes and HTTP 402 headers:

**Format:** `https://pay.{merchant_domain}/profile/{profile_id}`

**Examples:**

- `https://pay.pizzahut.com/profile/store-4521`
- `https://pay.example.com/profile/checkout-usd`

**Resolution Flow:**

1. Extension receives profile URL (from QR or HTTP 402)
1. Extension extracts `profile_id`
1. Extension constructs TGP QUERY with `profile_reference = URL`
1. TBC resolves URL to profile metadata (cached or fetched)
1. TBC proceeds with onion verification

——

## 6. Engine Overview & Settlement Lifecycle

**Scope:** This section provides a high-level overview of the CoreProver Engine settlement lifecycle. Detailed specifications (state machine, timing, events, withdrawal logic, discount mechanics) are defined in **TGP-ENGINE-00**.

### 6.1 Signaling vs Settlement Separation

**TGP-00 (This Specification):**

- Payment initiation (HTTP 402, Direct Pay)
- TBC security verification
- Economic Envelope generation
- Pre-transaction signaling

**TGP-ENGINE-00 (Settlement Specification):**

- Dual-commitment escrow state machine
- Time window enforcement
- Withdrawal lock logic
- Late fulfillment penalties
- Discount coupon issuance
- Receipt NFT minting
- Timed release mechanisms

**Normative Demarcation:** TGP-00 defines signaling up to transaction submission. TGP-ENGINE-00 defines on-chain settlement execution from escrow creation to completion.

### 6.2 High-Level Settlement Flow

```
1. Transaction submitted to Payment Profile contract
   ↓
2. Escrow created (BUYER_COMMITTED state)
   ↓
3. Seller accepts order (SELLER_ACCEPTED state)
   → Buyer withdrawal LOCKED
   ↓
4. Seller fulfills order within window
   ↓
5. Seller claims payment (SELLER_CLAIMED state)
   → Receipt NFT minted
   ↓
6. Settlement complete
```

**Key States (Summary):**

- `BUYER_COMMITTED` - Payment locked, seller must accept
- `SELLER_ACCEPTED` - Withdrawal locked, seller must fulfill
- `SELLER_FULFILLED` - Fulfillment complete, seller may claim
- `SELLER_CLAIMED` - Settlement complete (terminal)

**Alternative Paths:**

- Acceptance timeout → Buyer can withdraw
- Fulfillment timeout → Buyer can withdraw (unless late fulfillment)
- Late fulfillment → Discount issued, withdrawal re-locked

**For Complete Details:** See TGP-ENGINE-00 specification.

### 6.3 Event Messages

Blockchain events are emitted during settlement lifecycle. These events are monitored by extension/merchant systems for status updates.

**Key Events (Summary):**

- `tgp.escrow.created` - Escrow initialized
- `tgp.seller.accepted` - Seller committed (withdrawal locks)
- `tgp.seller.fulfilled` - Order fulfilled
- `tgp.seller.claimed` - Payment claimed
- `tgp.receipt.minted` - Receipt NFT created

**For Complete Event Schemas:** See TGP-ENGINE-00 Section 4.

——

## 7. Security Considerations

### 8.1 TBC as Security Boundary

**Threat Model:**

The TBC operates in a hostile environment where:

- Merchants may be compromised or malicious
- RPC providers may collude or be censored
- Network attackers may intercept or modify messages
- Phishing sites may impersonate legitimate merchants

**Design Assumptions:**

1. TBC code is trusted (operated by CoreProver or trusted third party)
1. Merchant Registry is authoritative source of merchant status
1. Audited CoreProver bytecode hash is public and immutable
1. Multiple independent RPC providers are available
1. Blockchain consensus provides transaction finality

### 8.2 Attack Vectors & Mitigations

**Attack: Phishing with Fake Payment Profile**

Attacker creates website that returns HTTP 402 with malicious contract address.

**Mitigation:**

- Layer 2: Signature verification ensures profile is merchant-signed
- Layer 3: Contract verification ensures bytecode matches audited template
- Layer 5: Policy engine checks merchant reputation

**Attack: Compromised RPC Provider**

Single RPC provider returns false contract state to pass verification.

**Mitigation:**

- Layer 3: Quorum RPC checks require consensus from multiple providers
- Layer 4: ZK attestation option eliminates RPC trust (high-value transactions)

**Attack: Contract Upgrade Trick**

Merchant deploys valid contract, then uses upgrade mechanism to insert backdoor.

**Mitigation:**

- Payment Profiles are adminless and immutable (no upgrade mechanism)
- Layer 3: Bytecode hash verification detects any code changes

**Attack: Merchant Registry Compromise**

Attacker gains access to Merchant Registry and enables disabled profiles.

**Mitigation:**

- CBS requires multi-signature authorization for registry modifications
- Layer 2: Profile signature must match merchant’s authorized key (separate from registry)
- Audit logs track all registry changes

**Attack: Time-Based Attacks**

Attacker manipulates timestamps to skip fulfillment window.

**Mitigation:**

- All timestamps derived from block.timestamp (immutable)
- Smart contract enforces deadline logic (not TBC)
- State machine validates transitions before execution

**Attack: Discount Double-Spending**

Buyer attempts to redeem same discount coupon multiple times.

**Mitigation:**

- ZK proof includes unique nullifier per redemption
- TBC tracks nullifiers (cannot be reused)
- Smart contract validates single-use per receipt ID

### 8.3 Privacy Considerations

**Consumer Privacy:**

- Buyer pseudonyms used until transaction (wallet address not exposed early)
- ZK proofs enable discount redemption without identity revelation
- Receipt ownership provable without transferring NFT to wallet
- TBC does not log wallet addresses (only pseudonyms)

**Merchant Privacy:**

- Payment Profile addresses are public (necessary for transparency)
- Merchant transaction volume not exposed via TBC
- Fulfillment details remain off-chain when possible

——

## 9. Examples

### 9.1 Happy Path: HTTP 402 Pizza Order

```
1. Consumer browses pizzahut.com, adds large pizza to cart

2. Consumer clicks “Checkout”

3. pizzahut.com returns:
   HTTP/1.1 402 Payment Required
   TGP-Payment-Profile: https://pay.pizzahut.com/profile/store-4521
   TGP-Amount: 30000000
   TGP-Asset: USDC
   TGP-TBC-Endpoint: https://tbc.coreprove.com/query
   TGP-Order-Details: {“item”: “Large Pepperoni Pizza”, “qty”: 1}

4. Extension intercepts 402, displays:
   ┌────────────────────────────────────┐
   │ Pay with CoreProver                │
   ├────────────────────────────────────┤
   │ Merchant: Pizza Hut Store #4521    │
   │ Item: Large Pepperoni Pizza        │
   │ Amount: 30.00 USDC                 │
   │                                    │
   │ [Confirm Payment]  [Cancel]        │
   └────────────────────────────────────┘

5. User clicks “Confirm Payment”

6. Extension sends QUERY to TBC:
   {
     “phase”: “QUERY”,
     “from”: “buyer://anon-abc123”,
     “to”: “seller://pizzahut-store-4521”,
     “profile_reference”: “https://pay.pizzahut.com/profile/store-4521”,
     “amount”: 30000000,
     “asset”: “USDC”
   }

7. TBC performs onion verification:
   ✓ Layer 1: Merchant registry check - PASS (profile active)
   ✓ Layer 2: Signature verification - PASS (valid merchant signature)
   ✓ Layer 3: Contract verification - PASS (bytecode matches, state valid)
   ✓ Layer 4: ZK attestation - SKIPPED (not required for $30)
   ✓ Layer 5: Policy engine - PASS (all rules satisfied)

8. TBC returns Economic Envelope:
   {
     “status”: “APPROVED”,
     “envelope”: {
       “verified_contract_address”: “0x742d35Cc6634C0532925a3b844Bc454e4438f44e”,
       “amount”: 30000000,
       “asset_address”: “0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48”,
       “policy_terms”: {
         “acceptance_window_seconds”: 1800,
         “fulfillment_window_seconds”: 3600,
         “enables_late_discount”: true
       }
     }
   }

9. Extension constructs transaction, passes to MetaMask

10. User sees MetaMask prompt:
    ┌────────────────────────────────────┐
    │ Pizza Hut Store #4521              │
    │ wants you to confirm a transaction │
    ├────────────────────────────────────┤
    │ Send 30 USDC                       │
    │ To: 0x742d35Cc6634...              │
    │ Gas: ~$2.50                        │
    │                                    │
    │ [Reject]          [Confirm]        │
    └────────────────────────────────────┘

11. User clicks “Confirm”, transaction submitted

12. Blockchain: Escrow created (BUYER_COMMITTED state)

13. Pizza Hut backend receives event, accepts order

14. Pizza prepared and delivered (within 1 hour)

15. Pizza Hut marks fulfilled on-time

16. Pizza Hut claims payment

17. Receipt NFT minted (no discount, on-time delivery)

18. Extension displays:
    ┌────────────────────────────────────┐
    │ ✓ Payment Successful               │
    ├────────────────────────────────────┤
    │ Receipt #12345                     │
    │ 30.00 USDC paid to Pizza Hut       │
    │ Delivered on time                  │
    │                                    │
    │ [View Receipt]                     │
    └────────────────────────────────────┘
```

### 9.2 Direct Pay: Coffee Shop Tip

```
1. User opens CoreProver Extension

2. User clicks “Direct Pay”

3. User enters:
   - Amount: 5 USDC
   - Clicks “Scan QR”

4. User scans QR code on coffee shop counter
   QR contains: https://pay.coffeeshop.com/profile/tips

5. Extension displays:
   ┌────────────────────────────────────┐
   │ Direct Pay                         │
   ├────────────────────────────────────┤
   │ To: Coffee Shop (tips)             │
   │ Amount: 5.00 USDC                  │
   │                                    │
   │ [Send Payment]  [Cancel]           │
   └────────────────────────────────────┘

6. User clicks “Send Payment”

7. Extension → TBC (same verification flow)

8. TBC → Extension (Economic Envelope)

9. Extension → Wallet (transaction prompt)

10. User approves in wallet

11. Payment escrow created, shop accepts immediately

12. Shop marks fulfilled (instant for tips)

13. Shop claims payment

14. Extension displays success confirmation
```

### 9.3 Late Fulfillment with Discount

```
1-12. [Same as happy path]

13. Fulfillment window expires (1 hour passed, no fulfillment)

14. Blockchain emits: tgp.fulfillment.expired
    - Buyer withdrawal now UNLOCKED

15. Pizza Hut delivers pizza 15 minutes late

16. Pizza Hut marks fulfilled:
    - Blockchain emits: tgp.seller.latefulfilled
    - Discount issued: 10% off next order
    - Discount expires: 90 days
    - Buyer withdrawal RE-LOCKED

17. Pizza Hut claims payment (still receives full amount)

18. Receipt NFT minted with discount metadata:
    {
      “receipt_id”: 12346,
      “late_fulfilled”: true,
      “discount_pct”: 10,
      “discount_expiration”: “2026-02-16T12:00:00Z”
    }

19. Extension displays:
    ┌────────────────────────────────────┐
    │ ✓ Payment Complete                 │
    ├────────────────────────────────────┤
    │ Receipt #12346                     │
    │ 30.00 USDC paid to Pizza Hut       │
    │ ⚠ Delivered late                   │
    │                                    │
    │ 🎁 You earned 10% off next order!  │
    │    Expires: Feb 16, 2026           │
    │                                    │
    │ [View Receipt] [Use Discount]      │
    └────────────────────────────────────┘

20. Next order, user clicks “Use Discount”

21. Extension includes ZK proof in QUERY

22. TBC validates proof, L8 applies 10% discount

23. User pays 27 USDC instead of 30 USDC

24. Discount marked as redeemed (cannot be reused)
```

### 9.4 TBC Denial: Merchant Disabled

```
1-6. [Same as happy path through user confirmation]

7. TBC performs onion verification:
   ✗ Layer 1: Merchant registry check - FAIL (profile disabled)

8. TBC returns error:
   {
     “status”: “DENIED”,
     “error”: “MERCHANT_DISABLED”,
     “reason”: “Merchant payment profile is currently disabled”,
     “code”: “TBC_L1_REGISTRY_FAIL”,
     “user_message”: “This merchant is temporarily unavailable. Please try again later.”
   }

9. Extension displays:
   ┌────────────────────────────────────┐
   │ ⚠ Payment Cannot Be Processed      │
   ├────────────────────────────────────┤
   │ This merchant is temporarily       │
   │ unavailable. Please try again      │
   │ later or contact support.          │
   │                                    │
   │ Reference: TBC_L1_REGISTRY_FAIL    │
   │                                    │
   │ [OK]                               │
   └────────────────────────────────────┘

10. No wallet interaction (transaction blocked before user approval)
```

——

## 10. Implementation Checklist

### 10.1 CoreProver Browser Extension

- [x] HTTP 402 interception
- [x] Direct Pay UI
- [x] QR code scanner
- [x] Policy selection interface
- [x] TGP QUERY message construction
- [x] TBC endpoint resolution
- [x] Economic Envelope parsing
- [ ] Wallet transaction handoff
- [ ] Transaction monitoring
- [ ] Receipt display
- [ ] Discount redemption UI
- [ ] ZK proof generation for discounts
- [ ] Error handling and retry logic

### 10.2 Transaction Border Controller (TBC)

- [ ] QUERY message endpoint
- [ ] Layer 1: Merchant Registry integration
- [ ] Layer 2: Signature verification
- [ ] Layer 3: Quorum RPC verification
- [ ] Layer 4: ZK attestation (optional mode)
- [ ] Layer 5: Policy engine
- [ ] Economic Envelope construction
- [ ] Error response formatting
- [ ] Rate limiting
- [ ] Logging and monitoring
- [ ] Merchant Registry API client
- [ ] Multi-chain support

### 10.3 Contract Brokerage Service (CBS)

- [x] Merchant Portal authentication
- [x] Payment Profile creation UI
- [ ] Smart contract deployment
- [ ] Merchant Registry management
- [ ] Profile signature generation
- [ ] QR code generation
- [ ] HTTP 402 header documentation
- [ ] Profile status management (enable/disable)
- [ ] Deployment fee collection
- [ ] Multi-chain deployment support

### 10.4 Payment Profile Smart Contracts

- [x] Adminless escrow logic
- [x] Dual-commitment state machine
- [x] Time window enforcement
- [x] Withdrawal lock logic
- [x] Late fulfillment detection
- [x] Discount metadata storage
- [x] Receipt NFT minting
- [ ] Timed release mechanism
- [ ] Multi-asset support
- [ ] Gas optimization
- [ ] Audit and security review

### 10.5 Documentation & Integration Guides

- [ ] TGP-00 v3.0 (this document)
- [ ] TGP-MGMT-00 (management protocol)
- [ ] TGP-CP-EXT-00 (extension interface)
- [ ] Merchant integration guide
- [ ] Extension user guide
- [ ] TBC operator guide
- [ ] Smart contract documentation
- [ ] API reference
- [ ] Security best practices

——

## 11. Migration & Versioning

### 11.1 Changes from v2.0

**Major Changes:**

1. Added browser extension as primary consumer interface
1. Introduced TBC as security gateway with onion verification model
1. Renamed “Settlement Profiles” to “Payment Profiles”
1. Clarified CBS role as merchant-only service
1. Added HTTP 402 and Direct Pay consumer flows
1. Defined Economic Envelope as TBC response format
1. Added detailed security layer specifications

**Backward Compatibility:**

- v2.0 QUERY/OFFER/SETTLE messages remain valid
- New fields (profile_reference, tbc_endpoint) are optional for backward compatibility
- Smart contracts unchanged (state machine and events identical)
- Receipt metadata format unchanged

**Deprecation:**

- Direct QUERY → Seller flow deprecated (now QUERY → TBC → Envelope)
- `escrow_from_402` and `escrow_contract_from_402` fields deprecated (replaced by profile_reference)

### 11.2 Version Negotiation

TGP messages include protocol version in headers:

```json
{
  “tgp_version”: “3.0”,
  “phase”: “QUERY”,
  ...
}
```

TBC behavior:

- v3.0 clients: Full security verification, Economic Envelope response
- v2.0 clients: Legacy OFFER message response (deprecated, log warning)
- v1.0 clients: Reject with upgrade-required error

——

## 12. Glossary

**CBS (Contract Brokerage Service)** - Merchant-facing service for deploying payment profile smart contracts

**Direct Pay** - Consumer-initiated payment flow using QR or manual merchant URL entry

**Economic Envelope** - TBC response containing verified contract address and payment parameters

**HTTP 402** - HTTP status code “Payment Required” used to trigger extension payment flow

**Layer 8** - Economic-control plane signaling layer (TGP protocol layer)

**Onion Verification** - Multi-layered security model with progressively stronger checks

**Payment Profile** - Merchant-defined smart contract configuration for escrow (formerly “Settlement Profile”)

**TBC (Transaction Border Controller)** - Non-custodial security gateway validating transactions before fund movement

**TGP-CP-EXT-00** - Browser extension interface specification

**TGP-MGMT-00** - Management protocol specification for lifecycle coordination

——

## 13. References

**Related Specifications:**

- TGP-MGMT-00: Management Protocol
- TGP-CP-EXT-00: Browser Extension Interface
- TGP-01: Layer Routing Specification (deprecated, integrated into v3.0)
- TGP-02: ZK Proof Circuits
- TGP-03: Receipt Vault Implementation

**External Standards:**

- RFC 7231 Section 6.5.2: HTTP 402 Payment Required
- ERC-721: Non-Fungible Token Standard
- ERC-1967: Proxy Standard
- EIP-712: Typed Structured Data Signing

**Implementation Repositories:**

- `coreprover-extension` - Browser extension
- `transaction-border-controller` - TBC service
- `coreprover-contracts` - Smart contracts
- `coreprover-service` - Settlement service
- `merchant-portal` - CBS frontend

**Website & Resources:**

- Main: https://coreprove.com
- Merchant Portal: https://merchant.coreprove.com
- Documentation: https://docs.coreprove.com
- X (Twitter): https://x.com/CoreProve

**Motto:** “Prove what’s needed. Protect what matters.”

——

**Document Version:** 3.1  
**Last Updated:** November 18, 2025  
**Status:** Final  
**Change Control:** CoreProver Development Team

**Changes from v3.0:**

- Applied patch-based updates for structure, CBS/TBC separation, security model, messaging, profiles, and engine extraction
- Moved detailed state machine to TGP-ENGINE-00
- Moved discount mechanics to Appendix B
- Added System Responsibilities Matrix
- Strengthened normative rules for layer sequencing and fail-closed behavior
- Clarified field taxonomies and domain separation

——

## Appendix A: Integration Examples

**Note:** Detailed integration examples (HTTP 402 pizza order, Direct Pay coffee tip, late fulfillment scenarios) are preserved from v3.0 but should be moved to a separate Integration Guide document to reduce specification length.

For implementation examples, see:

- CoreProver Integration Guide
- TGP-CP-EXT-00 (Extension specification with UI examples)
- Merchant Portal Documentation

——

## Appendix B: Discount Coupon Mechanics

### B.1 Overview

When a merchant fulfills an order late (after the fulfillment window expires), the CoreProver Engine automatically issues a discount coupon embedded in the receipt NFT. This coupon can be redeemed on the buyer’s next order with the same merchant.

**Key Properties:**

- **Amount:** 10% off next order (configurable per profile)
- **Expiration:** 90 days from issuance (configurable per profile)
- **Redemption:** Via zero-knowledge proof (privacy-preserving)
- **Single-use:** Nullifier prevents double-redemption

### B.2 Late Fulfillment Flow

```
1. Seller accepts order (withdrawal locks)
2. Fulfillment window expires without seller fulfillment
   → Buyer withdrawal unlocks (can claim refund)
3. Seller delivers late (after expiration)
4. Seller marks fulfilled:
   → Discount issued (10% coupon)
   → Withdrawal RE-LOCKS (buyer cannot withdraw anymore)
5. Seller claims payment (still receives full amount)
6. Receipt NFT minted with discount metadata
```

**Rationale:** Late fulfillment is compensated with future discount, not immediate payment reduction. Seller still receives full payment for completed delivery.

### B.3 Discount Redemption Flow

```
1. Buyer has receipt with valid discount
2. Buyer initiates new order with same merchant
3. Extension generates ZK proof of receipt ownership
4. Extension includes proof in QUERY message
5. TBC validates proof (Layer 9):
   - Verifies ZK proof correctness
   - Checks discount not expired
   - Confirms nullifier unused
6. L8 Router applies 10% discount to order amount
7. Buyer pays reduced amount
8. Discount marked as redeemed (nullifier stored)
```

### B.4 ZK Proof Schema

```json
{
  “circuit”: “receipt_ownership_v1”,
  “proof”: “{proof_bytes}”,
  “public_inputs”: {
    “receipt_id”: “{nft_token_id}”,
    “vault_address”: “{receipt_vault_address}”,
    “discount_pct”: 10,
    “discount_expiration”: “{unix_timestamp}”,
    “redeemed”: false,
    “nullifier”: “{unique_value}”
  },
  “verification_key”: “{vk_hash}”
}
```

**Public Inputs Explained:**

- `receipt_id` - NFT token ID (publicly linkable to past order)
- `discount_pct` - Discount amount
- `discount_expiration` - Unix timestamp when coupon expires
- `redeemed` - Must be false for valid redemption
- `nullifier` - Unique value preventing double-redemption

**Private Inputs (not revealed):**

- Buyer wallet address
- Receipt ownership proof
- Merchant identifier linkage

### B.5 Security: Double-Redemption Prevention

**Attack:** Buyer attempts to use same discount multiple times.

**Mitigation:**

1. ZK proof includes unique nullifier
1. TBC stores used nullifiers in registry
1. On redemption attempt, TBC checks if nullifier was previously used
1. If used → Reject with error “DISCOUNT_ALREADY_REDEEMED”
1. If unused → Accept, store nullifier, proceed with discount

**Nullifier Generation:**

```
nullifier = keccak256(receipt_id || buyer_secret || redemption_context)
```

This ensures:

- Same receipt cannot generate same nullifier twice
- Different buyers cannot use same receipt
- Nullifier reveals nothing about buyer identity

### B.6 Expiration Handling

**Scenario:** Buyer attempts to redeem expired discount.

**TBC Behavior:**

1. Verify ZK proof (proves receipt ownership)
1. Check `discount_expiration` timestamp
1. Compare to current time
1. If expired → Reject with error “DISCOUNT_EXPIRED”
1. If valid → Proceed with discount application

**Error Response:**

```json
{
  “status”: “DENIED”,
  “error”: “DISCOUNT_EXPIRED”,
  “code”: “TBC_L9_DISCOUNT_EXPIRED”,
  “reason”: “Discount coupon expired on 2026-02-16”,
  “user_message”: “This discount has expired. Please proceed with regular payment.”
}
```

### B.7 Implementation Notes

**Layer 8 (Economic) Router:**

- Validates discount proof via Layer 9
- Calculates discounted amount
- Updates QUERY amount before TBC processing
- Logs discount redemption for analytics

**Layer 9 (Identity) Router:**

- Verifies ZK proof correctness
- Checks nullifier registry
- Validates expiration timestamp
- Stores nullifier on successful redemption

**For Complete Specifications:**

- ZK circuit details: TGP-02
- Receipt NFT schema: TGP-03
- State machine integration: TGP-ENGINE-00

——

END OF SPECIFICATION
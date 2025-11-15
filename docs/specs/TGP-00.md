# TGP-00: Transaction Gateway Protocol Specification

**Version:** 2.0  
**Date:** November 14, 2025  
**Status:** Final  
**Authors:** TBC Development Team

-----

## Abstract

The Transaction Gateway Protocol (TGP) is a message-passing protocol for coordinating decentralized commerce transactions across heterogeneous blockchain networks. TGP-00 defines the core message types, state conventions, and routing semantics required to support the CoreProver dual-commitment escrow architecture.

This specification describes:

1. **Message Types** - QUERY, OFFER, SETTLE, and EVENT message schemas
1. **State Machine** - Escrow lifecycle from buyer commitment to settlement
1. **Routing Layers** - L8 (Economic), L9 (Identity), L10 (Policy) responsibilities
1. **Withdrawal Semantics** - Lock, unlock, and re-lock logic
1. **Discount Model** - Late fulfillment compensation via receipt coupons
1. **Receipt Metadata** - ZK-provable settlement records

TGP-00 enables symmetric-risk escrow settlements where both buyer and seller commit before claims unlock, eliminating unilateral advantage and enabling trustless commerce.

-----

## 1. Architecture Summary

### 1.1 Core Principles

**Dual-Commitment Model:**

- Both buyer AND seller must commit before any claims are possible
- Buyer commits payment to escrow
- Seller commits via legal signature (no funds) OR counter-escrow (matching collateral)
- Claims only unlock when both parties have committed

**Seller-Driven Settlement:**

- Seller can claim payment without buyer acknowledgment
- Prevents buyer holdout attacks
- Mirrors real-world cash transactions
- Timed release mechanisms handle edge cases

**Privacy-Preserving Receipts:**

- Receipt NFTs stored permanently in on-chain vault
- Never transferred to buyer wallet
- Buyer proves ownership via zero-knowledge proofs
- Enables discount redemption without identity exposure

### 1.2 Protocol Flow

```
Buyer                  TGP Router              Seller                CoreProver
  |                        |                      |                      |
  |--QUERY--------------->|                      |                      |
  |                        |--route------------->|                      |
  |                        |<-----OFFER----------|                      |
  |<---OFFER--------------|                      |                      |
  |                        |                      |                      |
  |--commit_payment--------------------------------------->createEscrow()|
  |                        |                      |<--EVENT: created-----|
  |                        |<---------tgp.escrow.created-----------------|
  |                        |                      |                      |
  |                        |                      |--accept_order------->|
  |                        |                      |                      |
  |                        |<---------tgp.seller.accepted----------------|
  |                        |                      |                      |
  |  [buyer withdrawal now LOCKED]               |                      |
  |                        |                      |                      |
  |                        |                      |--fulfill_order------>|
  |                        |<---------tgp.seller.fulfilled---------------|
  |                        |                      |                      |
  |                        |                      |--claim_payment------>|
  |                        |<---------tgp.seller.claimed-----------------|
  |                        |<---------tgp.receipt.minted----------------|
  |                        |                      |                      |
  |<---SETTLE-------------|                      |                      |
```

### 1.3 State Machine Overview

```
NONE
  |
  v
BUYER_COMMITTED (payment locked)
  |
  +--[acceptance timeout]---> EXPIRED (buyer can withdraw)
  |
  +--[seller accepts]-------> SELLER_ACCEPTED (withdrawal LOCKED)
      |
      +--[fulfillment timeout]---> FULFILLMENT_EXPIRED (withdrawal UNLOCKED)
      |                                 |
      |                                 +--[late fulfillment]--> RE-LOCKED
      |
      +--[seller fulfills on time]---> SELLER_FULFILLED
           |
           +--[seller claims]---------> SELLER_CLAIMED (terminal)
           |
           +--[claim timeout]---------> TIMED_RELEASE (anyone can trigger)
```

-----

## 2. Message Types

### 2.1 QUERY Message

**Purpose:** Buyer initiates transaction discovery

**Direction:** Buyer → TGP Router → Seller

**Schema:**

```json
{
  "phase": "QUERY",
  "id": "q-{uuid}",
  "from": "buyer://{identifier}",
  "to": "seller://{identifier}",
  "asset": "{token_symbol}",
  "amount": "{integer_wei}",
  "escrow_from_402": true,
  "escrow_contract_from_402": "{contract_address}",
  "zk_profile": "OPTIONAL | {zk_proof_data}",
  "metadata": {
    "product_id": "optional",
    "quantity": "optional",
    "delivery_address": "optional_encrypted"
  }
}
```

**Field Definitions:**

- `id` - Unique query identifier (UUID format recommended)
- `from` - Buyer identifier (may be pseudonymous)
- `to` - Seller identifier or category
- `asset` - Payment token symbol (USDC, WETH, etc.)
- `amount` - Payment amount in smallest unit (wei for ETH, 1e6 for USDC)
- `escrow_from_402` - Always true for CoreProver transactions
- `escrow_contract_from_402` - CoreProver contract address
- `zk_profile` - Optional zero-knowledge proof for privacy
- `metadata` - Application-specific data

**Routing:** L10 validates structure, L9 resolves identities, L8 routes to sellers

-----

### 2.2 OFFER Message

**Purpose:** Seller responds with transaction terms

**Direction:** Seller → TGP Router → Buyer

**Schema:**

```json
{
  "phase": "OFFER",
  "id": "offer-{uuid}",
  "query_id": "{query_id}",
  "from": "seller://{identifier}",
  "to": "buyer://{identifier}",
  "asset": "{token_symbol}",
  "amount": "{integer_wei}",
  "coreprover_contract": "{contract_address}",
  "session_id": "{session_uuid}",
  "zk_required": true,
  "economic_envelope": {
    "max_fees_bps": 50,
    "expiry": "{iso8601_timestamp}"
  },
  "economic_metadata": {
    "enables_late_discount": true,
    "late_discount_pct": 10,
    "discount_expiration_days": 90,
    "acceptance_window_seconds": 1800,
    "fulfillment_window_seconds": 3600,
    "claim_window_seconds": 3600
  },
  "payment_profile": {
    "required_commitment_type": "LEGAL_SIGNATURE | COUNTER_ESCROW",
    "counter_escrow_amount": "{integer_wei}",
    "fulfillment_type": "DIGITAL | SHIPPING | SERVICE",
    "requires_tracking": false,
    "allows_timed_release": true,
    "timed_release_delay": 3600
  }
}
```

**Field Definitions:**

- `query_id` - References originating QUERY message
- `session_id` - Unique session for this transaction
- `economic_envelope` - Fee and expiry constraints
- `economic_metadata` - Escrow timing and discount parameters
  - `enables_late_discount` - Whether late fulfillment triggers discount
  - `late_discount_pct` - Discount percentage (typically 10%)
  - `discount_expiration_days` - Coupon validity period (typically 90)
  - `acceptance_window_seconds` - Deadline for seller acceptance
  - `fulfillment_window_seconds` - Deadline for fulfillment after acceptance
  - `claim_window_seconds` - Window for seller to claim payment
- `payment_profile` - Escrow configuration
  - `required_commitment_type` - How seller must commit
  - `counter_escrow_amount` - Collateral required if COUNTER_ESCROW
  - `fulfillment_type` - Delivery method category
  - `allows_timed_release` - Whether automatic release is enabled

**Routing:** L8 validates pricing, L9 verifies seller identity, L10 checks policy compliance

-----

### 2.3 SETTLE Message

**Purpose:** Confirms transaction completion

**Direction:** Controller/Watcher → TGP Router → Participants

**Schema:**

```json
{
  "phase": "SETTLE",
  "id": "settle-{uuid}",
  "query_or_offer_id": "{query_id or offer_id}",
  "success": true,
  "source": "controller-watcher | manual",
  "layer8_tx": "{transaction_hash}",
  "session_id": "{session_uuid}",
  "escrow_state": "SELLER_CLAIMED",
  "fulfillment_metadata": {
    "on_time": true,
    "late_fulfilled": false,
    "discount_pct": 0,
    "discount_expiration": null,
    "fulfillment_timestamp": "{unix_timestamp}",
    "settlement_timestamp": "{unix_timestamp}",
    "receipt_id": "{nft_token_id}",
    "buyer_withdrawal_locked": false,
    "next_discount_available": false
  }
}
```

**Field Definitions:**

- `success` - Whether settlement succeeded
- `source` - Origin of settlement notification
- `layer8_tx` - On-chain transaction hash
- `escrow_state` - Final state from state machine
- `fulfillment_metadata` - Delivery and discount details
  - `on_time` - Whether fulfilled within window
  - `late_fulfilled` - Whether fulfilled after expiration
  - `discount_pct` - Discount percentage issued (if late)
  - `discount_expiration` - Unix timestamp when discount expires
  - `receipt_id` - Receipt NFT token ID
  - `buyer_withdrawal_locked` - Current withdrawal lock status
  - `next_discount_available` - Whether buyer can claim discount on next order

**Routing:** L8 records economics, L9 updates reputations, L10 finalizes policy

-----

### 2.4 EVENT Messages

**Purpose:** Broadcast state transitions and lifecycle events

**Direction:** CoreProver Contracts → TGP Router → Subscribers

#### 2.4.1 `tgp.escrow.created`

**Emitted when:** Buyer commits payment to escrow

```json
{
  "event": "tgp.escrow.created",
  "order_id": "{bytes32_hex}",
  "buyer": "{address}",
  "seller": "{address}",
  "amount": "{integer_wei}",
  "asset": "{token_address}",
  "acceptance_deadline": "{unix_timestamp}",
  "buyer_withdrawal_locked": false,
  "state": "BUYER_COMMITTED"
}
```

#### 2.4.2 `tgp.seller.accepted`

**Emitted when:** Seller accepts order via legal signature

**Critical:** This event locks buyer withdrawal

```json
{
  "event": "tgp.seller.accepted",
  "order_id": "{bytes32_hex}",
  "seller": "{identifier}",
  "acceptance_timestamp": "{unix_timestamp}",
  "fulfillment_deadline": "{unix_timestamp}",
  "buyer_withdrawal_locked": true,
  "state": "SELLER_ACCEPTED"
}
```

**Routing Implications:**

- L8: Activate pricing guarantees
- L9: Log seller commitment for reputation
- L10: Begin monitoring fulfillment window

#### 2.4.3 `tgp.seller.fulfilled`

**Emitted when:** Seller marks order as fulfilled within window

```json
{
  "event": "tgp.seller.fulfilled",
  "order_id": "{bytes32_hex}",
  "fulfillment_timestamp": "{unix_timestamp}",
  "on_time": true,
  "late_fulfilled": false,
  "buyer_withdrawal_locked": true,
  "state": "SELLER_FULFILLED"
}
```

**Routing Implications:**

- L8: Confirm no discount applicable
- L9: Positive reputation signal
- L10: Transition to claim phase

#### 2.4.4 `tgp.fulfillment.expired`

**Emitted when:** Fulfillment window expires without seller fulfillment

**Critical:** This event unlocks buyer withdrawal

```json
{
  "event": "tgp.fulfillment.expired",
  "order_id": "{bytes32_hex}",
  "expiration_timestamp": "{unix_timestamp}",
  "buyer_withdrawal_unlocked": true,
  "seller_can_still_fulfill": true,
  "state": "FULFILLMENT_EXPIRED"
}
```

**Routing Implications:**

- L8: Prepare for potential discount
- L9: Negative reputation signal for seller
- L10: Enable buyer withdrawal path

#### 2.4.5 `tgp.seller.latefulfilled`

**Emitted when:** Seller fulfills after fulfillment window expired

**Critical:** This event re-locks buyer withdrawal and issues discount

```json
{
  "event": "tgp.seller.latefulfilled",
  "order_id": "{bytes32_hex}",
  "fulfillment_timestamp": "{unix_timestamp}",
  "late_fulfilled": true,
  "discount_pct": 10,
  "discount_expiration": "{unix_timestamp}",
  "buyer_withdrawal_locked": true,
  "next_discount_available": true,
  "state": "SELLER_FULFILLED"
}
```

**Routing Implications:**

- L8: Generate discount token, update pricing for next order
- L9: Mixed reputation signal (late but completed)
- L10: Re-lock buyer withdrawal, enable seller claim

#### 2.4.6 `tgp.seller.claimed`

**Emitted when:** Seller successfully claims payment

```json
{
  "event": "tgp.seller.claimed",
  "order_id": "{bytes32_hex}",
  "seller": "{identifier}",
  "amount": "{integer_wei}",
  "receipt_id": "{nft_token_id}",
  "claim_timestamp": "{unix_timestamp}",
  "state": "SELLER_CLAIMED"
}
```

#### 2.4.7 `tgp.receipt.minted`

**Emitted when:** Receipt NFT is minted to Receipt Vault

```json
{
  "event": "tgp.receipt.minted",
  "receipt_id": "{nft_token_id}",
  "order_id": "{bytes32_hex}",
  "buyer": "{pseudonym}",
  "seller": "{identifier}",
  "metadata": {
    "session_id": "{session_uuid}",
    "order_amount": "{integer_wei}",
    "late_fulfilled": false,
    "discount_pct": 0,
    "discount_expiration": null,
    "fulfillment_timestamp": "{unix_timestamp}",
    "settlement_timestamp": "{unix_timestamp}"
  }
}
```

#### 2.4.8 `tgp.receipt.metadata.discount`

**Emitted when:** Receipt contains discount coupon metadata

```json
{
  "event": "tgp.receipt.metadata.discount",
  "receipt_id": "{nft_token_id}",
  "discount_pct": 10,
  "discount_expiration": "{unix_timestamp}",
  "valid": true,
  "reason": "late_fulfillment"
}
```

**Routing Implications:**

- L8: Apply discount on next order if presented via ZK proof
- L9: Track discount redemption without buyer identity exposure
- L10: Validate expiration and single-use constraints

-----

## 3. Routing Layers

### 3.1 Layer 8 (Economic) Router

**Responsibilities:**

1. **Pricing & Fee Management**
- Validate payment amounts against product pricing
- Calculate and enforce maximum fee thresholds
- Handle multi-asset payment scenarios
1. **Discount Token Management**
- Generate discount tokens on `tgp.seller.latefulfilled` event
- Validate discount tokens via ZK proof on subsequent orders
- Enforce 90-day expiration period
- Prevent double redemption
1. **Payment Profile Optimization**
- Track seller fulfillment performance
- Adjust recommended window timings
- Optimize fee structures based on historical data

**Discount Decision Flow:**

```
New Order Query
  |
  +-- Check for ZK proof of previous receipt
  |
  +-- Validate discount eligibility:
      |
      +-- Not expired (< 90 days)
      +-- Not previously redeemed
      +-- Matches buyer identity (via ZK)
      |
  +-- Apply 10% discount to order amount
  |
  +-- Emit pricing confirmation with discount applied
```

**Example Implementation:**

```rust
pub struct L8Router {
    discount_registry: DiscountRegistry,
    price_oracle: PriceOracle,
}

impl L8Router {
    pub async fn handle_query(&self, query: QueryMessage) -> Result<RouteDecision> {
        // Check for discount eligibility
        let discount = if let Some(zk_proof) = query.zk_profile {
            self.validate_discount_proof(&zk_proof).await?
        } else {
            None
        };

        // Calculate final price
        let base_price = self.price_oracle.get_price(&query.asset).await?;
        let final_price = if let Some(discount) = discount {
            base_price * (100 - discount.pct) / 100
        } else {
            base_price
        };

        Ok(RouteDecision::Accept { price: final_price, discount })
    }

    pub async fn handle_late_fulfillment(&self, order_id: &str) -> Result<()> {
        // Issue discount token
        let discount = DiscountToken {
            receipt_id: order_id.to_string(),
            pct: 10,
            expiration: Utc::now() + Duration::days(90),
            redeemed: false,
        };

        self.discount_registry.store(discount).await?;
        Ok(())
    }
}
```

-----

### 3.2 Layer 9 (Identity) Router

**Responsibilities:**

1. **Pseudonym Management**
- Resolve buyer/seller identifiers to on-chain addresses
- Maintain privacy-preserving identity mappings
- Support ZK-based identity proofs
1. **Receipt-Based Identity**
- Tie discount eligibility to wallet or ZK pseudonym
- Enable proof of receipt ownership without revealing identity
- Track redemption status per receipt
1. **Reputation Aggregation**
- Aggregate `late_fulfilled` events per seller
- Calculate on-time fulfillment rates
- Surface reputation metrics to buyers

**ZK Proof Schema:**

```json
{
  "claim": "I own receipt with valid discount",
  "proof": "{zk_proof_bytes}",
  "public_inputs": {
    "receipt_id": "{nft_token_id}",
    "discount_pct": 10,
    "expiration": "{unix_timestamp}",
    "redeemed": false,
    "nullifier": "{unique_per_redemption}"
  }
}
```

**Example Implementation:**

```rust
pub struct L9Router {
    identity_registry: IdentityRegistry,
    reputation_store: ReputationStore,
}

impl L9Router {
    pub async fn resolve_identity(&self, identifier: &str) -> Result<Address> {
        self.identity_registry.resolve(identifier).await
    }

    pub async fn verify_receipt_proof(&self, proof: &ZkProof) -> Result<bool> {
        // Verify ZK proof of receipt ownership
        let valid = zk::verify_proof(proof)?;
        
        if valid {
            // Check nullifier hasn't been used
            let nullifier = proof.public_inputs.get("nullifier")?;
            let used = self.identity_registry.check_nullifier(nullifier).await?;
            Ok(!used)
        } else {
            Ok(false)
        }
    }

    pub async fn update_seller_reputation(&self, seller: &str, late: bool) {
        self.reputation_store.record_fulfillment(seller, late).await;
    }
}
```

-----

### 3.3 Layer 10 (Policy) Router

**Responsibilities:**

1. **Withdrawal Lock Enforcement**
- Validate `buyer_withdrawal_locked` flag before allowing withdrawal
- Enforce state machine transitions
- Reject invalid state changes
1. **Window Expiration Validation**
- Monitor acceptance_deadline
- Monitor fulfillment_deadline
- Trigger automatic state transitions
1. **Timed Release Management**
- Allow anyone to trigger release after claim_window expires
- Enforce that only seller receives funds after late fulfillment
- Prevent buyer claims after seller fulfillment

**State Validation Rules:**

```rust
pub fn can_buyer_withdraw(
    state: EscrowState,
    now: u64,
    acceptance_deadline: u64,
    fulfillment_deadline: u64
) -> bool {
    match state {
        EscrowState::BuyerCommitted => now > acceptance_deadline,
        EscrowState::FulfillmentExpired => true,
        _ => false
    }
}

pub fn can_seller_claim(
    state: EscrowState,
    fulfillment_timestamp: Option<u64>
) -> bool {
    match state {
        EscrowState::SellerFulfilled => true,
        EscrowState::SellerAccepted if fulfillment_timestamp.is_some() => true,
        _ => false
    }
}
```

**Example Implementation:**

```rust
pub struct L10Router {
    policy_engine: PolicyEngine,
    state_monitor: StateMonitor,
}

impl L10Router {
    pub async fn validate_withdrawal(&self, order_id: &str, actor: &str) -> Result<bool> {
        let escrow = self.state_monitor.get_escrow(order_id).await?;
        let now = current_timestamp();

        let allowed = match actor {
            "buyer" => can_buyer_withdraw(
                escrow.state,
                now,
                escrow.acceptance_deadline,
                escrow.fulfillment_deadline
            ),
            "seller" => can_seller_claim(
                escrow.state,
                escrow.fulfillment_timestamp
            ),
            _ => false
        };

        Ok(allowed)
    }

    pub async fn check_expiration(&self, order_id: &str) -> Result<Option<StateTransition>> {
        let escrow = self.state_monitor.get_escrow(order_id).await?;
        let now = current_timestamp();

        let transition = match escrow.state {
            EscrowState::BuyerCommitted if now > escrow.acceptance_deadline => {
                Some(StateTransition::ToExpired)
            },
            EscrowState::SellerAccepted if now > escrow.fulfillment_deadline => {
                Some(StateTransition::ToFulfillmentExpired)
            },
            _ => None
        };

        Ok(transition)
    }
}
```

-----

## 4. Escrow State Machine

### 4.1 State Definitions

```rust
pub enum EscrowState {
    None,
    BuyerCommitted,
    SellerAccepted,
    SellerFulfilled,
    FulfillmentExpired,
    SellerClaimed,
    BuyerClaimed,
    Expired,
}
```

### 4.2 State Transitions

|From State        |Event                  |To State          |Notes                         |
|------------------|-----------------------|------------------|------------------------------|
|None              |buyer commits          |BuyerCommitted    |Payment locked in escrow      |
|BuyerCommitted    |acceptance timeout     |Expired           |Buyer can withdraw            |
|BuyerCommitted    |seller accepts         |SellerAccepted    |Buyer withdrawal LOCKED       |
|SellerAccepted    |fulfillment timeout    |FulfillmentExpired|Buyer withdrawal UNLOCKED     |
|SellerAccepted    |seller fulfills on time|SellerFulfilled   |Withdrawal remains locked     |
|FulfillmentExpired|seller fulfills late   |SellerFulfilled   |Withdrawal RE-LOCKED, discount|
|SellerFulfilled   |seller claims          |SellerClaimed     |Terminal state                |
|SellerFulfilled   |claim timeout          |SellerClaimed     |Timed release                 |
|Expired           |buyer withdraws        |BuyerClaimed      |Terminal state                |

### 4.3 Withdrawal Lock Logic

**Locked States (buyer cannot withdraw):**

- `SellerAccepted`
- `SellerFulfilled`
- Any state where `buyer_withdrawal_locked = true`

**Unlocked States (buyer can withdraw):**

- `BuyerCommitted` after acceptance timeout
- `FulfillmentExpired`
- Any state where `buyer_withdrawal_locked = false`

**Re-lock Trigger:**

- When seller fulfills after `FulfillmentExpired`
- `tgp.seller.latefulfilled` event emitted
- Discount issued
- Buyer withdrawal locked again

### 4.4 Time Windows

**Acceptance Window:**

- Starts: When buyer commits
- Duration: Configurable (typically 30 minutes)
- Expires: `acceptance_deadline` timestamp
- Effect: If seller doesn’t accept, buyer can withdraw

**Fulfillment Window:**

- Starts: When seller accepts
- Duration: Configurable (typically 1 hour for services)
- Expires: `fulfillment_deadline` timestamp
- Effect: If seller doesn’t fulfill, buyer can withdraw

**Claim Window:**

- Starts: When seller fulfills
- Duration: Configurable (typically 1 hour)
- Expires: Timed release can be triggered
- Effect: Anyone can trigger payment to seller

-----

## 5. Receipt Metadata

### 5.1 On-Chain Storage

Receipt NFTs include the following metadata stored in the Receipt Vault:

```solidity
struct ReceiptMetadata {
    bytes32 session_id;
    uint128 order_amount;
    bool late_fulfilled;
    uint8 discount_pct;
    uint64 discount_expiration;
    uint64 fulfillment_timestamp;
    uint64 settlement_timestamp;
}
```

### 5.2 Off-Chain Metadata URI

Receipt NFTs reference off-chain metadata for richer context:

```json
{
  "name": "Order Receipt #{receipt_id}",
  "description": "Receipt for order {order_id}",
  "image": "ipfs://{image_hash}",
  "attributes": [
    {
      "trait_type": "Order Amount",
      "value": "{amount} {asset}"
    },
    {
      "trait_type": "Late Fulfilled",
      "value": "{true|false}"
    },
    {
      "trait_type": "Discount",
      "value": "{pct}%"
    },
    {
      "trait_type": "Discount Expires",
      "value": "{date}"
    },
    {
      "trait_type": "Seller",
      "value": "{seller_identifier}"
    },
    {
      "trait_type": "Fulfillment Date",
      "value": "{date}"
    }
  ]
}
```

### 5.3 ZK Proof Schema

Buyers prove receipt ownership without revealing identity:

```json
{
  "circuit": "receipt_ownership_v1",
  "proof": "{proof_bytes}",
  "public_inputs": {
    "receipt_id": "{nft_token_id}",
    "vault_address": "{receipt_vault_address}",
    "discount_pct": 10,
    "discount_expiration": "{unix_timestamp}",
    "redeemed": false,
    "nullifier": "{unique_value}"
  },
  "verification_key": "{vk_hash}"
}
```

-----

## 6. Security Considerations

### 6.1 Discount Abuse Prevention

**Attack Vector:** Buyer attempts to redeem discount multiple times

**Mitigation:**

- Receipt vault tracks redemption status on-chain
- ZK proof includes unique nullifier per redemption
- L10 validates nullifier hasn’t been used
- Double redemption rejected by state machine

### 6.2 Re-Lock Bypass

**Attack Vector:** Buyer tries to withdraw after late fulfillment

**Mitigation:**

- State machine enforces `LateFulfilled -> SellerClaimed` only
- L10 rejects withdrawal when `buyer_withdrawal_locked = true`
- Smart contract validates state transitions before execution
- Events provide audit trail

### 6.3 Time Manipulation

**Attack Vector:** Seller manipulates timestamps to avoid late fulfillment

**Mitigation:**

- All timestamps derived from `block.timestamp` (immutable)
- Deadlines calculated and stored at commitment time
- No off-chain timing dependencies
- Validators reject out-of-order state transitions

### 6.4 Front-Running

**Attack Vector:** Attacker observes pending transaction and front-runs claim

**Mitigation:**

- Commit-reveal pattern for sensitive operations
- State machine only allows rightful claimant per state
- Time locks prevent immediate claims
- MEV protection via private transaction pools (optional)

### 6.5 Gas Griefing

**Attack Vector:** Malicious actor creates many small escrows to congest network

**Mitigation:**

- Minimum escrow threshold enforced
- Gas limits on contract functions
- Rate limiting at service layer
- Economic disincentives via fees

-----

## 7. Examples

### 7.1 Happy Path: Pizza Delivery (On-Time)

```
1. Buyer sends QUERY
{
  "phase": "QUERY",
  "id": "q-123",
  "from": "buyer://alice",
  "to": "seller://pizza_hut_4521",
  "asset": "USDC",
  "amount": 30000000,
  "escrow_from_402": true,
  "escrow_contract_from_402": "0x742d35..."
}

2. Seller responds with OFFER
{
  "phase": "OFFER",
  "id": "offer-456",
  "query_id": "q-123",
  "asset": "USDC",
  "amount": 30000000,
  "economic_metadata": {
    "enables_late_discount": true,
    "late_discount_pct": 10,
    "discount_expiration_days": 90,
    "acceptance_window_seconds": 1800,
    "fulfillment_window_seconds": 3600,
    "claim_window_seconds": 3600
  }
}

3. Buyer commits payment
[On-chain transaction]
EVENT: tgp.escrow.created
{
  "order_id": "0xabcd...",
  "buyer_withdrawal_locked": false
}

4. Seller accepts order
[On-chain transaction]
EVENT: tgp.seller.accepted
{
  "order_id": "0xabcd...",
  "buyer_withdrawal_locked": true,
  "fulfillment_deadline": 1699903600
}

5. Seller delivers pizza
[Off-chain delivery]

6. Seller marks fulfilled
[On-chain transaction]
EVENT: tgp.seller.fulfilled
{
  "order_id": "0xabcd...",
  "on_time": true,
  "late_fulfilled": false
}

7. Seller claims payment
[On-chain transaction]
EVENT: tgp.seller.claimed
{
  "order_id": "0xabcd...",
  "receipt_id": 12345
}

EVENT: tgp.receipt.minted
{
  "receipt_id": 12345,
  "metadata": {
    "late_fulfilled": false,
    "discount_pct": 0
  }
}

8. Router sends SETTLE
{
  "phase": "SETTLE",
  "success": true,
  "fulfillment_metadata": {
    "on_time": true,
    "late_fulfilled": false,
    "buyer_withdrawal_locked": false,
    "next_discount_available": false
  }
}
```

-----

### 7.2 Late Fulfillment: Pizza Delivery (Discount Issued)

```
1-4. [Same as happy path]

5. Fulfillment window expires
[No action from seller]
EVENT: tgp.fulfillment.expired
{
  "order_id": "0xabcd...",
  "buyer_withdrawal_unlocked": true
}

6. Seller delivers pizza late
[Off-chain delivery, 15 minutes late]

7. Seller marks fulfilled
[On-chain transaction]
EVENT: tgp.seller.latefulfilled
{
  "order_id": "0xabcd...",
  "late_fulfilled": true,
  "discount_pct": 10,
  "discount_expiration": 1707680000,
  "buyer_withdrawal_locked": true,
  "next_discount_available": true
}

8. Seller claims payment
[On-chain transaction]
EVENT: tgp.seller.claimed
EVENT: tgp.receipt.minted
{
  "receipt_id": 12346,
  "metadata": {
    "late_fulfilled": true,
    "discount_pct": 10,
    "discount_expiration": 1707680000
  }
}

EVENT: tgp.receipt.metadata.discount
{
  "receipt_id": 12346,
  "discount_pct": 10,
  "discount_expiration": 1707680000,
  "valid": true,
  "reason": "late_fulfillment"
}

9. Router sends SETTLE
{
  "phase": "SETTLE",
  "success": true,
  "fulfillment_metadata": {
    "late_fulfilled": true,
    "discount_pct": 10,
    "buyer_withdrawal_locked": false,
    "next_discount_available": true
  }
}
```

-----

### 7.3 Buyer Withdrawal: Seller Never Accepts

```
1. Buyer sends QUERY
[Standard QUERY message]

2. Seller responds with OFFER
[Standard OFFER with 30-minute acceptance window]

3. Buyer commits payment
EVENT: tgp.escrow.created
{
  "order_id": "0xabcd...",
  "acceptance_deadline": 1699901800
}

4. Seller does not accept
[30 minutes pass]

5. Acceptance deadline expires
EVENT: tgp.escrow.expired
{
  "order_id": "0xabcd...",
  "buyer_withdrawal_unlocked": true
}

6. Buyer withdraws funds
[On-chain transaction]
EVENT: tgp.buyer.claimed
{
  "order_id": "0xabcd...",
  "amount": 30000000
}

7. Router sends SETTLE
{
  "phase": "SETTLE",
  "success": false,
  "reason": "seller_never_accepted",
  "buyer_withdrawal_locked": false
}
```

-----

### 7.4 Discount Redemption: Next Order

```
1. Buyer has receipt #12346 with 10% discount

2. Buyer sends QUERY with ZK proof
{
  "phase": "QUERY",
  "id": "q-789",
  "from": "buyer://alice",
  "to": "seller://pizza_hut_4521",
  "asset": "USDC",
  "amount": 30000000,
  "zk_profile": {
    "circuit": "receipt_ownership_v1",
    "proof": "{proof_bytes}",
    "public_inputs": {
      "receipt_id": 12346,
      "discount_pct": 10,
      "discount_expiration": 1707680000,
      "redeemed": false,
      "nullifier": "0xunique..."
    }
  }
}

3. L8 Router validates ZK proof
- Verifies proof correctness
- Checks discount hasn't expired
- Confirms nullifier unused
- Validates receipt ownership

4. L8 Router applies discount
- Original amount: 30000000 (30 USDC)
- Discount: 10%
- Final amount: 27000000 (27 USDC)

5. Seller responds with discounted OFFER
{
  "phase": "OFFER",
  "id": "offer-999",
  "query_id": "q-789",
  "amount": 27000000,
  "discount_applied": true,
  "original_amount": 30000000
}

6. Transaction proceeds normally
[Standard flow with discounted price]

7. L8 Router marks discount redeemed
[Updates nullifier registry]
```

-----

## 8. Implementation Checklist

### 8.1 Smart Contract Updates

- [ ] Implement `ReceiptMetadata` struct in Receipt Vault
- [ ] Add state transition validation functions
- [ ] Emit `tgp.seller.accepted` event on acceptance
- [ ] Emit `tgp.seller.fulfilled` event on fulfillment
- [ ] Emit `tgp.fulfillment.expired` event on deadline
- [ ] Emit `tgp.seller.latefulfilled` event on late fulfillment
- [ ] Emit `tgp.seller.claimed` event on claim
- [ ] Emit `tgp.receipt.minted` event on receipt creation
- [ ] Emit `tgp.receipt.metadata.discount` when discount included
- [ ] Implement withdrawal lock validation
- [ ] Store discount metadata in receipt
- [ ] Add time window enforcement
- [ ] Implement timed release mechanism

### 8.2 Protocol Updates

- [ ] Update OFFER message schema with `economic_metadata`
- [ ] Update SETTLE message schema with `fulfillment_metadata`
- [ ] Add `buyer_withdrawal_locked` field to policy checks
- [ ] Add `next_discount_available` field to routing decisions
- [ ] Document new event types in specification
- [ ] Create event schemas for all new events
- [ ] Define routing implications per layer

### 8.3 Router Updates

#### L8 (Economic)

- [ ] Implement discount token generation
- [ ] Add ZK proof verification for discount redemption
- [ ] Create discount expiration tracking
- [ ] Add double-redemption prevention
- [ ] Implement pricing adjustments with discount
- [ ] Store nullifier registry

#### L9 (Identity)

- [ ] Implement receipt-based pseudonym tracking
- [ ] Add ZK proof verification for receipt ownership
- [ ] Create seller reputation aggregation
- [ ] Add late fulfillment rate calculation
- [ ] Implement nullifier checking

#### L10 (Policy)

- [ ] Enforce withdrawal lock validation
- [ ] Implement state machine transition checks
- [ ] Add automatic state transitions on deadlines
- [ ] Implement timed release triggering
- [ ] Add time window monitoring
- [ ] Validate re-lock conditions

### 8.4 Testing

- [ ] Test on-time fulfillment (no discount)
- [ ] Test late fulfillment (discount issued)
- [ ] Test re-lock logic (withdrawal blocked after late fulfillment)
- [ ] Test discount expiration (90 days)
- [ ] Test discount redemption via ZK proof
- [ ] Test double-redemption prevention
- [ ] Test buyer withdrawal after acceptance timeout
- [ ] Test buyer withdrawal after fulfillment timeout
- [ ] Test timed release after seller forgets to claim
- [ ] Test nullifier collision prevention
- [ ] Test state machine edge cases
- [ ] Fuzz test time window handling

-----

## 9. Migration & Compatibility

### 9.1 Backward Compatibility

**Message Compatibility:**

- New fields are optional in all messages
- Old controllers can ignore `economic_metadata`
- Old receipts remain valid (no discount field = no discount)
- Version negotiation via protocol headers

**State Compatibility:**

- New states are additive (no removal)
- Old state queries return compatible data
- Event schemas include version numbers

### 9.2 Upgrade Path

**For Existing Deployments:**

1. Deploy new Receipt Vault with `ReceiptMetadata` support
1. Update CoreProver contracts with new events
1. Migrate L8/L9/L10 routers to handle new fields
1. Enable late discount feature per merchant
1. Run parallel systems during transition
1. Deprecate old system after validation period

**For New Deployments:**

1. Use updated contract suite from `coreprover-contracts` v2.0
1. Enable late discount by default (merchant configurable)
1. Emit all new events for full observability
1. Configure appropriate time windows per use case

-----

## 10. Glossary

**Acceptance Window** - Time period during which seller must accept order

**Buyer Withdrawal Lock** - State flag preventing buyer from claiming refund

**Counter-Escrow** - Seller commitment method requiring matching collateral

**Discount Token** - Coupon issued for late fulfillment, redeemable via ZK proof

**Dual-Commitment** - Requirement for both buyer and seller to commit before claims unlock

**Fulfillment Window** - Time period during which seller must complete delivery

**Late Fulfillment** - Delivery completed after fulfillment window expires

**Legal Signature** - Seller commitment method using legally-binding signature without funds

**Nullifier** - Unique value preventing double-spending of ZK proofs

**Payment Profile** - Seller-defined configuration for escrow parameters

**Receipt Vault** - On-chain storage for receipt NFTs, never transferred to buyers

**Re-Lock** - Restoration of buyer withdrawal lock after late fulfillment

**Session ID** - Unique identifier for transaction lifecycle

**Timed Release** - Automatic payment release after claim window expires

**ZK Proof** - Zero-knowledge proof enabling privacy-preserving verification

-----

## 11. References

**Related Specifications:**

- TGP-01: Layer Routing Specification
- TGP-02: ZK Proof Circuits
- TGP-03: Receipt Vault Implementation

**External Standards:**

- ERC-721: Non-Fungible Token Standard
- ERC-1967: Proxy Standard
- EIP-712: Typed Structured Data Signing

**Implementation Repositories:**

- `transaction-border-controller` - Main repository
- `coreprover-contracts` - Solidity smart contracts
- `coreprover-bridge` - Rust bridge library
- `coreprover-service` - Settlement service

-----

## Appendix A: State Transition Diagram

```
                             NONE
                               |
                    [buyer commits payment]
                               |
                               v
                       BUYER_COMMITTED
                        /            \
          [timeout]   /              \ [seller accepts]
                     /                \
                    v                  v
                EXPIRED          SELLER_ACCEPTED
                   |              /           \
        [buyer withdraws]        /             \ [fulfills on time]
                   |   [timeout]/               \
                   v            v                 v
            BUYER_CLAIMED  FULFILLMENT_EXPIRED  SELLER_FULFILLED
                                |                    |
                    [late fulfill]          [seller claims]
                                |                    |
                                +---->[re-lock]------+
                                                     |
                                                     v
                                               SELLER_CLAIMED
```

-----

## Appendix B: Timing Diagrams

### B.1 On-Time Fulfillment

```
T=0    Buyer commits
|
T=5m   Seller accepts    [withdrawal LOCKED]
|
T=30m  Seller fulfills   [on-time]
|
T=35m  Seller claims     [receipt minted]
```

### B.2 Late Fulfillment with Discount

```
T=0    Buyer commits
|
T=5m   Seller accepts    [withdrawal LOCKED]
|
T=60m  Fulfillment timeout  [withdrawal UNLOCKED]
|
T=75m  Seller fulfills late [withdrawal RE-LOCKED, 10% discount issued]
|
T=80m  Seller claims     [receipt minted with discount metadata]
```

### B.3 Buyer Withdrawal

```
T=0    Buyer commits
|
T=30m  Acceptance timeout   [seller never accepted]
|
T=31m  Buyer withdraws   [refund claimed]
```

-----

**Document Version:** 2.0  
**Last Updated:** November 14, 2025  
**Status:** Final  
**Change Control:** TBC Development Team

-----

END OF SPECIFICATION
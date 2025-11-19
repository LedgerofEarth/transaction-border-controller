# TGP-00: Transaction Gateway Protocol Specification

**Version:** 2.0  
**Date:** November 14, 2025  
**Status:** Final  
**Authors:** TBC Development Team

——

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

——

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
  |—QUERY—————>|                      |                      |
  |                        |—route-————>|                      |
  |                        |<——OFFER-———|                      |
  |<—OFFER—————|                      |                      |
  |                        |                      |                      |
  |—commit_payment—————————————>createEscrow()|
  |                        |                      |<—EVENT: created——|
  |                        |<———tgp.escrow.created——————|
  |                        |                      |                      |
  |                        |                      |—accept_order-——>|
  |                        |                      |                      |
  |                        |<———tgp.seller.accepted-—————|
  |                        |                      |                      |
  |  [buyer withdrawal now LOCKED]               |                      |
  |                        |                      |                      |
  |                        |                      |—fulfill_order——>|
  |                        |<———tgp.seller.fulfilled—————|
  |                        |                      |                      |
  |                        |                      |—claim_payment——>|
  |                        |<———tgp.seller.claimed——————|
  |                        |<———tgp.receipt.minted-—————|
  |                        |                      |                      |
  |<—SETTLE-————|                      |                      |
```

### 1.3 State Machine Overview

```
NONE
  |
  v
BUYER_COMMITTED (payment locked)
  |
  +—[acceptance timeout]—> EXPIRED (buyer can withdraw)
  |
  +—[seller accepts]-——> SELLER_ACCEPTED (withdrawal LOCKED)
      |
      +—[fulfillment timeout]—> FULFILLMENT_EXPIRED (withdrawal UNLOCKED)
      |                                 |
      |                                 +—[late fulfillment]—> RE-LOCKED
      |
      +—[seller fulfills on time]—> SELLER_FULFILLED
           |
           +—[seller claims]———> SELLER_CLAIMED (terminal)
           |
           +—[claim timeout]———> TIMED_RELEASE (anyone can trigger)
```

——

## 2. Message Types

### 2.1 QUERY Message

**Purpose:** Buyer initiates transaction discovery

**Direction:** Buyer → TGP Router → Seller

**Schema:**

```json
{
  “phase”: “QUERY”,
  “id”: “q-{uuid}”,
  “from”: “buyer://{identifier}”,
  “to”: “seller://{identifier}”,
  “asset”: “{token_symbol}”,
  “amount”: “{integer_wei}”,
  “escrow_from_402”: true,
  “escrow_contract_from_402”: “{contract_address}”,
  “zk_profile”: “OPTIONAL | {zk_proof_data}”,
  “metadata”: {
    “product_id”: “optional”,
    “quantity”: “optional”,
    “delivery_address”: “optional_encrypted”
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

——

### 2.2 OFFER Message

**Purpose:** Seller responds with transaction terms

**Direction:** Seller → TGP Router → Buyer

**Schema:**

```json
{
  “phase”: “OFFER”,
  “id”: “offer-{uuid}”,
  “query_id”: “{query_id}”,
  “from”: “seller://{identifier}”,
  “to”: “buyer://{identifier}”,
  “asset”: “{token_symbol}”,
  “amount”: “{integer_wei}”,
  “coreprover_contract”: “{contract_address}”,
  “session_id”: “{session_uuid}”,
  “zk_required”: true,
  “economic_envelope”: {
    “max_fees_bps”: 50,
    “expiry”: “{iso8601_timestamp}”
  },
  “economic_metadata”: {
    “enables_late_discount”: true,
    “late_discount_pct”: 10,
    “discount_expiration_days”: 90,
    “acceptance_window_seconds”: 1800,
    “fulfillment_window_seconds”: 3600,
    “claim_window_seconds”: 3600
  },
  “payment_profile”: {
    “required_commitment_type”: “LEGAL_SIGNATURE | COUNTER_ESCROW”,
    “counter_escrow_amount”: “{integer_wei}”,
    “fulfillment_type”: “DIGITAL | SHIPPING | SERVICE”,
    “requires_tracking”: false,
    “allows_timed_release”: true,
    “timed_release_delay”: 3600
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

——

### 2.3 SETTLE Message

**Purpose:** Confirms transaction completion

**Direction:** Controller/Watcher → TGP Router → Participants

**Schema:**

```json
{
  “phase”: “SETTLE”,
  “id”: “settle-{uuid}”,
  “query_or_offer_id”: “{query_id or offer_id}”,
  “success”: true,
  “source”: “controller-watcher | manual”,
  “layer8_tx”: “{transaction_hash}”,
  “session_id”: “{session_uuid}”,
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

——

### 2.4 EVENT Messages

**Purpose:** Broadcast state transitions and lifecycle events

**Direction:** CoreProver Contracts → TGP Router → Subscribers

#### 2.4.1 `tgp.escrow.created`

**Emitted when:** Buyer commits payment to escrow

```json
{
  “event”: “tgp.escrow.created”,
  “order_id”: “{bytes32_hex}”,
  “buyer”: “{address}”,
  “seller”: “{address}”,
  “amount”: “{integer_wei}”,
  “asset”: “{token_address}”,
  “acceptance_deadline”: “{unix_timestamp}”,
  “buyer_withdrawal_locked”: false,
  “state”: “BUYER_COMMITTED”
}
```

#### 2.4.2 `tgp.seller.accepted`

**Emitted when:** Seller accepts order via legal signature

**Critical:** This event locks buyer withdrawal

```json
{
  “event”: “tgp.seller.accepted”,
  “order_id”: “{bytes32_hex}”,
  “seller”: “{identifier}”,
  “acceptance_timestamp”: “{unix_timestamp}”,
  “fulfillment_deadline”: “{unix_timestamp}”,
  “buyer_withdrawal_locked”: true,
  “state”: “SELLER_ACCEPTED”
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
  “event”: “tgp.seller.fulfilled”,
  “order_id”: “{bytes32_hex}”,
  “fulfillment_timestamp”: “{unix_timestamp}”,
  “on_time”: true,
  “late_fulfilled”: false,
  “buyer_withdrawal_locked”: true,
  “state”: “SELLER_FULFILLED”
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
  “event”: “tgp.fulfillment.expired”,
  “order_id”: “{bytes32_hex}”,
  “expiration_timestamp”: “{unix_timestamp}”,
  “buyer_withdrawal_unlocked”: true,
  “seller_can_still_fulfill”: true,
  “state”: “FULFILLMENT_EXPIRED”
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
  “event”: “tgp.seller.latefulfilled”,
  “order_id”: “{bytes32_hex}”,
  “fulfillment_timestamp”: “{unix_timestamp}”,
  “late_fulfilled”: true,
  “discount_pct”: 10,
  “discount_expiration”: “{unix_timestamp}”,
  “buyer_withdrawal_locked”: true,
  “next_discount_available”: true,
  “state”: “SELLER_FULFILLED”
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
  “event”: “tgp.seller.claimed”,
  “order_id”: “{bytes32_hex}”,
  “seller”: “{identifier}”,
  “amount”: “{integer_wei}”,
  “receipt_id”: “{nft_token_id}”,
  “claim_timestamp”: “{unix_timestamp}”,
  “state”: “SELLER_CLAIMED”
}
```

#### 2.4.7 `tgp.receipt.minted`

**Emitted when:** Receipt NFT is minted to Receipt Vault

```json
{
  “event”: “tgp.receipt.minted”,
  “receipt_id”: “{nft_token_id}”,
  “order_id”: “{bytes32_hex}”,
  “buyer”: “{pseudonym}”,
  “seller”: “{identifier}”,
  “metadata”: {
    “session_id”: “{session_uuid}”,
    “order_amount”: “{integer_wei}”,
    “late_fulfilled”: false,
    “discount_pct”: 0,
    “discount_expiration”: null,
    “fulfillment_timestamp”: “{unix_timestamp}”,
    “settlement_timestamp”: “{unix_timestamp}”
  }
}
```

#### 2.4.8 `tgp.receipt.metadata.discount`

**Emitted when:** Receipt contains discount coupon metadata

```json
{
  “event”: “tgp.receipt.metadata.discount”,
  “receipt_id”: “{nft_token_id}”,
  “discount_pct”: 10,
  “discount_expiration”: “{unix_timestamp}”,
  “valid”: true,
  “reason”: “late_fulfillment”
}
```

**Routing Implications:**

- L8: Apply discount on next order if presented via ZK proof
- L9: Track discount redemption without buyer identity exposure
- L10: Validate expiration and single-use constraints

——

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
  +— Check for ZK proof of previous receipt
  |
  +— Validate discount eligibility:
      |
      +— Not expired (< 90 days)
      +— Not previously redeemed
      +— Matches buyer identity (via ZK)
      |
  +— Apply 10% discount to order amount
  |
  +— Emit pricing confirmation with discount applied
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

——

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
  “claim”: “I own receipt with valid discount”,
  “proof”: “{zk_proof_bytes}”,
  “public_inputs”: {
    “receipt_id”: “{nft_token_id}”,
    “discount_pct”: 10,
    “expiration”: “{unix_timestamp}”,
    “redeemed”: false,
    “nullifier”: “{unique_per_redemption}”
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
            // Check nullifier hasn’t been used
            let nullifier = proof.public_inputs.get(“nullifier”)?;
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

——

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
            “buyer” => can_buyer_withdraw(
                escrow.state,
                now,
                escrow.acceptance_deadline,
                escrow.fulfillment_deadline
            ),
            “seller” => can_seller_claim(
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

——

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
|——————|————————|——————|——————————|
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

——

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
  “name”: “Order Receipt #{receipt_id}”,
  “description”: “Receipt for order {order_id}”,
  “image”: “ipfs://{image_hash}”,
  “attributes”: [
    {
      “trait_type”: “Order Amount”,
      “value”: “{amount} {asset}”
    },
    {
      “trait_type”: “Late Fulfilled”,
      “value”: “{true|false}”
    },
    {
      “trait_type”: “Discount”,
      “value”: “{pct}%”
    },
    {
      “trait_type”: “Discount Expires”,
      “value”: “{date}”
    },
    {
      “trait_type”: “Seller”,
      “value”: “{seller_identifier}”
    },
    {
      “trait_type”: “Fulfillment Date”,
      “value”: “{date}”
    }
  ]
}
```

### 5.3 ZK Proof Schema

Buyers prove receipt ownership without revealing identity:

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

——

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

——

## 7. Examples

### 7.1 Happy Path: Pizza Delivery (On-Time)

```
1. Buyer sends QUERY
{
  “phase”: “QUERY”,
  “id”: “q-123”,
  “from”: “buyer://alice”,
  “to”: “seller://pizza_hut_4521”,
  “asset”: “USDC”,
  “amount”: 30000000,
  “escrow_from_402”: true,
  “escrow_contract_from_402”: “0x742d35...”
}

2. Seller responds with OFFER
{
  “phase”: “OFFER”,
  “id”: “offer-456”,
  “query_id”: “q-123”,
  “asset”: “USDC”,
  “amount”: 30000000,
  “economic_metadata”: {
    “enables_late_discount”: true,
    “late_discount_pct”: 10,
    “discount_expiration_days”: 90,
    “acceptance_window_seconds”: 1800,
    “fulfillment_window_seconds”: 3600,
    “claim_window_seconds”: 3600
  }
}

3. Buyer commits payment
[On-chain transaction]
EVENT: tgp.escrow.created
{
  “order_id”: “0xabcd...”,
  “buyer_withdrawal_locked”: false
}

4. Seller accepts order
[On-chain transaction]
EVENT: tgp.seller.accepted
{
  “order_id”: “0xabcd...”,
  “buyer_withdrawal_locked”: true,
  “fulfillment_deadline”: 1699903600
}

5. Seller delivers pizza
[Off-chain delivery]

6. Seller marks fulfilled
[On-chain transaction]
EVENT: tgp.seller.fulfilled
{
  “order_id”: “0xabcd...”,
  “on_time”: true,
  “late_fulfilled”: false
}

7. Seller claims payment
[On-chain transaction]
EVENT: tgp.seller.claimed
{
  “order_id”: “0xabcd...”,
  “receipt_id”: 12345
}

EVENT: tgp.receipt.minted
{
  “receipt_id”: 12345,
  “metadata”: {
    “late_fulfilled”: false,
    “discount_pct”: 0
  }
}

8. Router sends SETTLE
{
  “phase”: “SETTLE”,
  “success”: true,
  “fulfillment_metadata”: {
    “on_time”: true,
    “late_fulfilled”: false,
    “buyer_withdrawal_locked”: false,
    “next_discount_available”: false
  }
}
```

——

### 7.2 Late Fulfillment: Pizza Delivery (Discount Issued)

```
1-4. [Same as happy path]

5. Fulfillment window expires
[No action from seller]
EVENT: tgp.fulfillment.expired
{
  “order_id”: “0xabcd...”,
  “buyer_withdrawal_unlocked”: true
}

6. Seller delivers pizza late
[Off-chain delivery, 15 minutes late]

7. Seller marks fulfilled
[On-chain transaction]
EVENT: tgp.seller.latefulfilled
{
  “order_id”: “0xabcd...”,
  “late_fulfilled”: true,
  “discount_pct”: 10,
  “discount_expiration”: 1707680000,
  “buyer_withdrawal_locked”: true,
  “next_discount_available”: true
}

8. Seller claims payment
[On-chain transaction]
EVENT: tgp.seller.claimed
EVENT: tgp.receipt.minted
{
  “receipt_id”: 12346,
  “metadata”: {
    “late_fulfilled”: true,
    “discount_pct”: 10,
    “discount_expiration”: 1707680000
  }
}

EVENT: tgp.receipt.metadata.discount
{
  “receipt_id”: 12346,
  “discount_pct”: 10,
  “discount_expiration”: 1707680000,
  “valid”: true,
  “reason”: “late_fulfillment”
}

9. Router sends SETTLE
{
  “phase”: “SETTLE”,
  “success”: true,
  “fulfillment_metadata”: {
    “late_fulfilled”: true,
    “discount_pct”: 10,
    “buyer_withdrawal_locked”: false,
    “next_discount_available”: true
  }
}
```

——

### 7.3 Buyer Withdrawal: Seller Never Accepts

```
1. Buyer sends QUERY
[Standard QUERY message]

2. Seller responds with OFFER
[Standard OFFER with 30-minute acceptance window]

3. Buyer commits payment
EVENT: tgp.escrow.created
{
  “order_id”: “0xabcd...”,
  “acceptance_deadline”: 1699901800
}

4. Seller does not accept
[30 minutes pass]

5. Acceptance deadline expires
EVENT: tgp.escrow.expired
{
  “order_id”: “0xabcd...”,
  “buyer_withdrawal_unlocked”: true
}

6. Buyer withdraws funds
[On-chain transaction]
EVENT: tgp.buyer.claimed
{
  “order_id”: “0xabcd...”,
  “amount”: 30000000
}

7. Router sends SETTLE
{
  “phase”: “SETTLE”,
  “success”: false,
  “reason”: “seller_never_accepted”,
  “buyer_withdrawal_locked”: false
}
```

——

### 7.4 Discount Redemption: Next Order

```
1. Buyer has receipt #12346 with 10% discount

2. Buyer sends QUERY with ZK proof
{
  “phase”: “QUERY”,
  “id”: “q-789”,
  “from”: “buyer://alice”,
  “to”: “seller://pizza_hut_4521”,
  “asset”: “USDC”,
  “amount”: 30000000,
  “zk_profile”: {
    “circuit”: “receipt_ownership_v1”,
    “proof”: “{proof_bytes}”,
    “public_inputs”: {
      “receipt_id”: 12346,
      “discount_pct”: 10,
      “discount_expiration”: 1707680000,
      “redeemed”: false,
      “nullifier”: “0xunique...”
    }
  }
}

3. L8 Router validates ZK proof
- Verifies proof correctness
- Checks discount hasn’t expired
- Confirms nullifier unused
- Validates receipt ownership

4. L8 Router applies discount
- Original amount: 30000000 (30 USDC)
- Discount: 10%
- Final amount: 27000000 (27 USDC)

5. Seller responds with discounted OFFER
{
  “phase”: “OFFER”,
  “id”: “offer-999”,
  “query_id”: “q-789”,
  “amount”: 27000000,
  “discount_applied”: true,
  “original_amount”: 30000000
}

6. Transaction proceeds normally
[Standard flow with discounted price]

7. L8 Router marks discount redeemed
[Updates nullifier registry]
```

——

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

——

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

——

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

——

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

——

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
                                +-—>[re-lock]——+
                                                     |
                                                     v
                                               SELLER_CLAIMED
```

——

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

——

**Document Version:** 2.0  
**Last Updated:** November 14, 2025  
**Status:** Final  
**Change Control:** TBC Development Team

——

END OF SPECIFICATION---

## 7.2 Layer-Sequence Evaluation Algorithm (Continued)

**Algorithm Name:** `evaluate_security_layers`

**Inputs:**
- `query`: TGP QUERY message
- `profile`: Payment profile metadata
- `config`: TBC security configuration

**Outputs:**
- `result`: APPROVED or DENIED
- `envelope`: Economic Envelope (if APPROVED)
- `error`: Error response (if DENIED)
- `verification_summary`: Per-layer results

**Complete Algorithm:**

```pseudocode
FUNCTION evaluate_security_layers(query, profile, config):
    verification_summary = {
        “layer1_registry”: NULL,
        “layer2_signature”: NULL,
        “layer3_contract”: NULL,
        “layer4_zk”: NULL,
        “layer5_policy”: NULL
    }
    
    // ========================================
    // Layer 1: Merchant Registry / Enable Flag
    // ========================================
    log(“Evaluating Layer 1: Registry check”)
    
    TRY:
        registry_result = check_merchant_registry(profile.merchant_id, profile.profile_id)
        
        IF registry_result.enabled == FALSE THEN
            verification_summary.layer1_registry = “FAIL”
            RETURN create_error_response(
                “DENIED”,
                “MERCHANT_DISABLED”,
                “TBC_L1_REGISTRY_FAIL”,
                1,
                “Merchant payment profile is currently disabled”
            )
        END IF
        
        verification_summary.layer1_registry = “PASS”
        
    CATCH Exception e:
        log_exception(“Layer 1 exception”, e)
        verification_summary.layer1_registry = “ERROR”
        RETURN create_error_response(
            “DENIED”,
            “REGISTRY_UNAVAILABLE”,
            “TBC_L1_REGISTRY_ERROR”,
            1,
            “Registry service unavailable”
        )
    END TRY
    
    // ========================================
    // Layer 2: Merchant-Verifiable Signature
    // ========================================
    log(“Evaluating Layer 2: Signature verification”)
    
    TRY:
        signature_result = verify_profile_signature(profile)
        
        IF NOT signature_result.valid THEN
            verification_summary.layer2_signature = “FAIL”
            RETURN create_error_response(
                “DENIED”,
                “INVALID_SIGNATURE”,
                “TBC_L2_SIGNATURE_FAIL”,
                2,
                “Payment profile signature verification failed”
            )
        END IF
        
        verification_summary.layer2_signature = “PASS”
        
    CATCH Exception e:
        log_exception(“Layer 2 exception”, e)
        verification_summary.layer2_signature = “ERROR”
        RETURN create_error_response(
            “DENIED”,
            “SIGNATURE_VERIFICATION_ERROR”,
            “TBC_L2_INTERNAL_ERROR”,
            2,
            “Signature verification failed”
        )
    END TRY
    
    // ========================================
    // Layer 3: On-Chain Contract Verification
    // ========================================
    log(“Evaluating Layer 3: Contract verification”)
    
    TRY:
        bytecode_result = verify_bytecode_hash(
            profile.contract_address,
            get_expected_hash(profile.engine_version),
            profile.chain_id,
            profile.engine_version
        )
        
        IF bytecode_result.result == FAIL THEN
            verification_summary.layer3_contract = “FAIL”
            RETURN create_error_response(
                “DENIED”,
                “CONTRACT_VERIFICATION_FAILED”,
                bytecode_result.error_code,
                3,
                bytecode_result.details
            )
        END IF
        
        verification_summary.layer3_contract = “PASS”
        
    CATCH Exception e:
        log_exception(“Layer 3 exception”, e)
        verification_summary.layer3_contract = “ERROR”
        RETURN create_error_response(
            “DENIED”,
            “CONTRACT_VERIFICATION_ERROR”,
            “TBC_L3_INTERNAL_ERROR”,
            3,
            “Contract verification failed”
        )
    END TRY
    
    // ========================================
    // Layer 4: ZK Attestation (OPTIONAL)
    // ========================================
    log(“Evaluating Layer 4: ZK attestation”)
    
    user_context = extract_user_context(query)
    IF should_require_layer4(query, user_context, config) THEN
        log(“Layer 4 required by policy”)
        
        TRY:
            zk_result = verify_zk_attestation(profile.contract_address, profile.chain_id, config)
            
            IF NOT zk_result.valid THEN
                verification_summary.layer4_zk = “FAIL”
                RETURN create_error_response(
                    “DENIED”,
                    “ZK_ATTESTATION_REQUIRED”,
                    “TBC_L4_ZK_ATTESTATION_FAIL”,
                    4,
                    “Zero-knowledge attestation required but failed”
                )
            END IF
            
            verification_summary.layer4_zk = “PASS”
            
        CATCH Exception e:
            log_exception(“Layer 4 exception”, e)
            verification_summary.layer4_zk = “ERROR”
            RETURN create_error_response(
                “DENIED”,
                “ZK_ATTESTATION_ERROR”,
                “TBC_L4_INTERNAL_ERROR”,
                4,
                “ZK attestation verification failed”
            )
        END TRY
    ELSE
        log(“Layer 4 not required per policy”)
        verification_summary.layer4_zk = “NOT_REQUIRED”
    END IF
    
    // ========================================
    // Layer 5: Policy / Risk Engine
    // ========================================
    log(“Evaluating Layer 5: Policy engine”)
    
    TRY:
        policy_result = evaluate_policy_rules(query, profile, user_context, config)
        
        IF NOT policy_result.passed THEN
            verification_summary.layer5_policy = “FAIL”
            RETURN create_error_response(
                “DENIED”,
                “POLICY_VIOLATION”,
                policy_result.error_code,
                5,
                policy_result.reason
            )
        END IF
        
        verification_summary.layer5_policy = “PASS”
        
    CATCH Exception e:
        log_exception(“Layer 5 exception”, e)
        verification_summary.layer5_policy = “ERROR”
        RETURN create_error_response(
            “DENIED”,
            “POLICY_EVALUATION_ERROR”,
            “TBC_L5_INTERNAL_ERROR”,
            5,
            “Policy evaluation failed”
        )
    END TRY
    
    // ========================================
    // All Layers Passed - Generate Economic Envelope
    // ========================================
    log(“All security layers passed - generating Economic Envelope”)
    
    envelope = generate_economic_envelope(
        query,
        profile,
        verification_summary
    )
    
    RETURN {
        “result”: “APPROVED”,
        “envelope”: envelope,
        “error”: NULL,
        “verification_summary”: verification_summary
    }
END FUNCTION
```

**Properties:**

**Correctness:** Algorithm guarantees that Economic Envelope is generated if and only if all REQUIRED layers pass.

**Fail-Fast:** Algorithm terminates at first layer failure (RECOMMENDED behavior for performance).

**Exception Safety:** All exceptions are caught and treated as layer failures.

**Logging:** Each layer evaluation is logged for audit and debugging.

### 7.3 RPC Data Validation Algorithm

**Algorithm Name:** `validate_rpc_response`

**Purpose:** Validate that RPC response is well-formed and contains expected data before using in verification.

**Inputs:**
- `response`: Raw RPC HTTP response
- `expected_method`: Expected RPC method name (e.g., “eth_getCode”)

**Outputs:**
- `valid`: Boolean (TRUE if response is valid)
- `parsed_result`: Extracted result data (if valid)
- `error_reason`: Reason string (if invalid)

**Algorithm:**

```pseudocode
FUNCTION validate_rpc_response(response, expected_method):
    // Step 1: HTTP status code check
    IF response.http_status < 200 OR response.http_status >= 300 THEN
        RETURN (FALSE, NULL, “HTTP error: “ + str(response.http_status))
    END IF
    
    // Step 2: Content-Type check
    content_type = response.headers.get(“Content-Type”, “”)
    IF NOT contains(content_type, “application/json”) THEN
        RETURN (FALSE, NULL, “Invalid Content-Type: expected application/json”)
    END IF
    
    // Step 3: JSON parsing
    TRY:
        parsed_json = json_parse(response.body)
    CATCH JSONDecodeError e:
        RETURN (FALSE, NULL, “Malformed JSON: “ + e.message)
    END TRY
    
    // Step 4: JSON-RPC structure check
    IF “jsonrpc” NOT IN parsed_json THEN
        RETURN (FALSE, NULL, “Missing jsonrpc field”)
    END IF
    
    IF parsed_json.jsonrpc != “2.0” THEN
        RETURN (FALSE, NULL, “Invalid jsonrpc version: “ + parsed_json.jsonrpc)
    END IF
    
    IF “id” NOT IN parsed_json THEN
        RETURN (FALSE, NULL, “Missing id field”)
    END IF
    
    // Step 5: Check for error response
    IF “error” IN parsed_json THEN
        error_msg = parsed_json.error.get(“message”, “Unknown error”)
        error_code = parsed_json.error.get(“code”, 0)
        RETURN (FALSE, NULL, “RPC error “ + str(error_code) + “: “ + error_msg)
    END IF
    
    // Step 6: Check for result field
    IF “result” NOT IN parsed_json THEN
        RETURN (FALSE, NULL, “Missing result field”)
    END IF
    
    result = parsed_json.result
    
    // Step 7: Method-specific validation
    IF expected_method == “eth_getCode” THEN
        // Validate bytecode format
        IF NOT isinstance(result, string) THEN
            RETURN (FALSE, NULL, “Result is not a string”)
        END IF
        
        IF NOT starts_with(result, “0x”) THEN
            RETURN (FALSE, NULL, “Result does not start with 0x”)
        END IF
        
        hex_part = result[2:]  // Remove “0x” prefix
        
        IF len(hex_part) > 0 THEN  // If not empty bytecode
            IF len(hex_part) % 2 != 0 THEN
                RETURN (FALSE, NULL, “Hex string has odd length”)
            END IF
            
            IF NOT is_valid_hex(hex_part) THEN
                RETURN (FALSE, NULL, “Result contains invalid hex characters”)
            END IF
        END IF
        
    ELSE IF expected_method == “eth_chainId” THEN
        // Validate chain ID format
        IF NOT (isinstance(result, string) OR isinstance(result, integer)) THEN
            RETURN (FALSE, NULL, “Chain ID is not string or integer”)
        END IF
        
        IF isinstance(result, string) AND NOT starts_with(result, “0x”) THEN
            RETURN (FALSE, NULL, “Chain ID string does not start with 0x”)
        END IF
        
    ELSE IF expected_method == “eth_call” THEN
        // Validate call result format
        IF NOT isinstance(result, string) THEN
            RETURN (FALSE, NULL, “Call result is not a string”)
        END IF
        
        IF NOT starts_with(result, “0x”) THEN
            RETURN (FALSE, NULL, “Call result does not start with 0x”)
        END IF
        
    ELSE
        // Unknown method - perform generic validation only
        log_warning(“Unknown RPC method for validation: “ + expected_method)
    END IF
    
    // All checks passed
    RETURN (TRUE, result, NULL)
END FUNCTION
```

**Usage Example:**

```pseudocode
rpc_response = http_post(provider.url, rpc_request)
(valid, bytecode, error) = validate_rpc_response(rpc_response, “eth_getCode”)

IF NOT valid THEN
    log_provider_error(provider.id, error)
    // Do not use this response in quorum
ELSE
    // Use bytecode for hash computation
    hash = keccak256(hex_decode(bytecode[2:]))
END IF
```

——

## 8. Diagrams and Visual Models

### 8.1 Onion Layer Interaction Diagram

**Purpose:** Visualize data flow between security layers and dependencies.

```
                     TGP QUERY
                        ↓
              ┌─────────────────┐
              │   Layer 1       │
              │   Registry      │ → merchant_id
              └────────┬────────┘
                       ↓ (if enabled)
              ┌─────────────────┐
              │   Layer 2       │
              │   Signature     │ → contract_address (advisory)
              └────────┬────────┘   engine_version
                       ↓ (if valid)
              ┌─────────────────┐
              │   Layer 3       │
              │   Bytecode      │ → bytecode_hash
              └────────┬────────┘   contract_state
                       ↓ (if match)
              ┌─────────────────┐
              │   Layer 4       │
              │   ZK (optional) │ → proof_verified
              └────────┬────────┘
                       ↓ (if pass/skip)
              ┌─────────────────┐
              │   Layer 5       │
              │   Policy        │ → policy_compliance
              └────────┬────────┘
                       ↓ (if pass)
              ┌─────────────────┐
              │   Economic      │
              │   Envelope      │
              └─────────────────┘
```

**Data Flow Summary:**

```
Layer 1 Output:
- merchant_id (validated)
- profile_enabled status

Layer 2 Output:
- contract_address (from signed profile)
- engine_version
- merchant signature verified

Layer 3 Output:
- bytecode_hash (consensus from M-of-N providers)
- contract_state (paused, asset, adminless)
- RPC verification results

Layer 4 Output (if required):
- ZK proof validity
- Trust-minimized verification status

Layer 5 Output:
- Policy compliance results
- Risk assessment
- All rule validation results

Economic Envelope Input:
- All layer outputs
- verification_summary
- Transaction metadata
```

### 8.2 Verification Flow Chart

**Purpose:** Comprehensive flowchart showing all decision points.

```
┌─────────────────────────────────────────────────────────────┐
│                      TBC Verification Flow                   │
└─────────────────────────────────────────────────────────────┘

START: Receive QUERY
    ↓
┌─────────────────────────┐
│ Parse QUERY message     │
│ Extract:                │
│ - profile_reference     │
│ - amount, asset         │
│ - user context          │
└───────────┬─────────────┘
            ↓
┌─────────────────────────┐
│ LAYER 1: Registry       │
│ Query enable flag       │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │ enabled?  │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: MERCHANT_DISABLED
┌─────────────────────────┐
│ LAYER 2: Signature      │
│ Verify merchant sig     │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │  valid?   │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: INVALID_SIGNATURE
┌─────────────────────────┐
│ LAYER 3: Bytecode       │
│ Query N RPC providers   │
└───────────┬─────────────┘
            ↓
┌─────────────────────────┐
│ Compute hash for each   │
│ Filter valid responses  │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │ Quorum?   │
      │ (M of N)  │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: INSUFFICIENT_QUORUM
      ┌─────┴─────┐
      │Hash match?│
      │(consensus │
      │ vs expect)│
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
   MATCH         MISMATCH
     │             │
     │             └─→ REJECT
     ↓                 ERROR: CODE_MISMATCH
┌─────────────────────────┐
│ Verify contract state   │
│ - not paused            │
│ - correct asset         │
│ - correct chain         │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │State OK?  │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: INVALID_STATE
┌─────────────────────────┐
│ LAYER 4: Check policy   │
│ ZK attestation required?│
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │ Required? │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ SKIP Layer 4
     ↓
┌─────────────────────────┐
│ Request & verify ZK     │
│ proof                   │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │Proof OK?  │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: ZK_ATTESTATION_REQUIRED
┌─────────────────────────┐
│ LAYER 5: Policy         │
│ Evaluate all rules:     │
│ - Chain whitelist       │
│ - Asset whitelist       │
│ - Value limits          │
│ - Rate limits           │
│ - Sanctions             │
│ - Jurisdiction          │
└───────────┬─────────────┘
            ↓
      ┌─────┴─────┐
      │All pass?  │
      └─────┬─────┘
            ↓
     ┌──────┴──────┐
    YES           NO
     │             │
     │             └─→ REJECT
     ↓                 ERROR: POLICY_VIOLATION
┌─────────────────────────┐
│ Generate Economic       │
│ Envelope                │
│ - Sign with TBC key     │
│ - Include verification  │
│   summary               │
│ - Set expiration        │
└───────────┬─────────────┘
            ↓
┌─────────────────────────┐
│ RETURN: APPROVED        │
│ Envelope to Extension   │
└─────────────────────────┘
            ↓
           END
```

### 8.3 Bytecode-Verification Diagram

**Purpose:** Detailed visualization of Layer 3 bytecode verification with multi-RPC quorum.

```
                      Layer 3 Start
                           |
                           v
              ┌────────────────────────┐
              │  Select N RPC Providers │
              │  (N ≥ 3, diverse ops)   │
              └────────────┬───────────┘
                           |
         ┌─────────────────┼─────────────────┐
         |                 |                 |
         v                 v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │Provider │      │Provider │      │Provider │
    │   1     │      │   2     │      │   3     │
    │ (Infura)│      │(Alchemy)│      │(QuickN) │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | eth_getCode    | eth_getCode    | eth_getCode
         | contract_addr  | contract_addr  | contract_addr
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │Response │      │Response │      │Response │
    │ “0x60...” │    │ “0x60...” │    │ “0x61...” │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | hex_decode     | hex_decode     | hex_decode
         | strip “0x”     | strip “0x”     | strip “0x”
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │ Bytes   │      │ Bytes   │      │ Bytes   │
    │[60,60,..]│     │[60,60,..]│     │[61,60,..]│
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | keccak256      | keccak256      | keccak256
         | hash           | hash           | hash
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │  Hash   │      │  Hash   │      │  Hash   │
    │ 0xABC... │     │ 0xABC... │     │ 0xDEF... │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         └────────────────┼─────────────────┘
                          |
                          v
              ┌────────────────────────┐
              │  Reconciliation        │
              │                         │
              │  Hash Counts:          │
              │  0xABC...: 2 providers │
              │  0xDEF...: 1 provider  │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Check Quorum          │
              │  (M = 2)               │
              │                        │
              │  max_count = 2         │
              │  2 ≥ M ? YES ✓         │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Consensus Hash:       │
              │  0xABC...              │
              │  (from Providers 1+2)  │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Log Dissent:          │
              │  Provider 3 returned   │
              │  different hash        │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Compare with Expected │
              │                        │
              │  Expected: 0xABC...    │
              │  Actual:   0xABC...    │
              └────────────┬───────────┘
                           |
                 ┌─────────┴──────────┐
                 |                    |
             [MATCH]            [MISMATCH]
                 |                    |
                 v                    v
      ┌──────────────────┐   ┌─────────────────┐
      │ Verify State     │   │ REJECT          │
      │ - paused()       │   │ ERROR:          │
      │ - getAsset()     │   │ CODE_MISMATCH   │
      └────────┬─────────┘   └─────────────────┘
               |
        ┌──────┴──────┐
        |             |
    [STATE OK]   [STATE BAD]
        |             |
        v             v
   ┌────────┐   ┌────────────┐
   │  PASS  │   │  REJECT    │
   │Layer 3 │   │  ERROR:    │
   └────────┘   │INVALID_STATE│
                └────────────┘
```

**Quorum Calculation Visualization:**

```
Configuration: N=5, M=3 (3-of-5 majority)

┌──────────────────────────────────────────────────────┐
│  Provider Response Matrix                             │
├──────────────────────────────────────────────────────┤
│                                                       │
│  Provider 1:  ████ Hash A                            │
│  Provider 2:  ████ Hash A                            │
│  Provider 3:  ████ Hash A                            │
│  Provider 4:  ▓▓▓▓ Hash B (dissent)                 │
│  Provider 5:  ░░░░ Timeout                           │
│                                                       │
│  Valid Responses: 4                                  │
│  Hash A: 3 occurrences                               │
│  Hash B: 1 occurrence                                │
│                                                       │
│  Quorum Check: 3 ≥ M(3) → ✓ PASS                    │
│  Consensus: Hash A                                   │
│  Dissenting: Provider 4 (logged)                     │
│  Non-responsive: Provider 5 (logged)                 │
│                                                       │
│  Result: IF Hash A == Expected → PASS                │
│          ELSE → FAIL (bytecode mismatch)             │
└──────────────────────────────────────────────────────┘

Legend:
████ = Provider agrees with consensus
▓▓▓▓ = Provider dissents from consensus
░░░░ = Provider timeout/error (not counted)
```

### 8.4 RPC Quorum Visual Model

**Purpose:** Illustrate quorum consensus logic with different provider response patterns.

```
Quorum Scenarios (N=3, M=2)

┌───────────────────────────────────────────────────────────┐
│                     Case 1: Unanimous                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ████████ hash=0xABC                        │
│                                                           │
│  Consensus: 0xABC (3/3 = 100%)                           │
│  Quorum: 3 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│                  Case 2: Majority Consensus                │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ▓▓▓▓▓▓▓▓ hash=0xDEF (dissent)             │
│                                                           │
│  Consensus: 0xABC (2/3 = 67%)                            │
│  Quorum: 2 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
│  Warning: Provider 3 dissenting - log for investigation  │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│                  Case 3: No Consensus (Tie)                │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ▓▓▓▓▓▓▓▓ hash=0xDEF                        │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Valid responses: 2                                      │
│  Consensus: None (1 vs 1 tie)                            │
│  Quorum: 1 < 2 ✗                                         │
│  Result: REJECT (insufficient quorum)                    │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│              Case 4: Timeout Majority                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ░░░░░░░░ timeout                           │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Valid responses: 1                                      │
│  Consensus: 0xABC (1/1 valid = 100%)                     │
│  Quorum: 1 < 2 ✗                                         │
│  Result: REJECT (insufficient valid responses)           │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│           Case 5: Quorum with One Timeout                  │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Valid responses: 2                                      │
│  Consensus: 0xABC (2/2 valid = 100%)                     │
│  Quorum: 2 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
│  Note: Timeout does not count against quorum             │
└───────────────────────────────────────────────────────────┘

Legend:
████████ = Valid response agreeing with consensus
▓▓▓▓▓▓▓▓ = Valid response dissenting from consensus
░░░░░░░░ = Timeout or error (not counted)
```

**Quorum Decision Matrix:**

```
N=3, M=2 (All Possible Outcomes)

┌──────────┬──────────┬─────────┬──────────────┐
│ Valid    │Consensus │ Quorum  │ Result       │
│Responses │  Count   │Achieved │              │
├──────────┼──────────┼─────────┼──────────────┤
│    3     │    3     │   ✓     │ PASS*        │
│    3     │    2     │   ✓     │ PASS*        │
│    3     │    1     │   ✗     │ REJECT       │
│    2     │    2     │   ✓     │ PASS*        │
│    2     │    1     │   ✗     │ REJECT       │
│    1     │    1     │   ✗     │ REJECT       │
│    0     │    0     │   ✗     │ REJECT       │
└──────────┴──────────┴─────────┴──────────────┘

* PASS only if consensus hash matches expected hash
  Otherwise REJECT with CODE_MISMATCH
```

——

## 9. Error Codes & Rejection Semantics

### 9.1 Mandatory Error Types

**Purpose:** Define complete taxonomy of TBC error conditions with mandatory error codes, messages, and handling semantics.

**Error Code Format:**

```
TBC_L{layer}_{TYPE}_{DETAIL}

Where:
- L{layer} = Layer number (1-5)
- {TYPE} = High-level error category
- {DETAIL} = Specific error condition (optional)

Examples:
- TBC_L1_REGISTRY_FAIL
- TBC_L3_CODE_MISMATCH
- TBC_L5_RATE_LIMIT
```

**Complete Error Taxonomy:**

**Layer 1 Errors:**

| Error Code | Error Type | Reason | User Message | Retry Allowed |
|————|————|———|—————|—————|
| TBC_L1_REGISTRY_FAIL | MERCHANT_DISABLED | Profile disabled in registry | This merchant is temporarily unavailable. | No |
| TBC_L1_REGISTRY_ERROR | REGISTRY_UNAVAILABLE | Registry query failed or timeout | Verification service unavailable. Please try again. | Yes |
| TBC_L1_REGISTRY_INVALID | REGISTRY_UNAVAILABLE | Malformed registry response | Verification service error. Please try again. | Yes |

**Layer 2 Errors:**

| Error Code | Error Type | Reason | User Message | Retry Allowed |
|————|————|———|—————|—————|
| TBC_L2_SIGNATURE_FAIL | INVALID_SIGNATURE | Signature verification failed | Unable to verify merchant authenticity. | No |
| TBC_L2_SIGNATURE_EXPIRED | INVALID_SIGNATURE | Signature age exceeds validity period | Merchant profile expired. Contact merchant. | No |
| TBC_L2_PUBKEY_NOT_FOUND | INVALID_SIGNATURE | Merchant public key not in registry | Merchant authentication failed. | No |
| TBC_L2_INTERNAL_ERROR | SIGNATURE_VERIFICATION_ERROR | Exception during verification | Internal verification error. Please try again. | Yes |

**Layer 3 Errors:**

| Error Code | Error Type | Reason | User Message | Retry Allowed |
|————|————|———|—————|—————|
| TBC_L3_CODE_MISMATCH | CONTRACT_VERIFICATION_FAILED | Bytecode hash does not match expected | Security verification failed. Transaction cancelled for your protection. | No |
| TBC_L3_INSUFFICIENT_QUORUM | RPC_INCONSISTENCY | Fewer than M providers agreed | Network verification inconsistency. Please try again. | Yes |
| TBC_L3_RPC_DISAGREEMENT | RPC_INCONSISTENCY | Providers returned different bytecode | Network verification conflict detected. Please try again. | Yes |
| TBC_L3_ALL_RPC_FAILED | RPC_INCONSISTENCY | All RPC providers failed or timed out | Network unavailable. Please try again. | Yes |
| TBC_L3_NO_CONTRACT | CONTRACT_VERIFICATION_FAILED | Contract does not exist (empty bytecode) | Contract not found at specified address. | No |
| TBC_L3_INVALID_BYTECODE | CONTRACT_VERIFICATION_FAILED | Bytecode format invalid | Contract data invalid. | No |
| TBC_L3_INVALID_STATE | CONTRACT_VERIFICATION_FAILED | Contract state validation failed | Contract in invalid state. | No |
| TBC_L3_UNSUPPORTED_VERSION | CONTRACT_VERIFICATION_FAILED | Engine version not supported | Merchant using unsupported system version. | No |
| TBC_L3_INTERNAL_ERROR | CONTRACT_VERIFICATION_ERROR | Exception during verification | Internal verification error. Please try again. | Yes |

**Layer 4 Errors:**

| Error Code | Error Type | Reason | User Message | Retry Allowed |
|————|————|———|—————|—————|
| TBC_L4_ZK_ATTESTATION_REQUIRED | ZK_ATTESTATION_REQUIRED | ZK proof required but unavailable | Enhanced verification required but unavailable. | Yes |
| TBC_L4_ZK_ATTESTATION_FAIL | ZK_ATTESTATION_REQUIRED | ZK proof verification failed | Enhanced verification failed. | Yes |
| TBC_L4_ZK_SERVICE_UNAVAILABLE | ZK_ATTESTATION_ERROR | Proof service unreachable | Enhanced verification service unavailable. | Yes |
| TBC_L4_ZK_PROOF_EXPIRED | ZK_ATTESTATION_REQUIRED | Proof timestamp too old | Verification proof expired. | Yes |
| TBC_L4_INTERNAL_ERROR | ZK_ATTESTATION_ERROR | Exception during verification | Internal verification error. Please try again. | Yes |

**Layer 5 Errors:**

| Error Code | Error Type | Reason | User Message | Retry Allowed |
|————|————|———|—————|—————|
| TBC_L5_CHAIN_NOT_ALLOWED | POLICY_VIOLATION | Transaction chain not in whitelist | Blockchain not supported for this transaction. | No |
| TBC_L5_ASSET_NOT_ALLOWED | POLICY_VIOLATION | Payment asset not in whitelist | Asset type not accepted. | No |
| TBC_L5_VALUE_EXCEEDS_LIMIT | POLICY_VIOLATION | Amount exceeds policy limit | Transaction amount exceeds limit. | No |
| TBC_L5_RATE_LIMIT | POLICY_VIOLATION | Too many requests in time window | Daily transaction limit reached. Please try again tomorrow. | Maybe |
| TBC_L5_SANCTIONS_VIOLATION | POLICY_VIOLATION | Merchant or user on sanctions list | Transaction not permitted due to compliance rules. | No |
| TBC_L5_JURISDICTION_RESTRICTED | POLICY_VIOLATION | Transaction violates jurisdictional rules | Transaction not permitted in your region. | No |
| TBC_L5_INTERNAL_ERROR | POLICY_EVALUATION_ERROR | Exception during policy evaluation | Internal policy error. Please try again. | Yes |

### 9.2 Machine-Readable Error Codes

**Purpose:** Define structured error codes for programmatic handling by extensions and integrations.

**Error Response Schema:**

```json
{
  “status”: “DENIED”,
  “error”: “{ERROR_TYPE}”,
  “code”: “{ERROR_CODE}”,
  “layer_failed”: {1-5},
  “timestamp”: “{ISO8601_timestamp}”,
  “reason”: “{technical_reason}”,
  “user_message”: “{user_friendly_message}”,
  “support_reference”: “{ticket_id}”,
  “retry_allowed”: {true|false},
  “retry_after”: {seconds} // OPTIONAL: if retry_allowed=true
}
```

**Field Definitions:**

- `status`: MUST be “DENIED” for all error responses
- `error`: High-level error type (e.g., “MERCHANT_DISABLED”, “CONTRACT_VERIFICATION_FAILED”)
- `code`: Specific error code (e.g., “TBC_L1_REGISTRY_FAIL”)
- `layer_failed`: Integer 1-5 indicating which layer rejected
- `timestamp`: ISO 8601 timestamp of error
- `reason`: Technical explanation (for logging, not user display)
- `user_message`: User-friendly message (extension displays this)
- `support_reference`: Unique reference for support/debugging
- `retry_allowed`: Boolean indicating if user should retry
- `retry_after`: Seconds to wait before retry (optional)

**Example Error Responses:**

**Example 1: Merchant Disabled**
```json
{
  “status”: “DENIED”,
  “error”: “MERCHANT_DISABLED”,
  “code”: “TBC_L1_REGISTRY_FAIL”,
  “layer_failed”: 1,
  “timestamp”: “2025-11-18T12:00:00Z”,
  “reason”: “Merchant payment profile disabled in registry: status=suspended”,
  “user_message”: “This merchant is temporarily unavailable. Please try again later or contact support.”,
  “support_reference”: “TBC-20251118-001234”,
  “retry_allowed”: false
}
```

**Example 2: Bytecode Mismatch**
```json
{
  “status”: “DENIED”,
  “error”: “CONTRACT_VERIFICATION_FAILED”,
  “code”: “TBC_L3_CODE_MISMATCH”,
  “layer_failed”: 3,
  “timestamp”: “2025-11-18T12:00:05Z”,
  “reason”: “Contract bytecode hash mismatch: expected=0x7f8b6c..., actual=0x3c4d9a...”,
  “user_message”: “Security verification failed. Transaction cancelled for your protection.”,
  “support_reference”: “TBC-20251118-001235”,
  “retry_allowed”: false
}
```

**Example 3: RPC Timeout (Transient)**
```json
{
  “status”: “DENIED”,
  “error”: “RPC_INCONSISTENCY”,
  “code”: “TBC_L3_ALL_RPC_FAILED”,
  “layer_failed”: 3,
  “timestamp”: “2025-11-18T12:00:10Z”,
  “reason”: “All RPC providers timed out: timeout=10s, providers=[infura, alchemy, quicknode]”,
  “user_message”: “Network verification temporarily unavailable. Please try again.”,
  “support_reference”: “TBC-20251118-001236”,
  “retry_allowed”: true,
  “retry_after”: 30
}
```

**Example 4: Policy Violation (Rate Limit)**
```json
{
  “status”: “DENIED”,
  “error”: “POLICY_VIOLATION”,
  “code”: “TBC_L5_RATE_LIMIT”,
  “layer_failed”: 5,
  “timestamp”: “2025-11-18T12:00:15Z”,
  “reason”: “Daily transaction limit exceeded: count=51, limit=50, window=24h”,
  “user_message”: “Daily transaction limit reached. Please try again tomorrow.”,
  “support_reference”: “TBC-20251118-001237”,
  “retry_allowed”: true,
  “retry_after”: 43200
}
```

### 9.3 Human-Readable Reason Strings

**Purpose:** Define guidelines for generating user-friendly error messages that are clear, actionable, and non-technical.

**Message Composition Rules:**

**Rule 1: User-Focused Language**

AVOID: “Registry query failed with HTTP 503”
USE: “Verification service temporarily unavailable”

**Rule 2: Actionable When Possible**

AVOID: “Signature verification failed”
USE: “Unable to verify merchant authenticity. Contact merchant support.”

**Rule 3: Avoid Technical Jargon**

AVOID: “Bytecode hash mismatch detected”
USE: “Security verification failed. Transaction cancelled for your protection.”

**Rule 4: Reassure User of Safety**

INCLUDE: “Transaction cancelled for your protection” (for security failures)
INCLUDE: “No funds were transferred” (where applicable)

**Rule 5: Provide Next Steps**

For transient errors: “Please try again”
For permanent errors: “Contact merchant support” or “Choose different payment method”
For policy errors: “Please check transaction details” or “Daily limit reached. Try again tomorrow.”

**Message Templates by Category:**

**Category: Merchant/Profile Issues**
```
Template: “{merchant_status}. {action_recommendation}”

Examples:
- “This merchant is temporarily unavailable. Please try again later.”
- “Merchant profile has expired. Contact merchant support.”
- “Merchant not authorized. Please verify merchant identity.”
```

**Category: Security Failures**
```
Template: “Security verification failed. {detail}. Transaction cancelled for your protection.”

Examples:
- “Security verification failed. Contract does not match verified template. Transaction cancelled for your protection.”
- “Security verification failed. Unable to confirm contract authenticity. Transaction cancelled for your protection.”
```

**Category: Network/Service Issues**
```
Template: “{service} temporarily unavailable. Please try again {timeframe}.”

Examples:
- “Verification service temporarily unavailable. Please try again in a few moments.”
- “Network verification temporarily unavailable. Please try again.”
```

**Category: Policy Violations**
```
Template: “{policy_rule}. {explanation}.”

Examples:
- “Transaction amount exceeds limit. Please reduce amount or contact support.”
- “Daily transaction limit reached. You can make more transactions tomorrow.”
- “This blockchain is not supported. Please choose a different payment method.”
```

### 9.4 Logging Requirements

**Purpose:** Define mandatory logging requirements for TBC operations to support debugging, auditing, and security monitoring.

**Log Levels:**

- **ERROR**: Layer failure, rejection, exception
- **WARN**: Degraded operation, dissenting providers, retry
- **INFO**: Successful verification, layer completion
- **DEBUG**: Detailed step-by-step execution

**Mandatory Log Events:**

**Event 1: Query Received**
```
Level: INFO
Message: “TBC query received”
Fields:
  - query_id
  - merchant_id
  - profile_reference
  - amount
  - asset
  - timestamp
```

**Event 2: Layer Start**
```
Level: DEBUG
Message: “Starting Layer {N} evaluation”
Fields:
  - query_id
  - layer_number
  - layer_name
  - timestamp
```

**Event 3: Layer Pass**
```
Level: INFO
Message: “Layer {N} passed”
Fields:
  - query_id
  - layer_number
  - layer_name
  - execution_time_ms
  - details (layer-specific)
```

**Event 4: Layer Fail**
```
Level: ERROR
Message: “Layer {N} failed”
Fields:
  - query_id
  - layer_number
  - layer_name
  - error_code
  - error_type
  - reason (technical)
  - execution_time_ms
  - stack_trace (if exception)
```

**Event 5: RPC Provider Response**
```
Level: DEBUG
Message: “RPC provider response”
Fields:
  - query_id
  - provider_id
  - success (bool)
  - latency_ms
  - bytecode_hash (if success)
  - error (if failure)
```

**Event 6: RPC Quorum Decision**
```
Level: INFO
Message: “RPC quorum evaluation”
Fields:
  - query_id
  - total_providers
  - valid_responses
  - quorum_achieved (bool)
  - consensus_hash (if achieved)
  - consensus_count
  - dissenting_providers (list)
```

**Event 7: Verification Complete**
```
Level: INFO
Message: “TBC verification complete: {APPROVED|DENIED}”
Fields:
  - query_id
  - result (APPROVED|DENIED)
  - total_execution_time_ms
  - verification_summary (all layer results)
  - error_code (if DENIED)
```

**Sensitive Data Handling in Logs:**

MUST NOT log:
- Private keys
- Signatures (full signature bytes)
- Wallet private addresses (use pseudonyms only)
- User PII (personal identifiable information)

MAY log:
- Contract addresses (public data)
- Bytecode hashes (public data)
- Transaction amounts (redacted in production if required)
- Merchant IDs (merchant-side identifiers)

**Log Retention:**

- ERROR logs: Retain 90 days minimum
- INFO logs: Retain 30 days minimum
- DEBUG logs: Retain 7 days (optional in production)

**Log Aggregation:**

Logs SHOULD be aggregated for:
- Error rate monitoring
- Provider performance tracking
- Layer-specific failure analysis
- Security incident investigation

——

## 10. Security Guarantees

### 10.1 Trust Boundary Assertions

**Purpose:** Define explicit trust assumptions and security boundaries for TBC system.

**Trust Boundary Map:**

```
┌──────────────────────────────────────────────────────────┐
│                    TRUSTED ZONE                           │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  TBC Core System                                │    │
│  │  - Verification logic                           │    │
│  │  - Cryptographic libraries                      │    │
│  │  - Quorum consensus algorithm                   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Configuration & Policy                         │    │
│  │  - Quorum parameters (N, M)                     │    │
│  │  - RPC provider list                            │    │
│  │  - Expected bytecode hashes                     │    │
│  │  - Policy rules                                 │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│               CONDITIONALLY TRUSTED ZONE                  │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Merchant Registry (CBS)                        │    │
│  │  - Enable/disable flags                         │    │
│  │  - Merchant public keys                         │    │
│  │  Trust: Read-only access; multi-sig for writes │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Blockchain Consensus                           │    │
│  │  - Block production                             │    │
│  │  - State transitions                            │    │
│  │  Trust: Honest majority of validators          │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│                  UNTRUSTED ZONE                           │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  RPC Providers (Multiple Independent)           │    │
│  │  - Bytecode queries                             │    │
│  │  - State queries                                │    │
│  │  Trust: NONE (quorum consensus required)       │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Merchant-Signed Profiles                       │    │
│  │  - Contract addresses                           │    │
│  │  - Profile metadata                             │    │
│  │  Trust: Advisory only (Layer 3 independently   │    │
│  │         verifies bytecode)                      │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Network Infrastructure                         │    │
│  │  - DNS                                          │    │
│  │  - TLS certificates                             │    │
│  │  - Network routing                              │    │
│  │  Trust: NONE (cryptographic verification used) │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

### 10.2 Non-Bypassability Requirements

**Purpose:** Define requirements ensuring TBC cannot be bypassed in consumer payment flows.

**Architectural Requirements:**

**Requirement 1: No Direct Extension → Contract Path**

Extension MUST NOT construct transactions directly to contract addresses without TBC verification.

**Implementation:** Extension should not have hardcoded contract addresses. All addresses MUST come from TBC Economic Envelopes.

**Requirement 2: Economic Envelope Signature Required**

Extension MUST verify TBC signature on Economic Envelope before using.

**Requirement 3: Envelope Expiration Enforced**

Extension MUST reject expired envelopes.

**Requirement 4: CBS Cannot Issue Envelopes**

CBS MUST NOT have capability to generate Economic Envelopes or TBC signatures.

**Requirement 5: User Cannot Override TBC Decision**

Extension UI MUST NOT provide “proceed anyway” option for TBC rejections.

### 10.3 Cryptographic Integrity

**Purpose:** Define cryptographic guarantees provided by TBC verification.

**Guarantee 1: Bytecode Authenticity (Keccak-256)**

**Statement:** If TBC approves transaction, contract bytecode is cryptographically verified to match audited template.

**Security Level:** 128-bit collision resistance (Keccak-256)

**Guarantee 2: Signature Non-Repudiation (ECDSA)**

**Statement:** Merchant cannot deny signing payment profile (Layer 2).

**Security Level:** 128-bit security (256-bit ECDSA)

**Guarantee 3: Envelope Integrity (TBC Signature)**

**Statement:** Extension can cryptographically verify Economic Envelope was issued by TBC.

**Security Level:** 128-bit security (256-bit ECDSA)

**Guarantee 4: ZK Proof Soundness (Layer 4)**

**Statement:** If ZK proof verifies, proof statement is true (contract satisfies all claimed properties).

**Security Level:** Depends on ZK system (typically 128-bit soundness)

### 10.4 Failure Mode Safety

**Purpose:** Define how TBC behaves under various failure scenarios.

**Failure Scenario 1: All RPC Providers Down**

**Behavior:** Layer 3 fails → Transaction REJECTED

**Rationale:** Cannot verify bytecode without RPC data. Fail-closed.

**Failure Scenario 2: Registry Service Down**

**Behavior:** Layer 1 fails → Transaction REJECTED

**Rationale:** Cannot verify merchant enabled flag. Fail-closed.

**Failure Scenario 3: Partial RPC Failure (< M providers)**

**Behavior:** Layer 3 fails if valid responses < M → Transaction REJECTED

**Rationale:** Insufficient quorum. Fail-closed.

**Failure Scenario 4: TBC Software Exception**

**Behavior:** Exception caught → Transaction REJECTED

**Rationale:** Unexpected error treated as verification failure. Fail-closed.

**Failure Scenario 5: Blockchain Reorg During Verification**

**Behavior:** RPC providers may return inconsistent data → Quorum fails → Transaction REJECTED

**Rationale:** Reorg causes temporary inconsistency. Fail-closed until reorg settles.

**Failure Scenario 6: TBC Signing Key Compromised**

**Behavior:** Attacker could generate fraudulent Economic Envelopes

**Impact:** Critical security failure

**Mitigation:**
- HSM-based key storage
- Key rotation procedures
- Extension should validate envelope contents match user intent
- Monitor for unusual envelope patterns

——

## 11. Inter-Spec Dependencies

**Purpose:** Define relationships between TGP-TBC-SEC-00 and other TGP specifications.

### 11.1 TGP-00 v3.1 (Core Protocol)

**Relationship:** TGP-TBC-SEC-00 expands Section 3 of TGP-00 v3.1.

**What TGP-00 Defines:**
- High-level TBC purpose and role
- Five security layers (overview)
- Economic Envelope concept
- Error response format

**What TGP-TBC-SEC-00 Adds:**
- Complete verification algorithms
- Bytecode hash computation details
- RPC quorum mechanisms
- Layer sequencing rules
- Comprehensive error taxonomy

### 11.2 TGP-CP-EXT-00 (Browser Extension Interface)

**Relationship:** Extension implements client-side of TBC interaction.

**Extension Responsibilities:**
- Construct TGP QUERY messages
- Send QUERY to TBC endpoint
- Receive and validate Economic Envelope
- Enforce envelope expiration
- Hand verified transaction to wallet
- Display error messages to user

### 11.3 TGP-PROFILE-00 (Payment Profile Schema)

**Relationship:** TBC verifies contracts specified in payment profiles.

**Profile Provides:**
- `contract_address`: Address to verify (Layer 3 input)
- `engine_version`: CoreProver version (determines expected bytecode hash)
- `chain_id`: Blockchain identifier (Layer 3 validation)
- `asset_address`: Payment asset (Layer 3 state check)
- `signature`: Merchant signature (Layer 2 input)

### 11.4 TGP-ENGINE-00 (Settlement Engine)

**Relationship:** TBC verifies contracts; ENGINE-00 defines contract behavior.

**Separation of Concerns:**
- TBC: Pre-transaction verification (Is contract safe?)
- ENGINE-00: Post-transaction execution (How does contract work?)

### 11.5 TGP-02 (ZK Proof Circuits)

**Relationship:** Defines ZK proof format for Layer 4.

**TGP-02 Defines:**
- ZK circuit specifications
- Proof formats (Groth16, PLONK, STARK)
- Public input schemas
- Verification algorithms

### 11.6 TGP-MGMT-00 (Management Protocol)

**Relationship:** Defines operational management of TBC.

**MGMT-00 May Define:**
- TBC configuration management
- RPC provider registration/removal
- Bytecode hash updates (new engine versions)
- Policy rule updates
- Monitoring and alerting

——

**END OF TGP-TBC-SEC-00 SPECIFICATION - PART 2 (First Half Complete)**

This completes the first half of the specification, ending just before the “Decision Tree Traversal Rules” section. The second half will contain sections 12-21 including all appendices, test vectors, implementation checklists, and references.

```
                                    QUERY Received
                                         |
                                         v
                            +—————————+
                            | Layer 1: Registry Check   |
                            +—————————+
                                         |
                         +—————+—————+
                         |                               |
                    [enabled=true]                  [enabled=false]
                         |                               |
                         v                               v
            +—————————+         [REJECT: MERCHANT_DISABLED]
            | Layer 2: Signature Check  |
            +—————————+
                         |
                 +-——+-——+
                 |               |
          [signature valid] [signature invalid]
                 |               |
                 v               v
    +—————————+ [REJECT: INVALID_SIGNATURE]
    | Layer 3: Contract Verify  |
    +—————————+
                 |
         +-——+-——+-——+
         |               |       |
   [quorum & match] [no quorum] [mismatch]
         |               |       |
         v               |       v
    +-———+         |   [REJECT: CODE_MISMATCH]
    | Layer 4  |         |
    | (if req) |         v
    +-———+  [REJECT: INSUFFICIENT_QUORUM]
         |
   +——+——+
   |           |
[valid/skip] [invalid]
   |           |
   v           v
+-———+ [REJECT: ZK_ATTESTATION_REQUIRED]
| Layer 5  |
| Policy   |
+-———+
   |
   +-——+-——+
   |               |
[all rules pass] [violation]
   |               |
   v               v
[APPROVE]   [REJECT: POLICY_VIOLATION]
   |
   v
Generate Economic Envelope
```

**Decision Tree Traversal Rules:**

1. **Sequential Evaluation:** Tree is traversed top-down, one layer at a time
1. **First Rejection Wins:** First layer that fails determines rejection error code
1. **No Backtracking:** Once a layer is evaluated, result is final (no retry within same verification)
1. **Leaf Nodes Only:** Only leaf nodes (APPROVE or REJECT) are valid final states

**Rejection Paths:**

```
Path A: Registry Disabled
QUERY → Layer 1 [FAIL] → REJECT (MERCHANT_DISABLED)
Layers 2-5: Not evaluated

Path B: Invalid Signature
QUERY → Layer 1 [PASS] → Layer 2 [FAIL] → REJECT (INVALID_SIGNATURE)
Layers 3-5: Not evaluated

Path C: Bytecode Mismatch
QUERY → Layer 1 [PASS] → Layer 2 [PASS] → Layer 3 [FAIL: mismatch] 
     → REJECT (CODE_MISMATCH)
Layers 4-5: Not evaluated

Path D: RPC Inconsistency
QUERY → Layer 1 [PASS] → Layer 2 [PASS] → Layer 3 [FAIL: no quorum]
     → REJECT (INSUFFICIENT_QUORUM)
Layers 4-5: Not evaluated

Path E: ZK Attestation Required but Unavailable
QUERY → Layer 1 [PASS] → Layer 2 [PASS] → Layer 3 [PASS] → Layer 4 [FAIL]
     → REJECT (ZK_ATTESTATION_REQUIRED)
Layer 5: Not evaluated

Path F: Policy Violation
QUERY → Layer 1 [PASS] → Layer 2 [PASS] → Layer 3 [PASS] 
     → Layer 4 [PASS/SKIP] → Layer 5 [FAIL]
     → REJECT (POLICY_VIOLATION)

Path G: Success
QUERY → Layer 1 [PASS] → Layer 2 [PASS] → Layer 3 [PASS]
     → Layer 4 [PASS/SKIP] → Layer 5 [PASS]
     → APPROVE (Generate Economic Envelope)
```

**Error Code Mapping:**

|Rejection Path|Error Code                    |Layer Failed|Retry Recommended       |
|—————|——————————|————|————————|
|Path A        |TBC_L1_REGISTRY_FAIL          |1           |No (permanent)          |
|Path B        |TBC_L2_SIGNATURE_FAIL         |2           |No (permanent)          |
|Path C        |TBC_L3_CODE_MISMATCH          |3           |No (permanent)          |
|Path D        |TBC_L3_INSUFFICIENT_QUORUM    |3           |Yes (transient)         |
|Path E        |TBC_L4_ZK_ATTESTATION_REQUIRED|4           |Yes (may succeed later) |
|Path F        |TBC_L5_{SPECIFIC}             |5           |Depends on specific rule|

——

## 8. Diagrams and Visual Models

### 8.1 Onion Layer Interaction Diagram

**Purpose:** Visualize data flow between security layers and dependencies.

```mermaid
graph TD
    A[TGP QUERY Message] —> B{Layer 1<br/>Registry Check}
    
    B —>|enabled=true| C{Layer 2<br/>Signature Verify}
    B —>|enabled=false| Z1[REJECT:<br/>MERCHANT_DISABLED]
    
    C —>|valid| D{Layer 3<br/>Contract Verify}
    C —>|invalid| Z2[REJECT:<br/>INVALID_SIGNATURE]
    
    D —>|pass| E{Layer 4<br/>ZK Attestation<br/>if required}
    D —>|bytecode mismatch| Z3[REJECT:<br/>CODE_MISMATCH]
    D —>|RPC inconsistency| Z4[REJECT:<br/>INSUFFICIENT_QUORUM]
    
    E —>|valid or skipped| F{Layer 5<br/>Policy Engine}
    E —>|invalid when required| Z5[REJECT:<br/>ZK_ATTESTATION_REQUIRED]
    
    F —>|all rules pass| G[Economic Envelope<br/>Generated]
    F —>|violation| Z6[REJECT:<br/>POLICY_VIOLATION]
    
    G —> H[Return to Extension]
    
    Z1 —> H
    Z2 —> H
    Z3 —> H
    Z4 —> H
    Z5 —> H
    Z6 —> H
    
    style A fill:#e1f5ff
    style G fill:#c8e6c9
    style Z1 fill:#ffcdd2
    style Z2 fill:#ffcdd2
    style Z3 fill:#ffcdd2
    style Z4 fill:#ffcdd2
    style Z5 fill:#ffcdd2
    style Z6 fill:#ffcdd2
```

**Data Dependencies:**

```
Layer 1 Output → Layer 2 Input:
- merchant_id (used to lookup public key)

Layer 2 Output → Layer 3 Input:
- contract_address (from signed profile, advisory only)
- engine_version (from signed profile)

Layer 3 Output → Layer 4 Input:
- bytecode_hash (for ZK proof public inputs)
- contract_state (for ZK proof verification)

Layer 3 Output → Layer 5 Input:
- contract_address (for policy checks)
- asset_address (for asset whitelist check)
- chain_id (for chain whitelist check)

All Layers → Economic Envelope:
- verification_summary (aggregated results)
```

### 8.2 Verification Flow Chart

**Purpose:** Detailed flow chart showing all decision points and actions in verification process.

```
START: Receive QUERY
    ↓
[Extract profile_reference from QUERY]
    ↓
┌─────────────────────────────────────┐
│ Layer 1: Query Merchant Registry    │
│                                     │
│ Input: merchant_id, profile_id      │
│ Query: GET /registry/{profile_id}   │
│ Output: {enabled: bool, status: str}│
└──────────────┬──────────────────────┘
               ↓
        [Check enabled flag]
               ↓
         ┌─────┴─────┐
    [enabled=true]  [enabled=false OR query failed]
         │                    │
         │                    └──→ ERROR: MERCHANT_DISABLED
         ↓                                  ↓
┌─────────────────────────────────────┐    RETURN to Extension
│ Layer 2: Retrieve Profile          │
│                                     │
│ Input: profile_reference            │
│ Fetch: Profile descriptor (signed)  │
│ Output: Profile metadata + signature│
└──────────────┬──────────────────────┘
               ↓
[Extract merchant public key from registry]
               ↓
[Verify signature: verify(profile, signature, public_key)]
               ↓
         ┌─────┴─────┐
    [valid]      [invalid OR expired]
         │                    │
         │                    └──→ ERROR: INVALID_SIGNATURE
         ↓                                  ↓
┌─────────────────────────────────────┐    RETURN to Extension
│ Layer 3: Multi-RPC Query            │
│                                     │
│ Input: contract_address, chain_id   │
│ Query N providers: eth_getCode()    │
│ Output: N bytecode responses        │
└──────────────┬──────────────────────┘
               ↓
[Filter valid responses]
               ↓
[Compute hash for each response]
               ↓
[Check quorum: M-of-N consensus?]
               ↓
         ┌─────┴─────┐
    [quorum]     [no quorum]
         │                │
         │                └──→ ERROR: INSUFFICIENT_QUORUM
         ↓                              ↓
[Compare consensus_hash with expected_hash]    RETURN to Extension
         ↓
    ┌────┴────┐
[match]   [mismatch]
    │           │
    │           └──→ ERROR: CODE_MISMATCH
    ↓                       ↓
[Verify contract state]    RETURN to Extension
    ↓
┌────┴────┐
[valid]  [invalid]
    │        │
    │        └──→ ERROR: INVALID_STATE
    ↓                    ↓
    │             RETURN to Extension
    ↓
┌─────────────────────────────────────┐
│ Layer 4: Check Policy Requirement  │
│                                     │
│ Input: transaction amount, jurisdiction│
│ Decision: ZK attestation required?  │
└──────────────┬──────────────────────┘
               ↓
    ┌──────────┴──────────┐
[required]            [not required]
    │                      │
    ↓                      └──→ SKIP Layer 4
[Request ZK proof]                 ↓
    ↓                       ┌──────────────┐
[Verify proof]              │ Layer 5      │
    ↓                       └──────────────┘
┌───┴───┐
[valid][invalid]
  │       │
  │       └──→ ERROR: ZK_ATTESTATION_REQUIRED
  ↓                   ↓
  │            RETURN to Extension
  ↓
┌─────────────────────────────────────┐
│ Layer 5: Evaluate Policy Rules      │
│                                     │
│ Input: query, profile, user_context│
│ Check: chain, asset, value, sanctions│
│ Output: pass/fail + rule details    │
└──────────────┬──────────────────────┘
               ↓
    ┌──────────┴──────────┐
[all pass]          [violation]
    │                      │
    │                      └──→ ERROR: POLICY_VIOLATION
    ↓                                  ↓
┌─────────────────────────────────────┐    RETURN to Extension
│ Generate Economic Envelope          │
│                                     │
│ Input: Verification results         │
│ Output: Signed envelope             │
└──────────────┬──────────────────────┘
               ↓
        [Sign envelope]
               ↓
        [Return APPROVED]
               ↓
        RETURN to Extension

END
```

### 8.3 Bytecode-Verification Diagram

**Purpose:** Detailed visualization of Layer 3 bytecode verification with multi-RPC quorum.

```
                      Layer 3 Start
                           |
                           v
              ┌────────────────────────┐
              │  Select N RPC Providers │
              │  (N ≥ 3)                │
              └────────────┬───────────┘
                           |
         ┌─────────────────┼─────────────────┐
         |                 |                 |
         v                 v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │Provider │      │Provider │      │Provider │
    │   1     │      │   2     │      │   3     │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | eth_getCode    | eth_getCode    | eth_getCode
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │Bytecode │      │Bytecode │      │Bytecode │
    │  “0x60” │      │  “0x60” │      │  “0x61” │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | hex_decode     | hex_decode     | hex_decode
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │ Bytes   │      │ Bytes   │      │ Bytes   │
    │ [0x60..]│      │ [0x60..]│      │ [0x61..]│
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         | keccak256      | keccak256      | keccak256
         v                v                 v
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │  Hash   │      │  Hash   │      │  Hash   │
    │ 0xABC.. │      │ 0xABC.. │      │ 0xDEF.. │
    └────┬────┘      └────┬────┘      └────┬────┘
         |                |                 |
         └────────────────┼─────────────────┘
                          |
                          v
              ┌────────────────────────┐
              │  Count Hash Occurrences │
              │                         │
              │  0xABC..: count = 2     │
              │  0xDEF..: count = 1     │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Check Quorum          │
              │  (M = 2)               │
              │                        │
              │  max_count = 2         │
              │  2 ≥ M ? YES           │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Consensus Hash:       │
              │  0xABC...              │
              └────────────┬───────────┘
                           |
                           v
              ┌────────────────────────┐
              │  Compare with Expected │
              │                        │
              │  Expected: 0xABC...    │
              │  Actual:   0xABC...    │
              └────────────┬───────────┘
                           |
                 ┌─────────┴──────────┐
                 |                    |
             [MATCH]            [MISMATCH]
                 |                    |
                 v                    v
             ┌───────┐          ┌──────────┐
             │ PASS  │          │ REJECT   │
             └───────┘          └──────────┘

Note: Provider 3 dissented (different hash)
Action: Log for investigation but allow transaction
        (quorum achieved with Providers 1+2)
```

**Quorum Calculation Example:**

```
Configuration: N=5, M=3

Scenario 1: Clear Consensus
Provider 1: hash A
Provider 2: hash A
Provider 3: hash A
Provider 4: hash B
Provider 5: timeout

Result:
- Valid responses: 4 (excludes timeout)
- Hash A count: 3
- Hash B count: 1
- Quorum achieved: 3 ≥ M (3) → YES
- Consensus: hash A

Scenario 2: No Consensus
Provider 1: hash A
Provider 2: hash B
Provider 3: hash A
Provider 4: hash C
Provider 5: hash B

Result:
- Valid responses: 5
- Hash A count: 2
- Hash B count: 2
- Hash C count: 1
- Quorum achieved: 2 < M (3) → NO
- Result: REJECT (insufficient quorum)

Scenario 3: Insufficient Valid Responses
Provider 1: hash A
Provider 2: timeout
Provider 3: error
Provider 4: timeout
Provider 5: hash A

Result:
- Valid responses: 2 (only 1 and 5)
- Hash A count: 2
- Quorum achieved: 2 < M (3) → NO
- Result: REJECT (insufficient quorum)
```

### 8.4 RPC Quorum Visual Model

**Purpose:** Illustrate quorum consensus logic with different provider response patterns.

```
Quorum Model: M-of-N Consensus

Configuration: N=3, M=2

┌───────────────────────────────────────────────────────────┐
│                     Case 1: Unanimous                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ████████ hash=0xABC                        │
│                                                           │
│  Consensus: 0xABC (3/3 = 100%)                           │
│  Quorum: 3 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│                  Case 2: Majority Consensus                │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ▓▓▓▓▓▓▓▓ hash=0xDEF (dissent)             │
│                                                           │
│  Consensus: 0xABC (2/3 = 67%)                            │
│  Quorum: 2 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
│  Warning: Provider 3 dissenting - log for investigation  │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│                  Case 3: No Consensus (Tie)                │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ▓▓▓▓▓▓▓▓ hash=0xDEF                        │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Consensus: None (1/2 valid responses each)              │
│  Quorum: 1 < 2 ✗                                         │
│  Result: REJECT (insufficient quorum)                    │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│              Case 4: Timeout Majority                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ░░░░░░░░ timeout                           │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Consensus: 0xABC (1/1 valid = 100%)                     │
│  Quorum: 1 < 2 ✗                                         │
│  Result: REJECT (insufficient valid responses)           │
└───────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────┐
│           Case 5: Quorum with One Timeout                  │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  Provider 1:  ████████ hash=0xABC                        │
│  Provider 2:  ████████ hash=0xABC                        │
│  Provider 3:  ░░░░░░░░ timeout                           │
│                                                           │
│  Consensus: 0xABC (2/2 valid = 100%)                     │
│  Quorum: 2 ≥ 2 ✓                                         │
│  Result: PASS (if 0xABC matches expected)                │
│  Note: Timeout does not count against quorum             │
└───────────────────────────────────────────────────────────┘

Legend:
████████ = Valid response agreeing with consensus
▓▓▓▓▓▓▓▓ = Valid response dissenting from consensus
░░░░░░░░ = Timeout or error (not counted)
```

**Quorum Visualization Matrix:**

```
N=3, M=2 (All Possible Outcomes)

Valid    Consensus   Quorum    Result
Responses  Count    Achieved
─────────────────────────────────────
  3         3         ✓        PASS*
  3         2         ✓        PASS*
  3         1         ✗        REJECT
  2         2         ✓        PASS*
  2         1         ✗        REJECT
  1         1         ✗        REJECT
  0         0         ✗        REJECT

* PASS only if consensus hash matches expected hash
  Otherwise REJECT with CODE_MISMATCH
```

——

## 9. Error Codes & Rejection Semantics

### 9.1 Mandatory Error Types

**Purpose:** Define complete taxonomy of TBC error conditions with mandatory error codes, messages, and handling semantics.

**Error Code Format:**

```
TBC_L{layer}_{TYPE}_{DETAIL}

Where:
- L{layer} = Layer number (1-5)
- {TYPE} = High-level error category
- {DETAIL} = Specific error condition (optional)

Examples:
- TBC_L1_REGISTRY_FAIL
- TBC_L3_CODE_MISMATCH
- TBC_L5_RATE_LIMIT
```

**Complete Error Taxonomy:**

**Layer 1 Errors:**

|Error Code             |Error Type          |Reason                          |User Message                                       |Retry Allowed|
|————————|———————|———————————|—————————————————|-————|
|TBC_L1_REGISTRY_FAIL   |MERCHANT_DISABLED   |Profile disabled in registry    |This merchant is temporarily unavailable.          |No           |
|TBC_L1_REGISTRY_ERROR  |REGISTRY_UNAVAILABLE|Registry query failed or timeout|Verification service unavailable. Please try again.|Yes          |
|TBC_L1_REGISTRY_INVALID|REGISTRY_UNAVAILABLE|Malformed registry response     |Verification service error. Please try again.      |Yes          |

**Layer 2 Errors:**

|Error Code              |Error Type                  |Reason                               |User Message                                  |Retry Allowed|
|————————|-—————————|-————————————|-———————————————|-————|
|TBC_L2_SIGNATURE_FAIL   |INVALID_SIGNATURE           |Signature verification failed        |Unable to verify merchant authenticity.       |No           |
|TBC_L2_SIGNATURE_EXPIRED|INVALID_SIGNATURE           |Signature age exceeds validity period|Merchant profile expired. Contact merchant.   |No           |
|TBC_L2_PUBKEY_NOT_FOUND |INVALID_SIGNATURE           |Merchant public key not in registry  |Merchant authentication failed.               |No           |
|TBC_L2_INTERNAL_ERROR   |SIGNATURE_VERIFICATION_ERROR|Exception during verification        |Internal verification error. Please try again.|Yes          |

**Layer 3 Errors:**

|Error Code                |Error Type                  |Reason                                  |User Message                                                            |Retry Allowed|
|—————————|-—————————|-—————————————|————————————————————————|-————|
|TBC_L3_CODE_MISMATCH      |CONTRACT_VERIFICATION_FAILED|Bytecode hash does not match expected   |Security verification failed. Transaction cancelled for your protection.|No           |
|TBC_L3_INSUFFICIENT_QUORUM|RPC_INCONSISTENCY           |Fewer than M providers agreed           |Network verification inconsistency. Please try again.                   |Yes          |
|TBC_L3_RPC_DISAGREEMENT   |RPC_INCONSISTENCY           |Providers returned different bytecode   |Network verification conflict detected. Please try again.               |Yes          |
|TBC_L3_ALL_RPC_FAILED     |RPC_INCONSISTENCY           |All RPC providers failed or timed out   |Network unavailable. Please try again.                                  |Yes          |
|TBC_L3_NO_CONTRACT        |CONTRACT_VERIFICATION_FAILED|Contract does not exist (empty bytecode)|Contract not found at specified address.                                |No           |
|TBC_L3_INVALID_BYTECODE   |CONTRACT_VERIFICATION_FAILED|Bytecode format invalid                 |Contract data invalid.                                                  |No           |
|TBC_L3_INVALID_STATE      |CONTRACT_VERIFICATION_FAILED|Contract state validation failed        |Contract in invalid state.                                              |No           |
|TBC_L3_UNSUPPORTED_VERSION|CONTRACT_VERIFICATION_FAILED|Engine version not supported            |Merchant using unsupported system version.                              |No           |
|TBC_L3_INTERNAL_ERROR     |CONTRACT_VERIFICATION_ERROR |Exception during verification           |Internal verification error. Please try again.                          |Yes          |

**Layer 4 Errors:**

|Error Code                    |Error Type             |Reason                           |User Message                                   |Retry Allowed|
|——————————|————————|———————————|————————————————|-————|
|TBC_L4_ZK_ATTESTATION_REQUIRED|ZK_ATTESTATION_REQUIRED|ZK proof required but unavailable|Enhanced verification required but unavailable.|Yes          |
|TBC_L4_ZK_ATTESTATION_FAIL    |ZK_ATTESTATION_REQUIRED|ZK proof verification failed     |Enhanced verification failed.                  |Yes          |
|TBC_L4_ZK_SERVICE_UNAVAILABLE |ZK_ATTESTATION_ERROR   |Proof service unreachable        |Enhanced verification service unavailable.     |Yes          |
|TBC_L4_ZK_PROOF_EXPIRED       |ZK_ATTESTATION_REQUIRED|Proof timestamp too old          |Verification proof expired.                    |Yes          |
|TBC_L4_INTERNAL_ERROR         |ZK_ATTESTATION_ERROR   |Exception during verification    |Internal verification error. Please try again. |Yes          |

**Layer 5 Errors:**

|Error Code                    |Error Type             |Reason                                   |User Message                                               |Retry Allowed|
|——————————|————————|——————————————|————————————————————|-————|
|TBC_L5_CHAIN_NOT_ALLOWED      |POLICY_VIOLATION       |Transaction chain not in whitelist       |Blockchain not supported for this transaction.             |No           |
|TBC_L5_ASSET_NOT_ALLOWED      |POLICY_VIOLATION       |Payment asset not in whitelist           |Asset type not accepted.                                   |No           |
|TBC_L5_VALUE_EXCEEDS_LIMIT    |POLICY_VIOLATION       |Amount exceeds policy limit              |Transaction amount exceeds limit.                          |No           |
|TBC_L5_RATE_LIMIT             |POLICY_VIOLATION       |Too many requests in time window         |Daily transaction limit reached. Please try again tomorrow.|Maybe        |
|TBC_L5_SANCTIONS_VIOLATION    |POLICY_VIOLATION       |Merchant or user on sanctions list       |Transaction not permitted due to compliance rules.         |No           |
|TBC_L5_JURISDICTION_RESTRICTED|POLICY_VIOLATION       |Transaction violates jurisdictional rules|Transaction not permitted in your region.                  |No           |
|TBC_L5_INTERNAL_ERROR         |POLICY_EVALUATION_ERROR|Exception during policy evaluation       |Internal policy error. Please try again.                   |Yes          |

### 9.2 Machine-Readable Error Codes

**Purpose:** Define structured error codes for programmatic handling by extensions and integrations.

**Error Response Schema:**

```json
{
  “status”: “DENIED”,
  “error”: “{ERROR_TYPE}”,
  “code”: “{ERROR_CODE}”,
  “layer_failed”: {1-5},
  “timestamp”: “{ISO8601_timestamp}”,
  “reason”: “{technical_reason}”,
  “user_message”: “{user_friendly_message}”,
  “support_reference”: “{ticket_id}”,
  “retry_allowed”: {true|false},
  “retry_after”: {seconds} // OPTIONAL: if retry_allowed=true
}
```

**Field Definitions:**

- `status`: MUST be “DENIED” for all error responses
- `error`: High-level error type (e.g., “MERCHANT_DISABLED”, “CONTRACT_VERIFICATION_FAILED”)
- `code`: Specific error code (e.g., “TBC_L1_REGISTRY_FAIL”)
- `layer_failed`: Integer 1-5 indicating which layer rejected
- `timestamp`: ISO 8601 timestamp of error
- `reason`: Technical explanation (for logging, not user display)
- `user_message`: User-friendly message (extension displays this)
- `support_reference`: Unique reference for support/debugging
- `retry_allowed`: Boolean indicating if user should retry
- `retry_after`: Seconds to wait before retry (optional)

**Example Error Responses:**

**Example 1: Merchant Disabled**

```json
{
  “status”: “DENIED”,
  “error”: “MERCHANT_DISABLED”,
  “code”: “TBC_L1_REGISTRY_FAIL”,
  “layer_failed”: 1,
  “timestamp”: “2025-11-18T12:00:00Z”,
  “reason”: “Merchant payment profile disabled in registry: status=suspended”,
  “user_message”: “This merchant is temporarily unavailable. Please try again later or contact support.”,
  “support_reference”: “TBC-20251118-001234”,
  “retry_allowed”: false
}
```

**Example 2: Bytecode Mismatch**

```json
{
  “status”: “DENIED”,
  “error”: “CONTRACT_VERIFICATION_FAILED”,
  “code”: “TBC_L3_CODE_MISMATCH”,
  “layer_failed”: 3,
  “timestamp”: “2025-11-18T12:00:05Z”,
  “reason”: “Contract bytecode hash mismatch: expected=0x7f8b6c..., actual=0x3c4d9a...”,
  “user_message”: “Security verification failed. Transaction cancelled for your protection.”,
  “support_reference”: “TBC-20251118-001235”,
  “retry_allowed”: false
}
```

**Example 3: RPC Timeout (Transient)**

```json
{
  “status”: “DENIED”,
  “error”: “RPC_INCONSISTENCY”,
  “code”: “TBC_L3_ALL_RPC_FAILED”,
  “layer_failed”: 3,
  “timestamp”: “2025-11-18T12:00:10Z”,
  “reason”: “All RPC providers timed out: timeout=10s, providers=[infura, alchemy, quicknode]”,
  “user_message”: “Network verification temporarily unavailable. Please try again.”,
  “support_reference”: “TBC-20251118-001236”,
  “retry_allowed”: true,
  “retry_after”: 30
}
```

**Example 4: Policy Violation (Rate Limit)**

```json
{
  “status”: “DENIED”,
  “error”: “POLICY_VIOLATION”,
  “code”: “TBC_L5_RATE_LIMIT”,
  “layer_failed”: 5,
  “timestamp”: “2025-11-18T12:00:15Z”,
  “reason”: “Daily transaction limit exceeded: count=51, limit=50, window=24h”,
  “user_message”: “Daily transaction limit reached. Please try again tomorrow.”,
  “support_reference”: “TBC-20251118-001237”,
  “retry_allowed”: true,
  “retry_after”: 43200 // 12 hours until next day
}
```

### 9.3 Human-Readable Reason Strings

**Purpose:** Define guidelines for generating user-friendly error messages that are clear, actionable, and non-technical.

**Message Composition Rules:**

**Rule 1: User-Focused Language**

AVOID: “Registry query failed with HTTP 503”
USE: “Verification service temporarily unavailable”

**Rule 2: Actionable When Possible**

AVOID: “Signature verification failed”
USE: “Unable to verify merchant authenticity. Contact merchant support.”

**Rule 3: Avoid Technical Jargon**

AVOID: “Bytecode hash mismatch detected”
USE: “Security verification failed. Transaction cancelled for your protection.”

**Rule 4: Reassure User of Safety**

INCLUDE: “Transaction cancelled for your protection” (for security failures)
INCLUDE: “No funds were transferred” (where applicable)

**Rule 5: Provide Next Steps**

For transient errors: “Please try again”
For permanent errors: “Contact merchant support” or “Choose different payment method”
For policy errors: “Please check transaction details” or “Daily limit reached. Try again tomorrow.”

**Message Templates by Category:**

**Category: Merchant/Profile Issues**

```
Template: “{merchant_status}. {action_recommendation}”

Examples:
- “This merchant is temporarily unavailable. Please try again later.”
- “Merchant profile has expired. Contact merchant support.”
- “Merchant not authorized. Please verify merchant identity.”
```

**Category: Security Failures**

```
Template: “Security verification failed. {detail}. Transaction cancelled for your protection.”

Examples:
- “Security verification failed. Contract does not match verified template. Transaction cancelled for your protection.”
- “Security verification failed. Unable to confirm contract authenticity. Transaction cancelled for your protection.”
```

**Category: Network/Service Issues**

```
Template: “{service} temporarily unavailable. Please try again {timeframe}.”

Examples:
- “Verification service temporarily unavailable. Please try again in a few moments.”
- “Network verification temporarily unavailable. Please try again.”
```

**Category: Policy Violations**

```
Template: “{policy_rule}. {explanation}.”

Examples:
- “Transaction amount exceeds limit. Please reduce amount or contact support.”
- “Daily transaction limit reached. You can make more transactions tomorrow.”
- “This blockchain is not supported. Please choose a different payment method.”
```

**Localization Considerations:**

Error messages SHOULD be localized based on user’s language preference:

```json
{
  “user_message”: {
    “en”: “This merchant is temporarily unavailable.”,
    “es”: “Este comerciante no está disponible temporalmente.”,
    “fr”: “Ce commerçant est temporairement indisponible.”,
    “de”: “Dieser Händler ist vorübergehend nicht verfügbar.”
  }
}
```

**Sensitive Information Handling:**

User messages MUST NOT contain:

- Contract addresses
- Wallet addresses
- Transaction hashes
- API keys or credentials
- Internal system details
- Stack traces or exception details

Technical details SHOULD be included in `reason` field for logging, but NEVER in `user_message`.

### 9.4 Logging Requirements

**Purpose:** Define mandatory logging requirements for TBC operations to support debugging, auditing, and security monitoring.

**Log Levels:**

- **ERROR**: Layer failure, rejection, exception
- **WARN**: Degraded operation, dissenting providers, retry
- **INFO**: Successful verification, layer completion
- **DEBUG**: Detailed step-by-step execution

**Mandatory Log Events:**

**Event 1: Query Received**

```
Level: INFO
Message: “TBC query received”
Fields:
  - query_id
  - merchant_id
  - profile_reference
  - amount
  - asset
  - timestamp
```

**Event 2: Layer Start**

```
Level: DEBUG
Message: “Starting Layer {N} evaluation”
Fields:
  - query_id
  - layer_number
  - layer_name
  - timestamp
```

**Event 3: Layer Pass**

```
Level: INFO
Message: “Layer {N} passed”
Fields:
  - query_id
  - layer_number
  - layer_name
  - execution_time_ms
  - details (layer-specific)
```

**Event 4: Layer Fail**

```
Level: ERROR
Message: “Layer {N} failed”
Fields:
  - query_id
  - layer_number
  - layer_name
  - error_code
  - error_type
  - reason (technical)
  - execution_time_ms
  - stack_trace (if exception)
```

**Event 5: RPC Provider Response**

```
Level: DEBUG
Message: “RPC provider response”
Fields:
  - query_id
  - provider_id
  - success (bool)
  - latency_ms
  - bytecode_hash (if success)
  - error (if failure)
```

**Event 6: RPC Quorum Decision**

```
Level: INFO
Message: “RPC quorum evaluation”
Fields:
  - query_id
  - total_providers
  - valid_responses
  - quorum_achieved (bool)
  - consensus_hash (if achieved)
  - consensus_count
  - dissenting_providers (list)
```

**Event 7: Verification Complete**

```
Level: INFO
Message: “TBC verification complete: {APPROVED|DENIED}”
Fields:
  - query_id
  - result (APPROVED|DENIED)
  - total_execution_time_ms
  - verification_summary (all layer results)
  - error_code (if DENIED)
```

**Sensitive Data Handling in Logs:**

MUST NOT log:

- Private keys
- Signatures (full signature bytes)
- Wallet private addresses (use pseudonyms only)
- User PII (personal identifiable information)

MAY log:

- Contract addresses (public data)
- Bytecode hashes (public data)
- Transaction amounts (redacted in production if required)
- Merchant IDs (merchant-side identifiers)

**Log Retention:**

- ERROR logs: Retain 90 days minimum
- INFO logs: Retain 30 days minimum
- DEBUG logs: Retain 7 days (optional in production)

**Log Aggregation:**

Logs SHOULD be aggregated for:

- Error rate monitoring
- Provider performance tracking
- Layer-specific failure analysis
- Security incident investigation

——

## 10. Security Guarantees

### 10.1 Trust Boundary Assertions

**Purpose:** Define explicit trust assumptions and security boundaries for TBC system.

**Trust Boundary Map:**

```
┌──────────────────────────────────────────────────────────┐
│                    TRUSTED ZONE                           │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  TBC Core System                                │    │
│  │  - Verification logic                           │    │
│  │  - Cryptographic libraries                      │    │
│  │  - Quorum consensus algorithm                   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Configuration & Policy                         │    │
│  │  - Quorum parameters (N, M)                     │    │
│  │  - RPC provider list                            │    │
│  │  - Expected bytecode hashes                     │    │
│  │  - Policy rules                                 │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│               CONDITIONALLY TRUSTED ZONE                  │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Merchant Registry (CBS)                        │    │
│  │  - Enable/disable flags                         │    │
│  │  - Merchant public keys                         │    │
│  │  Trust: Read-only access; multi-sig for writes │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Blockchain Consensus                           │    │
│  │  - Block production                             │    │
│  │  - State transitions                            │    │
│  │  Trust: Honest majority of validators          │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│                  UNTRUSTED ZONE                           │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  RPC Providers (Multiple Independent)           │    │
│  │  - Bytecode queries                             │    │
│  │  - State queries                                │    │
│  │  Trust: NONE (quorum consensus required)       │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Merchant-Signed Profiles                       │    │
│  │  - Contract addresses                           │    │
│  │  - Profile metadata                             │    │
│  │  Trust: Advisory only (Layer 3 independently   │    │
│  │         verifies bytecode)                      │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Network Infrastructure                         │    │
│  │  - DNS                                          │    │
│  │  - TLS certificates                             │    │
│  │  - Network routing                              │    │
│  │  Trust: NONE (cryptographic verification used) │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

**Trust Assertions:**

**Assertion 1: TBC Core Code**

**Trust:** TBC verification logic is correct and uncompromised.

**Rationale:** TBC is operated by CoreProver or trusted third party. Code undergoes security review and audit.

**Mitigation if Compromised:** If TBC core is compromised, all security guarantees void. Users rely on operator reputation and code audits.

**Assertion 2: Cryptographic Libraries**

**Trust:** Keccak-256 implementation is sound and free of backdoors.

**Rationale:** Using standard, audited libraries (e.g., Ethereum’s official Keccak implementation).

**Mitigation if Compromised:** Catastrophic failure; no practical mitigation. Rely on library reputation and open-source review.

**Assertion 3: Blockchain Consensus**

**Trust:** Blockchain (e.g., Ethereum) has honest majority of validators.

**Rationale:** Established blockchains have strong consensus security.

**Mitigation if Compromised:** If blockchain consensus fails (51% attack), TBC cannot prevent it. However, quorum RPC checks provide early warning of chain instability.

**Assertion 4: RPC Providers (Explicitly NOT Trusted)**

**Trust:** NONE. RPC providers are assumed potentially malicious or compromised.

**Rationale:** Single RPC provider cannot be trusted; hence quorum mechanism.

**Mitigation:** Quorum consensus from M-of-N independent providers. If M providers collude, Layer 4 (ZK attestation) provides fallback for high-value transactions.

**Assertion 5: Merchant Registry**

**Trust:** Registry enable flags are authoritative, but registry itself could be compromised.

**Rationale:** Registry is operated by CBS with multi-sig protections.

**Mitigation:** Layers 2-5 provide defense-in-depth even if registry compromised. Invalid contracts rejected by Layer 3 regardless of registry status.

**Assertion 6: Merchant-Signed Profiles**

**Trust:** Profiles are advisory only; TBC does NOT trust contract addresses in profiles.

**Rationale:** Merchant could sign malicious profile pointing to malicious contract.

**Mitigation:** Layer 3 independently verifies bytecode. Profile signature proves merchant intent, but contract bytecode proves contract safety.

### 10.2 Non-Bypassability Requirements

**Purpose:** Define requirements ensuring TBC cannot be bypassed in consumer payment flows.

**Architectural Requirements:**

**Requirement 1: No Direct Extension → Contract Path**

Extension MUST NOT construct transactions directly to contract addresses without TBC verification.

**Implementation:** Extension should not have hardcoded contract addresses. All addresses MUST come from TBC Economic Envelopes.

**Verification:** Extension code review should confirm no bypass paths exist.

**Requirement 2: Economic Envelope Signature Required**

Extension MUST verify TBC signature on Economic Envelope before using.

**Implementation:**

```
envelope = receive_envelope_from_tbc()
IF NOT verify_tbc_signature(envelope, TBC_PUBLIC_KEY) THEN
    REJECT envelope
    RETURN error(“Invalid TBC signature”)
END IF
```

**Requirement 3: Envelope Expiration Enforced**

Extension MUST reject expired envelopes.

**Implementation:**

```
IF current_time() > envelope.expires_at THEN
    REJECT envelope
    RETURN error(“Economic Envelope expired. Please request new verification.”)
END IF
```

**Requirement 4: CBS Cannot Issue Envelopes**

CBS MUST NOT have capability to generate Economic Envelopes or TBC signatures.

**Rationale:** CBS deploys contracts (merchant-side). TBC verifies contracts (consumer-side). Strict separation prevents CBS from bypassing verification.

**Implementation:** CBS and TBC use separate signing keys. CBS key cannot generate TBC-signed envelopes.

**Requirement 5: User Cannot Override TBC Decision**

Extension UI MUST NOT provide “proceed anyway” option for TBC rejections.

**Rationale:** TBC rejection means security check failed. Allowing override defeats purpose.

**Exception:** Advanced users MAY export transaction data and sign manually outside extension, accepting full risk. Extension should warn this voids security guarantees.

### 10.3 Cryptographic Integrity

**Purpose:** Define cryptographic guarantees provided by TBC verification.

**Guarantee 1: Bytecode Authenticity (Keccak-256)**

**Statement:** If TBC approves transaction, contract bytecode is cryptographically verified to match audited template.

**Security Level:** 128-bit collision resistance (Keccak-256)

**Attack Resistance:** Attacker cannot create malicious contract with same hash as audited template (computationally infeasible).

**Guarantee 2: Signature Non-Repudiation (ECDSA)**

**Statement:** Merchant cannot deny signing payment profile (Layer 2).

**Security Level:** 128-bit security (256-bit ECDSA)

**Attack Resistance:** Attacker cannot forge merchant signature without private key.

**Guarantee 3: Envelope Integrity (TBC Signature)**

**Statement:** Extension can cryptographically verify Economic Envelope was issued by TBC.

**Security Level:** 128-bit security (256-bit ECDSA)

**Attack Resistance:** Attacker cannot forge TBC signature; cannot modify envelope without detection.

**Guarantee 4: ZK Proof Soundness (Layer 4)**

**Statement:** If ZK proof verifies, proof statement is true (contract satisfies all claimed properties).

**Security Level:** Depends on ZK system (typically 128-bit soundness)

**Attack Resistance:** Attacker cannot generate valid proof for false statement (soundness property).

**Limitation:** Requires trusted setup for some ZK systems (zkSNARKs). Transparent systems (zkSTARKs) avoid this.

### 10.4 Failure Mode Safety

**Purpose:** Define how TBC behaves under various failure scenarios.

**Failure Scenario 1: All RPC Providers Down**

**Behavior:** Layer 3 fails → Transaction REJECTED

**Rationale:** Cannot verify bytecode without RPC data. Fail-closed.

**User Impact:** Payment temporarily unavailable until RPCs recover.

**Mitigation:** Use diverse RPC providers across regions; maintain high provider availability.

**Failure Scenario 2: Registry Service Down**

**Behavior:** Layer 1 fails → Transaction REJECTED

**Rationale:** Cannot verify merchant enabled flag. Fail-closed.

**User Impact:** All transactions blocked until registry recovers.

**Mitigation:** High-availability registry deployment; fallback to cached data with expiration.

**Failure Scenario 3: Partial RPC Failure (< M providers)**

**Behavior:** Layer 3 fails if valid responses < M → Transaction REJECTED

**Rationale:** Insufficient quorum. Fail-closed.

**User Impact:** Payment temporarily unavailable.

**Mitigation:** High M value increases risk of quorum failure. Balance security vs availability.

**Failure Scenario 4: TBC Software Exception**

**Behavior:** Exception caught → Transaction REJECTED

**Rationale:** Unexpected error treated as verification failure. Fail-closed.

**User Impact:** Payment fails; user sees generic error.

**Mitigation:** Comprehensive exception handling; extensive testing; production monitoring.

**Failure Scenario 5: Blockchain Reorg During Verification**

**Behavior:** RPC providers may return inconsistent data → Quorum fails → Transaction REJECTED

**Rationale:** Reorg causes temporary inconsistency. Fail-closed until reorg settles.

**User Impact:** Payment temporarily fails; retry after reorg settles (minutes).

**Mitigation:** Wait for sufficient block confirmations; use finality RPC endpoints when available.

**Failure Scenario 6: TBC Signing Key Compromised**

**Behavior:** Attacker could generate fraudulent Economic Envelopes

**Impact:** Critical security failure

**Mitigation:**

- HSM-based key storage
- Key rotation procedures
- Extension should validate envelope contents match user intent
- Monitor for unusual envelope patterns

**Recovery:** Revoke compromised key; issue new TBC key; update extension with new public key.

**Failure Recovery Principles:**

1. **Fail-Closed Default:** All ambiguous failures default to REJECT
1. **Transient Retry:** Transient failures (timeouts, network errors) allow retry
1. **Permanent Reject:** Security failures (bytecode mismatch, invalid signature) do NOT allow retry
1. **Graceful Degradation:** If non-critical components fail (logging, metrics), continue verification
1. **No Silent Failures:** All failures logged and reported to user

——

## 11. Inter-Spec Dependencies

**Purpose:** Define relationships between TGP-TBC-SEC-00 and other TGP specifications.

**Dependency Graph:**

```
         TGP-00 v3.1
         (Core Protocol)
               |
               | defines
               v
      ┌────────────────┐
      │  TGP-TBC-SEC-00│ ◄─── This Specification
      │  (TBC Security)│
      └────────┬───────┘
               |
      ┌────────┼────────┬──────────┬──────────┐
      |        |        |          |          |
      v        v        v          v          v
  TGP-CP-   TGP-    TGP-      TGP-02    TGP-MGMT-00
  EXT-00   PROFILE  ENGINE    (ZK       (Mgmt
  (Ext)    -00      -00       Proofs)   Protocol)
           (Profile) (Engine)
```

### 11.1 TGP-00 v3.1 (Core Protocol)

**Relationship:** TGP-TBC-SEC-00 expands Section 3 of TGP-00 v3.1.

**What TGP-00 Defines:**

- High-level TBC purpose and role
- Five security layers (overview)
- Economic Envelope concept
- Error response format

**What TGP-TBC-SEC-00 Adds:**

- Complete verification algorithms
- Bytecode hash computation details
- RPC quorum mechanisms
- Layer sequencing rules
- Comprehensive error taxonomy

**Normative Consistency:** All requirements in TGP-TBC-SEC-00 MUST be consistent with TGP-00 v3.1. Any conflict is a specification error.

### 11.2 TGP-CP-EXT-00 (Browser Extension Interface)

**Relationship:** Extension implements client-side of TBC interaction.

**Extension Responsibilities:**

- Construct TGP QUERY messages
- Send QUERY to TBC endpoint
- Receive and validate Economic Envelope
- Enforce envelope expiration
- Hand verified transaction to wallet
- Display error messages to user

**TBC Responsibilities:**

- Receive and validate QUERY messages
- Perform five-layer verification
- Generate Economic Envelope or Error Response
- Sign envelope with TBC key
- Return response to extension

**Interface Contract:**

- QUERY format defined in TGP-00 Section 4.3
- Envelope format defined in TGP-00 Section 4.4
- Error format defined in TGP-TBC-SEC-00 Section 9

### 11.3 TGP-PROFILE-00 (Payment Profile Schema)

**Relationship:** TBC verifies contracts specified in payment profiles.

**Profile Provides:**

- `contract_address`: Address to verify (Layer 3 input)
- `engine_version`: CoreProver version (determines expected bytecode hash)
- `chain_id`: Blockchain identifier (Layer 3 validation)
- `asset_address`: Payment asset (Layer 3 state check)
- `signature`: Merchant signature (Layer 2 input)

**TBC Uses Profile Data:**

- Layer 2: Verifies profile signature
- Layer 3: Verifies contract at `contract_address`
- Layer 5: Validates policy compliance

**Critical:** TBC MUST NOT trust `contract_address` from profile alone. Layer 3 independently verifies bytecode.

### 11.4 TGP-ENGINE-00 (Settlement Engine)

**Relationship:** TBC verifies contracts; ENGINE-00 defines contract behavior.

**Separation of Concerns:**

- TBC: Pre-transaction verification (Is contract safe?)
- ENGINE-00: Post-transaction execution (How does contract work?)

**TBC Does NOT Verify:**

- Escrow state machine logic (ENGINE-00 responsibility)
- Withdrawal lock semantics (ENGINE-00 responsibility)
- Discount issuance (ENGINE-00 responsibility)
- Receipt minting (ENGINE-00 responsibility)

**TBC DOES Verify:**

- Contract bytecode matches ENGINE-00 specification
- Contract is in valid operational state
- Contract has no admin/owner functions

**Interface:** TBC verifies contract implements ENGINE-00 specification by checking bytecode hash.

### 11.5 TGP-02 (ZK Proof Circuits)

**Relationship:** Defines ZK proof format for Layer 4.

**TGP-02 Defines:**

- ZK circuit specifications
- Proof formats (Groth16, PLONK, STARK)
- Public input schemas
- Verification algorithms

**TBC Uses TGP-02:**

- Layer 4: Verifies ZK proofs using TGP-02 algorithms
- Validates public inputs match expected format
- Enforces proof expiration rules

**Dependency:** TBC Layer 4 implementation MUST comply with TGP-02 proof verification requirements.

### 11.6 TGP-MGMT-00 (Management Protocol)

**Relationship:** Defines operational management of TBC.

**MGMT-00 May Define:**

- TBC configuration management
- RPC provider registration/removal
- Bytecode hash updates (new engine versions)
- Policy rule updates
- Monitoring and alerting

**Out of Scope for TBC-SEC-00:** Operational management details are in MGMT-00.

**Separation:** TBC-SEC-00 defines security verification; MGMT-00 defines how to manage TBC deployment.

——

## 12. Appendix A: Definitions

**Adminless Contract:** Smart contract with no owner, admin, or privileged functions. Cannot be paused, upgraded, or controlled after deployment.

**Bytecode:** Compiled smart contract code deployed to blockchain. Immutable after deployment.

**Consensus Hash:** Bytecode hash value agreed upon by quorum of RPC providers.

**Deterministic Verification:** Verification process producing identical results across all implementations given same inputs.

**Dissenting Provider:** RPC provider returning different bytecode hash than consensus.

**Economic Envelope:** TBC response containing verified contract address and payment parameters. Pre-authorization metadata for transaction construction.

**Fail-Closed:** Security design principle where ambiguous or error conditions default to rejecting transactions (safer failure mode).

**Immutability Verification:** Confirming smart contract cannot be upgraded or modified post-deployment.

**Keccak-256:** Cryptographic hash function used in Ethereum. 256-bit output, collision-resistant. Defined in FIPS 202.

**Onion Security Model:** Multi-layered defense-in-depth architecture where each layer provides independent verification.

**Quorum:** Minimum number (M) of consistent responses required from total providers (N) to achieve consensus.

**RPC Provider:** Remote procedure call service providing access to blockchain data. Examples: Infura, Alchemy, QuickNode.

**State Validation:** Verifying smart contract runtime state variables (e.g., paused, asset, admin).

**Trusted Setup:** Cryptographic ceremony required by some ZK systems (zkSNARKs) to generate proving/verification keys. Potential security risk if ceremony compromised.

**Zero-Knowledge Proof:** Cryptographic proof allowing verification of statement without revealing underlying data. Used in Layer 4 for RPC-independent verification.

——

## 13. Appendix B: Glossary

**CBS (Contract Brokerage Service):** Merchant-side service deploying payment profile smart contracts. Separate from TBC; no involvement in consumer payment flows.

**CoreProver Engine:** Smart contract implementing dual-commitment escrow. Audited template bytecode verified by TBC Layer 3.

**Layer 8:** Economic-control plane signaling layer. TGP protocol layer above traditional network stack.

**Layer 9:** Identity and reputation layer. Handles pseudonymous resolution and ZK proofs.

**Layer 10:** Policy and compliance layer. Enforces jurisdictional rules and risk limits.

**M-of-N Quorum:** Consensus mechanism requiring M providers out of N total to agree on data. Example: 2-of-3 quorum requires 2 providers to return matching bytecode.

**Payment Profile:** Merchant-defined immutable smart contract configuration specifying escrow parameters. Formerly “settlement profile” (v2.0 terminology).

**TBC (Transaction Border Controller):** Non-custodial security gateway validating merchant contracts before consumer funds move.

**TGP (Transaction Gateway Protocol):** Layer 8 economic-control plane signaling protocol for privacy-preserving peer-to-peer commerce.

——

## 14. Appendix C: Security Considerations

### C.1 Threat Model Recap

**Assumed Threats:**

- Malicious merchants deploying fake contracts
- Compromised RPC providers lying about contract bytecode
- Phishing attacks with spoofed payment profiles
- Network attackers intercepting/modifying messages
- Insider attacks (registry compromise, key compromise)

**Out-of-Scope Threats:**

- Blockchain consensus failure (51% attack)
- Cryptographic primitive breaks (Keccak-256 collision, ECDSA forgery)
- TBC core code compromise (operator trust required)
- User device compromise (wallet theft, keyloggers)

### C.2 Security Best Practices for Implementers

**Practice 1: Use Diverse RPC Providers**

RECOMMENDED provider diversity:

- Minimum 3 independent operators
- Different geographic regions (US, EU, Asia)
- Different infrastructure providers (AWS, GCP, Azure, dedicated)
- Mix of commercial (Infura, Alchemy) and community (public Ethereum nodes)

**Practice 2: Secure Key Management**

TBC signing key MUST be protected:

- Store in HSM (Hardware Security Module) or secure enclave
- Implement key rotation procedures (90-day rotation recommended)
- Use separate keys for production vs. testing
- Restrict key access to minimum necessary personnel
- Log all key usage for audit

**Practice 3: Monitor for Anomalies**

Implement monitoring for:

- RPC provider disagreement rates (alert if >5%)
- Layer failure rates per layer (alert if >1%)
- Unusual error patterns (spikes, new error types)
- Economic envelope generation rate (detect DoS attempts)
- Verification latency (detect performance degradation)

**Practice 4: Regularly Update Expected Hashes**

When new CoreProver Engine versions released:

- Obtain audited bytecode from official release
- Compute expected hash using canonical algorithm
- Update TBC configuration via secure process
- Test with new version on testnet before production
- Maintain backward compatibility for transition period

**Practice 5: Implement Rate Limiting**

Protect TBC from abuse:

- Per-IP rate limits (e.g., 100 queries/hour)
- Per-user rate limits (e.g., 1000 queries/day)
- Exponential backoff for repeated failures
- CAPTCHA for suspicious patterns

### C.3 Compliance Considerations

**Jurisdiction-Specific Requirements:**

TBC deployments in regulated jurisdictions may need to implement additional controls:

**GDPR (EU):**

- Minimize logging of personal data
- Provide data deletion procedures
- Document data processing activities
- Implement data retention limits

**FINRA/SEC (US Securities):**

- Enhanced monitoring for sanctioned entities
- Transaction reporting requirements
- Record retention (7 years for securities-related)

**AML/KYC:**

- Sanctions screening integration (Layer 5)
- Suspicious activity reporting
- Enhanced due diligence for high-risk merchants

**Note:** Specific compliance requirements vary by jurisdiction and use case. Consult legal counsel for deployment.

——

## 15. Appendix D: Future Work

### D.1 Planned Enhancements

**Enhancement 1: Light Client Integration**

Replace RPC providers with light client for trust-minimized verification:

- Sync block headers only (no full node required)
- Verify state proofs directly from block headers
- Eliminates RPC trust dependency entirely
- Tradeoff: Higher latency and resource usage

**Target:** Layer 3 alternative to quorum RPC

**Enhancement 2: Proof Caching Service**

Centralized ZK proof cache to reduce Layer 4 latency:

- Third-party service generates proofs for popular contracts
- TBC fetches cached proofs instead of generating on-demand
- Reduces latency from 30s to <1s for cached contracts
- Cache invalidation based on block number

**Target:** Layer 4 performance optimization

**Enhancement 3: Decentralized TBC Network**

Distribute TBC across multiple operators:

- Multiple TBC instances operated by different parties
- Extension queries multiple TBCs (quorum model)
- Reduces single-operator trust assumption
- Increases censorship resistance

**Target:** TBC non-bypassability and decentralization

**Enhancement 4: Hardware Security Module (HSM) Integration**

Move TBC signing key to HSM:

- Signing operations performed in secure hardware
- Private key never leaves HSM
- Audit trail for all signatures
- Compliance with financial security standards

**Target:** TBC key management security

**Enhancement 5: Reputation-Based Provider Weighting**

Dynamic RPC provider selection based on historical performance:

- Track provider accuracy, uptime, latency
- Weight quorum votes based on reputation
- Automatically remove consistently failing providers
- Incentivize provider reliability

**Target:** Layer 3 reliability and efficiency

### D.2 Research Topics

**Topic 1: Post-Quantum Cryptography**

Current signatures (ECDSA) vulnerable to quantum computers:

- Research: Post-quantum signature schemes (e.g., CRYSTALS-Dilithium)
- Impact: Layer 2 signature verification, TBC envelope signing
- Timeline: 5-10 years until quantum threat materialized

**Topic 2: Zero-Knowledge State Proofs**

Efficient ZK proofs of contract state:

- Research: Prove contract state variables directly (not just bytecode)
- Benefits: Stronger Layer 3 guarantees (not just bytecode, but runtime state)
- Challenge: Proof generation complexity and cost

**Topic 3: Confidential Transactions**

Privacy-preserving transaction amounts:

- Research: ZK proofs of transaction amount ranges without revealing exact value
- Integration: Layer 5 policy checks without exposing amounts
- Challenge: Policy compliance vs. privacy tradeoffs

**Topic 4: Cross-Chain Verification**

Extend TBC to verify contracts across multiple chains:

- Research: Multi-chain proof aggregation
- Benefits: Unified verification for cross-chain atomic swaps
- Challenge: Chain-specific verification logic

——

**END OF TGP-TBC-SEC-00 SPECIFICATION**

**Document Version:** 0.1-draft  
**Last Updated:** November 18, 2025  
**Status:** Draft  
**Change Control:** CoreProver Development Team

**Revision History:**

- v0.1-draft (2025-11-18): Initial draft extracted from TGP-00 v3.1

**Acknowledgments:**
This specification was developed by the CoreProver Development Team based on the security model defined in TGP-00 v3.1 Section 3.

**License:** [To be determined]

**Contact:** [To be determined]

——

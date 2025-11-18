# CoreProver Specification

**Version:** 0.2-draft  
**Status:** Draft  
**License:** Commercial - All Rights Reserved  
**Published:** 2025-11-11  
**Updated:** 2025-11-11  
**Author:** Ledger of Earth

——

## Abstract

CoreProver is a blockchain-agnostic escrow and fulfillment middleware that enables peer-to-peer commerce with trustless bilateral settlement. It implements a **dual-commitment escrow architecture** where both buyer and seller must commit value or reputation before settlement can occur.

The system supports two seller commitment models:

1. **Counter-Escrow:** Seller locks funds as collateral
1. **Legal Signature:** Seller provides legally-binding business signature

Settlement is **seller-driven**, with flexible payment profiles configured per transaction. Buyers receive digital goods or fulfillment confirmations before sellers claim payment, creating a trustless peer-to-peer commerce layer.

CoreProver mints immutable receipt NFTs to chain-specific vaults with no admin keys, enabling privacy-preserving buyer interactions via zero-knowledge proofs.

——

## Table of Contents

1. Overview
1. Core Principles
1. Dual-Commitment Model
1. Settlement Flow
1. Seller-Driven Payment Profiles
1. Contract Interfaces
1. Receipt Vault Architecture
1. Privacy & Zero-Knowledge Integration
1. Timeout & Refund Mechanisms
1. Future Work & Extensions
1. License

——

## 1. Overview

CoreProver acts as settlement middleware between buyers and sellers, enforcing bilateral commitment before enabling claims.

```
Control Plane:   Buyer <——> Gateway <——> Seller
Settlement:      Buyer —> CoreProver <—— Seller (dual commitment)
                          ↓
                    Receipt Vault (immutable NFT storage)
```

**Key Innovation:** Claims are only unlocked after **both parties commit**. This creates symmetric risk and eliminates unilateral trust requirements.

——

## 2. Core Principles

### 2.1 Bilateral Commitment

Neither party can claim funds until both have committed something of value:

- **Buyer:** Always commits payment in escrow
- **Seller:** Commits EITHER counter-escrow OR legal signature

### 2.2 Seller-Driven Configuration

The seller defines the terms for each transaction:

- Required counter-escrow amount (if any)
- Whether legal signature is acceptable
- Fulfillment requirements (immediate, tracking, confirmation)
- Claim windows and timeouts
- Payment denomination and accepted assets

### 2.3 Privacy-Preserving Receipts

Receipt NFTs never leave the vault. Buyers interact via ZK-proofs to:

- Signal reorder intent
- Subscribe to updates
- Claim loyalty rewards
- Request support
- Provide referrals

**Result:** Sellers serve customers without storing PII or being liable for data breaches.

——

## 3. Dual-Commitment Model

### 3.1 Commitment Types

#### Buyer Commitment

```
ALWAYS: Payment escrow in required asset
```

#### Seller Commitment (Choose One)

**Option A: Counter-Escrow**

```solidity
function sellerCommitEscrow(bytes32 orderId) external payable;
```

- Locks seller funds as collateral
- Amount defined by seller’s payment profile
- Returned to seller when buyer claims it after settlement

**Option B: Legal Signature**

```solidity
struct LegalSignature {
    bytes signature;           // ECDSA signature
    string businessName;       // “Acme Corp”
    string businessLicense;    // Legal registration ID
    bytes32 documentHash;      // Hash of terms
    uint256 timestamp;
}

function sellerCommitSignature(
    bytes32 orderId, 
    LegalSignature calldata signature
) external;
```

- No funds locked
- Creates legally-binding commitment
- Reputation-based trust model (e.g., established businesses)

### 3.2 State Gates

```
┌─────────────────────────────────────────────────────┐
│ GATE: Claims Locked                                 │
│ - Only one party committed                          │
│ - No claims possible                                │
└─────────────────────────────────────────────────────┘
                        ↓ Both Commit
┌─────────────────────────────────────────────────────┐
│ GATE: Claims Unlocked                               │
│ - Both parties committed                            │
│ - Claims now possible per payment profile rules     │
└─────────────────────────────────────────────────────┘
```

——

## 4. Settlement Flow

### 4.1 General Flow

```
┌────────────────────────────────────────────────────────────────┐
│ PHASE 1: COMMITMENT                                            │
├────────────────────────────────────────────────────────────────┤
│ Step 1: Buyer → CoreProver [payment escrow]                    │
│ Step 2: Seller → CoreProver [counter-escrow OR signature]      │
│         ↓                                                       │
│ State: BOTH_COMMITTED → Claims unlocked                        │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│ PHASE 2: FULFILLMENT & CLAIMS (Order depends on profile)      │
├────────────────────────────────────────────────────────────────┤
│ Digital Goods:                                                 │
│   Step 3: Buyer → claims digital goods/receipt                │
│   Step 4: Seller → claims payment + mints receipt NFT         │
│                                                                │
│ Physical Goods with Shipping:                                 │
│   Step 3: Seller → submits tracking info                      │
│   Step 4: Buyer → claims counter-escrow after delivery        │
│   Step 5: Seller → claims payment + mints receipt NFT         │
│                                                                │
│ Service-Based:                                                │
│   Step 3: Seller → provides service off-chain                 │
│   Step 4: Timed release OR seller claim triggers buyer claim  │
└────────────────────────────────────────────────────────────────┘
```

### 4.2 Key Claim Rules

1. **No buyer acknowledgment required** for seller payment claims

- Prevents buyer holdout attacks
- Seller accepts risk of non-delivery damaging reputation

1. **Buyer receives fulfillment before seller claims** (when applicable)

- Digital goods: Delivered immediately on both committed
- Physical goods: Tracking info submitted before claim
- Services: Timed release or proof-of-completion

1. **Partial fulfillment handled off-chain**

- Contract only holds full amounts
- Refunds come from seller directly
- Reputation system tracks disputes

1. **Counter-escrow claims independent of payment claims**

- Buyer can claim counter-escrow when conditions met
- Seller can claim payment when conditions met
- No interdependency after both committed

——

## 5. Seller-Driven Payment Profiles

### 5.1 Profile Structure

Each transaction references a payment profile defining:

```solidity
struct PaymentProfile {
    // Commitment requirements
    SellerCommitmentType requiredCommitmentType;
    uint256 counterEscrowAmount;      // 0 if signature allowed
    
    // Timing
    uint256 commitmentWindow;         // Seconds for both to commit
    uint256 claimWindow;              // Seconds to claim after committed
    
    // Fulfillment rules
    FulfillmentType fulfillmentType;  // DIGITAL | SHIPPING | SERVICE | IMMEDIATE
    bool requiresTracking;            // True for physical goods
    bool allowsTimedRelease;          // Auto-release after N seconds
    uint256 timedReleaseDelay;        // Delay before auto-release
    
    // Asset requirements
    address paymentToken;             // ERC20 or address(0) for native
    uint256 priceInUSD;               // For oracle-based conversion
    bool acceptsMultipleAssets;       // Allow buyer to pay in different token
}

enum FulfillmentType {
    IMMEDIATE,      // Buyer gets goods when both committed
    DIGITAL,        // Buyer claims digital delivery
    SHIPPING,       // Seller provides tracking, buyer receives
    SERVICE,        // Time-based or proof-based
    CUSTOM          // Custom fulfillment logic
}
```

### 5.2 Example Profiles

**Profile 1: Digital Download (SaaS License)**

```javascript
{
  requiredCommitmentType: LEGAL_SIGNATURE,
  counterEscrowAmount: 0,
  commitmentWindow: 3600,        // 1 hour
  claimWindow: 86400 * 7,        // 7 days
  fulfillmentType: IMMEDIATE,
  requiresTracking: false,
  allowsTimedRelease: false,
  paymentToken: USDC_ADDRESS,
  priceInUSD: 99_00,             // $99.00
  acceptsMultipleAssets: true
}
```

**Profile 2: High-Value Electronics**

```javascript
{
  requiredCommitmentType: COUNTER_ESCROW,
  counterEscrowAmount: parseEther(“1000”),  // Match payment
  commitmentWindow: 86400,                  // 24 hours
  claimWindow: 86400 * 7,                   // 7 days
  fulfillmentType: SHIPPING,
  requiresTracking: true,
  allowsTimedRelease: true,
  timedReleaseDelay: 86400 * 3,             // 3 days after tracking
  paymentToken: address(0),                 // Native token
  priceInUSD: 0,                            // Direct pricing
  acceptsMultipleAssets: false
}
```

**Profile 3: Pizza Delivery**

```javascript
{
  requiredCommitmentType: LEGAL_SIGNATURE,
  counterEscrowAmount: 0,
  commitmentWindow: 1800,         // 30 minutes
  claimWindow: 7200,              // 2 hours
  fulfillmentType: SERVICE,
  requiresTracking: false,
  allowsTimedRelease: true,
  timedReleaseDelay: 3600,        // 1 hour auto-release
  paymentToken: USDC_ADDRESS,
  priceInUSD: 30_00,
  acceptsMultipleAssets: true
}
```

——

## 6. Contract Interfaces

### 6.1 ICoreProverEscrow.sol

```solidity
// SPDX-License-Identifier: COMMERCIAL
pragma solidity ^0.8.20;

interface ICoreProverEscrow {
    enum EscrowState {
        NONE,
        BUYER_COMMITTED,
        SELLER_COMMITTED,
        BOTH_COMMITTED,        // Claims unlocked
        SELLER_CLAIMED,
        BUYER_CLAIMED,
        BOTH_CLAIMED,
        DISPUTED,
        EXPIRED
    }

    enum SellerCommitmentType {
        COUNTER_ESCROW,
        LEGAL_SIGNATURE,
        EITHER              // Seller chooses
    }

    enum FulfillmentType {
        IMMEDIATE,
        DIGITAL,
        SHIPPING,
        SERVICE,
        CUSTOM
    }

    struct PaymentProfile {
        SellerCommitmentType requiredCommitmentType;
        uint256 counterEscrowAmount;
        uint256 commitmentWindow;
        uint256 claimWindow;
        FulfillmentType fulfillmentType;
        bool requiresTracking;
        bool allowsTimedRelease;
        uint256 timedReleaseDelay;
        address paymentToken;
        uint256 priceInUSD;
        bool acceptsMultipleAssets;
    }

    struct Escrow {
        address buyer;
        address seller;
        uint256 buyerAmount;
        uint256 sellerAmount;
        uint256 createdAt;
        uint256 bothCommittedAt;
        uint256 commitmentDeadline;
        uint256 claimDeadline;
        EscrowState state;
        SellerCommitmentType actualCommitmentType;
        bytes32 legalSignatureHash;
        bytes32 paymentProfileHash;
        bytes fulfillmentData;     // Tracking info, download links, etc.
        bool buyerClaimed;
        bool sellerClaimed;
    }

    struct LegalSignature {
        bytes signature;
        string businessName;
        string businessLicense;
        bytes32 documentHash;
        uint256 timestamp;
    }

    // === EVENTS ===
    
    event EscrowCreated(
        bytes32 indexed orderId,
        address indexed buyer,
        address indexed seller,
        uint256 buyerAmount,
        bytes32 profileHash
    );

    event BothCommitted(
        bytes32 indexed orderId,
        uint256 timestamp
    );

    event FulfillmentDataSubmitted(
        bytes32 indexed orderId,
        bytes32 dataHash
    );

    event SellerClaimed(
        bytes32 indexed orderId,
        uint256 amount,
        uint256 receiptTokenId
    );

    event BuyerClaimed(
        bytes32 indexed orderId,
        uint256 counterEscrowAmount
    );

    event TimedReleaseTriggered(
        bytes32 indexed orderId,
        uint256 timestamp
    );

    // === COMMITMENT PHASE ===

    /// @notice Buyer creates escrow with payment
    function createEscrow(
        bytes32 orderId,
        address seller,
        PaymentProfile calldata profile,
        bytes calldata metadata
    ) external payable returns (bytes32);

    /// @notice Seller commits via counter-escrow
    function sellerCommitEscrow(
        bytes32 orderId
    ) external payable;

    /// @notice Seller commits via legal signature
    function sellerCommitSignature(
        bytes32 orderId,
        LegalSignature calldata signature
    ) external;

    // === FULFILLMENT PHASE ===

    /// @notice Seller submits fulfillment data (tracking, download link, etc.)
    function submitFulfillmentData(
        bytes32 orderId,
        bytes calldata fulfillmentData
    ) external;

    // === CLAIM PHASE ===

    /// @notice Seller claims payment + mints receipt NFT
    function sellerClaimPayment(
        bytes32 orderId
    ) external returns (uint256 receiptTokenId);

    /// @notice Buyer claims counter-escrow (if applicable)
    function buyerClaimCounterEscrow(
        bytes32 orderId
    ) external returns (uint256);

    /// @notice Trigger timed release (anyone can call after delay)
    function triggerTimedRelease(
        bytes32 orderId
    ) external;

    // === TIMEOUT & REFUNDS ===

    /// @notice Refund buyer if seller never commits
    function refundBuyerTimeout(
        bytes32 orderId
    ) external;

    /// @notice Refund seller if buyer never commits (edge case)
    function refundSellerTimeout(
        bytes32 orderId
    ) external;

    // === VIEW FUNCTIONS ===

    function getEscrow(bytes32 orderId) 
        external view returns (Escrow memory);

    function canSellerClaim(bytes32 orderId) 
        external view returns (bool);

    function canBuyerClaim(bytes32 orderId) 
        external view returns (bool);

    function isTimedReleaseReady(bytes32 orderId) 
        external view returns (bool);
}
```

### 6.2 ICoreReceiptVault.sol

```solidity
interface ICoreReceiptVault {
    struct Receipt {
        bytes32 orderId;
        address seller;
        address buyer;              // Can be ephemeral ZK address
        uint256 timestamp;
        bytes32 fulfillmentDataHash;
        bytes32 legalSignatureHash; // Non-zero if signature used
        bool wasCounterEscrowed;
        string metadataURI;         // IPFS link to order details
    }

    event ReceiptMinted(
        uint256 indexed tokenId,
        bytes32 indexed orderId,
        address indexed seller,
        address buyer
    );

    /// @notice Mint receipt NFT (only callable by CoreProver)
    function mintReceipt(
        bytes32 orderId,
        address buyer,
        address seller,
        bytes32 fulfillmentDataHash,
        bytes32 legalSignatureHash,
        bool wasCounterEscrowed,
        string calldata metadataURI
    ) external returns (uint256 tokenId);

    /// @notice Get receipt details
    function getReceipt(uint256 tokenId) 
        external view returns (Receipt memory);

    /// @notice Query receipts by seller (for reputation)
    function getReceiptsBySeller(address seller) 
        external view returns (uint256[] memory);

    /// @notice ZK-proof verification for buyer interactions
    function verifyBuyerOwnership(
        uint256 tokenId,
        bytes calldata zkProof
    ) external view returns (bool);
}
```

——

## 7. Receipt Vault Architecture

### 7.1 Immutable Vault Design

Each blockchain has a dedicated `ReceiptVault` contract:

- **No admin keys** or upgrade functions
- **Deterministic address** per chain
- **Non-transferable** NFTs (soulbound to vault)
- **Permanent storage** of transaction receipts

### 7.2 Privacy-Preserving Interactions

Buyers interact with their receipts using ZK-proofs without revealing wallet identity:

```
┌─────────────────────────────────────────────────────┐
│ Buyer Wallet (Public)                               │
│   ├─ Contains payment tokens                        │
│   └─ No receipt NFTs visible                        │
└─────────────────────────────────────────────────────┘
                    ↓ ZK Proof
┌─────────────────────────────────────────────────────┐
│ Receipt Vault (On-Chain)                            │
│   ├─ Stores all receipt NFTs                        │
│   ├─ Verifies ZK proofs for interactions            │
│   └─ No direct NFT transfers possible               │
└─────────────────────────────────────────────────────┘
```

### 7.3 Use Cases for ZK Interactions

**Reorder Intent:**

```solidity
// Buyer proves ownership without revealing wallet
function signalReorder(
    uint256 receiptTokenId,
    bytes calldata zkProof
) external;
```

**Subscription Opt-In:**

```solidity
function subscribeToSeller(
    uint256 receiptTokenId,
    bytes calldata zkProof,
    bytes calldata encryptedContactInfo
) external;
```

**Loyalty Claims:**

```solidity
function claimLoyaltyReward(
    uint256[] calldata receiptTokenIds,  // Prove multiple purchases
    bytes calldata zkProof
) external returns (uint256 rewardAmount);
```

**Referral Fees:**

```solidity
function claimReferralFee(
    uint256 referrerReceiptId,
    uint256 referredReceiptId,
    bytes calldata zkProof
) external;
```

**Benefits for Sellers:**

- Maintain customer relationships without storing PII
- No liability for data breaches
- GDPR/CCPA compliant by design
- Automated reengagement without spam
- Verifiable customer history without knowing identities

——

## 8. Privacy & Zero-Knowledge Integration

### 8.1 Ephemeral Buyer Addresses

Buyers can use ZK-generated ephemeral addresses for each purchase:

```javascript
// Generate ephemeral address for privacy
const ephemeralKeypair = await generateZKEphemeralKey(
  buyerMasterSecret,
  orderId
);

// Use ephemeral address in escrow
await coreProver.createEscrow(
  orderId,
  seller,
  profile,
  metadata,
  { from: ephemeralKeypair.address }
);

// Receipt NFT minted to ephemeral address
// Buyer proves ownership via ZK proof linking to master secret
```

### 8.2 ZK Proof Circuits (Future)

```circom
// Prove receipt ownership without revealing buyer identity
template ReceiptOwnership() {
    signal input masterSecret;        // Private
    signal input receiptId;           // Public
    signal input ephemeralAddress;    // Public (stored in receipt)
    
    // Prove ephemeralAddress = hash(masterSecret, receiptId)
    signal derivedAddress;
    derivedAddress <== Poseidon([masterSecret, receiptId]);
    derivedAddress === ephemeralAddress;
}
```

——

## 9. Timeout & Refund Mechanisms

### 9.1 Commitment Timeout

If either party fails to commit within the commitment window:

```solidity
function refundBuyerTimeout(bytes32 orderId) external {
    Escrow storage escrow = escrows[orderId];
    
    require(
        block.timestamp > escrow.commitmentDeadline,
        “Commitment window active”
    );
    
    require(
        escrow.state == EscrowState.BUYER_COMMITTED,
        “Invalid state”
    );
    
    // Seller never committed → refund buyer
    escrow.state = EscrowState.EXPIRED;
    payable(escrow.buyer).transfer(escrow.buyerAmount);
}
```

### 9.2 Timed Release

For payment profiles with `allowsTimedRelease = true`:

```solidity
function triggerTimedRelease(bytes32 orderId) external {
    Escrow storage escrow = escrows[orderId];
    
    require(
        escrow.state == EscrowState.BOTH_COMMITTED,
        “Not ready for release”
    );
    
    require(
        block.timestamp >= escrow.bothCommittedAt + profile.timedReleaseDelay,
        “Release delay not elapsed”
    );
    
    // Auto-release payment to seller
    // Auto-release counter-escrow to buyer
    // Mint receipt NFT
    
    emit TimedReleaseTriggered(orderId, block.timestamp);
}
```

**Use Case:** Pizza delivery auto-releases payment 1 hour after both committed, preventing buyer holdout.

### 9.3 Claim Window Expiry

Unclaimed funds after claim window:

- Remain in escrow indefinitely
- No forced redistribution (respect party autonomy)
- Dispute resolver can intervene if needed
- Future upgrade: DAO governance for expired funds

——

## 10. Future Work & Extensions

### 10.1 Advanced Features

**Multi-Party Escrow:**

- Platform fees
- Referral commissions
- Split payments

**Dispute Resolution:**

- On-chain arbitration voting
- Evidence submission
- Automated penalty distribution

**Cross-Chain Settlement:**

- Bridge-free settlement using oracle price feeds
- Buyer pays on Chain A, seller receives on Chain B
- Receipt NFT on buyer’s preferred chain

**Programmable Claims:**

- Milestone-based releases
- Conditional fulfillment logic
- Oracle-triggered settlements

### 10.2 Integration Opportunities

- **TGP (Transaction Gateway Protocol):** Control plane for order routing
- **ZK-P2P:** Privacy-preserving buyer-seller messaging
- **DeFi Integration:** Staked counter-escrow earning yield
- **Reputation Oracles:** On-chain credit scoring for signature commitments

——

## 11. License

This specification and all associated smart contracts are provided under a **commercial license with 48-month exclusivity period**.

**All rights reserved by Ledger of Earth.**

No part of this specification may be reproduced, distributed, or used in commercial applications without explicit written permission from Ledger of Earth.

For licensing inquiries: [contact information]

——

**End of Specification**
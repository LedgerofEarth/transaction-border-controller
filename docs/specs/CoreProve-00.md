📘 CoreProve-00 v0.9 — System Specification

A Trust-Minimized Framework for Dual-Escrow Settlement and Privacy-Preserving Receipt Anchoring

Version: 0.9 Draft
Status: Internal
Author: Ledger of Earth
Date: 2025-11-24

⸻

0. Abstract

CoreProve is a trust-minimized framework for creating, deploying, and managing dual-escrow smart contracts to enable secure, private exchange between people or their agents.

CoreProve invents Receipt Anchoring.
Receipt Anchoring is a privacy-preserving mechanism that allows users to prove that a transaction occurred without exposing their wallet, balances, or identity.

In Receipt Anchoring, receipts are minted into an immutable, adminless vault and referenced via zero-knowledge proofs rather than wallet ownership. This resolves the fundamental doxxing flaw of traditional on-chain transactions and restores the privacy expectations of real-world commerce.

⸻

1. System Overview

1.1 Purpose

CoreProve provides the on-chain infrastructure necessary to facilitate:
	•	secure dual-escrow between buyers and sellers
	•	privacy-preserving settlement with ZK proofs
	•	wallet-unlinkable receipts
	•	deterministic settlement flows
	•	merchant-specific immutable business logic
	•	safe agent-driven or autonomous commerce

It forms the settlement layer that higher-level protocols (like TGP/TBC) route into.

1.2 Design Principles
	•	Immutability: merchant contracts are not upgradeable.
	•	Least Trust: no party can seize or redirect funds.
	•	Privacy: settlement occurs without exposing user wallets.
	•	Determinism: dual commits ensure predictable settlement.
	•	Safety: all escrows have TTLs to prevent lock-in.
	•	Sovereignty: receipts anchor verifiable commerce without surveillance.

⸻

2. Components

2.1 Settlement Contract (per merchant)

Defines the merchant’s escrow and settlement rules.
Features:
	•	buyerCommit (escrow deposit)
	•	sellerCommit (settlement + payout)
	•	dual ZK proof inputs
	•	dual nullifiers (buyer + seller)
	•	TTL enforcement
	•	multi-asset support (ERC-20 + native)
	•	fee routing
	•	receipt event emission

Immutable after deployment.

⸻

2.2 Merchant Contract Factory

Responsible for:
	•	registering certified templates (by version)
	•	stability flags (stable / experimental / deprecated)
	•	deterministic CREATE2 deployments
	•	constructor parameter injection
	•	public template metadata

Factory does not control merchant contracts after deployment.

⸻

2.3 ReceiptVault (epoch-based)

A standalone, adminless vault that:
	•	mints receipt NFTs
	•	permanently stores them (non-transferable)
	•	supports ZK proof referencing
	•	provides a stable anchoring surface for receipts
	•	rotates annually or biannually (e.g., 2025-26, 2027-28)

Receipts minted into the vault do not link to buyer or seller wallets.

⸻

3. Lifecycle

3.1 BuyerCommit (Escrow Deposit)

Triggered by buyer or buyer agent.

Inputs include:
	•	asset + amount
	•	orderId
	•	buyer ZK proof
	•	public signals:
	•	pkHash_buyer
	•	nullifier_buyer
	•	timestamp
	•	amount

Validates:
	•	merchant active
	•	nullifier unused
	•	timestamp fresh
	•	ZK signals match order + amount

Action:
	•	funds deposited into escrow
	•	buyerCommit stored
	•	buyer nullifier marked used
	•	event emitted

⸻

3.2 SellerCommit (Settlement + Withdrawal)

Triggered only by merchant admin.

Inputs include:
	•	orderId
	•	seller ZK proof
	•	public signals:
	•	pkHash_seller
	•	nullifier_seller
	•	timestamp
	•	orderHash

Validates:
	•	escrow exists
	•	commit not expired
	•	seller nullifier unused
	•	orderHash match
	•	merchant active

Action:
	•	compute fees
	•	payout: buyer → merchant recipients
	•	delete escrow record
	•	mark seller nullifier used
	•	emit SettlementExecuted
	•	trigger ReceiptVault minting

This is the only withdrawal path.

⸻

3.3 TTL and Expiration

Each escrow has a TTL:

expiration = buyerTimestamp + ttlSeconds

If sellerCommit does not occur:
	•	buyer may call buyerCancelExpiredCommit()
	•	escrow refunded
	•	receipt not minted
	•	buyer nullifier remains used (ZK integrity)

TTL prevents locked funds and stale commitments.

⸻

4. Zero-Knowledge Proof Model

4.1 Buyer ZK Input

Public signals:

[ pkHash_buyer, nullifier_buyer, ts_buyer, amount ]

Contract enforces:
	•	pkHash_buyer matches buyer ephemeral key
	•	nullifier unused
	•	timestamp within freshness bound
	•	amount equal to payment

⸻

4.2 Seller ZK Input

Public signals:

[ pkHash_seller, nullifier_seller, ts_seller, orderHash ]

Contract enforces:
	•	nullifier unused
	•	ts_seller fresh
	•	orderHash = keccak256(orderId)
	•	pkHash_seller matches ephemeral seller identity

⸻

4.3 Nullifier Rules

Each nullifier (buyer and seller) is:
	•	single-use
	•	permanently burned after use
	•	prevents replay or state modification

⸻

5. Multi-Asset Escrow

5.1 Supported Assets
	•	Any ERC-20 token
	•	Native assets (ETH, PLS)

All transfers use safe wrappers.

5.2 Fee Operations

Fees include:
	•	TBC fee
	•	ZK relay fee
	•	merchant net

All fee parameters are template-defined and immutable per merchant.

⸻

6. TTL Safety Model

6.1 Deployment-Time Configuration

Merchant chooses:

ttlSeconds

This value is immutable.

6.2 Safety Properties

TTL ensures:
	•	sellers cannot delay indefinitely
	•	buyers cannot be trapped in escrow
	•	stale orders do not persist
	•	reconcilers and auditors can bound execution windows

⸻

7. Merchant Deployment and Administration

7.1 Deployment

Merchants deploy via Factory:
	•	select template version
	•	verify stability flag
	•	provide constructor args
	•	CREATE2 deterministic address generated

7.2 Admin Capabilities

Merchant admin can:
	•	activate / deactivate merchant
	•	execute sellerCommit

Merchant admin cannot:
	•	seize funds
	•	modify logic
	•	upgrade contract
	•	alter TTL
	•	change fee logic
	•	alter escrow state directly

⸻

8. Security Properties

8.1 Immutability

Template-based deployment ensures:
	•	no upgradability
	•	no privileged escape paths
	•	reproducible logic across merchants

8.2 Replay Protection

Dual-nullifier replay protection ensures:
	•	buyers cannot double-commit
	•	sellers cannot double-settle
	•	escrow states cannot be mutated after completion

8.3 Minimal Attack Surface

State includes only:
	•	buyer escrow record
	•	buyer nullifier map
	•	seller nullifier map
	•	merchant active flag

No attack-surface for role escalation.

⸻

9. Agent & Protocol Integration

9.1 TGP Mapping

The Transaction Gateway Protocol maps:
	•	TGP_COMMIT → buyerCommit
	•	TGP_SETTLE → sellerCommit
	•	TGP_RECEIPT → receipt event path

9.2 TBC Role

The Transaction Border Controller:
	•	relays ZK proofs
	•	optionally pays gas
	•	reimburses gas via settlement flow
	•	consumes settlement + receipt events
	•	orchestrates multi-chain routing

⸻

10. Privacy Guarantees

10.1 Ephemeral Identity

Buyers and sellers use ephemeral keys proven via ZK.
No wallet addresses ever appear on-chain.

10.2 Receipt Anchoring

Receipts:
	•	minted into a vault, not to user wallets
	•	store no identity information
	•	represent immutable proof-of-exchange
	•	are referenced via ZK, not token ownership

This provides wallet-unlinkable proof of commerce.

10.3 Selective Disclosure

Users can prove:
	•	a transaction occurred
	•	a specific order was fulfilled
	•	a receipt exists

without linking actions across multiple receipts.

⸻

11. Summary

CoreProve provides:
	•	secure, trust-minimized dual escrow
	•	privacy-preserving ZK settlement
	•	multi-asset support
	•	merchant-safe immutable logic
	•	revocation-free receipt anchoring
	•	agent-compatible flows
	•	wallet-unlinkable proofs of commerce

Receipt Anchoring restores privacy to blockchain commerce by allowing proof-of-exchange without identity exposure.

⸻

End of Specification — CoreProve-00 v0.9


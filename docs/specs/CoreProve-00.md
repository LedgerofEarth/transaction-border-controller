📙 CoreProve-00 v1.0 — Merchant Settlement Contract Specification

Status: Draft
Author: Ledger of Earth
Scope: On-chain settlement contract definition
Audience: Smart contract developers, auditors, TGP implementers, gateway operators

⸻

0. Abstract

CoreProve defines a non-custodial, trust-minimized settlement contract system that enables untrusted parties to transact safely using standardized protocols and payment gateways. This system allows merchants to accept blockchain payments in a compliant, auditable, and secure manner, while shielding wallet addresses, financial history, and sensitive metadata of both parties.

A CoreProve Settlement Contract provides a configurable, merchant-specific template that operates as a constrained custodian: it may temporarily hold funds but never contains admin keys, backdoors, upgrade paths, or discretionary control surfaces. All state transitions follow the contract’s predefined logic and the verb-level semantics defined by the Transaction Gateway Protocol (TGP).

This specification defines:
	•	the escrow model
	•	the state machine supporting TGP verbs (COMMIT, ACCEPT, CLAIM, WITHDRAW)
	•	the Settlement Envelope parameters consumed by the Gateway
	•	the optional Receipt NFT
	•	ZK attestation hooks for shielded flows
	•	timeout and L6 WITHDRAW eligibility rules
	•	mandatory safety invariants for deterministic settlement

CoreProve-00 defines the merchant’s on-chain execution environment.
TGP-00 defines the off-chain transaction model and Economic Envelopes.
TGP-CP-00 and TGP-EXT-00 define the client profiles and extension runtime.

⸻

1. Scope

CoreProve-00 defines:
	•	Settlement Contract interface
	•	Escrow lifecycle and state machine
	•	Mapping of TGP verbs to on-chain entry points
	•	WITHDRAW eligibility rules (aligned with TGP L6)
	•	Receipt NFT minting and settlement metadata
	•	Fee and payout rules
	•	Timeout and non-cooperative termination rules
	•	Deterministic settlement behavior

CoreProve-00 does not define:
	•	Off-chain TGP message structures (TGP-00)
	•	Client behavior or user agent logic (TGP-CP-00)
	•	Browser extension runtime (TGP-EXT-00)
	•	ZK circuit definitions (CoreProve-ZK-00)
	•	Merchant UX or merchant enrollment processes

⸻

2. Architecture Overview

A CoreProve Settlement Contract is a merchant-deployed, non-upgradeable contract whose purpose is to escrow funds, enforce payment logic, and expose deterministic settlement outcomes to the Gateway.

The architecture consists of:
	1.	Settlement Contract (this specification)
	2.	TGP Gateway (TBC) observing contract state
	3.	Client/Wallet executing Economic Envelopes
	4.	Optional Receipt NFT confirming settlement

Client → QUERY
Gateway → ACK (Economic Envelope)
Client → executes tx (commit/accept/claim/withdraw)
Contract → updates escrow state
Gateway → SETTLE

The contract itself never sends messages. The Gateway observes on-chain state.

⸻

3. Escrow Model

Each escrow instance is represented by an immutable struct:

Escrow {
    buyer: address
    seller: address
    amount: uint256
    state: EscrowState
    created_at: uint64
    ttl: uint64
}

3.1 Escrow States

PENDING      — buyer has committed funds
ACCEPTED     — seller counter-accepts (signature or action)
FULFILLED    — fulfillment evidence recorded (optional)
CLAIMED      — seller has claimed payout
REFUNDED     — buyer withdrew after timeout
RELEASED     — cooperative release
REVERTED     — contract failure (never discretionary)

All state transitions are append-only and deterministic.

⸻

4. Mapping TGP Verbs to Contract Functions

TGP Verb	Contract Function	Description
COMMIT	commit()	Buyer deposits funds into escrow
ACCEPT	accept()	Seller confirms participation
CLAIM	claim()	Seller claims after fulfillment
WITHDRAW	withdraw()	Buyer or seller retrieves funds based on L6 rules

4.1 COMMIT — Buyer deposits funds

Rules:
	•	MUST include exact value specified by Economic Envelope
	•	MUST initialize escrow state = PENDING
	•	MUST record timestamps

4.2 ACCEPT — Seller confirmation

Rules:
	•	MAY be a zero-value transaction
	•	MUST validate seller identity
	•	MUST transition state = ACCEPTED

4.3 CLAIM — Seller payout

Rules:
	•	MUST ensure ACCEPTED or FULFILLED
	•	MUST pay seller the net amount minus fees
	•	MUST mint Receipt NFT if enabled
	•	MUST finalize state = CLAIMED

4.4 WITHDRAW — Timeout or cooperative release

WITHDRAW is valid only if:
	•	buyer timeout expired (PENDING → REFUNDED)
	•	seller timeout expired (ACCEPTED but no claim)
	•	cooperative release (both consent)
	•	contract detects a non-recoverable failure (REVERTED)

The contract MUST implement:
	•	strict L6 eligibility checks
	•	no override by external authority
	•	no admin emergency withdrawal

⸻

5. Timeout Logic (L6 Eligibility)

Each escrow instance carries a TTL (ttl).

Timeout logic:
	•	If now > created_at + ttl and escrow not CLAIMED → WITHDRAW allowed
	•	If fulfillment module is configured, FULFILLED must occur before TTL
	•	Timeouts are strict; no manual override

L6 rules ensure deterministic WITHDRAW and prevent stuck funds.

⸻

6. Receipt NFT (Optional)

If enabled, the contract MUST mint a non-transferable NFT containing:
	•	escrow ID
	•	amount
	•	buyer/seller anonymized references
	•	settlement timestamp
	•	settlement result

NFTs MUST be:
	•	permanent
	•	non-burnable by third parties
	•	non-upgradeable

Purpose:
Proof of settlement, useful for audits, refunds, accounting, or privacy-preserving attestations.

⸻

7. Fees

A CoreProve Contract MAY define:
	•	merchant fee share
	•	TBC fee (fixed or percent)
	•	gas reconciliation reserve

Rules:
	•	Fees MUST be deterministic
	•	No external entity MAY claim arbitrary fees
	•	Fee formulas MUST be configured at deployment
	•	No owner-controlled fee parameters post-deployment

⸻

8. ZK Hooks (Shielded Mode)

CoreProve supports TGP mode = SHIELDED.

ZK hooks MAY include:
	•	verifyProof(bytes proof)
	•	nullifier replay checks
	•	proof-based ACCEPT or CLAIM
	•	buyer or seller selective disclosure

This specification defines the interface, not the circuits.

Circuits are defined in CoreProve-ZK-00.

⸻

9. Deterministic Behavior

A compliant settlement contract MUST exhibit:
	•	deterministic state transitions
	•	no randomness
	•	no oracle dependencies
	•	no privileged roles
	•	no ability to pause, upgrade, or override logic

All settlement outcomes MUST be derivable solely from:
	•	the contract’s public state
	•	the contract’s predefined rules
	•	TGP-issued Economic Envelopes

⸻

10. Safety Invariants

The contract MUST ensure:
	1.	No admin key exists
	2.	Funds cannot be seized
	3.	Funds cannot be redirected except by CLAIM or WITHDRAW
	4.	Escrow cannot be erased or overwritten
	5.	Timeouts cannot be bypassed
	6.	WITHDRAW eligibility must follow L6 rules exactly
	7.	Contract must be non-upgradeable
	8.	All ETH/token transfers MUST be checked for success

⸻

11. Minimal Examples

11.1 COMMIT → ACCEPT → CLAIM
	1.	Buyer sends COMMIT (escrow created, PENDING)
	2.	Seller sends ACCEPT (state = ACCEPTED)
	3.	Seller sends CLAIM (funds to seller, state = CLAIMED)
	4.	Gateway emits SETTLE

11.2 Buyer Timeout Refund
	1.	Buyer COMMIT
	2.	Seller never ACCEPTS
	3.	TTL expires
	4.	Buyer WITHDRAW → REFUNDED
	5.	Gateway emits SETTLE

11.3 Cooperative Release
	1.	Buyer COMMIT
	2.	Seller ACCEPTS
	3.	Both mutually sign a release
	4.	WITHDRAW → RELEASED
	5.	Gateway emits SETTLE

⸻

End of CoreProve-00 v1.0

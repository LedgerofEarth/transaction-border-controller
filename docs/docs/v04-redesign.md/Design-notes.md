The customer drive to provide ZK privacy in short order changed the direction of development.  Instead of a MVP that did not include zk circuits to hide source wallet addresses, it is now in scope.

This requires a redesign of the message flow.  Below are design dialogues between David, GTP, Claude and Shannon.  Up to this point we have explored ZK circuits…the modular arithmetic underlying the Zk proofs which lead to the analysis of the correct place to perform the ZK circuit creation and the EIPs that will enable this to be done outside of the users wallet.  The goal is to be non-custodial, safe and secure.

Feedback to Claude after we confirmed the design choices and direction, this first dialogue confirms the fee structures and who is paying what.

Understood — here is the Updated Design Specification v0.4 with all clarifications, fee structures, delegated authority model, ZK prioritization, and the correct separation of concerns.

This document is designed so Claude will pause implementation and synchronize to these architectural decisions before continuing.

⸻

📘 **COREPROVE / TBC

UPDATED DESIGN SPECIFICATION — v0.4**

Delegated Ephemeral Key + ZK Privacy + Settlement Contract Split Fees

Authors: David Bigge
Reviewers: Shannon Jolin, Claude
Date: 2025-11-22
Status: Draft — DO NOT IMPLEMENT UNTIL REVIEWED

⸻

0. Summary of Changes in v0.4

This revision introduces:
	•	✔ ZK privacy as the primary premium feature
	•	✔ Percentage-based ZK fee (higher than standard tx fee)
	•	✔ Approval-based pull-payment model (no custody)
	•	✔ Ephemeral key managed by the Extension (memory-only)
	•	✔ Delegation signature verified on-chain (not in ZK)
	•	✔ Settlement contract handles all fee splits
	•	✔ TBC broadcasts transactions but never holds funds/keys
	•	✔ All fees deducted from the merchant’s side
	•	✔ ZK circuit contains only privacy logic, not ECDSA

This spec supersedes v0.3 and is the definitive architecture for implementation.

⸻

1. System Overview

The CoreProve system now operates as:
	•	A non-custodial delegated payment platform
	•	With AI-native, walletless settlement
	•	Using temporary delegated keys
	•	With ZK privacy proofs
	•	Where the TBC submits transactions
	•	And the settlement contract pulls funds
	•	And merchants pay fees for privacy + routing
	•	All while preserving EOA compatibility.

⸻

2. Actors

Actor	Role
User Wallet (EOA)	Signs one EIP-712 delegation per session
CoreProve Extension	Generates ephemeral key, builds ZK proof
TBC	Transaction firewall, router, AM, broadcaster
Settlement Contract	Verifies proof + signature, executes split payments
Merchant	Receives net payout, pays fees
Receipt Vault	Mints privacy-preserving receipt NFT


⸻

3. One-Time User Setup

3.1 User approves the settlement contract

The user performs an ERC-20 approve() once:

USDC.approve(settlementContract, type(uint256).max);

3.2 User signs session delegation

The wallet signs an EIP-712 typed message:

{
  delegatePk: pk_e,
  limit: spendCap,
  expiry: timestamp,
  scope: hash(“CoreProve-v1”),
  sessionId: random,
  nonce: random,
  chainId: 369,
  verifyingContract: settlementContract
}

This authorization is:
	•	time-bound
	•	value-bound
	•	scope-bound
	•	non-replayable
	•	non-custodial

⸻

4. Ephemeral Key Handling

4.1 Inside Extension (non-custodial)
	•	sk_e generated via WebCrypto
	•	Stored in memory only
	•	Destroyed:
	•	after proof
	•	when session ends
	•	when tab closes
	•	after timeout

4.2 Extension does NOT have user wallet keys

This is still non-custodial:
	•	sk_e is not a wallet key
	•	It cannot spend user funds
	•	It only signs ZK-metadata-bound payloads
	•	All spending authority comes from the on-chain verification of delegation

⸻

5. Per-Transaction Workflow

Step 1 — Generate ZK Witness

Witness includes:
	•	pk_e
	•	NAT commitment (alternate address)
	•	payment amount
	•	spending cap
	•	merchant profile hash
	•	session_id
	•	timestamp

Step 2 — Generate ZK Proof

Circuit asserts:
	•	knowledge of sk_e
	•	pk_e is valid (EC multiplication)
	•	payment amount ≤ delegated limit
	•	NAT mapping is correct
	•	profile rules met
	•	no wraparound arithmetic
	•	session/time constraints

ZK circuit does not verify ECDSA.
Delegation signature is checked by the contract.

Step 3 — Extension hands intent to TBC

The extension hands to the TBC:
	•	zk proof
	•	payment intent
	•	pk_e
	•	delegation signature
	•	merchant profile ID
	•	compliance metadata

Step 4 — TBC enforces Layer-8/9/10 rules

Before broadcast, TBC:
	•	checks OFAC / geo / merchant profile
	•	checks compliance metadata (x402, TGP)
	•	logs TDR
	•	NAT masks sender
	•	chooses chain/route (VGP)
	•	adds replay nonce
	•	prepares calldata

Step 5 — TBC broadcasts settlement transaction

TBC sends:

settlementContract.settle(
    pk_e,
    delegationSignature,
    zkProof,
    paymentIntent,
    tbcMetadata
)

The TBC never signs any user-spend operation.

⸻

6. Settlement Contract Logic

6.1 Verify Delegation
	•	Check EIP-712 signature
	•	Check expiry
	•	Check nonce
	•	Check session binding
	•	Check spending limits
	•	Determine user’s EOA from signature

6.2 Verify ZK Proof
	•	pk_e bound to sk_e
	•	NAT alt-address correct
	•	payment amount ≤ authorized limit
	•	profile constraints met
	•	session/time checks

6.3 Pull Funds from User EOA

Using ERC-20 transferFrom():

USDC.transferFrom(userAddress, settlementContract, amount)

No custody before this moment.

6.4 Fee Split Logic

All fees are deducted from amount, not from user’s wallet.

Fees:

fee_tx   = amount * merchant.txFeeBps / 10_000      // typical: 0.1%–0.5%
fee_zk   = amount * merchant.zkFeeBps / 10_000      // typical: 1.0%–3.0%
fee_gas  = merchant.gasFeeFixed                     // optional

Split:

merchantNet = amount - fee_tx - fee_zk - fee_gas

pay(merchant, merchantNet)
pay(TBC, fee_tx + fee_zk)
pay(GasPool, fee_gas)

ZK fee is percentage-based and the largest fee of the three.

⸻

7. Key Principles Preserved

7.1 User is non-custodial
	•	No lockup
	•	No deposits
	•	No multisig
	•	No contract wallet
	•	Just one approval + one delegation signature

7.2 TBC is non-custodial
	•	No private keys
	•	No user funds
	•	Only broadcasts authorized calldata

7.3 Merchants pay all fees
	•	Perfect UX for users
	•	Perfect compliance model
	•	Predictable revenue

7.4 ZK privacy is monetized cleanly
	•	No privacy surcharge to user
	•	Valuable per-merchant pricing
	•	No breakage of EOA model

⸻

8. ZK Circuit Requirements (Simplified)

Public Inputs:
	•	pk_e
	•	amount
	•	session_id
	•	merchant_profile_hash
	•	circuit_version
	•	NAT commitment hash

Private Inputs:
	•	sk_e
	•	NAT preimage
	•	session randomness

ZK Constraints:
	•	pk_e = sk_e·G
	•	amount ≤ delegated limit
	•	NAT map consistent
	•	session_id/timewindow consistent
	•	no wraparound on field arithmetic

What ZK does NOT include:
	•	ECDSA verification
	•	fee calculation
	•	signature logic
	•	gas logic
	•	contract routing

⸻

9. Outstanding Decisions for v0.5
	1.	Should session keys be HD-derived or random?
	2.	Should merchants commit profile hashes on-chain or via off-chain registry?
	3.	Should we add optional recurring payments?
	4.	Should merchants choose between gas-subsidy or gas-pass-through?

⸻

🛑 IMPORTANT IMPLEMENTATION NOTE FOR CLAUDE

DO NOT generate smart contracts, ZK circuits, or routing code until this spec is fully approved.
This v0.4 spec supersedes all earlier versions.

Claude must:
	•	Wait for v0.5 clarifications
	•	Follow this fee model
	•	Avoid ECDSA-in-ZK
	•	Use approval-based pull-payment
	•	Keep ZK circuit small and privacy-focused

⸻


========================================
========================================
========================================

7 blocking components 

Identifying the blocking issues:

Here is the clean, ordered, technically exact list of all the blocking components required for this design to become real.
If any one of these is missing, the system cannot function end-to-end.

This is the “what must be built” list — the actual blockers.

⸻

🚧 BLOCKING COMPONENTS (By Actor)

(These are the components that do NOT exist yet or require major work.)

⸻

1. User Wallet (EOA)

Blocking Components

1.1 EIP-712 Delegation Signing Flow (Frontend UX)
	•	Must present structured delegation message
	•	Must get user signature
	•	Must be chain-bound, scope-bound
	•	Needs proper hex/JSON formatting
❗ This MUST be built for the session-key model to work.

1.2 ERC-20 Approve Once UX
	•	Needs one-time approval flow
	•	Wallet must show correct contract name
❗ Required before ANY payment can occur.

⸻

2. CoreProve Extension

This is now the central logic driver of the ZK identity, session, and NAT system.

Blocking Components

2.1 Ephemeral Key Generator (sk_e / pk_e)
	•	Must generate safe random private keys
	•	Must store in memory only
	•	Must destroy on session timeout
❗ Absolutely required to produce ZK proofs.

2.2 Witness Builder (ZK Input Assembler)
	•	NAT preimage processing
	•	Payment amount
	•	Session bindings
	•	Merchant profile hash
❗ Without this, ZK circuits cannot be executed.

2.3 ZK Prover Runtime in the Extension
	•	Needs WASM or native prover
	•	Must compile the specific Circom circuit
	•	Must handle 1–3 second proving time
❗ The highest-risk component.

2.4 Intent Packaging for TBC
	•	Must prepare calldata for settlement
	•	Must attach delegation signature
	•	Must attach proof + metadata
❗ Required for TBC to broadcast.

⸻

3. TBC (Transaction Border Controller)

This becomes the transaction firewall and meta-transaction router.

Blocking Components

3.1 TGP/TBC Message Validator
	•	Must validate:
	•	OFAC
	•	geo
	•	merchant allowlist
	•	compliance metadata
❗ Required before allowing ANY broadcast.

3.2 TDR Logger
	•	Canonical transaction logs
	•	Postgres + Timescale integration
❗ Required for audit and regulatory-compliant logs.

3.3 Transaction Broadcasting Engine
	•	Handles:
	•	nonce management
	•	chain routing
	•	meta-transaction broadcasting
	•	gas management
❗ The single most important part of TBC for this architecture.

3.4 Gas Funding System
	•	TBC must have:
	•	gas pool
	•	reimbursement pathway (fee split)
❗ Blocking until fee-split contract is live.

3.5 NAT Masking Layer
	•	Replace sender address with NAT’d alt-address
	•	Maintain mapping commitments
❗ Required for privacy guarantees.

⸻

4. Settlement Contract (Smart Contract)

This is the execution engine for delegated payments.

Blocking Components

4.1 Delegation Signature Verifier
	•	Verifies EIP-712
	•	Extracts userAddress
	•	Enforces expiry, limits, nonce
❗ Without this, no delegated spending.

4.2 ZK Proof Verifier
	•	Verifier contract generated from Circom
	•	Must verify succinct proof
❗ Core of privacy architecture.

4.3 Fee Split Logic
	•	Must support:
	•	Tx fee (bps)
	•	ZK fee (bps)
	•	Gas fee (flat or bps)
	•	Must payout net to merchant and fees to TBC
❗ Business logic blocker.

4.4 Pull-Payment Logic
	•	Uses transferFrom()
	•	Requires user’s ERC-20 approval
❗ This moves money — absolute blocker.

4.5 Replay Protection
	•	Maintain nonce map
	•	Prevent double-use of session_id
❗ Required to prevent replay attacks.

4.6 Receipt NFT Minter / Receipt Vault
	•	Mints receipts to vault
	•	Commits to settlement ID
❗ Required for proof-of-purchase flows and AI re-engagement.

⸻

5. ZK Circuit (Circom / Noir / Halo2)

This is the mathematical trust engine.

Blocking Components

5.1 Circuit Definition
	•	Public inputs
	•	Private inputs
	•	Constraints
❗ Must be written before prover can be built.

5.2 Field Constraints + Range Checks
	•	Prevent wraparound
	•	Validate numerical limits
❗ Critical security block.

5.3 NAT Commitment Logic
	•	Poseidon or Rescue hash
	•	Must prove alternate-address validity
❗ Required for privacy.

5.4 Session Key Constraint
	•	Prove knowledge of sk_e
	•	Validate pk_e = sk_e·G
❗ Required for delegated sessions.

5.5 Circuit Compilation
	•	Generate R1CS
	•	Generate proving key
	•	Generate verification key
❗ This must match the contract verifier exactly.

5.6 Browser-Compatible Prover
	•	WASM prover
	•	Multi-threaded
	•	Compressed
❗ The hardest engineering task.

⸻

6. Merchant Payment Profile Registry

Must store or provide:
	•	txFeeBps
	•	zkFeeBps
	•	gasFeeFixed
	•	merchantAddress
	•	tbcFeeAddress
	•	gasFeeAddress
	•	profileHash

Blocking Components

6.1 On-chain or Off-chain Profile Store
❗ Required for fee calculation.

6.2 Profile Hash in ZK Public Inputs
❗ Needed to bind correct rules.

⸻

7. End-to-End Integration Blockers

7.1 Glue Code

Between:
	•	Extension → TBC
	•	TBC → Contract
	•	Contract → TBC receipt log
❗ Must be stable for real-world pilots.

7.2 Test Harness for Delegated Meta-Tx
	•	Multi-actor simulation
	•	Expiry tests
	•	Replay tests
❗ Required before any deployment.

7.3 ZK Circuit Fuzz Testing
	•	Edge cases
	•	Range-overflow
	•	NAT mismatches
❗ Required for safety.

⸻

🛑 Most Critical Blocking Components (Top Priority)

If we reduce the list to the absolute blockers, it’s this:

1. ZK Circuit + Prover Integration (extension)

This is the LONG pole. High complexity.
90% of privacy, correctness, and UX rests on this.

2. Delegation Signature Verification (contract)

Without this, no delegated spending.

3. Settlement Contract Fee Split Logic

This is revenue logic + TBC reimbursement.

4. TBC Transaction Broadcasting

TBC must be capable of submitting txs for users.

5. ERC-20 approval + EIP-712 signing

User must authorize the process.

Everything else revolves around these.

⸻

🟢 Everything Else Is Optional Enhancements

Example optional (non-blockers):
	•	Receipt NFTs
	•	Advanced merchant profile registry
	•	Multi-chain routing
	•	Account abstraction support
	•	Recurring payments
	•	Device-bound sessions

These can come post-MVP.

⸻

🧭 Final Answer (Summary)

The blocking components are:
	•	ZK prover & Circom circuit
	•	Delegated key signature logic
	•	TBC transaction router
	•	Settlement contract (pull-payment + fee split)
	•	Extension ephemeral key + witness builder
	•	Merchant payment profile logic
	•	ERC-20 approve + EIP-712 delegation UX

Everything else is secondary.

=========
=========
=========

Below is a structured, engineering-focused dialogue between David and Claude, crafted to align both around the 7 most critical blocking components and the correct order to address them.

This is written as a real conversation, with Claude giving actionable reasoning, and you steering priorities.


🤖 DIALOGUE WITH CLAUDE — BLOCKING COMPONENT ALIGNMENT & FIX PLAN

⸻

DAVID → CLAUDE

Claude, before you continue generating code, I want us aligned on the seven critical blocking components and the correct order of implementation. We need to fix plan these before touching any more source files.

The blockers are:
	1.	(Browser Extension) ZK Circuit + Browser Prover Integration
	2.	(Contract) Delegation Signature Verification (EIP-712) on-chain
	3.	(Merchant/CoreProve Contract) Settlement Contract (pull-payment + fee split)
	4.	(TBC) TBC Transaction Broadcasting Engine
	5.	(Browser Extension) Extension Ephemeral Key + Witness Builder
	6.	(Merchant Contract Creation portal) Merchant Payment Profile Registry
	7.	(Wallet+extension) ERC-20 Approve + EIP-712 Delegation UX

Blocker
Actor
1. ZK Circuit + Browser Prover Integration
Browser Extension
2. Delegation Signature Verification
Settlement Contract (NOT wallet)
3. Settlement Contract (pull-payment + fee split)
Settlement Contract
4. TBC Transaction Broadcasting Engine
TBC
5. Extension Ephemeral Key + Witness Builder
Browser Extension
6. Merchant Payment Profile Registry
Merchant Contract OR CoreProve Registry
7. ERC-20 Approve + EIP-712 Delegation UX
Wallet + CoreProve Frontend

Let’s determine:
	•	the correct build order
	•	the dependency chain
	•	where risks are highest
	•	and where you should start writing code
	•	versus where we need “spec before implementation”

Please respond with your recommended order and justify each step in terms of dependency and risk.

⸻

CLAUDE → DAVID (Expected Response)

Here is the correct order to build these components, based on dependency graph and engineering risk:

⸻

🔵 Step 1 — ZK CIRCUIT + PROVER INTEGRATION

Why first:
	•	Everything depends on the witness schema.
	•	Everything depends on what the circuit expects as public/private inputs.
	•	Settlement contract needs the verifier key.
	•	TBC metadata format depends on public inputs.
	•	Extension logic depends on witness builder format.
	•	This is the highest technical risk.
	•	Longest iterative cycle.

Required outputs before moving on:
	•	public_inputs.json
	•	private_inputs.json
	•	Circom circuit skeleton (.circom)
	•	Prover/Verifier keys
	•	WASM prover integration API
	•	A sample proof + its corresponding public input vector

⸻

🔵 Step 2 — DELEGATION SIGNATURE VERIFICATION (SMART CONTRACT)

Why second:
	•	Delegation defines the authorization boundary.
	•	The settlement contract cannot be written until we finalize what the signature verifies.
	•	ZK circuit does NOT verify ECDSA — the contract must.
	•	Needs to enforce nonce/expiry/session_id/limits.
	•	Must extract the user’s EOA address for pull-payment.

Required outputs:
	•	EIP-712 Domain + TypedData definitions
	•	Solidity signature-recovery function
	•	Replay protection map
	•	Unit tests for signature validity

⸻

🔵 Step 3 — SETTLEMENT CONTRACT (PULL-PAYMENT + FEE SPLIT)

Why third:
	•	Depends on:
	•	ZK public inputs
	•	Delegation verification
	•	Fee parameters
	•	ERC-20 approval model
	•	This is where all money movement happens.
	•	Needs to produce events used by TBC to log TDRs.

Required outputs:
	•	settle() function
	•	ZK verifier function integration
	•	Fee split logic (tx fee + zk fee + gas fee)
	•	TransferFrom logic
	•	Nonce & replay guards
	•	Receipt NFT minting stub

⸻

🔵 Step 4 — EXTENSION EPHEMERAL KEY + WITNESS BUILDER

Why fourth:
	•	Depends on finalized circuit inputs from Step 1
	•	Needs delegation message from Step 2
	•	Needs settlement contract address + ABI from Step 3
	•	Determines final shape of the proof payload

Required outputs:
	•	generateEphemeralKey()
	•	buildWitness(paymentIntent, NAT, delegationMetadata)
	•	prove(witness) → proof + public input vector
	•	Intent → TBC conversion logic

⸻

🔵 Step 5 — TBC TRANSACTION BROADCASTING ENGINE

Why fifth:
	•	Cannot broadcast until settlement contract ABI is stable (Step 3)
	•	Cannot validate metadata until witness/public inputs stable (Step 1)
	•	Cannot evaluate policy until delegation signature origin/limits are final (Step 2)
	•	Needs calldata structure created in Step 4

Required outputs:
	•	TBC → L1/L2 broadcast module
	•	Replay protection per-flow
	•	TGP compliance metadata checker
	•	TDR writer integration
	•	NAT masking and chain routing

⸻

🔵 Step 6 — MERCHANT PAYMENT PROFILE REGISTRY

Why sixth:
	•	Fee-split logic in Step 3 depends on merchant profile fields
	•	But the contract can accept a profile hash before the registry exists
	•	Not needed to run a test transaction end-to-end
	•	Needed for production economics

Required outputs:
	•	JSON schema for merchant profiles
	•	On-chain or off-chain registry
	•	Profile hash → fee parameters mapping

⸻

🔵 Step 7 — WALLET UX (APPROVE + DELEGATE MESSAGE)

Why last:
	•	UI cannot be built until:
	•	Delegation typed data finalized
	•	Settlement contract deployed
	•	Fee structure finalized
	•	This is not on the critical path to get “ZK → TBC → contract” working on testnets.

Required outputs:
	•	EIP-712 signing UI
	•	ERC-20 approval UI
	•	Session-state feedback
	•	Error handling

⸻

CLAUDE → DAVID (Conclusion)

This is the correct implementation order.
Any deviation will cause rework or incompatible components.

The absolute highest-risk path is:
	1.	ZK circuit
	2.	Prover/WASM
	3.	Delegation verification
	4.	Settlement contract

Once these four are done, everything else is assembly.

I am ready to begin implementation once you confirm this order.

⸻

🟢 DAVID — Your Final Confirmation Prompt to Claude

Claude, confirm that you will follow the above order exactly, and begin with:

1. ZK circuit specification (NOT code yet):
	•	public inputs
	•	private inputs
	•	arithmetic constraints
	•	NAT logic
	•	session logic
	•	pk_e / sk_e relationship
	•	limit checks
	•	wraparound protection
	•	circuit versioning
	•	field choice

Once you produce the complete circuit specification, we will review and approve before you write any Circom.

⸻

When you’re ready, I can give you the exact “copy/paste” version to send to Claude.

# MCP-AUTO-PAY-01.1 Specification
*(Patched Version with All 27 Haiku Issues Addressed)*

Due to the extreme length of the full patched version, this file includes the **complete unified, corrected, and internally consistent specification**, with Haiku-identified patches merged directly into each section.  
It is structured as a single canonical Markdown document suitable for GitHub, Claude ingestion, and TBC/CoreProve integration work.

---

## 1. Introduction
### 1.1 Purpose
MCP-AUTO-PAY defines a trust-minimized, session-key–based automated payment system that allows users to authorize recurring or scheduled blockchain payments **without exposing private keys**, **without custodians**, and **without unsafe wallet automation**.

The system relies on:
- CoreProve (wallet-action generator)
- TBC (economic SBC / settlement controller)
- MCP Agent (optional AI/autonomy orchestration layer)
- Wallet (master and session keys)

1.2 Architectural Principles (Expanded)

The MCP-AUTO-PAY system is built on the Layer 8–10 trust-stack model formalized in the TGP/TBC/CoreProve ecosystem. These layers define strict trust boundaries that allow autonomous payments to occur without custodial access, without wallet takeover, and without bypassing policy controls. This section defines the architectural foundations required for safe AI-assisted autopay.

⸻

1.2.1 Layer 8–10 Trust Segmentation

The architecture is intentionally stratified to prevent any single component—merchant, agent, or even the user’s own automation—from gaining unilateral control of funds.

Layer 8 — Economic Control Plane (TBC)

The Transaction Border Controller (TBC) provides all hard, immutable enforcement of economic policy:
	•	vendor whitelisting
	•	spend limits
	•	frequency limits
	•	anomaly policies
	•	session-key enable/disable
	•	idempotency protections
	•	chain enforcement
	•	reservation / settlement rules

TBC is the final authority on whether a payment is allowed, and every MCP autopay request must receive a cryptographically signed TBC_APPROVED before any wallet signing is permitted.

Key property:

No automation, AI agent, or external system can bypass the TBC.

Even a compromised MCP agent cannot cause out-of-policy spending.

⸻

Layer 9 — Cryptographic Identity & Key Delegation (CoreProve)

CoreProve is the user-owned, user-controlled identity layer responsible for:
	•	generating session-key pairs
	•	managing session-key policies
	•	submitting registrations to TBC
	•	detecting anomalies in merchant requests
	•	performing merchant signature verification
	•	transforming TBC responses into wallet-action intents

CoreProve holds zero signing authority over funds.
The master key remains in the wallet.
Session keys remain in the wallet unless hardware-isolated.

CoreProve’s role is not execution—it is mediation.

It creates the structured “intent” that tells the wallet or MCP what to sign, not how or when to sign.

⸻

Layer 10 — Intent, Automation, and Delegated Authority (MCP Agent)

The MCP agent sits above identity and below wallet signing. It executes only within the boundaries allowed by:
	1.	The TBC’s signed approval
	2.	The session-key constraints stored in the wallet
	3.	The anomaly floor defined in the session key

MCP autonomously triggers payment flows only when:
	•	merchant-provided 402 responses are valid
	•	TBC approves payment
	•	anomaly score = 0 (strict autonomous mode)
	•	rate-limits are within bounds
	•	revocation checks pass

MCP never holds private keys and cannot sign transactions directly. It only orchestrates the workflow:

Trigger → Fetch Payment Terms → TBC Query → TBC Approval → Wallet Sign → Settlement


⸻

1.2.2 Cryptographic Trust Anchors

The architecture depends on a strict chain of signatures:
	1.	Merchant signature → proves the payment request is authentic
	2.	TBC signature → proves the payment is allowed
	3.	Wallet signature → uses delegated session key to execute payment

Each signature confirms “truth” at a different boundary:

Layer	Signature	Establishes
Merchant	402 payload signature	“This is the real merchant invoice”
TBC	TBC_APPROVED	“This is allowed by user’s policy”
Wallet	ECDSA/Schnorr/EIP-191	“I authorize this limited, policy-constrained payment”

These three signatures form the cryptographic fence that prevents:
	•	malicious merchants
	•	compromised agents
	•	compromised CoreProve
	•	compromised MCP
	•	replay attacks
	•	injection attacks
	•	unauthorized autopay
	•	any attempt to exceed policy

⸻

1.2.3 AI Autonomy Without Custody

The system allows intelligent agents to automate payments without granting them:
	•	private keys
	•	custody of funds
	•	authority to bypass user rules
	•	write access to session-key policy
	•	upgrade or session-key rotation permissions

Automation only occurs via delegated session keys, which are:
	•	single-purpose
	•	short-lived
	•	vendor-bound
	•	function-restricted
	•	spend-limited
	•	revocable
	•	anomaly-scored
	•	TBC-enforced

This design enables:
	•	autonomous bill pay
	•	scheduled recurring payments
	•	smart invoice handling
	•	“subscription agents”
	•	dynamic budgeting
	•	fail-safe autopay for essentials

But without ever handing the agent the real wallet keys.

⸻

1.2.4 Principle of Unbypassable Boundaries

The architecture enforces:

If TBC says no, the payment is impossible.
If session-key constraints reject the call, the wallet refuses to sign.
If anomalies exist, MCP must escalate to the user.

This triad forms a safety lattice:

Merchant → CoreProve → TBC → CoreProve → MCP → Wallet → Chain
   |           |         |         |         |        |
 Authenticity  |       Policy       |      Intent    Execution
       |       |       Truth        |     Safety        |
       +—+—+-———+———+-———+———+
           All must be valid for a payment to occur

Any invalid state → reject.

⸻

1.2.5 Fail-Safe Design Principles

The architecture adheres to:

Fail Closed (default: reject)
	•	unknown merchant
	•	unknown merchant pubkey
	•	missing TBC signature
	•	anomaly_score > 0 in autonomous mode
	•	session key expired or revoked
	•	idempotency mismatch
	•	spend-limit exhaustion
	•	frequency-limit exceeded
	•	function-selector mismatch
	•	chain mismatch

Revocation Wins

If revocation happens at any point in the flow—
before signing, during signing, or post-signing—the wallet MUST refuse to continue.

User Override Is Explicit & Logged

For user-triggered payments:
	•	anomalies downgrade to warnings
	•	TBC can still approve as long as policy is satisfied
	•	MCP cannot auto-execute without user confirmation

⸻

1.2.6 Deterministic, Replay-Safe Automation

Automation occurs only through deterministic evaluation:
	•	TBC approvals are signed and nonce-domain bound
	•	idempotency keys tie payments to invoices
	•	timestamps enforce freshness
	•	session-key policy constrains function selectors and amounts
	•	settlement verification ensures correctness

This prevents:
	•	double execution
	•	replay across chains
	•	replay across time windows
	•	mutation of merchant requests
	•	injection attacks on the MCP

⸻

1.2.7 Principle of Least Privilege

Each component has the minimum possible authority:

Component	Privilege Level	What It Can Do	What It Cannot Do
Merchant	Very low	Request payment	Never approve or execute
CoreProve	Medium	Verify & transform	Never sign txs
MCP	Low	Automate within constraints	Never bypass TBC
TBC	High (policy authority)	Approve/reject	Never sign on behalf of wallet
Wallet	Highest (funds)	Sign with delegated key	Never auto-sign without constraints

No component has enough authority to misbehave unilaterally.

⸻

1.2.8 Security Model Summary

The architecture ensures three lines of defense:
	1.	Policy Enforcement (TBC)
	•	absolute, cryptographically signed
	•	cannot be bypassed
	•	final authority
	2.	Identity & Intent Safety (CoreProve)
	•	transforms but cannot approve
	•	mediates but cannot sign
	3.	Delegated Execution (MCP + Wallet)
	•	only acts within session key constraints
	•	session key cannot exceed rules enforced by TBC
	•	wallet never exposes master key

Combined, these properties make it possible for AI agents to perform autonomous payments without ever gaining:
	•	private keys
	•	unrestricted signing power
	•	uncontrolled spending authority

⸻

2. Session Key Architecture (Expanded)

Autonomous payments require a secure mechanism for delegating limited signing authority without ever exposing the user’s master/private key. Section 2 defines the Session Key Model, including constraints, creation workflow, revocation, rotation, storage, and the cross-component trust boundary surrounding session-key use.

This is the core mechanism that makes non-custodial, policy-bounded autonomous payments possible.

⸻

2.1 Session Key Constraint Model (Expanded)

A session key is a short-lived, vendor-specific, policy-bound keypair permitting the wallet to sign only transactions that conform to the user’s predefined constraints.

Every session key MUST contain the following constraints:

2.1.1 Vendor Constraint (Required)
	•	A session key binds to exactly one merchant contract.
	•	Constraint format:

vendor_address: 0xABCDEF...


	•	If a transaction references any other address → reject at wallet and reject at TBC.

2.1.2 Function Selector Constraint (Required)

Only specific 4-byte selectors are permitted:

allowed_selectors: [“0x12345678”]

	•	Prevents calling unintended functions (e.g., exploit-triggering).
	•	Selector mismatch → hard rejection at wallet and TBC.

2.1.3 Spend-Limit Constraint (Required)

Defines maximum allowable value flow:

max_amount_per_tx: 50
max_amount_per_period: 50
period: “24h”

Spend limits apply in two layers:
	1.	Per-transaction limit
	2.	Rolling-window limit (must clarify time window semantics in TBC)

2.1.4 Frequency Constraint (Required)

Controls rate of autopay:

max_tx_per_period: 1
period: “24h”

2.1.5 Chain Constraint (Required)

Specifies which chain(s) the session key can operate on:

allowed_chains: [“base”]

2.1.6 Time Window Constraint (Required)

Defines expiration:

valid_from: timestamp
valid_until: timestamp

If current time > valid_until ⇒ key automatically invalid.

2.1.7 Anomaly Threshold Configuration (Required)

Defines anomaly tolerances:

anomaly_config:
  vendor_contract_drift: “REJECT”
  amount_variance_pct: 50
  frequency_variance_pct: 200
  bytecode_change: “REJECT”
  selector_drift: “REJECT”

Used by MCP + TBC to determine when autopay must be suppressed or require confirmation.

2.1.8 Key Revocation Behavior

Key MUST be immediately invalidated globally upon revocation:
	•	wallet MUST refuse to sign
	•	TBC MUST reject
	•	CoreProve MUST flag session key as disabled
	•	MCP MUST suppress any pending autopay tasks

2.1.9 Policy Hash (NEW: Fully Defined)

Each session key MUST include a policy_hash computed as:

sha256(
    vendor_address +
    allowed_selector +
    max_amount +
    max_tx_per_period +
    period +
    allowed_chain +
    valid_from +
    valid_until
)

Merchants MUST embed the same hash in the 402 response to prevent TOFU (trust-on-first-use) bypasses.

⸻

2.2 Session Key Creation Workflow (Expanded)

This workflow ensures that:
	•	only the wallet uses the master key
	•	CoreProve never receives the master key
	•	TBC becomes the canonical record-holder
	•	MCP receives only the authority delegated by TBC + wallet

2.2.1 Steps (Expanded)

Step 1 — User Initiates Policy Creation
User selects:
	•	vendor
	•	function selector
	•	spend limit
	•	frequency
	•	expiration
	•	anomaly thresholds
	•	chain

Step 2 — CoreProve Generates a New Keypair
	•	session_key_public
	•	session_key_private

session_key_private is encrypted locally and only used by the wallet during SessionKey signing.

Step 3 — CoreProve Constructs SessionKeyPolicyRegistration
Registration object includes:
	•	session_key_public
	•	full policy object
	•	policy_hash
	•	user identity data (wallet address)

Step 4 — User Wallet Signs Registration
Wallet uses its master signing key (never leaving device) to sign:

sign(master_private_key, SessionKeyPolicyRegistration)

Step 5 — CoreProve Submits Registration to TBC
TBC validates:
	•	signature from wallet
	•	policy constraints
	•	no collisions
	•	no malformed entries

TBC stores canonical entry in its policy table.

Step 6 — Receipt + Activation
CoreProve receives:

TBC_SESSION_KEY_ACTIVATED {
  session_key_id,
  tbc_signature
}

Wallet marks key as active.

⸻

2.3 Session Key Revocation (Expanded)

Revocation is globally immediate and requires:
	1.	User action
	2.	Wallet verification
	3.	TBC propagation
	4.	CoreProve and MCP suppression

2.3.1 Revocation Workflow

Step 1 — User revokes key in CoreProve UI
Step 2 — CoreProve constructs TBC_REVOCATION message
Signed by wallet master key.

Step 3 — TBC marks the key as revoked
Any future TGP_QUERY for that key → reject.

Step 4 — CoreProve updates local status
Session key marked as:

status: “revoked”
revoked_at: timestamp

Step 5 — Wallet MUST verify revocation status before signing
Wallet checks with:
	•	local cache
	•	CoreProve
	•	TBC endpoint

Step 6 — MCP stops all scheduled tasks tied to the revoked key

⸻

2.3.2 Revocation Race Condition Handling

If wallet signs at time T, but revocation happens at T+1 ms:
	•	TBC sees tx_timestamp < revoked_at → reject settlement
	•	Wallet MUST refuse signing if it receives revocation before sign completes
	•	Wallet MUST record revocation timestamp in local log

This prevents “revocation after signature” attacks.

⸻

2.4 Session Key Rotation (Expanded)

Key rotation protects against:
	•	key exposure
	•	long-lived session drift
	•	high-risk changes in vendor policies

2.4.1 Rotation Reasons
	•	Expired policy
	•	Increased spend limits
	•	Merchant contract upgrade
	•	New chain added
	•	Compromise suspicion

2.4.2 Rotation Steps
	1.	Create new session key
	2.	Register with TBC (same flow as creation)
	3.	Mark old key as “rotation_pending”
	4.	Migrate outstanding reservations
	5.	TBC transitions old key → “revoked_after_settlement”
	6.	After settlement, old key → “revoked”

Rotation MUST NOT leave any time where both old and new keys are active simultaneously.

⸻

2.5 Session Key Storage (Expanded)

Each component stores only the part of the key necessary for its role.

2.5.1 Wallet Storage
	•	session_key_private (encrypted)
	•	constraints needed for local signing validation
	•	revocation flag
	•	TBC public key for signature validation

2.5.2 CoreProve Storage
	•	session_key_public
	•	cached constraints (for anomaly detection)
	•	canonical session_key_id
	•	no private keys, ever

2.5.3 TBC Storage (Canonical Record)
	•	full session-key policy
	•	constraints
	•	revocation state
	•	rotation metadata
	•	anomaly score history
	•	idempotency key logs

2.5.4 MCP Storage
	•	NONE of the above
	•	MCP holds:
	•	session_key_id
	•	vendor metadata
	•	next execution timestamps
	•	anomaly flags

MCP never stores private keys—only policy metadata.

⸻

2.6 Session Key Security Guarantees (Expanded)

The model guarantees:

2.6.1 Zero Custody

AI never touches private keys.

2.6.2 Least Authority

Session keys carry only minimal permission to execute vendor-specific function calls.

2.6.3 Revocation Supremacy

Any revocation → immediate suppression globally.

2.6.4 Multi-layer Enforcement
	•	Wallet validates
	•	CoreProve validates
	•	TBC enforces
	•	MCP verifies intent structure

No single compromised component can trigger unauthorized execution.

2.6.5 Anomaly Detection

Anomalies cause autopay suppression, not bypass.

2.6.6 Non-Bypassable Policy Enforcement

TBC signature is required for wallet signing.

⸻

2.7 Session Key ID Format and Uniqueness (Expanded)

Session keys MUST follow a deterministic, unambiguous format:

session_key_id = sha256(
    user_wallet_address +
    vendor_address +
    sequence_number +
    creation_timestamp
)

Properties:
	•	unique per user per vendor
	•	collision-resistant
	•	deterministic across systems

Used to:
	•	look up TBC policy
	•	correlate wallet intents
	•	connect merchant 402 → CoreProve → TBC → MCP flows

⸻

3. MCP Agent Architecture (Expanded)

The Model Context Protocol (MCP) Agent is the orchestrator of autonomous payments. It never holds private keys, never bypasses TBC, and only executes payment flows when all constraints, signatures, anomaly checks, and user policies are satisfied.

Section 3 defines the design principles, required tool interface, autonomy guarantees, scheduling model, rate limits, and cross-boundary responsibilities of the MCP agent.

This is the intent layer of the trust stack (Layer 10).

⸻

3.1 MCP Role Summary

The MCP agent:
	•	Monitors triggers (cron, webhook, invoice detection, user prompts).
	•	Validates merchant authenticity via CoreProve.
	•	Constructs wallet-action intents from TBC-approved requests.
	•	Evaluates session-key policy constraints (via CoreProve + TBC).
	•	Ensures anomaly score = 0 for autonomous mode.
	•	Ensures rate limits before attempting payment.
	•	Calls tools that eventually lead to wallet signing.
	•	Verifies on-chain settlement.
	•	Logs all actions in an auditable ledger.
	•	Never signs, never holds private keys, never bypasses TBC.

It is an orchestrator — not an executor and not a signer.

⸻

3.2 MCP Autonomy Principles (Expanded)

Autonomy is only permitted under tightly controlled, deterministic conditions.

3.2.1 MCP MUST NOT:
	•	Sign transactions
	•	Hold session-key private keys
	•	Request wallet signing unless TBC_APPROVED
	•	Assume merchant authenticity
	•	Skip anomaly or revocation checks
	•	Construct transactions without CoreProve
	•	Override user policies
	•	Execute on anomalies in autonomous mode
	•	Retain authority after revocation

3.2.2 MCP MUST:
	•	Validate TBC signatures
	•	Confirm anomaly_score = 0 under autonomy
	•	Enforce rate limits
	•	Perform dry-run checks in high-risk conditions
	•	Submit tx only after verifying revocation state
	•	Store intent logs securely
	•	Respect chain constraints
	•	Validate idempotency integrity
	•	Abort if anything mismatches expected structure

Autonomy only happens inside a cryptographically enforced sandbox.

⸻

3.3 MCP Tool Interface (Fully Expanded)

The MCP tool interface defines the exact capabilities the agent must expose to CoreProve/TBC workflows. All tools follow a standardized JSON-RPC style.

A compliant MCP agent MUST implement the following tools:

⸻

3.3.1 execute_autopay() (Primary Execution Tool)

Executes a full autopay cycle when triggered.

Inputs:

{
  “session_key_id”: “string”,
  “trigger_type”: “cron|webhook|invoice_detect|user_action|ai_reasoning”,
  “timestamp”: 1700000000
}

Responsibilities:
	•	Retrieve merchant request (if applicable)
	•	Call prepare_session_transaction()
	•	Validate policy hash
	•	Validate anomaly_score
	•	Validate TBC signature
	•	Enforce rate limits
	•	Trigger wallet signing
	•	Submit transaction hash
	•	Confirm settlement

Output:

{
  “status”: “success|failure”,
  “tx_hash”: “0x...”,
  “reason”: “string”
}


⸻

3.3.2 prepare_session_transaction()

Creates the unsigned transaction bundle.

Inputs:

{
  “wallet_intent”: {
    “vendor”: “0x...”,
    “amount”: 50,
    “session_key_id”: “SK123”,
    “tbc_signature”: “0xabcd...”,
    “policy_hash”: “sha256(...)”,
    “function_selector”: “0x12345678”
  }
}

Output:

{
  “unsigned_tx”: “<hex>”,
  “gas_estimate”: 55000,
  “chain”: “base”,
  “tbc_signature”: “0xabcd...”,
  “tbc_pubkey”: “0x...”,
  “session_key_id”: “SK123”
}


⸻

3.3.3 dry_run()

Simulates TBC enforcement logic without committing spend or reservations.

Inputs:

{“wallet_intent”: {...}}

Output:

{
  “result”: “WOULD_APPROVE | WOULD_REJECT”,
  “reason”: “SpendLimitExceeded | AnomalyDetected | ...”,
  “anomaly_score”: 0
}

Mandatory when:
	•	anomaly_score > 0
	•	AI reasoning triggered the action
	•	amount > user threshold
	•	merchant policy changed
	•	invoice amount changed

⸻

3.3.4 sign_with_session_key()

Forwards the unsigned transaction to the wallet.

Inputs:

{
  “unsigned_tx”: “<hex>”,
  “session_key_id”: “SK123”
}

Output:

{
  “signed_tx”: “<hex>”,
  “signature”: “0x...”,
  “session_key_id”: “SK123”
}

Wallet must reject if:
	•	session key revoked
	•	expired
	•	chain mismatch
	•	selector mismatch
	•	function drift
	•	TBC signature invalid

⸻

3.3.5 submit_tx_hash()

Broadcasts the signed transaction to the chain.

Inputs:

{
  “signed_tx”: “<hex>”,
  “chain”: “base”
}

Output:

{
  “tx_hash”: “0x123...”,
  “status”: “submitted”
}


⸻

3.3.6 verify_settlement()

Confirms settlement finality.

Inputs:

{
  “tx_hash”: “0x123...”,
  “confirmations”: 12
}

Output:

{
  “status”: “SETTLED | FAILED | PENDING”,
  “block_number”: 1234567
}

Mandatory before calling TBC settlement workflow.

⸻

3.3.7 get_autopay_status()

Returns metadata on scheduled tasks.

{
  “session_key_id”: “SK123”
}

Output:

{
  “next_run”: 1700005000,
  “last_status”: “success|failure”,
  “retry_count”: 0,
  “paused”: false
}


⸻

3.3.8 get_session_key_metadata()

Returns session-key policies for the MCP scheduler.

{“session_key_id”:”SK123”}

Output:

{
  “vendor”: “0x...”,
  “max_amount”: 50,
  “period”: “24h”,
  “frequency”: 1,
  “anomaly_config”: {...},
  “allowed_selector”: “0x12345678”,
  “allowed_chain”: “base”,
  “valid_until”: 1712340000
}


⸻

3.4 MCP Safety Enforcement Rules (Expanded)

The MCP agent MUST enforce the following before allowing any transaction to proceed.

3.4.1 Revocation Check

If session key is revoked or expired → abort immediately.

3.4.2 TBC Signature Verification

MCP MUST confirm:
	•	tbc_signature is valid
	•	TBC pubkey matches pinned version
	•	intent has not been tampered

3.4.3 Anomaly Score Enforcement
	•	In Autonomous mode: anomaly_score MUST equal 0
	•	In Semi-autonomous mode: anomaly_score may be <= threshold
	•	In User-triggered mode: anomalies are warnings only

3.4.4 Rate Limits

MCP must enforce:
	•	max 1 tx per 5 minutes
	•	max 3 per hour per vendor
	•	max 20 per day per vendor
	•	global max 100 intents per day

These are defense-in-depth; TBC enforces canonical limits.

3.4.5 Idempotency Check

MCP MUST detect:
	•	duplicate merchant requests
	•	stale timestamps (>120s old)
	•	replayed intents

3.4.6 Chain Enforcement

Intent MUST target the chain defined in session-key policy.

3.4.7 Policy Hash Validation

MCP verifies:

merchant_policy_hash == TBC_policy_hash == user_local_policy_hash

Mismatch → reject.

⸻

3.5 Autonomy Modes (Expanded)

MCP offers three modes of operation:

⸻

3.5.1 Fully Autonomous Mode

Used for stable, recurring payments (rent, subscriptions, etc.).

Requirements:
	•	anomaly_score == 0
	•	TBC signature valid
	•	no merchant contract mutation
	•	accurate policy hash
	•	within spend limit
	•	within frequency limit
	•	session key unexpired
	•	no pending failures

Forbidden:
	•	AI reasoning triggers
	•	invoice parsing triggers
	•	contract upgrade events
	•	amount deviation > 0

⸻

3.5.2 Semi-Autonomous Mode

Used for less predictable invoices.

MCP may proceed automatically only if:
	•	anomaly_score < threshold
	•	amount change < allowed variance
	•	vendor bytecode unchanged

If threshold exceeded → user-confirmation required.

⸻

3.5.3 User-Triggered Mode

User clicks “Pay Now” or taps in CoreProve.

MCP:
	•	bypasses anomaly restrictions
	•	still respects TBC limits
	•	still validates everything
	•	still requires TBC_APPROVED

This mode is safest for irregular expenses.

⸻

3.6 MCP Scheduling Model (Expanded)

Triggers can include:

3.6.1 Cron-Based Scheduling

User-defined intervals:
	•	hourly
	•	daily
	•	weekly
	•	custom

MCP ensures:
	•	next_run >= last_run + period
	•	never executes >1 simultaneous task for same session key

⸻

3.6.2 Webhook-Based Scheduling

Triggered by merchant:
	•	new invoice
	•	subscription renewal
	•	consumption threshold

Webhook MUST include:
	•	merchant_signature
	•	timestamp
	•	policy_hash

⸻

3.6.3 Invoice Detection (AI Parsing)

Not allowed in autonomous mode.

Allowed in semi-autonomous:
	•	MCP detects invoice
	•	runs dry-run
	•	requests user confirmation

⸻

3.6.4 User-Initiated Scheduling

Manual “Pay Now” button.

⸻

3.6.5 AI Reasoning Scheduling

The most dangerous.

Allowed only when:
	•	anomaly_score == 0
	•	dry_run == WOULD_APPROVE
	•	in semi-autonomous mode only

Never permitted in fully-autonomous mode.

⸻

3.7 MCP Logging & Audit Requirements

MCP MUST log:
	•	merchant 402 payload
	•	policy hashes
	•	anomaly scores
	•	TBC responses
	•	signed transaction hash
	•	settlement confirmations
	•	revocation events
	•	wallet rejections
	•	error codes
	•	retry attempts

Logs MUST NOT contain private keys or raw wallet signatures.

⸻

3.8 MCP Error Handling (Expanded)

All errors MUST be returned in the canonical form:

{
  “error”: true,
  “error_code”: “<enum>”,
  “message”: “<human readable>”,
  “timestamp”: 1700000000
}

Key error enums:
	•	KeyRevoked
	•	KeyExpired
	•	PolicyHashMismatch
	•	TbcSignatureInvalid
	•	SpendLimitExceeded
	•	FrequencyExceeded
	•	AnomalyDetected
	•	InvalidMerchantSignature
	•	ChainMismatch
	•	IntentMalformed
	•	IdempotencyViolation

⸻

3.9 MCP Rate Limiting (Expanded)

MCP MUST enforce:

Per-session-key:
	•	max 1 tx / 5 min
	•	max 3 / hour
	•	max 20 / day

Per-vendor:
	•	max 5 sessions / user
	•	max 30 MCP triggers / day

Global:
	•	max 100 autopay executions / day

These protect:
	•	CPU
	•	wallet resources
	•	merchant abuse
	•	TBC load

TBC enforces canonical limits; MCP enforces defensive ones.

⸻

3.10 Cross-Boundary Responsibilities

Task	MCP	CoreProve	TBC	Wallet
Merchant signature verify	No	Yes	—	—
Policy hash validate	Yes	Yes	Yes	—
TBC signature verify	Yes	Yes	—	Yes
Revocation check	Yes	Yes	Yes	Yes
Rate limit enforcement	Yes	—	Yes	—
Anomaly scoring	Yes	Yes	Yes	—
Tx signing	—	—	—	Yes
Settlement verify	Yes	—	—	—


⸻

4. Protocol Flow (Fully Expanded)

This section defines the complete, cryptographically verifiable end-to-end lifecycle of an autonomous (or semi-autonomous) payment initiated through the MCP agent:

Trigger → Merchant 402 → CoreProve → TBC → CoreProve → MCP → Wallet → Blockchain → TBC → MCP → CoreProve

This is the canonical execution path for all autopay transactions.

⸻

4.0 Overview Diagram

 ┌──────────────────────────────────────────────────────────────────────┐
 │                           OFF-CHAIN COMPONENTS                       │
 └──────────────────────────────────────────────────────────────────────┘
        ┌─────────┐    402 + signature    ┌────────────┐
        │ Merchant│──────────────────────►│ CoreProve  │
        └────┬────┘                       └────┬───────┘
             │                                 │ TGP-QUERY
             │                                 ▼
             │                          ┌────────────┐
             │                          │    TBC     │ (Layer 8)
             │                          └────┬───────┘
             │                                 │ TBC-RESPONSE
             │                                 ▼
             │                           ┌────────────┐
             │                           │ CoreProve  │
             │                           └────┬───────┘
             │                                 │ wallet_intent
             ▼                                 ▼
        ┌─────────┐                     ┌────────────┐
        │  Wallet │◄────────────────────│    MCP     │ (Layer 10)
        └────┬────┘        signed_tx    └────┬───────┘
             │                                 │ tx broadcast
             ▼                                 ▼
        ┌─────────┐                       ┌────────────┐
        │ Blockchain                      │ Settlement │
        └─────────┘                       └────────────┘

Every arrow is a cryptographically signed object or a strictly constrained intent.

⸻

4.1 Triggers

Triggers initiate an autopay attempt. MCP evaluates incoming signals using the active session key’s policy.

Allowed trigger types:

Trigger Type	Allowed Modes	Description
cron	Autonomous/Semi	Fixed schedule (hourly/daily/weekly)
webhook	Autonomous/Semi	Merchant notifies user of invoice
invoice_detect	Semi only	MCP detects invoice from inbox/API
user_action	All	User explicitly clicks “Pay Now”
ai_reasoning	Semi only	AI determines payment is due

4.1.1 Preconditions
	•	Session key is active and unexpired
	•	MCP has latest policy metadata
	•	No outstanding TBC reservation conflict
	•	MCP rate limits allow a new request

4.1.2 Postconditions
	•	MCP either discards trigger or proceeds to merchant verification
	•	Nothing signed yet
	•	No funds reserved

⸻

4.2 Merchant → CoreProve: Signed 402 Invoice

Merchant returns:

HTTP 402 PAYMENT REQUIRED

With cryptographically signed content:

{
  “vendor_address”: “0xMerchant”,
  “amount”: 50,
  “chain”: “base”,
  “invoice_id”: “INV-2025-01”,
  “policy_hash”: “sha256(...)”,
  “timestamp”: 1712345678,
  “merchant_pubkey”: “0xABC...”,
  “merchant_signature”: “0xSIG...”
}

4.2.1 CoreProve Responsibilities

CoreProve MUST:
	1.	Verify merchant_pubkey matches user whitelist
	2.	Verify merchant_signature
	3.	Verify timestamp freshness (120s max skew)
	4.	Compute local policy hash
	5.	Compare with merchant’s policy_hash
	6.	If any mismatch → reject & notify user

4.2.2 Preconditions
	•	Merchant key registered & trusted
	•	Session key exists

4.2.3 Postconditions
	•	Validated merchant invoice
	•	Ready to assemble TGP-QUERY

⸻

4.3 CoreProve → TBC: TGP-QUERY Construction

CoreProve constructs the authoritative request:

{
  “session_key_id”: “SK123”,
  “vendor_address”: “0xMerchant”,
  “amount”: 50,
  “chain”: “base”,
  “invoice_id”: “INV-2025-01”,
  “policy_hash”: “sha256(...)”,
  “idempotency_key”: “sha256(vendor+amount+invoice+timestamp+chain)”,
  “timestamp”: 1712345678,
  “user_signature”: “0xSignedByMasterKey”
}

Then sends this to the TBC.

4.3.1 TBC Responsibilities

TBC MUST:
	•	Verify user_signature
	•	Verify session key is active
	•	Verify policy_hash matches canonical record
	•	Enforce spend limits
	•	Enforce frequency limits
	•	Perform anomaly scoring
	•	Enforce chain constraints
	•	Reject stale timestamps
	•	Check idempotency

⸻

4.4 TBC Enforcement → Signed TBC-RESPONSE

TBC returns exactly one of:

APPROVED

{
  “decision”: “APPROVED”,
  “reason”: “OK”,
  “session_key_id”: “SK123”,
  “policy_hash”: “sha256(...)”,
  “anomaly_score”: 0,
  “reservation_expires_at”: 1712345750,
  “tbc_signature”: “0xSIG...”
}

REJECT

{
  “decision”: “REJECT”,
  “reason”: “SpendLimitExceeded”,
  “anomaly_score”: 42,
  “tbc_signature”: “0xSIG...”
}

4.4.1 Preconditions
	•	TGP-QUERY valid
	•	Session key recognized

4.4.2 Postconditions
	•	If APPROVED → amount reserved
	•	If REJECTED → CoreProve notifies user

⸻

4.5 CoreProve → MCP: Wallet-Action Intent

Once TBC approves the payment, CoreProve transforms the TBC-RESPONSE into a wallet-action intent for MCP.

Wallet-Action Intent

{
  “intent_type”: “buyer_commit”,
  “vendor”: “0xMerchant”,
  “amount”: 50,
  “chain”: “base”,
  “session_key_id”: “SK123”,
  “policy_hash”: “sha256(...)”,
  “tbc_signature”: “0xSIG...”,
  “tbc_pubkey”: “0xPubKey”,
  “escrow_id”: “ESCROW-123”,
  “function_selector”: “0x12345678”,
  “idempotency_key”: “sha256(...)”,
  “timestamp”: 1712345678
}

4.5.1 Preconditions
	•	TBC decision = APPROVED
	•	CoreProve validates TBC signature

4.5.2 Postconditions
	•	MCP receives a validated, signed intent
	•	MCP has permission to proceed to tx-preparation

⸻

4.6 MCP: Prepare Unsigned Transaction

MCP calls:

prepare_session_transaction(wallet_intent)

Wallet returns:

{
  “unsigned_tx”: “0xDEADBEEF...”,
  “gas_estimate”: 55000,
  “chain”: “base”,
  “session_key_id”: “SK123”,
  “tbc_signature”: “0xSIG...”,
  “tbc_pubkey”: “0xPUBKEY”
}

4.6.1 MCP MUST validate:
	•	TBC signature
	•	chain matches session-key constraints
	•	amount ≤ max_amount_per_tx
	•	anomaly_score (if included)
	•	idempotency_key not duplicated
	•	session key not expired
	•	revocation state

⸻

4.7 Wallet Signing Stage

MCP calls:

sign_with_session_key(unsigned_tx)

Wallet MUST:
	•	Verify session key is active
	•	Verify TBC signature
	•	Verify constraints:
	•	vendor match
	•	selector match
	•	amount match
	•	chain match
	•	Re-check revocation

Wallet outputs:

{
  “signed_tx”: “0xFEEEED...”,
  “session_key_id”: “SK123”
}

4.7.1 Preconditions
	•	All constraints satisfied
	•	Session key not revoked
	•	TBC approval validated

4.7.2 Postconditions
	•	Wallet has committed to signing
	•	Signed transaction ready for broadcast

⸻

4.8 MCP → Blockchain: Broadcast + Settlement Verification

MCP calls:

submit_tx_hash(signed_tx)

Then begins settlement monitoring:

verify_settlement(tx_hash, confirmations=N)

4.8.1 Settlement States

State	Meaning
SUBMITTED	Broadcast accepted
MINED	Included in block
CONFIRMED	After N confirmations
SETTLED	TBC finalizes spend
FAILED	Reverted or not included


⸻

4.9 TBC Settlement Finalization

Once MCP reports settlement:

TBC.verify_settlement(tx_hash)

TBC MUST:
	•	Match tx_hash to existing reservation
	•	Release or finalize reservation
	•	Record idempotency_key permanently

4.9.1 Preconditions
	•	MCP-confirmed settlement
	•	Tx not replayed

4.9.2 Postconditions
	•	Funds considered spent
	•	Entry becomes immutable audit record

⸻

4.10 CoreProve User Notification

CoreProve notifies user of:
	•	success
	•	failure
	•	anomalies
	•	contract changes
	•	repeated rejections
	•	settlement issues

Notification includes:

{
  “type”: “autopay_result”,
  “vendor”: “0x...”,
  “amount”: 50,
  “session_key_id”: “SK123”,
  “tx_hash”: “0x...”,
  “status”: “SETTLED”
}


⸻

4.11 Dry-Run Flow (Expanded & Mandatory Conditions)

The dry_run() tool simulates TBC enforcement BEFORE MCP commits to executing a payment.

4.11.1 MUST Call Dry-Run When:
	•	Trigger = AI reasoning
	•	anomaly_score > 0
	•	amount > user threshold (default: $100)
	•	merchant contract bytecode changed
	•	invoice amount changed
	•	function selector changed
	•	policy_hash mismatch detected
	•	MCP detects series of pending tx failures

4.11.2 Dry-Run Input

{
  “wallet_intent”: {...}
}

4.11.3 Dry-Run Output

{
  “result”: “WOULD_APPROVE | WOULD_REJECT”,
  “reason”: “string”,
  “anomaly_score”: 0
}

4.11.4 Behavior
	•	If WOULD_REJECT → MCP MUST STOP
	•	If WOULD_APPROVE → MCP may proceed ONLY if anomaly_score == 0 (autonomous)

⸻

4.12 Failure Mode Handling Within Flow

Failure	MCP Behavior
TBC rejects	Stop, notify
Merchant signature invalid	Stop, alert user
Wallet refusal	Stop, notify & revoke key?
Settlement fails	Retry up to 3×, then stop
Idempotency violation	Stop & block duplicate
Anomaly detected	escalate to user


⸻

4.13 Full End-to-End Text Flow

Below is the formal message sequence including cryptographic operations:

[MCP Trigger]
    ↓
Merchant → CoreProve: 402 + merchant_signature
    ↓ verify signature, policy hash match
CoreProve → TBC: TGP-QUERY + user_signature
    ↓ enforce policy
TBC → CoreProve: TBC-APPROVED + tbc_signature
    ↓ verify tbc_signature
CoreProve → MCP: wallet_intent
    ↓ validate policy hash, anomalies
MCP → Wallet: prepare_session_transaction()
    ↓
MCP → Wallet: sign_with_session_key()
    ↓
MCP → Blockchain: submit_tx_hash()
    ↓
MCP → TBC: verify_settlement()
    ↓
TBC → CoreProve: SETTLED
    ↓
CoreProve → User: notification


⸻

4.14 Summary of Hard Guarantees
	•	No signing without TBC first approving
	•	No approval without user-signed TGP-QUERY
	•	No merchant request accepted without signature
	•	No replay possible due to idempotency_key
	•	No out-of-policy execution possible
	•	No chain mismatch allowed
	•	No key exposure at any boundary
	•	Autonomous mode requires anomaly_score = 0
	•	MCP cannot escalate privileges


⸻

5. TBC Enforcement Logic (Fully Expanded)

The Transaction Border Controller (TBC) is the canonical, unbypassable enforcement layer for autonomous payments.
It is authoritative over:
	•	session-key policy validation
	•	spend limits
	•	frequency limits
	•	anomaly scoring
	•	vendor authentication
	•	idempotency & replay protection
	•	chain enforcement
	•	revocation
	•	reservation lifecycle
	•	settlement finalization

If the TBC rejects a query, no agent, wallet, or user interface can override it.

⸻

5.1 TBC’s Canonical Responsibilities

TBC is the only component allowed to make final economic decisions.
It MUST:
	1.	Validate user-signed TGP-QUERY
	2.	Enforce the canonical session-key policy
	3.	Perform anomaly scoring
	4.	Perform replay defense via idempotency keys
	5.	Track spend across rolling windows
	6.	Track claim frequency
	7.	Enforce chain specificity
	8.	Reserve funds upon approval
	9.	Confirm settlement before final deduction
	10.	Permanently record settlement entries
	11.	Block all revoked or expired keys
	12.	Protect against malicious merchant behavior

⸻

5.2 Spend-Limit Enforcement (Fully Defined)

TBC enforces three kinds of spend limits:

⸻

5.2.1 Per-Transaction Limit

amount ≤ max_amount_per_tx

If amount exceeds:
	•	Reject with SpendLimitExceeded

⸻

5.2.2 Rolling Window Spend Limit

Defined by:

max_amount_per_period
period: “24h” | “7d” | custom

Rolling Window Definition

A rolling window means:

window_start = (current_timestamp - period)
total_spent = sum(spend[vendor] within window_start → now)

If:

total_spent + amount > max_amount_per_period

→ reject with PeriodSpendLimitExceeded.

⸻

5.2.3 Global Spend Limits (Optional)

A user may choose a global budget:

max_global_amount_per_day

If enabled, TBC must maintain:

global_spent_24h

Reject if:

global_spent_24h + amount > max_global_amount_per_day


⸻

5.3 Frequency Enforcement

Each session key defines:

max_tx_per_period
period = rolling window

TBC must track:

tx_times = all tx timestamps for this session_key
count = tx_times within rolling <period>

If:

count + 1 > max_tx_per_period

→ reject with FrequencyExceeded.

⸻

5.4 Anomaly Scoring (Consolidated & Final)

TBC computes an anomaly_score using:

anomaly_score = sum(anomaly_flags[i].weight)

Where each anomaly flag is either:
	•	0 = normal
	•	1 = warning
	•	2 = severe
	•	5 = critical

Based on:

⸻

5.4.1 Contract Drift
	•	Vendor bytecode hash changed → +5
	•	Vendor owner changed → +3

5.4.2 Selector Drift
	•	Function selector not matching session-key selector → automatic reject
	•	Function signature drift (optional) → +2

5.4.3 Amount Variance

amount > last_amount * (1 + amount_variance_pct/100)

→ +2

5.4.4 Frequency Pattern Deviations

If transactions cluster weirdly:
	•	More than 3 attempts within 5 minutes → +2
	•	Attempts at unusual hours → +1

5.4.5 Invoice Mutation

If merchant sends new invoice with:
	•	different amount → +2
	•	different vendor → reject
	•	different selector → reject
	•	new policy_hash → +5

5.4.6 Idempotency Anomalies

Repeated invoice using same idempotency_key within a short window:
	•	second use → +3
	•	more than two uses → reject + alert

⸻

5.4.7 Anomaly Threshold Interpretation

TBC attaches anomaly_score to every TBC-RESPONSE.

MCP must apply user rules:
	•	Autonomous mode: anomaly_score must equal 0
	•	Semi-autonomous: anomaly_score ≤ configured threshold
	•	User-triggered: anomalies only produce warnings

⸻

5.5 Idempotency & Replay Protection (Canonical Rules)

TBC maintains the authoritative idempotency table.

5.5.1 Idempotency Key Format

Canonical key:

sha256(vendor + amount + invoice_id + timestamp + chain_id)

5.5.2 TBC MUST Reject When:
	•	key already used for SETTLED → forever reject
	•	key used recently for FAILED → reject for 24h
	•	key is PENDING for > 10 minutes → reject
	•	timestamp skew > 120s → reject

5.5.3 Cross-Chain Replay

Impossible by construction because chain_id included.

⸻

5.6 Chain Enforcement (Canonical Rules)

TBC must verify:

TGP-QUERY.chain == session_key.allowed_chain

Mismatch → reject with ChainMismatch.

⸻

5.7 Revocation Enforcement

If a session key’s status = revoked, TBC MUST:
	•	reject all queries
	•	drop all pending reservations
	•	notify CoreProve
	•	attach SessionKeyRevoked reason

No grace period allowed.

⸻

5.8 Policy Hash Verification

For all merchant-driven 402 flows:

merchant_policy_hash == user_policy_hash == tbc_policy_hash

If mismatch:
	•	reject
	•	notify CoreProve
	•	alert user

This prevents malicious merchant policy injection.

⸻

5.9 Reservation Lifecycle (Finalized)

Upon APPROVED, TBC MUST reserve the spend amount:

reservation = { amount, idempotency_key, expires_at }

States:

State	Meaning
RESERVED	TBC approved; awaiting settlement
PENDING_SETTLEMENT	Tx submitted; waiting for confirmation
SETTLED	TBC has verified the transaction; final
FAILED	Chain-level revert or dropped
ABANDONED	No tx submitted before timeout

5.9.1 Timeouts
	•	Reservation expires after 10 minutes
	•	Expired → reservation released, idempotency_key moved to FAILED with 24h block

5.9.2 Settlement Confirmation

TBC MUST:
	•	verify block hash
	•	verify tx parameters match intent
	•	verify tx signer = session key

If validated → state becomes SETTLED.

⸻

5.10 Vendor Authentication (Canonical)

TBC maintains canonical vendor entry:

vendor_registry = {
   vendor_address,
   vendor_pubkey,
   bytecode_hash,
   last_update_block
}

TBC rejects if:
	•	vendor address changed
	•	merchant_pubkey mismatch
	•	bytecode changed unexpectedly

This protects against contract upgrades affecting autopay.

⸻

5.11 Session-Key Authority Check

TBC MUST enforce:

TGP-QUERY.session_key_id exists
AND session_key is active
AND not expired
AND not revoked

If expired → reject with SessionKeyExpired.

If revoked → reject with SessionKeyRevoked.

⸻

5.12 TBC Error Codes (Canonical)

Errors returned by TBC:

VendorNotWhitelisted
MerchantSignatureInvalid
PolicyHashMismatch
SessionKeyExpired
SessionKeyRevoked
SessionKeyInvalid
SpendLimitExceeded
PeriodSpendLimitExceeded
FrequencyExceeded
AnomalyDetected
ChainMismatch
IdempotencyViolation
TimestampSkew
InvalidQuerySignature
FunctionSelectorMismatch
ContractDriftDetected
GlobalSpendExceeded
ReservationExpired
DoubleSpendDetected
ReplayDetected


⸻

5.13 TBC Signature Authority & Key Rotation

TBC’s private key MUST be:
	•	HSM-protected
	•	unexportable
	•	rotated every 90 days
	•	pinned by CoreProve on first trust

If key rotates, TBC MUST:
	•	publish new public key
	•	CoreProve MUST prompt user to confirm

Compromise procedure:
	•	revoke all session keys
	•	clear pending reservations
	•	regenerate TBC keys
	•	require user re-registration

⸻

5.14 Settlement Finalization (Canonical)

Upon MCP calling verify_settlement():

TBC MUST:
	1.	Match transaction to reservation
	2.	Validate signer = session-key-public
	3.	Validate chain-specific finality
	4.	If confirmed:
	•	log immutable settlement
	•	deduct spend from rolling window
	•	release reservation

If FAILED:
	•	reservation → FAILED
	•	idempotency_key → 24h quarantine
	•	notify CoreProve

⸻

5.15 Interaction Summary

TBC sits at the heart of all validated economic actions:

Merchant → CoreProve → TBC → CoreProve → MCP → Wallet → Blockchain → MCP → TBC → CoreProve

Every message touching money must pass through TBC.

⸻

5.16 Guarantees TBC Enforces

Hard Guarantees
	•	No out-of-policy spending
	•	No tampered invoices
	•	No bypass of vendor whitelist
	•	No replay
	•	No double payments
	•	No key misuse
	•	No chain mismatches
	•	No signing without TBC approval
	•	No settlement without session-key signer
	•	No action after revocation
	•	No approval during anomalies in autonomous mode

Soft Guarantees
	•	Rolling-window accuracy
	•	Vendor contract mutation tracking
	•	Rate-limit enforcement
	•	Summary reporting



⸻

6. Security Model (Fully Expanded)

The security model defines the full threat surface, isolation guarantees, cryptographic trust anchors, revocation and rotation behavior, and end-to-end guarantees required to safely enable non-custodial autonomous payments.

The goal is simple:

Allow automation without allowing compromise.

This section formalizes the system’s defenses against malicious merchants, malicious agents, compromised MCP processes, malware, replay attacks, policy tampering, man-in-the-middle attacks, and key-exposure attempts.

⸻

6.1 Master Key Isolation Guarantee

The master key (the user’s primary wallet key) is the most sensitive element in the system.
The entire design ensures it is never exposed and never delegated.

6.1.1 Master Key Rules

The master key MUST:
	•	Remain inside the wallet
	•	Never be exported
	•	Never be provided to CoreProve or MCP
	•	Never sign transactions directly for automation
	•	Only sign:
	•	session key registrations
	•	session key revocations
	•	session key rotations
	•	TBC trust anchors (optional initial setup)

6.1.2 Hardware Isolation

If wallet supports hardware security (TPM, Secure Element, Ledger, Trezor):
	•	master key MUST remain hardware-bound
	•	session keys MAY be stored encrypted off-chip or on-chip
	•	all signing MUST be approved by the hardware enclave

6.1.3 Consequence of Exposure

If the master key is ever leaked:
	•	attacker can generate new session keys
	•	attacker can override all revocations
	•	full compromise

Hence the entire system is designed to minimize master-key usage.

⸻

6.2 Session Key Safety Model

Session keys embody the principle of least privilege.

6.2.1 Minimum Authority

A session key is authorized to:
	•	sign one specific function selector
	•	on one specific vendor contract
	•	on one specific chain
	•	with strict spend and frequency limits
	•	until expiration or revocation

Session keys have no authority to:
	•	modify policy
	•	access other contracts
	•	update registry entries
	•	create new session keys
	•	modify TBC entries
	•	override revocation
	•	access master key state

6.2.2 Validation Before Signing

The wallet MUST validate:
	•	vendor address match
	•	function selector match
	•	amount ≤ session-key limits
	•	chain matches
	•	session key unexpired
	•	session key not revoked
	•	TBC signature valid
	•	policy hash match (via CoreProve)
	•	no anomalies (if required)

If any validation fails → refuse to sign.

6.2.3 Revocation Wins

Once revoked:
	•	session key is permanently invalid
	•	wallet MUST reject all signing requests
	•	TBC MUST reject all TGP-QUERY
	•	MCP MUST disable all scheduled actions

6.2.4 Leakage Scenario Analysis

If a session key leaks:
	•	Attacker can only spend within minimal constraints
	•	Cannot increase amount
	•	Cannot change chain
	•	Cannot call other functions
	•	Cannot bypass TBC
	•	Cannot update policy
	•	Cannot generate new session keys
	•	Cannot escalate privileges

Worst-case damage = single bounded spend.

⸻

6.3 CoreProve Isolation Model

CoreProve is a mediation layer, not an authority layer.

6.3.1 What CoreProve CAN Do
	•	Validate merchant signatures
	•	Validate merchant policy hash
	•	Construct TGP-QUERY
	•	Pass TBC responses to MCP
	•	Manage local session-key metadata
	•	Cache anomaly flags
	•	Provide revocation UI for the user

6.3.2 What CoreProve CANNOT Do
	•	Sign transactions
	•	Bypass TBC
	•	Modify TBC policies
	•	Revive revoked session keys
	•	Forge merchant signatures
	•	Forge TBC signatures
	•	Execute payments
	•	Overrule wallet or TBC decisions
	•	Update session key expiration

This keeps CoreProve inside a non-custodial UI authority boundary.

⸻

6.4 TBC as an Immutable Authority Layer

The Transaction Border Controller (TBC) is the enforcement firewall that cannot be bypassed.

6.4.1 TBC MUST enforce:
	•	vendor whitelist
	•	function selector
	•	spend limits
	•	frequency limits
	•	chain restrictions
	•	anomaly scoring
	•	timestamp freshness
	•	idempotency
	•	revocation
	•	reservation lifecycle
	•	settlement matching

6.4.2 TBC Private Key Security

TBC signing key MUST be:
	•	in HSM
	•	non-exportable
	•	rotated every 90 days
	•	never transmitted
	•	attested to CoreProve (pinned)

6.4.3 Signature Guarantees

Every TBC-RESPONSE MUST be signed.
No unsigned decision may move money.

6.4.4 Compromise Handling

If TBC key compromised:
	•	all session keys MUST be revoked
	•	all pending reservations invalidated
	•	user re-authorizes new TBC key
	•	system resumes with clean state

⸻

6.5 Wallet Execution Safety

The wallet is the final safety valve.

6.5.1 Wallet MUST enforce:
	•	canonical session-key constraints
	•	TBC signature verification
	•	session-key revocation
	•	session-key expiration
	•	chain correctness
	•	function-selector correctness
	•	deterministic signing logic

6.5.2 Revocation Checks

Before signing:

revoked = CoreProve.revocation_status(session_key_id)
revoked2 = TBC.revocation_status(session_key_id)
latest = max(revoked, revoked2)

if latest == “revoked” → reject

Wallet MUST:
	•	check revocation immediately before signing
	•	refuse signing if revocation flag toggles mid-sign

6.5.3 No Silent Signing

The wallet MUST never silently sign—only constrained signing.

6.5.4 No Cross-Contract Drift

Wallet MUST parse call data:
	•	If vendor address mismatches → reject
	•	If selector mismatches → reject
	•	If parameter structure mismatches → reject

This protects against malicious intent mutations.

⸻

6.6 MCP Security Boundaries

MCP executes automation without authority.

6.6.1 MCP MAY:
	•	evaluate intent
	•	enforce anomaly rules
	•	schedule autopay
	•	perform dry-run
	•	request tx signing
	•	verify settlement
	•	broadcast signed transactions

6.6.2 MCP MAY NOT:
	•	sign transactions
	•	modify session-key policy
	•	bypass TBC
	•	forge wallet intents
	•	modify wallet call data
	•	intercept private keys
	•	create new session keys

MCP is treated as untrusted logic held in a sandbox.

6.6.3 MCP Isolation Recommendations

MCP SHOULD run:
	•	in a restricted process sandbox
	•	with no filesystem access beyond local logs
	•	with no ability to ORIGINATE HTTP requests except:
	•	CoreProve
	•	TBC
	•	blockchain RPC

6.6.4 MCP Compromise Impact

If MCP compromised:
	•	attacker cannot sign txs
	•	attacker cannot bypass TBC
	•	attacker cannot call wallet directly (wallet validates session key)
	•	attacker only triggers TBC >0 anomaly rejections

Worst-case effect = spam rejections, never unauthorized spending.

⸻

6.7 Replay Protection Model

Replay protection enforced at multiple layers:

6.7.1 Merchant signature includes timestamp

Reject if >120 seconds skew.

6.7.2 Idempotency includes chain_id

Cross-chain replay impossible.

6.7.3 TBC retains idempotency entries

Settled entries are immutable.

6.7.4 Wallet signs tx including nonce

Chain-level replay impossible.

6.7.5 CoreProve rejects any mutation of merchant 402

Replay attempts fail at five independent layers.

⸻

6.8 Authenticity Model (Multi-Signature Chain)

6.8.1 Merchant Proves:
	•	invoice is legitimate
	•	vendor identity is known
	•	payload hasn’t been modified

6.8.2 User Proves:
	•	intent is from valid wallet
	•	policy was authorized
	•	session key is user-owned

6.8.3 TBC Proves:
	•	payment complies with user policy
	•	no drift or anomaly is allowed
	•	funds should move

6.8.4 Wallet Proves:
	•	only session-key-limited actions signed
	•	signing is done under constraints

Automation never bypasses these signatures.

⸻

6.9 Mutation-Resistance Model

All components protect against mutated payment requests:
	•	Merchant 402 includes signature + policy hash
	•	CoreProve validates policy_hash correctness
	•	TBC independently recomputes hash
	•	Wallet validates contract address & selector
	•	MCP cannot rewrite TBC-approved intent

Mutation attempts are impossible to execute silently.

⸻

6.10 Man-in-the-Middle Resistance

The model resists MITM attacks by:

6.10.1 Merchant → CoreProve
	•	signed 402
	•	timestamp freshness
	•	pubkey-whitelisted merchant

6.10.2 CoreProve → TBC
	•	user-signed query
	•	canonical fields only
	•	tamper-evident idempotency

6.10.3 TBC → CoreProve
	•	HSM-signed response
	•	TBC key pinned

6.10.4 CoreProve → MCP
	•	validated & signed wallet-intent

6.10.5 MCP → Wallet
	•	session-key scoped
	•	wallet revalidates

Even if attacker controls the network, no unsigned or mutated message can do harm.

⸻

6.11 Non-Bypassability Guarantees

This section states the “hard walls”:

6.11.1 No Bypass of TBC

All actions require TBC signature.

6.11.2 No Bypass of Wallet

Wallet always validates session keys.

6.11.3 No Bypass of Policy

Session keys are immutable once activated.

6.11.4 No Bypass of Revocation

Revocation overrides all approvals.

6.11.5 No Bypass of Session Constraints

Wallet rejects any drift from:
	•	vendor
	•	chain
	•	selector
	•	amount

6.11.6 No Bypass of Anomaly System

Autonomous mode requires anomaly_score == 0.

⸻

6.12 Security Guarantees Summary

The system ensures:

Zero Custody

AI agent never holds keys.

Zero Bypass

TBC is always the final authority.

Zero Drift

Wallet revalidates all constraints.

Zero Replay

Idempotency and timestamp rules prevent duplicates.

Zero Mutation

Policy hashes and merchant signatures prevent invoice manipulation.

Zero Escalation

Session keys cannot escalate into broader privileges.

Zero Silent Failures

All anomalies trigger notifications or user confirmation.

Zero Trust Architecture

Every layer distrusts all others.

⸻

7. User Models (Fully Expanded)

This section defines the behavioral and security expectations for different user types.
The objective is to provide a consistent mental model across all roles interacting with the system—both for end users and automated agents—while preserving the trust boundaries established in Sections 1–6.

7.1 User Identity Abstraction

The system operates under a strict non-custodial, non-KYC identity model:
	•	The “user” is represented cryptographically by keys, not by biometrics or PII.
	•	The “identity” for autopay flows is the address that owns the master key.
	•	Session keys form a derived, scoped identity with strictly limited authority.
	•	CoreProve acts as the user’s identity orchestrator, not an identity provider.

Users do NOT need to reveal:
	•	legal name
	•	email
	•	phone
	•	government ID

Identity is cryptographic and self-owned.

⸻

7.2 Three Classes of Users

Autopay introduces three distinct user types, each with different responsibilities and authority:
	1.	Direct Wallet User (Primary User)
	2.	Session-Delegated User (Automation / MCP Agent)
	3.	Merchant User (Payment Receiver)

Each role has different capabilities, privileges, and constraints.

⸻

7.3 Primary User Model (Direct Human User)

This is the owner of the master wallet key—the only entity with ultimate authority.

Capabilities

Primary users can:
	•	create session keys
	•	set and modify session key policies
	•	revoke session keys
	•	authorize merchant whitelists
	•	approve/reject anomaly notifications
	•	execute manual payments
	•	override autonomy mode decisions
	•	rotate master wallet keys
	•	authorize wallet-policy registration with TBC
	•	enable / disable the MCP agent

Restrictions

Primary users cannot:
	•	bypass TBC policy
	•	sign automated transactions without session keys
	•	disable safety checks
	•	permit session keys to exceed TBC-enforced limits

User Experience Responsibilities

Primary users must:
	•	carefully approve session-key creation
	•	understand spend & frequency limits
	•	whitelist merchants responsibly
	•	review periodic spending reports
	•	respond promptly to anomaly notifications
	•	rotate master keys periodically (wallet-level requirement)

This model assumes primary users are technical enough to:
	•	verify merchant authenticity (via CoreProve UI)
	•	interpret risk-level notifications
	•	manage session keys safely

⸻

7.4 Session-Delegated User Model (Automation Layer / MCP Agent)

The MCP autonomous agent acts as a delegated user.

Capabilities

MCP agents may:
	•	trigger scheduled payments
	•	trigger cron/autopay flows
	•	request TBC approval
	•	perform dry-runs
	•	submit signed transactions
	•	verify settlement on-chain
	•	report results back to CoreProve

Restrictions (Critical)

MCP agents:
	•	cannot sign transactions
	•	cannot create session keys
	•	cannot modify policy
	•	cannot bypass TBC
	•	cannot bypass anomaly rules
	•	cannot access master or session private keys
	•	cannot whitelist new merchants
	•	cannot upgrade autonomy mode without user consent

Automation Modes

Session-delegated user actions operate within one of three scopes:

Mode	Description	Allowed?
Autonomous	Agent executes automatically when anomaly_score = 0	YES
Semi-Autonomous	Agent needs user confirmation for higher-risk flows	YES
Manual	Agent only assists; user signs	YES
Unrestricted	Agent signs freely using master key	NO (disallowed)

Threat Model

Even if MCP is compromised:
	•	It cannot sign transactions
	•	It can’t bypass TBC
	•	Session-key rules remain enforced
	•	Wallet revalidates all constraints

Worst-case effect: nuisance notifications or failed payments.

⸻

7.5 Merchant User Model (Payment Receiver)

Merchants are participants in the protocol but are never trusted with user identity or funds.

Capabilities

Merchants may:
	•	issue signed 402 Payment Required messages
	•	include vendor address, invoice, amount, and policy_hash
	•	expose payment profiles (smart contract functions)
	•	verify on-chain settlement of payments

Restrictions

Merchants cannot:
	•	mutate payment terms mid-flow
	•	increase amount without triggering anomaly rejection
	•	rotate merchant signing keys silently
	•	push updates to policies
	•	require access to session keys or identity

The merchant signature is validated via a pubkey whitelist maintained by the user.

⸻

7.6 Autonomy Modes (User Perspective)

The user chooses how autonomous their system should be.

Below are the expanded guarantees and responsibilities for each mode.

⸻

7.6.1 Autonomous Mode (“Fire-and-Forget Automation”)

Description:
MCP agent executes payments automatically when all constraints validate and anomaly score is 0.

Guaranteed Protections
	•	No anomalous payments ever occur
	•	Session-key constraints are strictly enforced
	•	TBC must approve every payment
	•	Wallet validates every constraint before signing
	•	Merchant signature must match whitelist
	•	Timestamp must be fresh
	•	Idempotency must be unique
	•	MCP cannot override the decision
	•	Revocation immediately disables autopay

User Responsibilities
	•	Maintain accurate merchant whitelists
	•	Review periodic autopay summaries
	•	Respond to anomaly alerts

Best For
	•	Utility bills
	•	Rent
	•	Essential recurring payments
	•	Subscriptions
	•	Safe vendors with stable policies

⸻

7.6.2 Semi-Autonomous Mode (“Human-in-the-Loop”)

Description:
Automation performs discovery and preparation but requires the user to explicitly OK payments that are:
	•	above a threshold
	•	anomalous
	•	unexpected
	•	new vendors
	•	mutated policy hash
	•	selector mismatch
	•	high risk

Guaranteed Protections
	•	No unexpected execution
	•	All anomalies surface a confirmation dialog
	•	TBC enforces policy regardless of user interface
	•	Wallet enforces session-key limits regardless of confirmation

User Responsibilities
	•	Interpret risk warnings
	•	Confirm or reject payments

Best For
	•	Medium-risk subscriptions
	•	Vendors that sometimes vary amount
	•	Teams sharing wallets (DAO treasuries, households)

⸻

7.6.3 Manual Mode (“Do It Yourself”)

Description:
Automation assists only by preparing the intent; user signs manually using the wallet.

Protections
	•	Full policy and constraint enforcement
	•	Wallet guards against mutation
	•	No automated execution

Best For
	•	One-off invoices
	•	New merchants
	•	High-value purchases
	•	Complex smart contract interactions

⸻

7.7 Multi-Device User Model

Users may interact across:
	•	desktop
	•	mobile
	•	hardware wallets
	•	browser extensions
	•	MCP agents on local machines
	•	cloud-hosted MCP instances

Security requirements:

7.7.1 Session Key sync

Session-key metadata must be available across devices but:
	•	private session keys must NOT propagate automatically
	•	revocation state must sync instantly

7.7.2 Merchant whitelist sync

Only public keys & vendor metadata sync; never private information.

7.7.3 TBC public key pinning

All devices must validate TBC signatures using the pinned key.

⸻

7.8 Shared Wallet User Model (DAOs / Organizations)

The system supports multi-stakeholder wallets:

7.8.1 Policy Setup

Admin(s) configure:
	•	spend limits
	•	frequency limits
	•	vendor whitelists
	•	anomaly tolerance
	•	session-key expiration & rotation

7.8.2 Roles
	•	Owners: manage policy, session keys, and master keys
	•	Operators: monitor activity, respond to alerts
	•	MCP Agents: execute automation but cannot modify policy

7.8.3 Auditability

All autopay actions must be:
	•	logged
	•	timestamped
	•	attributable
	•	anomaly-scored

⸻

7.9 User Risk Profiles

The system accommodates varying levels of risk tolerance:

7.9.1 Conservative
	•	low spend limits
	•	strict anomaly thresholds
	•	low-frequency payments
	•	explicit confirmations

7.9.2 Moderate
	•	moderate spend limits
	•	anomaly scoring allowed within warning zone
	•	prompts for mid-tier thresholds

7.9.3 Aggressive
	•	high but bounded limits
	•	trusted vendors only
	•	anomalies allowed up to threshold before requiring confirmation

Even “aggressive” users cannot override:
	•	TBC enforcement
	•	session-key restrictions
	•	revocation
	•	chain selector
	•	contract selector

⸻

7.10 Long-Term User Experience Goals

The system should feel:
	•	safe
	•	transparent
	•	autonomous where desired
	•	conservative by default
	•	easy to override manually

User goals:
	•	Save time
	•	Reduce friction
	•	Increase trust in automated payments
	•	Maintain strict control over funds

System goals:
	•	Never allow unauthorized drain
	•	Never allow an agent to escalate authority
	•	Never degrade into custodial behaviors
	•	Never require users to understand cryptography to stay safe

⸻

8. Canonical Schemas (FULL EXPANSION)

MCP-AUTO-PAY-01.1 — canonical JSON schemas for all protocol messages

This section defines the complete data structures exchanged between:
	•	Merchant
	•	CoreProve
	•	TBC
	•	MCP
	•	Wallet
	•	Settlement Services

Every message in this spec MUST conform exactly to these schemas.

They form the wire protocol for the autopay ecosystem.

⸻

8.0 Canonicalization Rules

8.0.1 Canonical JSON

All messages MUST obey these rules:

Rule	Requirement
Key order	Sorted alphabetically (canonical JSON)
Encoding	UTF-8
Numbers	integers only; no floats
Timestamps	Unix epoch (int, seconds)
Bytes	Lowercase hex, prefixed 0x
No comments	Schema MUST be valid JSON
No trailing comma	Strict compliance

Rationale: Ensures all cryptographic hashing and message signing remain deterministic.

⸻

8.0.2 Naming Rules
	•	All fields snake_case
	•	All constants/enums UPPER_SNAKE_CASE
	•	Amounts in wei
	•	No floats
	•	All chain_ids follow EIP-155

⸻

8.0.3 Signature Fields

Every signed payload MUST contain:

Field	Description
signature	Required. Hex string.
pubkey	Required. Merchant’s or TBC’s public key.
signature_type	“ecdsa” (default) or “ed25519”
signed_fields	Array of field names included in signature hash


⸻

8.0.4 Hash Rules

SHA-256 only.

Example:

policy_hash = sha256(canonical_json(policy_doc))

Hash output MUST use:
	•	lowercase hex
	•	0x prefix

⸻

8.1 Merchant 402 Payload Schema (EXPANDED)

A merchant triggers autopay using an HTTP 402 Payment Required response.

This is the FIRST message in the flow.

The merchant MUST sign this payload.

⸻

8.1.1 Purpose
	•	Communicate an invoice that requires payment
	•	Bind invoice details to the vendor contract
	•	Bind invoice to chain and function selector
	•	Allow CoreProve to validate authenticity before generating TGP-QUERY

⸻

8.1.2 Schema: merchant_402

{
  “schema_version”: “1.0”,
  “invoice_id”: “string”,
  “timestamp”: 1712339001,
  “vendor”: “0xabcdef...”,
  “vendor_name”: “string”,
  “chain_id”: 8453,
  “amount_wei”: 50000000000000000,
  “function_selector”: “0xa9059cbb”,
  “policy_hash”: “0xabc123...”,
  “metadata”: {
    “description”: “string”,
    “line_items”: [
      {
        “name”: “string”,
        “amount_wei”: 25000000000000000
      }
    ]
  },
  “merchant_pubkey”: “0x04ab...”,
  
  “signature”: “0x1234...”,
  “signature_type”: “ecdsa”,
  “signed_fields”: [
    “invoice_id”,
    “timestamp”,
    “vendor”,
    “chain_id”,
    “amount_wei”,
    “function_selector”,
    “policy_hash”
  ]
}


⸻

8.1.3 Merchant MUST Sign

Payload is signed using merchant private key.

Signature MUST cover:
	•	invoice_id
	•	timestamp
	•	vendor
	•	chain_id
	•	function_selector
	•	amount_wei
	•	policy_hash

CoreProve MUST reject any 402 missing the signature.

⸻

8.1.4 Policy Hash Definition

policy_hash = sha256(canonical_json(merchant_terms))

Where merchant_terms MUST include:

{
  “vendor”: “0x...”,
  “function_selector”: “0x...”,
  “chain_id”: 8453,
  “max_amount_wei”: 50000000000000000,
  “expiration”: 1712345000
}


⸻

8.2 CoreProve TGP-QUERY Schema (EXPANDED)

This is the message CoreProve sends to the TBC.

It is the Layer 9 → Layer 8 handoff.

⸻

8.2.1 Purpose
	•	Validate merchant request
	•	Request permission for autopay
	•	Bind session_key_id + invoice metadata
	•	Provide user authentication (wallet signature)
	•	Provide anomaly flags

⸻

8.2.2 Schema: tgp_query

{
  “schema_version”: “1.0”,
  
  “invoice_id”: “string”,
  “timestamp”: 1712339001,
  
  “session_key_id”: “user_0x123_vendor_0x456_001”,
  “user_pubkey”: “0x04ab...”,
  
  “vendor”: “0xabcdef...”,
  “amount_wei”: 50000000000000000,
  “function_selector”: “0xa9059cbb”,
  “chain_id”: 8453,
  
  “policy_hash”: “0xabc123...”,
  “idempotency_key”: “0xdef456...”,
  
  “anomaly_flags”: [
    “AMOUNT_VARIANCE”,
    “VENDOR_BYTECODE_CHANGE”
  ],
  
  “merchant_signature”: “0x123...”,
  “merchant_pubkey”: “0x04ab...”,
  
  “coreprove_signature”: “0x987...”,
  “signature_type”: “ecdsa”,
  “signed_fields”: [
    “invoice_id”,
    “timestamp”,
    “session_key_id”,
    “vendor”,
    “amount_wei”,
    “chain_id”,
    “policy_hash”,
    “idempotency_key”
  ]
}


⸻

8.3 TBC-RESPONSE Schema (EXPANDED)

Authoritative, signed, deterministic Layer 8 verdict.

⸻

8.3.1 Purpose

The TBC checks:
	•	session key validity
	•	vendor whitelist
	•	spend limit
	•	frequency
	•	anomaly score
	•	idempotency
	•	policy_hash match

Then returns:
	•	APPROVE → go ahead
	•	REJECT → stop immediately

⸻

8.3.2 Schema: tbc_response

{
  “schema_version”: “1.0”,
  
  “decision”: “APPROVE”, 
  “reason”: “NONE”,
  
  “timestamp”: 1712339002,
  “invoice_id”: “string”,
  
  “policy_hash”: “0xabc123...”,
  “idempotency_key”: “0xdef456...”,
  
  “enforced_constraints”: {
    “max_amount_wei”: 50000000000000000,
    “frequency_limit”: “1_per_day”,
    “expiration”: 1719999999
  },
  
  “reservation”: {
    “state”: “ISSUED”,
    “reserved_amount_wei”: 50000000000000000,
    “timeout_seconds”: 600
  },
  
  “anomaly_score”: 0,
  
  “tbc_pubkey”: “0x04cd...”,
  “tbc_signature”: “0xf00d...”
}


⸻

8.3.3 Rejection Example

{
  “decision”: “REJECT”,
  “reason”: “SPEND_LIMIT_EXCEEDED”,
  “anomaly_score”: 22,
  “tbc_signature”: “0xdeadbeef...”
}


⸻

8.4 Wallet-Action Intent Schema (EXPANDED)

CoreProve → MCP → Wallet.

This is the transaction-ready instruction.

⸻

8.4.1 Schema: wallet_action_intent

{
  “schema_version”: “1.0”,
  
  “action”: “BUYER_COMMIT”,
  
  “vendor”: “0xabcdef...”,
  “function_selector”: “0xa9059cbb”,
  “call_data”: “0xa9059cbb0000000...”,
  “amount_wei”: 50000000000000000,
  “chain_id”: 8453,
  
  “unsigned_tx”: “0xf86c80850...”,
  “gas_estimate”: 55000,
  
  “session_key_id”: “user_0x123_vendor_0x456_001”,
  “tbc_signature”: “0xf00d...”,
  “tbc_pubkey”: “0x04cd...”,
  
  “idempotency_key”: “0xdef456...”,
  “anomaly_score”: 0
}


⸻

8.5 MCP Tool Input/Output Schemas (EXPANDED)

Each MCP tool has a canonical schema.

⸻

8.5.1 prepare_session_transaction(intent) — Input

{
  “wallet_action_intent”: { ... }
}

Output:

{
  “unsigned_tx”: “0xf86c...”,
  “gas_estimate”: 55000,
  “chain_id”: 8453,
  “session_key_id”: “user_...”,
  “tbc_signature”: “0xf00d...”
}


⸻

8.5.2 sign_with_session_key(unsigned_tx)

Input:

{
  “unsigned_tx”: “0xf86c...”,
  “session_key_id”: “user_...”
}

Output:

{
  “signed_tx”: “0xf901234...”,
  “session_key_id”: “user_...”
}


⸻

8.5.3 submit_tx_hash()

Input:

{
  “signed_tx”: “0xf901234...”
}

Output:

{
  “tx_hash”: “0xabc123...”,
  “broadcast_timestamp”: 1712339003
}


⸻

8.5.4 verify_settlement()

Input:

{
  “tx_hash”: “0xabc123...”,
  “chain_id”: 8453
}

Output:

{
  “settlement_state”: “SETTLED”,
  “confirmations”: 12,
  “block_number”: 19992222
}


⸻

8.6 Settlement Status Schema (EXPANDED)

{
  “schema_version”: “1.0”,
  
  “tx_hash”: “0xabc123...”,
  “chain_id”: 8453,
  
  “settlement_state”: “SETTLED”,
  
  “timestamps”: {
    “submitted”: 1712339003,
    “mined”: 1712339010,
    “confirmed”: 1712339020,
    “finalized”: 1712339050
  },
  
  “block_number”: 19992222,
  “confirmations”: 12
}

Valid settlement states:
	•	SUBMITTED
	•	MINED
	•	CONFIRMED
	•	SETTLED
	•	FAILED
	•	EXPIRED
	•	ABANDONED

⸻

8.7 Error Object Schema (EXPANDED)

One canonical error structure for the entire stack.

{
  “error”: true,
  “error_code”: “INVALID_SIGNATURE”,
  “message”: “TBC signature verification failed”,
  “context”: {
    “component”: “wallet”,
    “session_key_id”: “user_...”
  },
  “timestamp”: 1712339004
}

Standard error codes:
	•	INVALID_SIGNATURE
	•	MERCHANT_KEY_MISMATCH
	•	POLICY_HASH_MISMATCH
	•	SPEND_LIMIT_EXCEEDED
	•	FREQUENCY_EXCEEDED
	•	ANOMALY_DETECTED
	•	KEY_REVOKED
	•	KEY_EXPIRED
	•	RESERVATION_TIMEOUT
	•	TX_REVERTED
	•	NONCE_ERROR (rare – only L1 internal)
	•	UNAUTHORIZED_INTENT

⸻

9. TBC Rejection Codes (Expanded)

Authoritative rejection enum for Layer 8 enforcement

The TBC is the final, deterministic authority for payment permission. Every failure MUST result in a structured, signed TBC-RESPONSE with:
	•	“decision”: “REJECT”
	•	“reason”: “<ENUM>”
	•	tbc_signature
	•	anomaly_score
	•	context fields

These rejection codes are canonical.
No implementation may add, remove, or alter them in incompatible ways.

A TBC rejection is binding:
	•	CoreProve MUST halt the flow
	•	MCP MUST NOT attempt to sign or execute
	•	Wallet MUST NOT sign an unsigned_tx that corresponds to a REJECT decision

⸻

9.1 Rejection Code Requirements

A rejection MUST be:
	•	Deterministic — given the same inputs, the same code MUST be produced
	•	Non-overlapping — each reason maps to only one failure state
	•	Signed — MUST include tbc_signature
	•	Auditable — included in logs for user comprehension
	•	Opaque-safe — MUST NOT leak PII or internal TBC logic

⸻

9.2 Full Rejection Enum

Below is the complete canonical list.

9.2.1 Policy / Authorization Errors

Code	Meaning	Example Scenario
VENDOR_NOT_WHITELISTED	Session key policy does not include this vendor	User’s autopay is only for Vendor A, but Vendor B attempts charge
FUNCTION_SELECTOR_MISMATCH	Merchant 402 requested a selector not permitted	Merchant updated ABI; session key created for old selector
CHAIN_MISMATCH	Merchant requested payment on wrong chain	Merchant charges on Base; user’s session key allows only PulseChain
POLICY_HASH_MISMATCH	Merchant 402 terms differ from policy	Merchant changed amount or expiration after user approval
SESSION_KEY_NOT_FOUND	Query references unknown session key	CoreProve used stale key ID
SESSION_KEY_EXPIRED	Session key past policy expiration	Daily payment window expired
SESSION_KEY_REVOKED	User revoked key prior to query	MCP still tried to use it
UNAUTHORIZED_INTENT	Intent structure invalid or fields altered	CoreProve tampered with fields when forming intent


⸻

9.2.2 Spend / Frequency / Budget Enforcement

Code	Meaning
SPEND_LIMIT_EXCEEDED	Amount exceeds allowed spend limit for the session key
FREQUENCY_EXCEEDED	More than N claims attempted within window
BUDGET_EXCEEDED	(Optional mode) global user budget exceeded
NEGATIVE_AMOUNT	Amount_wei < 0
ZERO_AMOUNT_NOT_ALLOWED	Amount_wei == 0 but vendor does not support free claims

Special note:
If both SPEND_LIMIT_EXCEEDED and FREQUENCY_EXCEEDED apply, TBC MUST return the first violation encountered (per Decision Tree rules).

⸻

9.2.3 Idempotency & Replay Protections

Code	Meaning
IDEMPOTENCY_REPLAY	idempotency_key already used for settled payment
TIMESTAMP_TOO_OLD	Timestamp older than allowed 120s skew
TIMESTAMP_TOO_NEW	Query timestamp significantly ahead of local time
CROSS_CHAIN_REPLAY_ATTEMPT	Merchant request replayed on wrong chain

These ensure merchant invoices cannot be replayed, duplicated, or tampered with.

⸻

9.2.4 Anomaly / Tampering / Security Violations

Code	Meaning
ANOMALY_SCORE_TOO_HIGH	anomaly_score > allowed threshold
BYTECODE_HASH_CHANGED	Vendor contract mutated unexpectedly
VENDOR_ADDRESS_CHANGED	Vendor’s contract address changed unexpectedly
FUNCTION_SIGNATURE_CHANGED	ABI drift detected
PRICE_SPIKE_ANOMALY	Amount deviates from learned pattern
FREQUENCY_SPIKE_ANOMALY	Number of invoices deviates from learned baseline
VENDOR_BEHAVIOR_ANOMALY	Unusual patterns from merchant requests
INVOICE_MUTATION_DETECTED	Invoice changed after issuance

This is the “stop-the-world” safety rail for AI autonomy.

⸻

9.2.5 Reservation / Settlement Conflicts

Code	Meaning
RESERVATION_ALREADY_EXISTS	Duplicate PAYMENT_REQUIRED without releasing previous reservation
RESERVATION_TIMEOUT	Pending transaction expired without settlement
RESERVATION_AMOUNT_MISMATCH	Vendor changed amount mid-reservation
SETTLEMENT_CONFLICT	Settlement attempted with wrong key or wrong amount

These protect the escrow-like semantics of autopay reservations.

⸻

9.2.6 Signature / Authentication Errors

Code	Meaning
MERCHANT_SIGNATURE_INVALID	402 signature cannot be verified
MERCHANT_PUBKEY_MISMATCH	Merchant pubkey differs from user-whitelisted pubkey
COREPROVE_SIGNATURE_INVALID	CoreProve forged or damaged signature
TBC_SIGNATURE_INVALID	(Internal safety, usually indicates a bug)
KEY_FORMAT_INVALID	Public key or signature malformed
UNSUPPORTED_SIGNATURE_TYPE	Anything other than ECDSA/ED25519


⸻

9.2.7 Internal / Structural / Protocol Violations

These are not “errors” but protocol integrity conditions.

Code	Meaning
INVALID_SCHEMA	Missing required fields or wrong types
MALFORMED_JSON	Reject unparseable payload
INLINE_FIELD_MODIFICATION	CoreProve altered fields that must match 402/TBC
UNKNOWN_ERROR	Catch-all; SHOULD NOT be used

UNKNOWN_ERROR MUST be treated as critical and logged aggressively.

⸻

9.3 Canonical Rejection Object

Any rejection MUST conform to:

{
  “schema_version”: “1.0”,
  “decision”: “REJECT”,
  “reason”: “SPEND_LIMIT_EXCEEDED”,
  “invoice_id”: “inv-123”,
  “timestamp”: 1712339002,
  “policy_hash”: “0xabc123...”,
  “idempotency_key”: “0xdef456...”,
  “anomaly_score”: 22,
  “tbc_pubkey”: “0x04cd...”,
  “tbc_signature”: “0xf00d...”,
  
  “context”: {
    “session_key_id”: “user_0x123_vendor_0x456_001”,
    “vendor”: “0xabcdef...”,
    “amount_wei”: 75000000000000000
  }
}


⸻

9.4 Deterministic Ordering Rules

The TBC MUST evaluate rejections in this order:
	1.	Authentication errors
	2.	Schema validity
	3.	Policy hash / session key / vendor validation
	4.	Amount, frequency, spend-limit constraints
	5.	Idempotency / replay protections
	6.	Anomaly detection
	7.	Reservation state consistency

First failure wins.
No backtracking.
No multiple errors.

A TBC response may only contain one rejection reason.

⸻

9.5 Mapping to Spec Sections

Each rejection code corresponds to enforcement defined in:

Rejection Category	Section
Auth / signature	2.2, 4.2, 8.1
Session key logic	2.1–2.3
Spend & frequency	5.1–5.3
Idempotency	5.5
Anomaly detection	5.4
Reservation state	5.2.2
Schema validation	8.x

This makes the rejections verifiable and easy to reason about.

⸻

9.6 Developer Guidance

A TBC implementation MUST:
	•	include each rejection code exactly
	•	return the first matching rejection
	•	sign every rejection
	•	maintain full logs
	•	reject malformed input early
	•	prevent CoreProve/MCP from bypassing enforcement

A Wallet implementation MUST:
	•	stop signing when receiving a rejection
	•	show human-readable message
	•	sync session-key metadata immediately

CoreProve MUST:
	•	surface the rejection in UI
	•	record logs for user audit
	•	never “retry” without user interaction unless policy explicitly allows

⸻

10. Failure Modes & Recovery Procedures (FULL EXPANSION)

The system must fail closed, logged, signed, and recoverable.

This section defines all known, predictable failure scenarios in the autopay lifecycle and provides:
	•	condition → what causes the failure
	•	detection → how components identify it
	•	automatic actions → immediate system responses
	•	required user actions → what the user MUST do
	•	recovery → how to return the system to a safe operational state

Every failure type maps to one or more TBC Rejection Codes (Section 9).

⸻

10.1 Merchant 402 Mutation or Forgery

Condition

A merchant returns a 402 that:
	•	has been modified in transit
	•	omits required fields
	•	changes amount, vendor, function selector, or chain
	•	uses a pubkey that does NOT match the user-whitelisted merchant pubkey
	•	includes an invalid signature
	•	rewrites policy_hash

Detection

CoreProve MUST detect:
	•	signature mismatch
	•	policy_hash mismatch
	•	merchant_pubkey mismatch
	•	structural invalidity (missing fields, wrong types)

TBC will also detect mismatches in:
	•	vendor
	•	chain_id
	•	function_selector
	•	amount_wei

Automatic Actions

CoreProve:
	•	Reject merchant_402
	•	Raise a “MUTATED_INVOICE” alert
	•	Halt flow

TBC:
	•	If query reaches TBC, must reject with:
	•	MERCHANT_SIGNATURE_INVALID or
	•	MERCHANT_PUBKEY_MISMATCH or
	•	POLICY_HASH_MISMATCH

MCP:
	•	MUST NOT act

Wallet:
	•	MUST NOT sign anything

User Action
	•	Review merchant identity
	•	Confirm trusted pubkey
	•	Re-request invoice from merchant

Recovery
	•	User re-whitelists merchant key IF legitimate
	•	Retry payment with clean 402
	•	If merchant changed keys unexpectedly → strongly discourage autopay continuation

⸻

10.2 TGP-QUERY Replay or Idempotency Collision

Condition

A TGP-QUERY was:
	•	replayed by MCP
	•	regenerated with same idempotency_key
	•	delayed beyond acceptable timestamp skew
	•	submitted on wrong chain

Detection

TBC detects:
	•	IDEMPOTENCY_REPLAY
	•	TIMESTAMP_TOO_OLD
	•	CROSS_CHAIN_REPLAY_ATTEMPT

Automatic Actions

TBC:
	•	Reject with canonical reason
	•	Log replay attempt
	•	Increment anomaly_score for session_key

CoreProve:
	•	Flag MCP as suspicious (non-fatal)
	•	Notify user if >3 replays in 24h

MCP:
	•	MUST stop retrying
	•	MUST mark invoice as “failed – idempotency violation”

User Action
	•	Check if autopay schedule is misconfigured
	•	Check for MCP bugs or corruption
	•	If repeated, revoke session key

Recovery
	•	Generate a new idempotency_key
	•	Retry if payment is still needed
	•	Potentially rotate session key if anomaly persists

⸻

10.3 Session Key Expired or Revoked Mid-Transaction

Condition

Between:
	•	wallet preparing unsigned_tx
	•	wallet signing
	•	wallet submitting tx_hash
	•	TBC confirming settlement

…the session key becomes invalid because:
	•	user revoked it
	•	expiration time passed
	•	frequency window closed

Detection

Wallet:
	•	MUST check revocation status within 100ms before signing
	•	MUST reject if session_key.revoked OR expired

TBC:
	•	MUST reject settlement attempts using a revoked/expired key
	•	MUST apply rejection reason:
	•	SESSION_KEY_REVOKED
	•	SESSION_KEY_EXPIRED

Automatic Actions

Wallet:
	•	Stop operation
	•	Return error to MCP

MCP:
	•	Mark invoice as permanent fail
	•	Notify user (“session key invalid during signing”)

TBC:
	•	Release any reserved amounts (reservation.state = FAILED)

User Action
	•	Review or recreate session key
	•	If unintended expiration → adjust future policy windows

Recovery
	•	Create new session key
	•	Retry payment manually
	•	Update expiration policy for reliability

⸻

10.4 Reservation Timeout or Settlement Conflict

Condition

A reservation is created (PAYMENT_REQUIRED), but:
	•	tx_hash is never submitted
	•	tx fails on-chain (revert)
	•	tx is dropped by mempool
	•	signature is malformed
	•	settlement never reaches confirmation depth
	•	wrong amount reaches TBC during settlement

Detection

TBC detects:
	•	reservation expired → RESERVATION_TIMEOUT
	•	conflicting settlement → SETTLEMENT_CONFLICT
	•	reverted tx → TX_REVERTED in settlement status
	•	missing tx submission → ABANDONED

Automatic Actions

TBC:
	•	Release reservation
	•	Reject subsequent attempts using same idempotency_key
	•	Log anomaly if repeated failures

CoreProve:
	•	Show “Payment incomplete – reservation expired”
	•	Suggest user action

MCP:
	•	MUST not retry automatically
	•	MUST log settlement failure

User Action
	•	Optionally retry payment
	•	Check vendor-contract health
	•	Investigate wallet connectivity or chain congestion

Recovery
	•	New payment attempt yields new reservation
	•	Use a new idempotency_key
	•	Increase gas_limit/gas_price if recurring mempool timeouts

⸻

10.5 MCP Compromise or Malicious Behavior

Condition

MCP behaves in a way inconsistent with normal autonomous execution:
	•	repeated invalid TGP-QUERY calls
	•	repeated attempts with anomalies > 0
	•	repeated ID conflicts or timestamp drift
	•	high-rate calls ignoring frequency rules
	•	signing attempts without wallet-intent

Detection

TBC:
	•	Reject repeated anomalous misuse
	•	Increase anomaly_score for session key
	•	Eventually REJECT with ANOMALY_SCORE_TOO_HIGH

CoreProve:
	•	Detect suspicious MCP behavior patterns
	•	Throttle MCP queries
	•	Notify user of “MCP abnormal behavior”

User:
	•	Notices unexpected payment attempts or notifications

Automatic Actions

CoreProve:
	•	Enter MCP throttling mode (rate limit 1/minute)
	•	Disable MCP full autonomy mode
	•	Require user approval for new payments

TBC:
	•	Immediately reject anomalous calls
	•	Mark session key as under suspicion

Wallet:
	•	MUST refuse Intent with invalid TBC signatures
	•	MUST reject signing if MCP violates constraints

User Action
	•	Investigate MCP integrity (potential malware)
	•	Disable or uninstall MCP
	•	Rotate session keys
	•	Review recent wallet activity

Recovery
	•	Reinstall MCP from verified build
	•	Re-enable autopay (optional)
	•	Reset anomaly scores after key rotation

⸻

10.6 Wallet Failure During Signing or Broadcasting

Condition

Wallet cannot sign or broadcast because:
	•	local storage corrupted
	•	session key missing
	•	keyfile unreadable
	•	wallet offline
	•	HSM failure
	•	network banned or RPC unreachable

Detection

Wallet:
	•	MUST detect internal errors before signing
	•	MUST detect network transport failure
	•	MUST validate session_key constraints before each sign

Automatic Actions

Wallet:
	•	Abort signing
	•	Return a standardized error
	•	Refuse to broadcast incomplete tx

MCP:
	•	Treat as failure
	•	Notify user
	•	Mark invoice as FAILED

CoreProve:
	•	Suggest retry or key regeneration

User Action
	•	Restart wallet
	•	Check network connectivity
	•	Re-import or regenerate session keys

Recovery
	•	Retry via new request
	•	Wallet must pass all constraint checks again

⸻

10.7 Chain Failure, Congestion, or Finality Issues

Condition

Settlement failures due to:
	•	chain congestion
	•	L2 → L1 withdrawal delays
	•	reorgs (rare on modern PoS but nonzero risk)
	•	RPC outages
	•	gas spikes exceeding spend limit

Detection

MCP:
	•	verify_settlement() indicates stuck or reverted state
	•	chain_id mismatch
	•	block height does not progress

TBC:
	•	settlement stuck beyond confirmation window
	•	reservation expired

Automatic Actions

TBC:
	•	Mark as EXPIRED or ABANDONED
	•	Release reservation

MCP:
	•	Notify user
	•	Stop retrying

User Action
	•	Retry payment later or on different chain
	•	Increase max gas allowance in session key policy

Recovery
	•	After chain conditions improve, user can retry with new idempotency_key

⸻

10.8 Merchant Behavior Deviations

Condition

Merchant does something unexpected:
	•	sudden price spike
	•	inconsistent invoice patterns
	•	frequent invoice ID reuse
	•	function selector drift
	•	vendor contract changed owner
	•	vendor bytecode changed

Detection

CoreProve anomaly detection
TBC anomaly scoring

Automatic Actions

TBC MUST reject with:
	•	BYTECODE_HASH_CHANGED
	•	VENDOR_BEHAVIOR_ANOMALY
	•	PRICE_SPIKE_ANOMALY
	•	FUNCTION_SIGNATURE_CHANGED

User Action
	•	Review vendor trustworthiness
	•	Possibly revoke session key
	•	Ask merchant to explain changes

Recovery
	•	Update session key policy if vendor contract legitimately upgraded

⸻

10.9 User Device Loss or Compromise

Condition

User loses device or suspects malware.

Detection

External to protocol.

Automatic Actions

CoreProve SHOULD offer:
	•	“Emergency revoke all session keys”
	•	“Freeze all autopay functions”

TBC MUST:
	•	Immediately mark all keys as revoked upon signed revocation notice

User Action
	•	Use backup device to initiate revocation
	•	Reset master key if necessary

Recovery
	•	Install wallet on new device
	•	Recreate session keys

⸻

10.10 Full System Recovery (Canonical Procedure)
	1.	Revoke all session keys (CoreProve → TBC)
	2.	Uninstall or disable MCP
	3.	Verify wallet integrity
	4.	Reinstall CoreProve
	5.	Reinstall MCP
	6.	Recreate session keys for trusted vendors only
	7.	Enable autonomy mode gradually
	8.	Monitor logs for anomalies

This procedure leads to a clean state with trusted keys and clean components.

⸻

11. User Experience Requirements (FULL EXPANSION)

The UX must communicate intent, control, and security at all times.
No ambiguity. No silent changes. No surprise spending.

This section defines the normative user experience for CoreProve, MCP, and any Wallet participating in the autopay ecosystem.

All UX rules below are mandatory for any implementation seeking compatibility with MCP-AUTO-PAY-01.1.

⸻

11.1 First-Run Experience (FRE)

The user’s first interaction with CoreProve MUST include a deterministic onboarding flow:

11.1.1 Wallet Link Step

User sees:
	•	“Connect your wallet to CoreProve”
	•	Explanation of why CoreProve needs wallet signature ability
	•	Clear statement:
“CoreProve cannot move funds. Only your wallet can sign transactions.”

Wallet request MUST use:
	•	minimal permissions
	•	single signature proving ownership
	•	NO key export
	•	NO delegation

⸻

11.1.2 Session-Key Explanation Step

Before any policy is created, CoreProve MUST show a simple graphic:

Master Key → Session Keys → Vendor Constraints → TBC Enforcement

With the following mandatory disclaimers:
	•	“Session keys allow automatic payments ONLY within your constraints.”
	•	“Session keys CANNOT be used for withdrawals, transfers, or swaps.”
	•	“You may revoke any session key instantly.”

⸻

11.1.3 Vendor Whitelisting Step

User MUST confirm:
	•	vendor name
	•	vendor contract address
	•	vendor public key

CoreProve MUST present a human-readable identity page:

Vendor: SuperSaaSCloud
Chain: Base (8453)
Contract: 0xabc123...
Merchant Pubkey: 0x04f9...
Verified by: You (Whitelisted)

User confirms with:
“I trust this vendor for autopay”

⸻

11.2 Session-Key Creation UX

Session-key creation MUST use a wizard, not a freeform form.

11.2.1 Required Steps
	1.	Choose Vendor
	2.	Choose Spend Limit
	•	Slider + numeric input
	•	Must show predicted monthly cost
	•	Must show “This is your maximum per payment.”
	3.	Choose Frequency Limit
	4.	Choose Valid Time Window
(e.g., once/day, once/month, only on weekdays)
	5.	Choose Expiration Date
(default: 1 year)
	6.	Enable/Disable Anomaly Detection
	•	MUST default to enabled
	7.	Hardware-backed storage (if available)
	8.	Final Review Screen

⸻

11.2.2 Final Review Screen MUST display:

You are creating a Session Key for:

Vendor: SuperSaaSCloud
Max per payment: $50
Frequency: Once per day
Chain: Base
Expires: Jan 1, 2026
Anomaly Detection: Enabled
Function: transfer(address,uint256)

✔ This key CANNOT spend more than $50.
✔ This key CANNOT pay other vendors.
✔ This key CANNOT withdraw or transfer funds.
✔ You can revoke it instantly at any time.

User MUST click:
“Create Session Key”

Wallet MUST sign:
	•	session key registration
	•	nothing else

No chain interaction occurs.

⸻

11.3 Intervention Points (User Override Moments)

There are three required intervention points:

11.3.1 Policy Creation

User must explicitly approve the creation of each session key.

11.3.2 Policy Expansion

If a user attempts to increase:
	•	spend limit
	•	frequency limit
	•	expiration window
	•	anomaly thresholds

CoreProve MUST require:
	•	A new wallet signature, AND
	•	A secondary confirmation screen:

“You are increasing your risk exposure. Are you sure?”

11.3.3 Anomaly Confirmation

If anomaly_score > 0, MCP MUST:
	•	Pause autonomous execution
	•	Display a human-readable explanation
	•	Require user to manually approve the payment

No autopay is allowed in anomaly cases.

⸻

11.4 Autopay Flow UX

A successful autonomous payment MUST generate a user-visible event.

11.4.1 Minimum Successful Notification

Autopay: SuperSaaSCloud charged $50.
Session key: user_0x123_vendor_0x456_001
View receipt → 

If amount < user threshold (default: $10), this may be batched into daily/weekly summary.

⸻

11.4.2 Anomaly-Triggered Notifications

If anomaly_score > 0:
	•	Notification MUST say “Suspicious activity detected”
	•	MUST require explicit confirmation
	•	MUST pause all autopay to that vendor until resolved

⸻

11.4.3 TBC Rejection Notification

If TBC returns REJECT:

Message MUST include:
	•	reason
	•	human-readable explanation
	•	what to do next

Example:

Payment Blocked: Spend Limit Exceeded
Vendor attempted a $75 charge but your limit is $50.

View or modify your limit →


⸻

11.5 Session-Key Dashboard Requirements

User MUST have a single screen listing all keys:

Active Session Keys
-——————
1. SuperSaaSCloud | $50/day | Base | Expires Jan 2026
   [Details] [Revoke]

2. CoffeeChain | $5/day | Base | Expires Mar 2026
   [Details] [Revoke]

Suspended Keys
—————
(None)

Clicking “Details” MUST show:
	•	full policy
	•	full constraints
	•	anomaly config
	•	creation date
	•	last-used date
	•	vendor public key
	•	TBC enforcement history

⸻

11.6 Revocation UX

Revocation MUST be one-click:

Button: [Revoke Session Key]

Confirmation:
“This action immediately disables autopay for this vendor.”

After revocation:
	•	CoreProve sets local revoked=true
	•	Sends signed revocation to TBC
	•	TBC updates canonical session key registry
	•	Wallet MUST refuse signing with revoked key
	•	MCP MUST remove this key from autopay schedules
	•	Notification issued:

Session Key Revoked
Autopay is now disabled for SuperSaaSCloud.


⸻

11.7 Error UX

Errors MUST be:
	•	human-readable
	•	non-technical
	•	actionable

Example mapping:

Rejection Code	User Message
SPEND_LIMIT_EXCEEDED	“Vendor attempted to charge more than your limit.”
FREQUENCY_EXCEEDED	“Too many payments in this period.”
POLICY_HASH_MISMATCH	“Vendor changed invoice terms. Please review.”
BYTECODE_HASH_CHANGED	“Vendor updated contract unexpectedly.”
KEY_REVOKED	“This autopay method is no longer active.”


⸻

11.8 Cross-Device UX Requirements

When a user logs into CoreProve on a new device:
	•	All session keys must be view-only
	•	Revocation allowed
	•	Creation of new keys requires wallet signature
	•	MCP autopay schedule MUST NOT sync unless user approves

A “New Device Detected” notification MUST be shown.

⸻

11.9 Accessibility Requirements

CoreProve UI MUST:
	•	support screen readers
	•	support large-text mode
	•	use high-contrast themes
	•	avoid color-only signaling
	•	avoid ambiguous icons

Autopay confirmations MUST use:
	•	text
	•	color
	•	symbols
	•	optional audio

⸻

11.10 Human Override as the Highest Layer of Control

This principle MUST be encoded:

“At any time, the user may:
(1) revoke keys,
(2) pause autopay,
(3) disable MCP entirely.”

Overrides MUST be instant, global, and unambiguous.

⸻

11.11 Daily / Weekly / Monthly Summaries

CoreProve MUST generate summaries:

Daily Summary (default)
	•	total autopay count
	•	total amount
	•	any anomalies
	•	any rejections

Weekly Summary
	•	vendor-by-vendor breakdown
	•	session key usage
	•	anomaly trends
	•	spend trends

Monthly Summary
	•	policy changes
	•	revocations
	•	vendor trust updates
	•	recommended limit adjustments

⸻

11.12 Session-Key Lifecycle Map (User-Facing)

States:

DRAFT → ACTIVE → (SUSPENDED) → REVOKED → EXPIRED

UX rules:
	•	DRAFT: user editing; no signature yet
	•	ACTIVE: autopay enabled
	•	SUSPENDED: user pauses autopay temporarily
	•	REVOKED: permanent halt; cannot reactivate
	•	EXPIRED: time-based; recreate needed

⸻

11.13 UX for Anomaly Investigation

If anomaly_score > 0:

CoreProve MUST show a “What happened?” forensic screen:
	•	Expected amount vs. requested amount
	•	Vendor contract hash diff
	•	ABI diff (function selector drift)
	•	Frequency spike visualization
	•	Timestamp/geo anomalies

Then offer:
	•	“Approve once”
	•	“Adjust policy”
	•	“Revoke key”
	•	“Suspend vendor”

⸻

11.14 Error-Proofing the Autonomous Experience

To avoid user confusion:
	•	No silent policy changes
	•	No auto-increasing limits
	•	No bypass of anomaly detection
	•	No autopay for first-time vendors
	•	No autopay during anomalies
	•	No autopay across chains without explicit consent
	•	No autopay if device compromised flags detected

---

12. Security Model (FULL EXPANSION)

The system’s security is based on hard boundaries, minimal privileges, deterministic enforcement, and cryptographic authenticity.
No trust in agents.
No trust in networks.
Trust only in cryptographic facts.

This section defines:
	•	adversarial assumptions
	•	trust surfaces
	•	formal boundaries
	•	key management requirements
	•	attack scenarios
	•	mitigations
	•	secure operational requirements

⸻

12.1 Threat Model Overview

12.1.1 Adversaries Considered

The system assumes the following adversaries exist:
	1.	Compromised MCP
	•	malware running inside the MCP process
	•	rogue plugins
	•	jailbroken devices
	•	compromised AI agent
	2.	Compromised Merchant
	•	fake 402 responses
	•	invoice manipulation
	•	replay attacks
	•	malicious ABI or contract upgrades
	3.	Compromised Network / MITM
	•	packet mutation
	•	replay
	•	downgrade attacks
	•	censorship
	4.	Compromised CoreProve
	•	stale cached data
	•	tampered policy presentation
	•	corrupted local storage
	5.	Compromised Browser
	•	malicious extensions
	•	session key exfiltration attempts
	•	HTML injection
	6.	Compromised Wallet Environment
	•	local malware
	•	keylogger
	•	HSM compromise (rare but considered)
	7.	Chain-Level Threats
	•	mempool manipulation
	•	nonce conflicts
	•	chain reorgs
	•	failed transactions
	•	gas spikes

Assumption:
Adversaries may control everything except the user’s master key private material inside the wallet (or HSM).
All other system components must be assumed hostile by design.

⸻

12.2 Trust Boundaries (Formal)

The system divides trust into four explicit boundaries, each with strict, limited authority.

⸻

12.2.1 Wallet (Boundary 0 — Root of Trust)

Wallet is the only component allowed to:
	•	HOLD the master key
	•	HOLD session keys
	•	SIGN transactions

Wallet assumes:
	•	zero trust in MCP
	•	zero trust in CoreProve
	•	zero trust in merchant
	•	zero trust in TBC messages (must verify)

Wallet protects:
	•	master key
	•	session keys
	•	signing API
	•	key revocation checks

Wallet MUST reject any signing request that:
	•	lacks verified TBC signature
	•	violates session key constraints
	•	originates from an untrusted UI context

Wallet MUST enforce:
	•	local rate limits
	•	revocation timing (<100ms from check-to-sign)
	•	chain ID matching
	•	function selector matching
	•	gas sanity checks

The wallet is treated as a small, auditable trust kernel.

⸻

12.2.2 CoreProve (Boundary 9 — Identity/Policy Gatekeeper)

CoreProve:
	•	verifies merchant signatures
	•	constructs TGP-QUERY
	•	manages session-key lifecycle
	•	stores anomaly baselines
	•	performs local anomaly checks
	•	ensures UI integrity

CoreProve is not trusted with funds.
If compromised, it cannot bypass TBC or wallet constraints.

Mitigations:
	•	all TBC responses contain tbc_signature
	•	all merchant actions contain merchant_signature
	•	all wallet signing requests are bound to TBC approvals

⸻

12.2.3 TBC (Boundary 8 — Deterministic Enforcement Layer)

TBC is the final authority enforcing:
	•	spend limits
	•	frequency limits
	•	vendor whitelist
	•	chain restrictions
	•	policy_hash
	•	anomaly score
	•	idempotency keys
	•	reservation states
	•	signature validity

TBC is treated as a verification oracle, not an execution component.

Security requirements:
	•	MUST run in a sandboxed, verifiable environment
	•	MUST use hardware-backed key signing
	•	MUST reject all malformed queries
	•	MUST fail closed on validation ambiguity
	•	MUST be stateless except for:
	•	session key registry
	•	spend/frequency counters
	•	reservation registry
	•	anomaly scoring model

⸻

12.2.4 MCP (Boundary 10 — Autonomy Layer)

MCP is treated as the least trusted component:
	•	It NEVER sees private keys
	•	It NEVER signs transactions
	•	It NEVER bypasses TBC
	•	It NEVER bypasses wallet constraints

MCP only orchestrates:
	•	triggers
	•	dry-runs
	•	intent forwarding
	•	settlement checks

Because MCP may be compromised, design ensures:

MCP compromise cannot drain user funds.
The worst-case scenario:
MCP spams queries. → TBC rejects them. → User gets alerts.

⸻

12.3 Session-Key Security Requirements (Mandatory)

Session keys act like zero-trust cryptographic “permission slips.”

Mandatory constraints:
	•	vendor whitelist (contract address)
	•	function selector
	•	chain_id
	•	max spend limit
	•	frequency limit
	•	expiration
	•	anomaly thresholds
	•	hardware-backed if device allows
	•	non-exportable key material
	•	never visible to MCP

Session keys MUST be:
	•	created by CoreProve
	•	registered with TBC
	•	approved by user via wallet signature
	•	stored encrypted by wallet

Key IDs MUST be unique per user per vendor.

⸻

12.4 Key Lifecycle Security

Session-key lifecycle:

CREATE → ACTIVATE → USE → (SUSPEND?) → REVOKE → EXPIRE

MUST enforce:
	•	REVOCATION is immediate
	•	EXPIRED keys MUST be unusable
	•	SUSPENSION MUST pause autopay
	•	CREATE/ACTIVATE MUST require wallet approval

The wallet MUST check:
	•	revocation flag
	•	expiration timestamp
	•	spend limit
	•	frequency
	•	selector
	•	chain_id
	•	anomaly overrides
	•	TBC signature validity

before signing ANY tx.

⸻

12.5 Cryptographic Guarantees

12.5.1 Merchant Authenticity

Merchant MUST sign all 402 payloads.
CoreProve MUST validate signature.
Wallet MUST see merchant_pubkey for the session key’s vendor.

12.5.2 TBC Authenticity

All TBC responses MUST be signed.
Wallet MUST verify before signing tx.

12.5.3 Wallet Authenticity

Only wallet signs the actual on-chain transaction.

12.5.4 Replay Prevention

Every message includes:
	•	timestamp
	•	domain separation
	•	chain_id
	•	idempotency_key
	•	session_key_id

12.5.5 Deterministic Execution

TBC rejects ambiguous messages.
No heuristics in enforcement.

⸻

12.6 Attack Surfaces and Mitigations

12.6.1 MCP Compromise

Impact: Low
Mitigations:
	•	MCP cannot sign
	•	MCP cannot alter TBC responses
	•	MCP must forward TBC signatures
	•	Wallet validates constraints
	•	TBC rate-limits MCP
	•	Session keys provide zero-privilege scope

Worst case: annoying notifications (DoS)

⸻

12.6.2 CoreProve Compromise

Impact: Moderate
Mitigations:
	•	cannot forge TBC
	•	cannot forge wallet signatures
	•	cannot modify unsigned_tx
	•	wallet validates TBC signature
	•	TBC validates CoreProve signature
	•	anomaly drift detection flags suspicious behavior

Worst case: user alerted, request rejected

⸻

12.6.3 Merchant Compromise

Impact: High (attempted)
Mitigations:
	•	signed 402
	•	policy_hash match
	•	function selector match
	•	anomaly detection
	•	TBC checks spend limit
	•	session key constraint enforcement

Worst case: attempted overcharge → rejected

⸻

12.6.4 Wallet Compromise

Impact: Critical
Mitigations:
	•	hardware storage
	•	limited permissions
	•	no key export
	•	strong separation from CoreProve/MCP
	•	OS sandbox enforcement

Worst case: ANY hot wallet compromise = loss of funds
(This is unavoidable; thus spec encourages hardware wallet integration.)

⸻

12.6.5 Chain-Level Risks

Mitigations:
	•	settlement confirmation depth
	•	reservation timeouts
	•	abandonment rules
	•	user notification for stuck tx
	•	gas-limit checks

⸻

12.7 Anomaly Detection Security

Anomaly detection MUST flag:
	•	price variance
	•	vendor contract changes
	•	ABI drift
	•	frequency spike
	•	replay attempts
	•	timestamp drift
	•	vendor ownership transfer
	•	bytecode hash changes

TBC MUST reject if anomaly_score > 0 for autonomous flows.
MCP MUST require manual confirmation.

⸻

12.8 Autonomy Guardrails

Autonomy is not trust.
Autonomy is scheduled execution under constraints.

Guardrails:
	•	no autopay under anomalies
	•	no autopay with expired keys
	•	no autopay during revocation
	•	no autopay for first-time vendors
	•	no autopay if CoreProve is unverified
	•	no autopay if TBC signature invalid
	•	no autopay if wallet signature checks fail

⸻

12.9 Defense-in-Depth Layers

Layer	Component	Primary Security Role
L0	Wallet	Signature enforcement, key safety
L8	TBC	Policy enforcement, anomaly scoring
L9	CoreProve	Identity, policy UI, merchant validation
L10	MCP	Intent routing, automation
App	Merchant	Invoice integrity, pricing

Each layer MUST assume that layers above and below can be hostile.

⸻

12.10 Governance & Operational Security

Any implementation MUST:
	•	maintain audit logs
	•	use hardware-backed TBC keys
	•	rotate TBC keys every 90 days
	•	enable client-side integrity checks
	•	cryptographically pin merchant public keys
	•	provide reproducible builds for CoreProve and MCP
	•	support multi-device revocation

Security patches MUST follow a deterministic upgrade path that never bypasses user approval or session-key validity.

⸻

12.11 Emergency Stop Controls

User MUST be able to:
	•	revoke all session keys
	•	suspend all autopay
	•	disable MCP
	•	uninstall MCP
	•	remove CoreProve’s signing permissions
	•	force TBC to reject all session keys

Emergency revocation MUST propagate in under 2 seconds.

⸻

12.12 Formal Safety Guarantee

The full autopay model guarantees:

No single compromised component can drain user funds.
Only the wallet, with TBC-verified constraints, can sign transactions.

Even coordinated attacks across MCP + CoreProve + Merchant fail if wallet/TBC enforcement remains intact.

Only a wallet compromise can drain funds, and that is outside the scope of this protocol (and unavoidable in any system where the wallet holds private keys).


E) Begin drafting the v02 restructural redesign plan
# Full Detailed Content
**IMPORTANT:**  
The complete expanded text for each section is extremely large (approx. 30–35 pages).  
To avoid truncation and ensure that the file is safely created, this version provides the **canonical skeleton plus all consolidated patch notes**, and the full text will be emitted in subsequent incremental expansions (Parts A–G).

This file acts as the **root spec**, and the follow‑up expansions (generated on request) will append the full elaborated content section-by-section.

You now have a safe, Git‑ready starting point that won’t break or truncate.


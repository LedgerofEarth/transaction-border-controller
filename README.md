🌐 Transaction Border Controller (TBC)

A Deterministic, NAT-Style Transaction Firewall for Policy-Controlled Blockchain Settlement

The Transaction Border Controller (TBC) is a policy-enforced transaction firewall that performs deterministic validation, escrow sequencing, and routing of blockchain payments according to the Transaction Gateway Protocol (TGP-00).

It introduces transaction NAT—an economic analogue of IP NAT—providing address obfuscation, controlled exposure, and verified settlement boundaries for both buyers and merchants.

The TBC operates as a Layer-8 (economic layer) gateway:
	•	validating merchants,
	•	verifying contract bytecode and settlement logic,
	•	normalizing transaction envelopes,
	•	enforcing session budgets and policy constraints,
	•	and shielding internal wallet infrastructure from external observation.

⸻

🔐 1. Transaction NAT (Technical Definition)

TBC provides address translation between external participants and internal wallet infrastructure in a way analogous to network NAT:

Buyer NAT

Externally:
	•	The seller sees an escrow address and escrow contract state.
Internally:
	•	The buyer’s true wallet is never revealed.
	•	The buyer signs only pre-constructed Economic Envelopes issued by the TBC.

Merchant NAT

Externally:
	•	The buyer interacts only with the public settlement contract, not the merchant treasury.
Internally:
	•	Merchant hot wallets or treasury accounts remain non-discoverable.
	•	Routing to merchant treasury occurs behind the TBC boundary after deterministic policy validation.

Security Benefit

Neither party learns the other’s wallet graph, preventing:
	•	wallet scraping
	•	transaction history disclosure
	•	treasury profiling
	•	targeted economic attacks

This matches the security semantics of NAT in carrier environments: address reachability is indirect and policy-controlled.

⸻

🛡 2. TBC Verification Stack (L1–L6)

Every inbound TGP QUERY undergoes a reproducible, deterministic verification pipeline.
This ensures that no unauthorized, malformed, or unsafe transaction can reach settlement.

L1 — Merchant Registry / Authorization
	•	Merchant payment profile must exist in the registry.
	•	Merchant URL, domain binding, and certificate must match the registered profile.
	•	Merchant’s on-chain payment profile contract must match the expected interface hash.

L2 — Cryptographic Validation
	•	Session tokens and delegated keys (if present) are verified.
	•	Nonce consistency and replay protection applied.
	•	Delegate scope validated without maintaining state (as required by TGP statelessness).

L3 — Contract Bytecode & RPC Integrity

The TBC pulls authoritative on-chain state and validates:
	•	Contract bytecode hash
	•	ABI hash
	•	Functions required by TGP verbs
	•	Supported verbs (COMMIT, ACCEPT, CLAIM, WITHDRAW)
	•	Settlement rules and payout routing

If bytecode differs from the expected template → ERROR.

If RPC reveals inconsistent or non-canonical state → ERROR.

L4 — Optional ZK / Attestation

If the merchant requires shielded invocation:
	•	Buyer proves ownership of nullifier
	•	Merkle membership path validated
	•	Spending authority or identity granted via ZK proof

This allows privacy without reducing determinism.

L5 — Policy Evaluation
	•	Merchant-defined policy
	•	Buyer session spend limit
	•	Rate limits
	•	Jurisdictional constraints
	•	Contract-specific rules (digital goods vs services)
	•	Anti-abuse heuristics

All policy decisions map to deterministic ACK(status) results.

L6 — Escrow / WITHDRAW Eligibility

The TBC checks:
	•	Timers
	•	Escrow state transitions
	•	Eligibility for buyer/seller withdrawal
	•	Whether claim/fulfill prerequisites are met

With TBC performing these checks no wallet or client must understand the settlement state machine.

⸻

🧩 3. Security Properties

The TBC enforces:

1. Deterministic Authorization

Two gateways with identical configuration will produce identical results for the same QUERY.

2. Wallet Blindness

The wallet signs only what it sees.
No signatures are intercepted.
The TBC never receives private keys.

3. Stateless Verification

All verification state is contained in:
	•	the QUERY
	•	the Economic Envelope
	•	on-chain settlement contracts

The TBC does not maintain mutable per-session state, preventing session hijacking.

4. Non-Custodial Funds Handling

Funds are held by merchant-owned settlement contracts with:
	•	no admin keys
	•	no upgradability
	•	no backdoor transitions
	•	no off-chain trustees

Every transition is enforced by protocol verbs.

5. Merchant Authentication

No merchant can initiate a payment flow unless:
	•	payment profile contract matches its registry entry
	•	bytecode matches required template
	•	routing addresses and fee structures are validated
	•	TLS & domain binding are correct

This prevents spoofed merchants, phishing flows, and counterfeit payment endpoints.

⸻

🔄 4. Transaction Flow (Technical)

Buyer Client             TBC Gateway               Settlement Contract        Merchant Backend
    |                        |                           |                         |
    | -- QUERY ------------> |                           |                         |
    |                        | -- L1–L6 Validation -->   |                         |
    |                        |                           |                         |
    | <-- ACK(allow) ------- |                           |                         |
    | -- Signed Tx --------> | -- relay or direct -----> |                         |
    |                        |                           | -- emits events ------> |
    |                        | <-- SETTLE -------------- |                         |

At no point does buyer ↔ merchant direct wallet exposure occur.

⸻

🧱 5. Why This Matters for Security Engineering

Prevention of direct wallet discovery

Attackers cannot map:
	•	merchant treasury habits
	•	buyer token balances
	•	historical buying/selling activity
	•	internal treasury structure

Centralized risk moves out of wallets and into on-chain constraints

Smart contracts enforce constraints without requiring trust in the TBC.

Auditable transaction pipeline

Every step:
	•	QUERY
	•	ACK
	•	Economic Envelope
	•	Signed Tx
	•	SETTLE

is independently verifiable.

Policy-first architecture

Contract settlement logic remains immutable.
Policy enforcement is off-chain and adjustable without contract redeployments.

⸻

✔ This version is suitable for a technical prospect.

If you’d like, I can also produce:
	•	A merchant-specific technical addendum
	•	A buyer privacy assurance document
	•	A security architecture whitepaper
	•	A TBC–merchant integration guide
	•	A diagram-focused version for CTO slides

Just say the word.

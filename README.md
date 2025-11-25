🌐 Transaction Border Controller (TBC)

A Trust-Minimized, Non-Custodial Transaction Firewall for Blockchain Commerce

Ledger of Earth — Version 0.8 (Active Development)

⸻

🔒 What the TBC Is

The Transaction Border Controller (TBC) is a trust-minimized, non-custodial, policy-aware transaction firewall that sits between buyers, merchants, and the blockchain.

It allows untrusted parties to transact safely by:
	•	Independently verifying settlement contracts
	•	Validating merchant authenticity
	•	Ensuring transaction integrity
	•	Orchestrating multi-step escrows
	•	Shielding wallet privacy
	•	Enforcing policy, limits, and jurisdictional rules

The TBC cannot hold user keys, cannot spend funds, and has no custody.
Wallets remain completely unmodified and fully sovereign.

This is the blockchain equivalent of a Session Border Controller (SBC) in telecom:
a neutral, policy-enforcing transaction firewall that protects both sides.

⸻

✨ Anchored in Satoshi’s Original Vision

Satoshi Nakamoto described the principle behind safe, trust-minimized two-party exchange:

“It’s cryptographically possible to make a risk-free trade.
The two parties would set up transactions on both sides such that when they both sign the transactions,
the second signer’s signature triggers the release of both.
The second signer can’t release one without releasing the other.”
— Satoshi Nakamoto, Dec 10, 2010

The TBC + CoreProve settlement contracts generalize this into a production-ready, multi-verb escrow system governed by the open Transaction Gateway Protocol (TGP-00).

⸻

📌 What the TBC Does

1. Merchant Verification & Fraud Prevention

The TBC performs deep validation before any payment is approved:
	•	Ensures payment profile contracts actually belong to the merchant
	•	Checks contract bytecode, deployed code hash, and RPC integrity
	•	Validates seller commitment signatures (or counter-escrow deposits)
	•	Confirms that the merchant’s escrow logic matches the advertised flow

If the settlement logic or merchant identity is suspicious, the TBC returns ERROR.

⸻

2. Buyer Protection

The TBC prevents:
	•	Overpayment
	•	Wrong-chain attacks
	•	Calldata manipulation
	•	Redirect-to-attacker address modifications
	•	Forced approval of malicious contract calls
	•	Infinite-drain or recursive-call attacks

All authorized transactions are returned as Economic Envelopes that must be executed verbatim.

⸻

3. Transaction NAT (Firewall Behavior)

Just as SBCs rewrite SIP messaging for safe routing, the TBC:
	•	Normalizes payment requests
	•	Sanitizes malformed or dangerous transaction fields
	•	Enforces chain consistency
	•	Determines routing:
	•	direct → RPC
	•	relay → TBC
	•	Removes ambiguities and dangerous optional fields
	•	Ensures wallets sign only safe, deterministic transactions

This creates NAT for blockchain commerce—a clean, safe transaction boundary.

⸻

4. Coordinated Escrow Sequencing

The TBC orchestrates the CoreProve settlement state machine:

COMMIT → ACCEPT → FULFILL → VERIFY → CLAIM → SETTLE

It enforces:
	•	Timeout logic
	•	WITHDRAW eligibility (L6 layer)
	•	Delivery confirmation
	•	Refund conditions
	•	Two-party fairness

The TBC monitors contract events and generates the terminal SETTLE message.

⸻

5. Privacy Protection for Both Parties

The TBC protects user identity and merchant privacy:
	•	Wallet addresses never leak to merchants
	•	Merchants avoid storing unnecessary customer data
	•	Buyers avoid exposing financial histories
	•	No linkable analytics or tracking
	•	No third-party relay of unneeded metadata

Only public information ever touches the TBC—never private keys, seeds, or internal wallet state.

⸻

🧭 System Architecture

Merchant
   ↓ (HTTP 402 / x402 / QR)
TGP Client (Browser Extension or Agent)
   ↓
Transaction Border Controller (TBC)
   ↓ (Economic Envelope)
Wallet (Unmodified Blind Signer)
   ↓
CoreProve Settlement Contract
   ↓
Blockchain Network

Wallets remain unchanged, unaware of TGP.

⸻

🛡 Security Properties

Non-Custodial
	•	The TBC holds no funds
	•	The settlement contract is constrained:
	•	no admin keys
	•	no privileged upgrade paths
	•	no discretionary withdrawals

Deterministic

Given identical input, all compliant TBCs produce the same output.

Verifiable

Every authorization relies on:

Layer	Verification
L1	Merchant registry
L2	Buyer/seller cryptographic validation
L3	Contract bytecode & RPC integrity
L4	ZK proofs (optional)
L5	Policy rules
L6	Escrow/WITHDRAW eligibility

If any layer fails → ERROR.

Wallet-Blind

Wallets sign normal transactions; TGP never modifies the wallet.

⸻

🔧 Repository Structure (Simplified)

/specs
    TGP-00.md          # Core signaling protocol
    TGP-CP-00.md       # Client behavior
    TGP-EXT-00.md      # Browser extension runtime
    CoreProve-00.md    # Settlement contract spec
    TBC-00.md          # Border controller spec

/coreprover-contracts  # Settlement contracts
/coreprover-service    # TBC gateway (Rust)
/coreprover-sdk        # Developer SDK
/tgp-extension         # Browser extension


⸻

🧪 MVP Pipeline
	1.	Merchant issues payment_required
	2.	Client generates QUERY
	3.	TBC validates everything (L1–L6)
	4.	TBC returns ACK allow with Economic Envelope
	5.	Wallet signs
	6.	Settlement contract executes escrow state transition
	7.	TBC monitors contract → emits SETTLE

This completes the full transaction lifecycle.

⸻

🧭 Ideal for
	•	Merchants handling crypto payments
	•	Wallet developers
	•	Agentic AI platforms
	•	Payment processors
	•	Protocol design teams
	•	Telecoms & carriers (multi-node TBC clusters)
	•	Compliance-driven organizations

⸻

🤝 Contributing

We welcome contributions to specs, code, routing logic, and wallet integrations.
Open a PR or start a discussion in the issue tracker.

⸻

📄 License
	•	Code components: 48-month commercial license
	•	Specifications: open for interoperability
	•	A “TBC vs. traditional payment processors” comparison chart

🌐 Transaction Border Controller (TBC) & Transaction Gateway Protocol (TGP)

A Carrier-Grade Economic Control Plane for Autonomous, Agent-Driven Blockchain Transactions
Created by Ledger of Earth
Version: 0.7 (Active Development)

⸻

📌 Overview

Modern blockchain wallets were never designed for autonomous agents, cross-domain compliance, session budgets, or multi-step escrow transactions.
Applications and AI systems today have no safe way to negotiate or route payments without exposing users to risk.

The Transaction Border Controller (TBC) and the Transaction Gateway Protocol (TGP) form a new Layer-8 economic control plane for secure, policy-governed blockchain transactions.

This system provides:
	•	Safe agent-driven payments
	•	Escrow-first settlement flows
	•	Policy-aware transaction governance
	•	Session-based spend limits
	•	Multi-chain routing logic
	•	Wallet-compatible transaction pipeline
	•	No key exposure, no custody, no wallet modification

It adapts the proven carrier-grade model of Session Border Controllers (VoIP) into a modern blockchain-native transaction firewall.

⸻

🧩 What This System Does

Using TGP + TBC, any transaction—human or agent-initiated—follows this pipeline:

Application (x402)
      ↓
TGP Client Runtime (browser extension)
      ↓
Transaction Border Controller (policy engine)
      ↓
Wallet (blind signer)
      ↓
Escrow / Payment Profile Contract (settlement state machine)
      ↓
Blockchain Network

This introduces, for the first time:
	•	Transaction NAT/Firewall behavior
	•	Escrow sequencing enforced by protocol
	•	Deterministic transaction construction
	•	Separation of authorization, policy, signing, and settlement
	•	Safe autonomous execution for agents

Wallets remain unmodified.
Users maintain full key control.
Policies live in the TBC.
Settlement logic lives on-chain.
Agents remain constrained and safe.

⸻

🚧 Repository Structure

/specs
   TGP-00.md                 # Core signaling protocol
   TGP-CP-00.md              # Client runtime profile
   TGP-EX-00.md              # Browser extension runtime
   TBC-00.md                 # Transaction Border Controller spec
   TxIP-00.md                # Signaling primitive
   x402-EXT.md               # Binding to x402 agent protocol
   appendices/               # Economic envelope, settlement receipts, etc.
   api/                      # TBC management API
   deprecated/               # Legacy VGP + early drafts

/coreprover-contracts        # Settlement contract tests & ABI
/coreprover-service          # TBC Gateway (Rust)
/coreprover-sdk              # Client-side SDK (TS/Rust)
/tgp-extension               # Browser extension implementation
/docs
   architecture/             # Architecture, topology, diagrams
   analysis/                 # Engineering analysis
   roadmap/                  # Rebuild plans, timelines


⸻

🔐 Key Architectural Components

1. TGP — Transaction Gateway Protocol

Defines QUERY/ACK signaling:
	•	QUERY requests policy guidance
	•	ACK returns transaction specifications
	•	Session-based transaction flow
	•	Routing mode: direct or relay
	•	Escrow verbs (commit, accept, fulfill, claim)

⸻

2. TBC — Transaction Border Controller

Think of this as a “transaction firewall” or “economic SBC.”

Responsibilities:
	•	Policy evaluation
	•	Jurisdiction + compliance boundaries
	•	Session tracking
	•	Settlement verb determination
	•	Transaction construction
	•	Relay of signed transactions

The TBC never sees private keys.

⸻

3. TGP Client Runtime (CP-00)

A standard for how client applications behave:
	•	Build QUERY messages
	•	Send → TBC
	•	Receive ACK
	•	Construct transaction
	•	Request wallet signature
	•	Route signed transaction

The Client holds no keys and alters no wallet behavior.

⸻

4. TGP Browser Extension (TGP-EX-00)

The default implementation of the Client runtime.
	•	Chrome MV3, Brave, Firefox, Safari compliant
	•	Detects x402 payment_required
	•	Injects the TGP Presence API
	•	Routes queries to TBC
	•	Hands final transactions to wallets

This enables wallet-agnostic integration.

Wallets do not need to adopt TGP—
they simply detect when the extension is active.

⸻

5. Payment Profile Contract (Settlement Layer)

On-chain state machine:

commit → accept → fulfill → verify → claim

This enforces:
	•	escrow logic
	•	delivery verification
	•	multi-step settlement flows
	•	dispute minimization
	•	transparent receipts (optional ZK proofs)

⸻

6. x402 Integration

The system is fully compatible with:
	•	Autonomous agents
	•	dApps
	•	Cross-domain payment negotiation
	•	Provider-to-client negotiation flows

An x402 payment_required event automatically triggers a TGP QUERY.

⸻

💡 Why This Matters

AI agents will soon manage:
	•	subscriptions
	•	settlements
	•	marketplace purchases
	•	resource allocation
	•	cross-domain compute payments
	•	multi-step digital delivery flows

Without TGP/TBC, they are unsafe.

This project is the first practical framework that:
	•	gives agents guardrails
	•	gives users policy control
	•	keeps wallets unchanged
	•	moves settlement logic onto the chain
	•	keeps key custody private
	•	works across any EVM chain (PulseChain first, EVM-wide next)

⸻

🧪 Demo Architecture (MVP)

The first working demo will showcase:
	•	x402 event detected
	•	Extension triggers TGP QUERY
	•	TBC returns commit transaction
	•	Wallet signs
	•	Contract logs settlement event
	•	TBC advances next verb (fulfill → claim)
	•	Session completes

This validates the full “economic control plane” pipeline.

⸻

🛠 Build & Development

Rust (TBC & CoreProver Service)

cd coreprover-service
cargo build
cargo test --workspace


⸻

Browser Extension (TGP-EX-00)

cd tgp-extension
npm install
npm run build

This outputs a manifest v3 extension ready for Chrome/Brave/Edge
and easily portable to Firefox/Safari.

⸻

Settlement Contract

cd coreprover-contracts
forge build
forge test


⸻

📅 Roadmap

Phase 1 — Foundations
	•	Implement QUERY/ACK engine
	•	Settlement ABI integration
	•	Basic policies

Phase 2 — Browser Extension
	•	Presence API
	•	x402 handler
	•	Signer routing

Phase 3 — End-to-End Demo
	•	Full commit → fulfill → claim flow
	•	Relay mode testing

Phase 4 — Agent Integration
	•	Autonomous but constrained spending
	•	x402 multi-step workflows

Phase 5 — Enterprise / Carrier Grade
	•	Multi-node clustering
	•	Federated TBCs
	•	Telemetry + Transaction Detail Records (TDRs)
	•	Zero-trust auditing

⸻

⚖️ Security Model
	•	No custody
	•	No keys visible to TBC or client
	•	Wallet remains final signing authority
	•	Public-key-only addressing
	•	Strict separation between policy, signing, and settlement
	•	HTTPS-only TBC interactions
	•	Replay-safe session identifiers
	•	Deterministic transaction construction

The TBC cannot spend user funds—
but it can deny or revise unsafe spending behaviors.

⸻

🔎 Audience

This project is built for:
	•	Blockchain wallets
	•	Agentic AI platforms
	•	dApp developers
	•	RPC providers
	•	Financial infrastructure
	•	Payment processors
	•	Protocol researchers
	•	L2/L3 builders

It is designed to be open, extensible, and network-neutral.

⸻

🤝 Contributing

We welcome:
	•	specification improvements
	•	implementation feedback
	•	wallet integration proposals
	•	agent compatibility testing
	•	research into policy engines / ZK receipts

Open a PR or start a discussion via issues.

⸻

🏛 License

The code components follow a 48-month commercial-use license.
The specifications are open for interoperability.

⸻

✉️ Contact

Ledger of Earth
Protocol Engineering & Architecture
(TBC/TGP Project)


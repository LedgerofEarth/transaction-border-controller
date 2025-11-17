📘 TBC Context Summary (Updated)

Version: 0.3
Status: Updated to reflect TGP-CP, TGP-EX, and TBC-00 architecture
Author: Ledger of Earth

⸻

1. Objective

The goal of the Transaction Border Controller (TBC) project is to define and implement a secure, policy-driven Layer-8 economic control plane for blockchain transactions.

The TBC enforces trust boundaries for:
	•	autonomous AI agents
	•	cross-domain payments
	•	multisite organizations
	•	merchant settlement flows
	•	user-defined spend / risk limits

Powered by:
	•	TxIP-00 (signaling primitive)
	•	TGP-00 (transaction gateway protocol)
	•	TGP-CP-00 (client runtime profile)
	•	TGP-EX-00 (browser extension runtime)
	•	X402-EXT (binding to the x402 agent negotiation standard)
	•	CoreProver settlement model
	•	Presence API for wallets

The TBC allows any wallet to become part of a safe, sessionized, escrow-enforced transaction pipeline without modification to wallet internals.

⸻

2. Current Status

✔ Specifications
	•	TxIP-00, TGP-00, and TGP-CP-00 drafted.
	•	TGP-EX-00 (extension runtime) finalized.
	•	TBC-00 API and routing spec defined.
	•	X402-EXT specification for triggering TGP flows is drafted.

✔ CoreProver / Settlement Layer
	•	Escrow state machine defined (commit → accept → fulfill → claim).
	•	Receipt vault model drafted.
	•	Zero-knowledge extensions outlined (ZKB-01 & ZKS-01).

✔ Implementation
	•	TBC Rust scaffolding complete.
	•	Query/ACK routing paths architected.
	•	Client runtime design finalized (browser extension first).

Next engineering focus:
	•	Implement full TGP QUERY/ACK pipeline
	•	Implement settlement contract ABI bindings
	•	Build end-to-end demo with test TBC + settlement contract

⸻

3. Short-Term Goals (MVP Phase)

1. Implement TGP QUERY/ACK Parsing and Policy Routing
	•	End-to-end from extension → TBC → extension → wallet
	•	Correct verb selection
	•	Deterministic transaction construction

2. Integrate Settlement Profile (CoreProver)
	•	commit()
	•	accept()
	•	fulfill()
	•	claim()
	•	full verb-loop routing through TBC

3. Build Test Harness & Simulator
	•	Simulated merchants
	•	Simulated buyer and agent flows
	•	Session continuation logic

4. Finalize x402 Integration Path
	•	Bind payment_required → TGP QUERY
	•	Support agent-sourced flows
	•	Ensure compatibility with MCP-based agents

5. Deliver the First End-to-End Demo
	•	“Protected transaction” showcase
	•	Commit → fulfill → claim sequence
	•	Wallet signature + TBC policy enforcement

⸻

4. Medium-Term Goals

Multi-Chain Routing
	•	PulseChain (MVP) → EVM expansion
	•	Define routing modes (direct/relay)
	•	Extend ACK structure for multi-chain transactions

Layer 9 / Layer 10 Integration
	•	Identity mapping layer (L9)
	•	signature identity → session identity
	•	Policy expression layer (L10)
	•	PEL-0.1 language for merchants & users
	•	on-TBC and off-chain rule evaluation

Unified Presence Layer
	•	Finalize TGP Presence API spec (wallet detection)
	•	Work with wallets to optionally display “Protected Mode Available”

High-Confidence Agent Interoperability
	•	Enable MCP agents to perform TGP flows autonomously
	•	Add guardrails for AI-initiated payments

⸻

5. Long-Term Goals

Carrier-Grade Deployment
	•	Clustered TBC nodes
	•	Transaction telemetry
	•	Trust-domain routing (enterprise or national-level)
	•	SLA monitoring
	•	Auditability and tamper-evident session logs

TDR (Transaction Detail Record) Infrastructure
	•	Immutable economic logs
	•	Inter-domain federation
	•	Cross-border payment trail compliance
	•	Optional ZK summarization

Global Multi-Agent Economy Enablement
	•	Autonomous agents negotiating payments
	•	Escrow-first transaction safety
	•	Zero-trust per-transaction policy enforcement
	•	Session-based authentication at scale

This establishes the TBC as the economic gatekeeper for all agent-driven blockchain transactions.

⸻

6. Summary

The TBC architecture introduces a new paradigm for blockchain transaction safety:
	•	Clients construct → TBC approves → Wallet signs → Escrow settles.
	•	No wallet modification required.
	•	No custody or key risk.
	•	Full policy control.
	•	Full agent compatibility.

The system is now specification-complete at the architectural level and ready for implementation of the core QUERY/ACK pipeline and escrow loop.

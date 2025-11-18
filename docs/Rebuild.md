🔧 Rebuild Plan — TBC / TGP System (Updated)

Version: 0.5
Status: Active Engineering Guide
Author: Ledger of Earth

This document defines the current rebuild plan for the Transaction Border Controller (TBC) ecosystem, integrating the new Client runtime, Browser Extension, Settlement Profile, and complete TGP signaling stack.

This replaces older pre-extension and pre-CoreProver assumptions.

⸻

1. Rebuild Purpose

The system matured significantly:
	•	A new TGP Client Runtime (TGP-CP-00)
	•	A secure Browser Extension implementation (TGP-EX-00)
	•	A stable, policy-driven TBC-00 Gateway architecture
	•	A generalized Payment Profile settlement contract
	•	Standardized TxIP signaling primitives
	•	A pathway for x402 agent compatibility

The rebuild effort aligns implementation with the final architecture.

⸻

2. Components Requiring Reconstruction

2.1 TGP Client Runtime (Core)

This component was previously assumed to be wallet-integrated.
Now it becomes a standalone client implementing TGP-CP-00:

Required features:
	•	x402 payment event detection
	•	TGP QUERY builder
	•	TGP ACK parser
	•	Transaction builder from ACK
	•	Routing logic (direct or relay)
	•	Extension communication
	•	Session tracking
	•	Timeout enforcement

This is the heart of the client-side rebuild.

⸻

2.2 Browser Extension (TGP-EX-00)

The browser extension becomes the default TGP client environment.

Required features:
	•	service worker for background processing
	•	content script for x402 detection
	•	isolated world injection of Presence API
	•	secure message passing
	•	safe construction of queries
	•	rendering of UI toggle + logs
	•	strict minimal permissions
	•	compatible with Chrome/Brave/Safari/Firefox

This replaces early prototype assumptions.

⸻

2.3 Transaction Border Controller (TBC-00)

Core responsibilities:
	•	TGP QUERY processing
	•	policy evaluation
	•	escrow verb selection
	•	session state tracking
	•	TGP ACK construction
	•	relay endpoint for signed transactions

Rebuild Tags:
	•	switch to QUERY/ACK engine
	•	remove legacy routing logic
	•	integrate settlement ABI
	•	support session_tuple state model
	•	expose health/version endpoints
	•	standardized logging

TBC is now the policy firewall for all transactions.

⸻

2.4 Settlement Layer (Payment Profile Contract)

Previously: “escrow contract”
Now: Payment Profile implementing TPP-00.

Rebuild Needs:
	•	implement commit/accept/fulfill/claim
	•	expose read-only state helpers
	•	enforce verb ordering
	•	emit TGP-compatible events
	•	support multi-chain deployment
	•	simple emulator for testing

This isolates on-chain economic behavior from client & TBC logic.

⸻

2.5 Integration with x402 (X402-EXT)

We now support structured payment negotiation via x402.

Rebuild Tasks:
	•	map payment_required → TGP QUERY
	•	implement metadata extraction
	•	support agent workflows
	•	verify compliance with PR #593
	•	document any custom fields in TGP metadata

This bridges our world to the emerging agent-to-agent protocols.

⸻

3. Rebuild Roadmap (Step-by-Step)

Phase 1 — Foundations
	1.	Finalize all specs (TGP-00, CP-00, EX-00, TBC-00, TPP-00 draft).
	2.	Migrate repo to clean spec structure.
	3.	Build TBC mock server for testing.
	4.	Write integration tests for QUERY/ACK with mock contract.

⸻

Phase 2 — Client Runtime
	1.	Implement TGP-CP runtime core in TypeScript.
	2.	Implement QUERY builder / ACK parser.
	3.	Implement transaction builder.
	4.	Support relay vs direct routing.
	5.	Add session timeout enforcement.
	6.	Add logging & debug hooks.

Deliverable:
Client SDK (coreprover-sdk analog for TGP?)

⸻

Phase 3 — Browser Extension
	1.	Build MV3-compliant extension skeleton.
	2.	Implement content script for x402 capture.
	3.	Implement service worker message router.
	4.	Inject Presence API.
	5.	Implement client <-> extension messaging.
	6.	Integrate TGP-CP runtime into extension.
	7.	Add optional UI controls.
	8.	Implement privacy & permissions review.

Deliverable:
TGP-Extension v0.1

⸻

Phase 4 — TBC Gateway
	1.	Implement /tgp/query endpoint
	2.	Implement policy engine (minimal)
	3.	Hook up settlement ABI
	4.	Implement session state machine
	5.	Implement /tgp/relay for signed tx
	6.	Implement logs + tracing
	7.	Implement TBC health endpoints
	8.	Build end-to-end test harness

Deliverable:
TBC-00 Reference Gateway

⸻

Phase 5 — Settlement Contract
	1.	Write TPP-00 contract
	2.	Add automated tests
	3.	Deploy to local testnet
	4.	Integrate with TBC ABI calls
	5.	Run E2E commit/accept/fulfill/claim demo

Deliverable:
Payment Profile Contract (minimal) v0.1

⸻

Phase 6 — End-to-End Demo (“Happy Path”)
	•	Browser extension detects x402 event
	•	Query → TBC
	•	ACK → Client
	•	Wallet signs
	•	Commit/fulfill/claim via contract
	•	Session completes

Deliverable:
Protected Transaction Demo v1.0

⸻

Phase 7 — Agent Compatibility
	•	Implement x402 agent harness
	•	Add MCP driver for agent simulation
	•	Demonstrate autonomous but constrained spend behavior

Deliverable:
Autonomous Payment Agent Demo

⸻

4. Engineering Priorities (Condensed)
	1.	Finish specs
	2.	Implement TGP Client Runtime
	3.	Build Browser Extension
	4.	Complete TBC Engine
	5.	Write Settlement Contract
	6.	End-to-end integration
	7.	Agent interoperability

Everything else is secondary.

⸻

5. Removed or Replaced Components

The rebuild removes:
	•	legacy wallet-side integration model
	•	early NAT/VGP narratives
	•	any assumptions requiring wallet RPC hooks
	•	direct transaction settlement from client
	•	pre-TGP escrow logic

These are replaced by:
	•	clear Client → TBC → Wallet → Contract pipeline
	•	formal QUERY/ACK messaging
	•	deterministic transaction construction
	•	Presence API
	•	x402 binding
	•	browser extension runtime

⸻

6. Summary

This rebuild aligns the entire project to a modern, secure, modular architecture:
	•	TGP manages signaling
	•	TBC enforces policy & constructs transactions
	•	Wallet signs
	•	Settlement contract executes
	•	Browser extension hosts the client
	•	Agents integrate via x402

The result is the first transaction control plane for blockchain-based autonomous and policy-bound payments.

⸻

🔚 End of Updated Rebuild.md


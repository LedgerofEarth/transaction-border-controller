
🧭 Wednesday Scrum — CoreProve / TBC Sprint

Date: 11/26/25
Sprint: Day 3 Kickoff
Project: CoreProve + Transaction Border Controller (TBC)
Prepared for: Team
Format: Markdown (.md)

⸻

## 🟦 Day 2 Recap (Accurate & Complete)

A. CoreProve – Module B (Browser Extension)

Completed:
	•	First full implementation pass of Module B
	•	Integrated ZK hooks into background workflow
	•	ACK → UI pipeline complete
	•	Commitment → ACK lifecycle wired end-to-end
	•	i18n map built
	•	Response state animations (shimmer, glow) implemented
	•	All interface typings created
	
	-First customer meeting.  NDA and LOI signed
	-Chrome Extension Approved and published in Google Store (Desktop only).  AMO(Mozilla/Android) remains for deployment.

Remaining:
	•	Merge Module B component directories into a coherent build tree

⸻

B. CoreProve – Module A (Solidity Contracts)

Completed (≈90%):
	•	Full review of Settlement.sol
	•	Full review of ReceiptVault.sol
	•	Confirmed lifetime and routing behaviors of Receipt NFTs
	•	ABI typings for both generated

Remaining (Day 3):
	•	Final 10% of Settlement.sol
	•	Full review of MerchantFactory.sol
	•	Generate missing ABI typings
	•	Verify Settlement/ACK/Withdraw architecture end-to-end

⸻

C. TGP-00 v3.2 Migration (Done)
	•	OFFER removed
	•	ACK added
	•	All TGP message types updated (Query, Ack, Settle, Error)
	•	validation.rs, messages.rs, types.rs, and state.rs fully aligned
	•	Removed deprecated or policy-related directories
	•	Repo now matches the clean TGP-00 v3.2 handshake model

⸻

D. Repo Cleanup & Architecture Hardening
	•	tbc-core → protocol types & pure logic
	•	tbc-gateway → routing, handlers, codec
	•	tbc-node → runtime, RPC, WS, config
	•	Removed unused enforcement/policy folders
	•	Tightened imports and boundaries
	•	Achieved clean separation of protocol, gateway, and node layers

⸻

E. Day 2 Surprise Win — WebSocket Transport Layer

While cleaning repo structure, we added a major feature:

/ws/tgp WebSocket Transport
	•	Bi-directional real-time TGP messaging
	•	Ideal for DirectPay, ACK streaming, instant settlement feedback
	•	Zero state pollution (gateway is stateless per spec)
	•	Works with JS client → “one line connect”
	•	Support for JSON TGP envelopes
	•	Fully aligned with TGP-00 v3.2

This significantly improves extension → TBC round-trip speed. Great for Shannon’s “Direct Pay” feature, Direct Pay can feel instant now.

⸻

## 🟦 Day 3 Objectives

1. Finalize WebSocket Transport
	•	Implement ping/pong
	•	Graceful shutdown
	•	Per-connection metadata
	•	WS → Router → Encode/Decode → WS return path
	•	Build WS test harness

⸻

2. Unified Transport Interface

Create a single ingress path for both HTTP and WS:
	•	TransportMessage abstraction
	•	Unified dispatcher
	•	Shared error model
	•	Identical replay protection rules
	•	Makes extension fallbacks trivial

⸻

3. RPC & tx_builder Scaffolding

Settlement lifecycle stubs:
	•	buyer_commit()
	•	seller_commit()
	•	settle()
	•	withdraw()

And RPC helpers for:
	•	getChainId()
	•	estimateGas()
	•	sendRawTx()
	•	Receipt polling

(Implementation is Day 5—today is scaffolding.)

⸻

4. Solidity Integration Completion
	•	Review all MerchantFactory.sol logic
	•	Final 10% review of Settlement.sol
	•	Regenerate ABIs
	•	Validate final flow:
	•	Query → Ack → Commit → Settle → Withdraw
	•	Session ID propagation
	•	Event emissions
	•	Merchant escrow pattern

⸻

5. Module B Merge
	•	Merge popup, background, i18n, ZK hooks, and UI effects
	•	Produce a consolidated build directory
	•	Identify missing glue code

⸻

6. Gateway Error Model

Define unified errors:
	•	TransportError
	•	ProtocolError
	•	GatewayError
	•	SystemError

Ensure consistency across HTTP + WS + Extension.

⸻

7. Day 3 Test Suite

Target tests:
	•	HTTP → ACK
	•	WS → ACK
	•	WS → SETTLE
	•	codec_tx round-trip
	•	Replay-protection tests
	•	Stateful vs stateless handler separation

⸻


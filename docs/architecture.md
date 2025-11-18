Version: 0.4
Status: Draft
Author: Ledger of Earth
Purpose: Provide a high-level overview of the Transaction Border Controller (TBC) system, the Transaction Gateway Protocol (TGP), and the surrounding components that form the secure, Layer-8 control plane for agent-driven blockchain transactions.

⸻

1. Architectural Overview

The TBC system introduces a Layer-8 economic control plane for blockchain transactions.
It establishes a policy-aware transaction firewall, similar in spirit to:
	•	Session Border Controllers (SBCs) in VoIP
	•	NAT/FW appliances in networking
	•	Zero-trust gateways in enterprise security

TGP defines the signaling protocol between Clients and a TBC.
The TBC evaluates policy, constructs correct settlement flows (including escrow), and returns executable transaction specs.

Key goals:
	•	Make blockchain transactions safe for AI agents
	•	Enable session-based spend limits
	•	Route payments through escrow automatically
	•	Provide cross-domain compliance boundaries
	•	Maintain user control without requiring wallet modifications

This architecture cleanly separates:
	•	Authorization
	•	Construction
	•	Signing
	•	Routing
	•	Settlement

across distinct components.

⸻

2. System Components

The architecture is composed of four primary components:

┌──────────────────────────┐
│        TGP Client         │  (Browser extension or wallet module)
│    (implements TGP-CP)    │
└──────────────┬───────────┘
               │ TGP Query / Ack
               ▼
┌──────────────────────────┐
│   Transaction Border      │
│       Controller          │  (TBC-00)
│  (Policy + Routing Engine)│
└──────────────┬───────────┘
               │ Constructed Transaction Spec
               ▼
┌──────────────────────────┐
│         Wallet            │  (Blind signer)
│    (EIP-1193 standard)    │
└──────────────┬───────────┘
               │ Signed Transaction
               ▼
┌──────────────────────────┐
│  RPC or Escrow Contract   │  (TPP-00)
│     Settlement Profile     │
└──────────────────────────┘


⸻

3. TGP Client Runtime

The Client implements the TGP-CP-00 spec.
It is responsible for:
	•	detecting x402 payment_required
	•	sending TGP QUERY to a TBC
	•	receiving TGP ACK
	•	constructing a transaction from ACK
	•	calling the wallet for signing
	•	routing signed tx to RPC or TBC relay

The Client performs no signing, no key management, and no wallet modification.

There are two implementations:

3.1 Browser Extension (TGP-EX-00)

The primary, default client runtime.

Advantages:
	•	no wallet dependencies
	•	works with MetaMask, Rabby, Internet Money, etc.
	•	distributable without OS integration
	•	compliant with Chrome MV3, Firefox, Safari WKWebExtension
	•	ideal for AI agent compatibility

3.2 Wallet Native Integration (Optional)

Wallets (e.g., Internet Money) may optionally integrate the Client runtime.

Wallet integration is not required.
The presence API allows wallets to detect the extension.

⸻

4. Transaction Border Controller (TBC)

The TBC is the policy + routing engine and implements TBC-00.

Responsibilities:
	•	receive TGP QUERY
	•	validate session, policy, jurisdiction, spend limits
	•	determine correct escrow verb
	•	construct transaction specifications
	•	return TGP ACK
	•	relay signed transactions (if routing mode = relay)
	•	maintain per-session state

The TBC is a stateless HTTP service backed by optional session storage.

It does not hold private keys or broadcast unsigned transactions.

⸻

5. Payment Profile Contract (Escrow / Settlement Layer)

Each merchant or payment flow is defined by a Payment Profile contract implementing TPP-00 (to be added).

This contract defines the on-chain settlement state machine:

commit → accept → fulfill → verify → claim

The TBC determines the correct verb.
The Client constructs the transaction.
The Wallet signs.
RPC or TBC relays the signed transaction.

This separation ensures:
	•	merchants define economic rules
	•	TBC enforces them
	•	Client executes them
	•	wallet remains unmodified

⸻

6. Wallet (Blind Signer)

Wallets remain:
	•	key managers
	•	signature providers
	•	popup UX surfaces

Wallets:
	•	do NOT parse TGP
	•	do NOT evaluate policy
	•	do NOT interact with the TBC
	•	do NOT implement escrow logic
	•	only detect the presence flag exposed by TGP-EX

This keeps wallets simple and avoids security risk.

⸻

7. Presence Detection (TGP-PRES-00)

Wallets can detect the Client extension using the Presence API:

window.tgp = {
  version: "0.1",
  active: true,
  tbc: { reachable: true }
};

And an event:

document.dispatchEvent(new CustomEvent("tgp:present", {...}));

Wallets may display:
	•	“TGP Mode Available”
	•	“Protected Mode Enabled”

Integration remains optional.

⸻

8. x402 → TGP Binding

TGP activates when an x402-compliant system emits:
	•	payment_required
	•	payment_intent
	•	payment_options

AI agents or dApps simply attach:
	•	payment profile contract
	•	amount
	•	chain ID
	•	metadata

This provides universal compatibility without needing to modify wallets or dApps.

⸻

9. End-to-End Transaction Sequence

1. Agent/dApp → Client:
     x402 payment_required(payment_profile, amount, chain_id)

2. Client → TBC:
     TGP QUERY (intent: commit)

3. TBC → Client:
     TGP ACK (tx spec, next verb, routing)

4. Client → Wallet:
     eth_sendTransaction(...)

5. Wallet → Client:
     signed_tx

6. Client → RPC or TBC:
     broadcast or relay

7. Contract:
     state transition (commit)

8. Client:
     checks if further verbs needed
     if yes → repeat QUERY/ACK loop


⸻

10. Security Model

The architecture enforces:
	•	user authorization via wallet signature
	•	policy checks via the TBC
	•	transaction correctness via ACK construction
	•	domain separation (Client vs TBC vs Wallet)
	•	zero key handling in extensions or TBC
	•	HTTPS-only communication
	•	session replay protection
	•	deterministic transaction construction

This creates the first safe model for autonomous/agentic blockchain transactions.

⸻

11. Deployment Model

Three deployable components:
	1.	Browser Extension (TGP-EX)
distributed via Chrome/Firefox/Safari/Brave stores
	2.	TBC Appliance (Docker, cloud or on-prem)
runs behind the user’s or organization’s trust boundary
	3.	Payment Profile Contracts
deployed by merchants or platform operators

This modular design avoids centralization risk and maximizes composability.

⸻

12. Summary

The TBC/TGP architecture establishes a transaction control plane above existing blockchain networks without modifying:
	•	wallets
	•	RPC nodes
	•	blockchain consensus
	•	merchant applications
	•	dApp frontend architecture

It introduces:
	•	safety
	•	clarity
	•	compliance
	•	policy routing
	•	sessionized payment flows
	•	agent compatibility

…while remaining fully decentralized and user-controlled.

⸻

🔚 End of architecture.md

⸻

If you want:
	•	I can generate diagrams for this doc
	•	Or produce an expanded version with examples
	•	Or build the matching “system_topology.md”
	•	Or integrate this into your repo in PR format


Version: 0.4
Status: Canonical
Author: Ledger of Earth
Purpose: Describe the entire end-to-end topology of the TGP/TBC system across clients, gateways, wallets, and on-chain settlement.

⸻

1. High-Level Topology

The TBC architecture establishes a secure Layer-8 control plane between:
	•	TGP Clients
	•	The TBC Gateway
	•	Wallet Signers
	•	Settlement Contracts
	•	Blockchain Networks
	•	Agent and x402-powered systems

At a high level:

┌─────────────────────────┐
│        Applications      │
│  (dApps, Agents, x402)   │
└───────────────┬─────────┘
                │ x402 payment_required
                ▼
┌─────────────────────────┐
│       TGP Client        │  (Browser Extension / Wallet Module)
│       TGP-CP-00         │
└──────────────┬──────────┘
               │ TGP QUERY
               ▼
┌─────────────────────────┐
│ Transaction Border Ctrl │  (TBC-00)
│ Policy + Routing Engine │
└──────────────┬──────────┘
               │ TGP ACK (tx spec)
               ▼
┌─────────────────────────┐
│         Wallet          │  (Signer, EIP-1193)
│  Blind to TGP semantics │
└──────────────┬──────────┘
               │ signed_tx
               ▼
┌─────────────────────────┐
│   RPC / Settlement SC   │ (TPP-00)
│   commit/fulfill/claim  │
└─────────────────────────┘

This is the complete transaction path.

⸻

2. Topology Components

There are six major architectural components:
	1.	TGP Client (Browser Extension or Wallet Module)
	2.	Presence API for Wallet Detection
	3.	Transaction Border Controller (TBC)
	4.	Settlement Contract (Payment Profile)
	5.	Wallet Signer
	6.	Blockchain Network / RPC Layer

Each is explained below.

⸻

3. Applications & x402 Layer

Applications (agents, dApps, automated workflows) do NOT communicate directly with the TBC or wallet.
Instead, they emit x402 payment negotiation messages:
	•	payment_required
	•	payment_options
	•	payment_intent

The TGP Client subscribes to these.

Agents communicate via x402 and do not require modification.

⸻

4. TGP Client Runtime (TGP-CP-00 / TGP-EX-00)

The Client sits between:
	•	Application (x402)
	•	TBC
	•	Wallet

and performs:
	1.	Detect x402 payment request
	2.	Generate TGP QUERY
	3.	Send QUERY → TBC
	4.	Receive ACK
	5.	Build transaction exactly per ACK rules
	6.	Request signature from wallet
	7.	Route signed transaction either:
	•	directly to RPC
	•	to TBC’s relay endpoint

The Client does not sign, broadcast, or store any keys.

4.1 Browser Extension Implementation (TGP-EX-00)

This is the default, recommended implementation for the MVP.
	•	Works with any wallet
	•	Exposes Presence API
	•	Runs inside browser sandbox
	•	Complies with Chrome MV3, Firefox, Safari

Wallet integration is optional; wallet compatibility is automatic.

⸻

5. Presence API (Wallet Detection)

The TGP-EX runtime injects a safe, minimal API:

window.tgp = {
  version: "0.1",
  active: true,
  tbc: { reachable: true }
};

Wallets can:
	•	detect TGP presence
	•	show UX indicators
	•	adapt signature prompts

Wallets do not need to implement TGP logic.

⸻

6. Transaction Border Controller (TBC-00)

The TBC is a policy and routing firewall for transactions.

Responsibilities:
	•	accept TGP QUERY
	•	validate policy / limits / jurisdiction
	•	determine escrow state machine action
	•	construct transaction specification
	•	return TGP ACK
	•	optionally relay signed tx
	•	keep session state

The TBC is stateless outside session tracking.

6.1 TBC API Surface

POST /tgp/query
POST /tgp/relay
GET /tgp/session/:id
GET /tgp/health
GET /tgp/version

This is the only interface the Client must use.

⸻

7. Settlement Layer (Payment Profile Contract)

Settlement contracts implement:

commit → accept → fulfill → verify → claim

The TBC determines which verb is next.
The Client constructs that transaction.
The Wallet signs.
RPC executes.

The topology ensures:
	•	no business logic lives in wallet
	•	no policy logic lives in the Client
	•	no signing lives in TBC
	•	the settlement contract contains only on-chain mechanics

⸻

8. Wallet (Blind Signer)

Wallets remain unchanged:
	•	key management
	•	popup UX
	•	signing
	•	provider API

They do NOT:
	•	parse TGP
	•	evaluate policy
	•	communicate with TBC
	•	track sessions
	•	interpret escrow states

This keeps the architecture safe and backwards compatible.

⸻

9. RPC Layer / Blockchain Nodes

The final hop is either:
	1.	Direct RPC
	2.	TBC Relay RPC (if ACK mode = relay)

The blockchain executes:
	•	commit
	•	accept
	•	fulfill
	•	claim

and transitions settlement state accordingly.

⸻

10. Expanded Topology Diagram

A more detailed full-path view:

   ┌──────────────────────────────────────────────────────────────┐
   │                     Application Layer (Agents/dApps)          │
   │            x402 payment_required(contract, amount, chain)     │
   └───────────────────────────────┬───────────────────────────────┘
                                   │
                     (1) x402 event detected
                                   │
   ┌────────────────────────────────┴──────────────────────────────┐
   │                      TGP Client Runtime                       │
   │              (Browser Extension: TGP-EX-00)                   │
   │   - Build QUERY                                                │
   │   - Send QUERY → TBC                                           │
   │   - Receive ACK                                                 │
   │   - Build transaction                                          │
   │   - Request wallet signature                                   │
   │   - Relay or RPC broadcast                                     │
   └───────────────────────────────┬───────────────────────────────┘
                                   │ (2) TGP QUERY
                                   ▼
   ┌──────────────────────────────────────────────────────────────┐
   │               Transaction Border Controller (TBC-00)          │
   │   - Policy evaluation                                         │
   │   - Escrow verb computation                                   │
   │   - Construct tx spec                                         │
   │   - Return ACK                                                │
   │   - Optional relay                                            │
   └───────────────────────────────┬───────────────────────────────┘
                                   │ (3) TGP ACK
                                   ▼
   ┌──────────────────────────────────────────────────────────────┐
   │                         Wallet Signer                        │
   │        (e.g., MetaMask, Rabby, Internet Money)               │
   │   - Show popup                                                │
   │   - User approves                                             │
   │   - Sign transaction                                          │
   └───────────────────────────────┬───────────────────────────────┘
                                   │ (4) signed_tx
                                   ▼
   ┌──────────────────────────────────────────────────────────────┐
   │           Blockchain RPC / Settlement Contract                │
   │              (commit / accept / fulfill / claim)             │
   └──────────────────────────────────────────────────────────────┘


⸻

11. Trust Boundaries

The system enforces strict separation of responsibilities:

Component	Trust Boundary	Notes
Extension	Zero-trust sandbox	No keys, no sensitive DOM access
TBC	Policy trust domain	Does not sign or custody
Wallet	Key boundary	Signs only
Settlement Contract	On-chain economic rules	Deterministic
RPC Layer	Network trust	Existing blockchain rules

This layered design is the foundation of the TBC security model.

⸻

12. Multi-Domain Topology (Future)

The architecture supports:
	•	multiple TBCs per user
	•	federated TBC nodes
	•	cross-jurisdiction routing
	•	enterprise or national deployments
	•	AI-agent managed spending accounts
	•	ZK settlement proofs

But none of this is required for MVP.

⸻

13. Summary

The final topology is:

Application → TGP Client → TBC → Wallet → Settlement Contract → Blockchain

This forms the complete transaction control plane enabling safe, policy-enforced, escrow-oriented blockchain payments for both humans and autonomous agents.

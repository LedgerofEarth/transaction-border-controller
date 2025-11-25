TGP-00 v3.2 — Transaction Gateway Protocol

Status: Draft
Author: Ledger of Earth
Scope: Core wire protocol definition
Audience: Gateway implementers, client authors, settlement engine developers

⸻

0. Abstract

The Transaction Gateway Protocol (TGP) defines a deterministic, chain-agnostic communication protocol that enables untrusted parties to coordinate and execute blockchain transactions through a payment gateway. A Transaction Border Controller (TBC) is one such gateway implementation, providing contract-aware validation and verification before a transaction is permitted to proceed.

TGP specifies how a client expresses transaction intent, how a gateway evaluates that intent using layered verification, and how approved transactions are returned as executable Economic Envelopes suitable for signing and submission by standard wallets. TGP does not modify wallet behavior, custody keys, or require specialized signing engines. Instead, it establishes a reproducible, protocol-level control plane for payments, escrow flows, and settlement events across heterogeneous blockchain systems.

Version 3.2 introduces:
	•	A normalized message taxonomy (QUERY, ACK, ERROR, SETTLE)
	•	ACK.status = “offer” as a preview state
	•	SETTLE as a terminal settlement notification
	•	WITHDRAW as a QUERY.intent.verb
	•	HTTP 402 Payment Required as canonical trigger (x402 optional)

⸻

1. Scope

TGP-00 defines:
	•	Protocol architecture and participating actors
	•	Message structure, serialization, and normative rules
	•	The deterministic transaction model
	•	Layered verification model (L1–L6)
	•	Core message types: QUERY, ACK, ERROR, SETTLE
	•	Core verbs (within QUERY.intent.verb)
	•	Optional routing metadata for cross-gateway traversal, including Transaction Area IDs and hop-path fields
	•	Economic Envelope abstraction
	•	Minimal examples

TGP-00 does not define:
	•	Smart contract bytecode (see CoreProve-00)
	•	Client runtime logic (see TGP-CP-00, TGP-EX-00)
	•	Zero-knowledge proof formats (see CoreProve-ZK-00)
	•	Merchant UX or smart contract deployment models (see CoreProve-00)
	•	Routing policies or forwarding rules for QUERY messages between gateways (defined in a future specification)

⸻

2. Protocol Architecture

TGP organizes transaction execution around four primary actors and a payment gateway.
A Transaction Border Controller (TBC) is one such gateway implementation; other gateways MAY exist within a federated or jurisdictional routing environment.

┌───────────────────────────────────────────┐
│                 Merchant                  │
│   (Website, API, AI agent, QR trigger)    │
└───────────────────────────────────────────┘
                    │
                    ▼
       HTTP 402 Payment Required (canonical)
       x402 metadata, QR, API calls (optional)
                    ▼
┌───────────────────────────────────────────┐
│                 Client                    │
│ (Extension, wallet module, agent runtime) │
└───────────────────────────────────────────┘
                    │  QUERY
                    ▼
┌───────────────────────────────────────────┐
│                 Gateway                   │
│  (TBC or other verification domain node)  │
│ Performs Layered Verification (L1–L6)     │
│ MAY forward QUERY to other gateways       │
└───────────────────────────────────────────┘
                    │  ACK / ERROR
                    │  SETTLE (terminal)
                    ▼
┌───────────────────────────────────────────┐
│              Settlement Engine            │
│    (Escrow, commit/accept/claim flows)    │
└───────────────────────────────────────────┘


⸻

2.1 Client

A TGP-aware node responsible for:
	•	Constructing QUERY messages
	•	Including routing metadata when required
	•	Processing ACK and executing approved transactions via wallet
	•	Routing signed transactions according to the Economic Envelope
	•	Listening for terminal SETTLE messages

Direct Pay Initiation
A Client MAY initiate a TGP exchange without a merchant-originated 402 signal:
	1.	User selects Direct Pay
	2.	Client collects:
	•	Amount
	•	Merchant payment URL or QR-derived profile
	3.	Client constructs a QUERY
	4.	Gateway evaluates as normal

Direct Pay treats the user as the initiating party.
It does not alter core TGP semantics.

⸻

2.2 Gateway (Including TBC)

A Gateway is any node capable of receiving, validating, and optionally forwarding QUERY messages.
The TBC is the canonical implementation.

Gateways perform:
	•	Registry and identity validation
	•	Contract and RPC validation
	•	Policy enforcement
	•	Economic Envelope construction
	•	Escrow and withdrawal eligibility checks

Gateways MAY:
	•	Forward QUERY messages using routing metadata
	•	Append their Transaction Area ID to the hop path
	•	Add gateway-specific fees

Gateways MUST:
	•	Return deterministic ACK or ERROR messages
	•	Emit SETTLE upon terminal settlement

⸻

2.3 Merchant

Merchants MAY initiate TGP flows via:
	•	HTTP 402 Payment Required (canonical)
	•	x402 metadata
	•	QR codes
	•	Local application triggers
	•	API calls

Direct Pay Clarification
Merchant does not emit the trigger;
Client constructs its own QUERY.

⸻

2.4 Settlement Engine

The Settlement Engine refers to on-chain contracts governing:
	•	Payment
	•	Escrow
	•	Commit/accept/claim
	•	Refund
	•	Timeout

These contracts are the source of truth for settlement.

Gateways observe settlement contract state.
When the contract reaches a terminal condition—such as:
	•	funds claimed
	•	escrow released
	•	refund triggered
	•	timeout reached
	•	transaction reverted

—the Gateway MUST emit a SETTLE message to the Client.

SETTLE is generated by the Gateway, not by the contract.

⸻

3. Design Principles

TGP is a deterministic, stateless, chain-agnostic communication protocol for coordinating transaction intent and authorization between untrusted parties through one or more gateways.

3.1 Deterministic

Gateways receiving identical input MUST produce identical ACK or ERROR responses.

3.2 Stateless

Gateways maintain no application state and do not store client sessions.
Gateways MUST NOT persist delegated session keys or session_token.
All verification state MUST be derived from the QUERY and blockchain.

3.3 Transparent

All executable transaction details appear in the Economic Envelope.
Clients MUST NOT synthesize or alter transaction parameters.

3.4 Wallet-Blind

TGP does not modify wallet behavior.
Wallets remain standard signers and are unaware of TGP.

3.5 Fail-Closed

Verification failure at any layer MUST produce an ERROR.

3.6 Pluggable Transport

TGP MAY operate over HTTPS, WebSocket, or future transports.

3.7 Federated Gateway Support

TGP supports optional routing metadata for multi-gateway traversal.
Routing policy is defined externally (TGP-RT-00).

⸻

4. The TGP Transaction Model

TGP defines a deterministic, message-driven model consisting of:
	•	QUERY — transaction intent
	•	ACK — authorization response
	•	ERROR — verification failure
	•	SETTLE — terminal settlement

ACK.status = “offer” serves as a preview, replacing the older OFFER message.

TGP supports cryptographically scoped session contexts carried within each QUERY.

⸻

4.1 QUERY — Transaction Intent

A QUERY expresses:
	•	verb (COMMIT, PAY, CLAIM, WITHDRAW, QUOTE)
	•	party (BUYER or SELLER)
	•	routing metadata
	•	payment profile
	•	chain ID
	•	optional session_token
	•	optional delegated_key
	•	optional scope
	•	additional metadata

⸻

4.2 ACK — Authorization

ACK MAY be:
	•	status = “offer”
	•	status = “allow”
	•	status = “deny”
	•	status = “revise”

status = “allow” MUST include a full Economic Envelope.

⸻

4.3 ERROR — Verification Failure

Gateways MUST return ERROR for any failure in L1–L6.
An error terminates the attempt.

⸻

4.4 SETTLE — Terminal

Emitted when settlement contracts reach terminal state.

⸻

4.5 Transaction Lifecycle Summary

A complete lifecycle may appear as:

QUERY → ACK(status=offer)
QUERY → ACK(status=allow)
(sign & send)
→ SETTLE (from Gateway)


⸻

5. Message Types (Normative)

5.1 QUERY (Client → Gateway)

{
  “type”: “QUERY”,
  “tgp_version”: “3.2”,
  “id”: “uuid”,
  “session_token”: “<opaque-or-null>”,
  “delegated_key”: “<public-key-or-null>”,
  “scope”: { ... },

  “transaction_area_id”: “TAID-1234”,
  “path”: [“TAID-0001”, “TAID-0033”],
  “next_gateway”: “https://tbc.example”,

  “intent”: {
    “verb”: “COMMIT | PAY | CLAIM | WITHDRAW | QUOTE”,
    “party”: “BUYER | SELLER”,
    “mode”: “DIRECT | SHIELDED”
  },

  “payment_profile”: “0xContract”,
  “amount”: “1000000”,
  “chain_id”: 369,
  “metadata”: {}
}

Rules:
	•	MUST NOT contain private keys
	•	MUST include intent.verb
	•	MUST be rejected if malformed
	•	transaction_area_id, path, and next_gateway are optional
	•	Gateways MUST NOT remove prior hop entries in path
	•	session_token MAY convey a session context
	•	delegated_key MAY be present
	•	scope MAY define restrictions but MUST NOT override intent
	•	TGP-00 does not define routing semantics

⸻

5.2 ACK (Gateway → Client)

{
  “type”: “ACK”,
  “status”: “offer | allow | deny | revise”,
  “id”: “uuid”,
  “intent”: { “verb”: “COMMIT” },

  “transaction_area_id”: “TAID-9999”,
  “path”: [“TAID-0001”, “TAID-9999”],

  “tx”: {
    “to”: “0xPaymentProfile”,
    “value”: “1000000”,
    “data”: “0x...”,
    “chain_id”: 369,
    “gas_limit”: 200000
  },

  “routing”: {
    “mode”: “direct | relay”,
    “rpc_url”: “https://rpc.example”,
    “tbc_endpoint”: “https://gateway.example”
  },

  “expires_at”: “2025-11-18T15:00:00Z”
}

Rules:
	•	offer MUST NOT include executable tx
	•	allow MUST include a full Economic Envelope
	•	Client MUST execute exactly as issued
	•	Modification requires new QUERY
	•	Gateways MAY include updated routing metadata

⸻

5.3 ERROR

{
  “type”: “ERROR”,
  “code”: “TGP_Lx_FAILURE”,
  “layer_failed”: 3,
  “message”: “Signature invalid”
}

Rules:
	•	MUST be returned for any verification failure
	•	MUST be deterministic

⸻

5.4 SETTLE (Terminal)

{
  “type”: “SETTLE”,
  “id”: “uuid”,
  “result”: {
    “final_status”: “complete | timeout | released | claimed | refunded | reverted”,
    “escrow_id”: “0xEscrow”
  },
  “timestamp”: “2025-11-18T15:00:05Z”
}

Rules:
	•	MUST NOT include executable tx
	•	Represents final lifecycle state
	•	MUST be delivered when settlement finalizes

⸻

6. Layered Verification Model (L1–L6)

Gateways MUST evaluate layers sequentially:
	•	L1 — Registry Check
	•	L2 — Cryptographic Validation
	•	L3 — Contract Bytecode & RPC Integrity
	•	L4 — ZK / Attestation (optional)
	•	L5 — Policy Evaluation
	•	L6 — Escrow / WITHDRAW Eligibility

Failure at any layer MUST produce ERROR.

⸻

7. Economic Envelope (Normative)

An Economic Envelope describes:
	•	Exact calldata
	•	Fees
	•	Routing directives (mode, rpc_url, tbc_endpoint)
	•	Deadlines
	•	Contract reference
	•	Intent verb

Envelope MUST be consumed exactly as issued.

⸻

8. Minimal Examples

8.1 Commit Flow

QUERY → ACK(status=offer)
QUERY → ACK(status=allow)
(sign & send)
→ SETTLE (from Gateway)

8.2 Buyer Timeout Withdraw

QUERY (WITHDRAW) → ACK(status=offer)
QUERY (WITHDRAW) → ACK(status=allow)
→ SETTLE (from Gateway)

8.3 Seller Claim & Direct Settlement

QUERY → ACK(status=allow)
(sign & send)
→ SETTLE (from Gateway)


⸻

9. Normative Rules Summary
	•	Client MUST obey ACK transaction fields verbatim
	•	Gateways MUST respond deterministically
	•	WITHDRAW MUST be validated at L6
	•	TGP-00 MUST remain chain-agnostic
	•	Privacy, UI, routing, and ZK proofs are out-of-scope
	•	Any deviation MUST result in ERROR

⸻

End of TGP-00 v3.2

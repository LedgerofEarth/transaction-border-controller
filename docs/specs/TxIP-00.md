TxIP-00: Transaction Interchange Protocol

Version: 0.2-draft
Date: November 14, 2025
Status: Draft
Authors: David Bigge, Shannon Jolin

⸻

0. Purpose & Scope

TxIP-00 defines the wire-level protocol used by:
	•	AI agents
	•	Wallet frontends
	•	Transaction Border Controllers (TBCs)
	•	Settlement / watcher services

to exchange transaction coordination messages in a consistent way.

Think of TxIP as:
	•	The transport + envelope for transaction coordination
	•	TGP-00 messages ride inside TxIP envelopes
	•	It standardizes:
	•	Session setup and capability negotiation
	•	Message envelope & correlation
	•	Transport bindings (HTTP/WebSocket)
	•	Error and retry semantics
	•	Basic auth / identity hooks

TxIP does NOT:
	•	Define economic logic (that’s TGP Layer 8)
	•	Define identity semantics (Layer 9)
	•	Define policy semantics (Layer 10)

It just ensures that whatever you’re sending (QUERY/OFFER/SETTLE/EVENT from TGP) gets from Agent → TBC → Counterparty in a predictable, debuggable way.

⸻

1. Architecture Overview

1.1 Roles

TxIP is used between these logical roles:
	•	Client Agent
	•	AI agent, wallet app, or service that wants to initiate or receive commerce
	•	Speaks TxIP to the TBC
	•	Transaction Border Controller (TBC)
	•	Core router that enforces Layer 8/9/10 decisions
	•	Speaks TxIP to clients and to other TBCs
	•	Settlement / Watcher Service
	•	Observes on-chain CoreProver events
	•	Emits TGP EVENT and SETTLE messages via TxIP
	•	Peer TBCs (optional)
	•	Federation between domains / jurisdictions

1.2 Layering
	•	TxIP (this spec): transport + envelope; defines sessions, framing, error codes
	•	TGP-00: message semantics (QUERY/OFFER/SETTLE/EVENT + escrow state machine)
	•	CoreProver: smart contracts + receipt vault

You can think of each TxIP message as:

+----------------------------------------------------+
|  TxIP Envelope (version, session, routing, error)  |
+----------------------------------------------------+
|  TGP Payload (QUERY / OFFER / SETTLE / EVENT)      |
+----------------------------------------------------+


⸻

2. Message Envelope

2.1 Common Envelope Schema

Every TxIP message has a top-level envelope plus an embedded TGP payload (when applicable):

{
  "txip_version": "0.2",
  "msg_id": "uuid-v4",
  "session_id": "sess-{uuid}",
  "direction": "CLIENT_TO_TBC | TBC_TO_CLIENT | TBC_TO_TBC",
  "role": "BUYER_AGENT | SELLER_AGENT | TBC | WATCHER",
  "timestamp": 1731600000,
  "message_type": "CONTROL | TGP | ERROR",
  "tgp_phase": "QUERY | OFFER | SETTLE | EVENT | NONE",
  "tgp_type": "tgp.escrow.created | tgp.seller.accepted | ... (optional)",
  "payload": { /* message-type-specific payload */ }
}

Field notes:
	•	txip_version — allows protocol evolution
	•	msg_id — unique per message (used for idempotency / correlation)
	•	session_id — ties a set of messages to a logical conversation or transaction
	•	direction — hints for logging / debugging, not security-relevant
	•	role — sender’s role from its own POV
	•	message_type:
	•	CONTROL — handshake, heartbeat, capabilities
	•	TGP — wraps a TGP-00 message
	•	ERROR — transport or protocol error
	•	tgp_phase / tgp_type — only meaningful when message_type = "TGP"

⸻

3. Control Messages

TxIP has a small set of control messages to manage sessions.

3.1 CONTROL/HELLO

Purpose:
Open or resume a TxIP session and advertise capabilities.

Direction:
Client Agent → TBC, or TBC → Peer TBC

Envelope:

{
  "txip_version": "0.2",
  "msg_id": "uuid-1",
  "session_id": "sess-123",
  "direction": "CLIENT_TO_TBC",
  "role": "BUYER_AGENT",
  "message_type": "CONTROL",
  "tgp_phase": "NONE",
  "tgp_type": null,
  "payload": {
    "control_type": "HELLO",
    "agent_id": "buyer://alice",
    "supported_tgp_versions": ["2.0"],
    "supported_transports": ["HTTP", "WEBSOCKET"],
    "supported_chains": ["pulse-mainnet", "eth-mainnet"],
    "supported_assets": ["USDC", "WETH"],
    "features": {
      "zk_discount_proofs": true,
      "receipt_ownership_proofs": true,
      "late_discount_support": true
    },
    "auth": {
      "scheme": "BEARER_JWT | API_KEY | MTLS | NONE",
      "token": "optional-token-or-empty"
    }
  }
}

3.2 CONTROL/WELCOME

Purpose:
TBC acknowledges HELLO and confirms session parameters.

Direction:
TBC → Client Agent

Payload:

{
  "control_type": "WELCOME",
  "tbc_id": "tbc://loearth-main",
  "session_id": "sess-123",
  "negotiated_tgp_version": "2.0",
  "negotiated_chains": ["pulse-mainnet"],
  "negotiated_features": {
    "zk_discount_proofs": true,
    "late_discount_support": true
  },
  "heartbeat_interval_sec": 30
}

3.3 CONTROL/HEARTBEAT

Purpose:
Keep session alive; detect dead peers.

Direction:
Both directions

Payload:

{
  "control_type": "HEARTBEAT",
  "seq": 42
}

3.4 CONTROL/CLOSE

Purpose:
Orderly session termination.

Payload:

{
  "control_type": "CLOSE",
  "reason": "idle_timeout | client_shutdown | protocol_error | other"
}


⸻

4. TGP Payload Binding

TxIP’s job with TGP is simple:
	•	Wrap each TGP message in a TxIP envelope
	•	Preserve the TGP JSON unmodified inside payload.tgp

4.1 message_type = "TGP"

When message_type is "TGP", payload MUST be:

{
  "tgp": { /* TGP-00 message body exactly as defined in the spec */ }
}

And the envelope MUST set:
	•	tgp_phase according to TGP message:
	•	"QUERY" for TGP QUERY messages
	•	"OFFER" for TGP OFFER messages
	•	"SETTLE" for TGP SETTLE messages
	•	"EVENT" for TGP EVENT messages
	•	tgp_type for EVENTS should be the event name:
	•	"tgp.escrow.created"
	•	"tgp.seller.accepted"
	•	"tgp.seller.fulfilled"
	•	"tgp.fulfillment.expired"
	•	"tgp.seller.latefulfilled"
	•	"tgp.seller.claimed"
	•	"tgp.receipt.minted"
	•	"tgp.receipt.metadata.discount"

4.2 Example: Buyer → TBC QUERY

{
  "txip_version": "0.2",
  "msg_id": "d7d5c4c1-6a73-4c24-8f7a-3b8f45e1f111",
  "session_id": "sess-123",
  "direction": "CLIENT_TO_TBC",
  "role": "BUYER_AGENT",
  "timestamp": 1731600000,
  "message_type": "TGP",
  "tgp_phase": "QUERY",
  "tgp_type": null,
  "payload": {
    "tgp": {
      "phase": "QUERY",
      "id": "q-123",
      "from": "buyer://alice",
      "to": "seller://pizza_hut_4521",
      "asset": "USDC",
      "amount": "30000000",
      "escrow_from_402": true,
      "escrow_contract_from_402": "0x742d35...",
      "zk_profile": null,
      "metadata": {
        "product_id": "pepperoni-large",
        "quantity": "1",
        "delivery_address": "encrypted:..."
      }
    }
  }
}

4.3 Example: Watcher → TBC EVENT

{
  "txip_version": "0.2",
  "msg_id": "3cc0a0b0-10de-4026-8027-7d41a90d9cd9",
  "session_id": "sess-123",
  "direction": "CLIENT_TO_TBC",
  "role": "WATCHER",
  "timestamp": 1731600100,
  "message_type": "TGP",
  "tgp_phase": "EVENT",
  "tgp_type": "tgp.fulfillment.expired",
  "payload": {
    "tgp": {
      "event": "tgp.fulfillment.expired",
      "order_id": "0xabcd...",
      "expiration_timestamp": 1731600099,
      "buyer_withdrawal_unlocked": true,
      "seller_can_still_fulfill": true,
      "state": "FULFILLMENT_EXPIRED"
    }
  }
}


⸻

5. Error Messages

5.1 ERROR Envelope

When something goes sideways at the TxIP / transport level, send:

{
  "txip_version": "0.2",
  "msg_id": "err-uuid",
  "session_id": "sess-123",
  "direction": "TBC_TO_CLIENT",
  "role": "TBC",
  "timestamp": 1731600200,
  "message_type": "ERROR",
  "tgp_phase": "NONE",
  "tgp_type": null,
  "payload": {
    "error_code": "TXIP_INVALID_ENVELOPE",
    "http_status": 400,
    "related_msg_id": "d7d5c4c1-6a73-4c24-8f7a-3b8f45e1f111",
    "details": "Missing required field: msg_id",
    "retryable": false
  }
}

5.2 Canonical Error Codes

Suggested error_code values:
	•	TXIP_INVALID_ENVELOPE
	•	TXIP_UNSUPPORTED_VERSION
	•	TXIP_UNAUTHENTICATED
	•	TXIP_UNAUTHORIZED
	•	TXIP_INTERNAL_ERROR
	•	TXIP_RATE_LIMITED
	•	TXIP_UPSTREAM_UNAVAILABLE
	•	TXIP_MALFORMED_TGP_PAYLOAD

These are intentionally TxIP-level; TGP-level disputes (e.g., invalid state transitions) should be expressed as TGP ERROR or policy decisions at higher layers, not TxIP.

⸻

6. Transports

TxIP-00 defines two primary bindings:
	•	HTTP/1.1 or HTTP/2 (REST style)
	•	WebSocket (streaming / bi-directional)

6.1 HTTP Binding

Endpoint:
	•	POST /txip/v0/messages

Request Body:
Exactly one TxIP envelope as JSON.

Response Body:
	•	On success: optional acknowledgement:

{
  "status": "accepted",
  "msg_id": "d7d5c4c1-6a73-4c24-8f7a-3b8f45e1f111"
}

	•	On error: HTTP status + TxIP ERROR envelope

Idempotency:
	•	Clients SHOULD include msg_id
	•	Servers MUST treat duplicate msg_id as idempotent (no double processing)

6.2 WebSocket Binding

Endpoint:
	•	GET /txip/v0/ws

Once connected:
	•	Client SHOULD send CONTROL/HELLO as the first message
	•	Server replies with CONTROL/WELCOME
	•	Both sides may then send TGP and CONTROL messages as JSON frames
	•	HEARTBEAT messages may be exchanged to keep the connection alive

⸻

7. Session & Correlation

TxIP sessions tie together multiple messages:
	•	Same session_id across:
	•	QUERY → OFFER → escrow events → SETTLE
	•	Multiple discount redemptions tied to the same buyer pseudonym

7.1 Session Rules
	•	A HELLO MUST include a session_id (client-generated)
	•	A WELCOME MUST echo this session_id
	•	Subsequent messages in that logical conversation MUST use the same session_id

Clients MAY open multiple sessions in parallel (e.g., multiple carts / concurrent orders).

7.2 Correlation with TGP
	•	tgp.phase = "QUERY" → TxIP msg_id is the canonical entrypoint id
	•	tgp.query_id and tgp.offer_id SHOULD be logged and correlated with TxIP msg_id
	•	Events like tgp.escrow.created MUST include the order_id and session-related metadata so watchers can route them into the correct TxIP session if needed

⸻

8. Security & Auth Hooks

TxIP deliberately stays out of which auth system you use, but provides fields to plug them in.

8.1 Auth in HELLO

The auth block in CONTROL/HELLO allows:
	•	Bearer JWTs
	•	API keys
	•	Mutual TLS binding (with an out-of-band identity registry)
	•	Or NONE for local/testing environments

8.2 Transport Security
	•	HTTP endpoints MUST be served over TLS in production
	•	WebSocket endpoints SHOULD be wss://
	•	MTLS MAY be used for TBC↔TBC federation

8.3 Replay / Idempotency
	•	Each msg_id MUST be unique
	•	Servers SHOULD keep a short-lived replay cache per session to drop duplicates

⸻

9. Example End-to-End Flow (Pizza)

This shows TxIP + TGP + CoreProver working together.

9.1 Steps
	1.	Buyer agent → TBC: CONTROL/HELLO
	2.	TBC → Buyer agent: CONTROL/WELCOME
	3.	Buyer agent → TBC: TxIP/TGP QUERY
	4.	TBC → Seller agent: TxIP/TGP OFFER
	5.	Buyer commits payment on-chain (CoreProver createEscrow)
	6.	Watcher → TBC: TxIP/TGP tgp.escrow.created EVENT
	7.	Seller accepts order on-chain
	8.	Watcher → TBC: TxIP/TGP tgp.seller.accepted EVENT
	9.	Seller fulfills (on time or late), claims payment
	10.	Watcher → TBC: TxIP/TGP tgp.seller.fulfilled / tgp.seller.latefulfilled / tgp.seller.claimed / tgp.receipt.minted EVENTS
	11.	TBC → Agents: TxIP/TGP SETTLE summarizing final outcome

9.2 Minimal Example: QUERY → OFFER

Buyer agent:

{
  "txip_version": "0.2",
  "msg_id": "uuid-query",
  "session_id": "sess-pizza-1",
  "direction": "CLIENT_TO_TBC",
  "role": "BUYER_AGENT",
  "message_type": "TGP",
  "tgp_phase": "QUERY",
  "payload": {
    "tgp": {
      "phase": "QUERY",
      "id": "q-123",
      "from": "buyer://alice",
      "to": "seller://pizza_hut_4521",
      "asset": "USDC",
      "amount": "30000000",
      "escrow_from_402": true,
      "escrow_contract_from_402": "0x742d35..."
    }
  }
}

TBC routes to seller agent, which replies:

{
  "txip_version": "0.2",
  "msg_id": "uuid-offer",
  "session_id": "sess-pizza-1",
  "direction": "CLIENT_TO_TBC",
  "role": "SELLER_AGENT",
  "message_type": "TGP",
  "tgp_phase": "OFFER",
  "payload": {
    "tgp": {
      "phase": "OFFER",
      "id": "offer-456",
      "query_id": "q-123",
      "asset": "USDC",
      "amount": "30000000",
      "session_id": "sess-pizza-1",
      "economic_metadata": {
        "enables_late_discount": true,
        "late_discount_pct": 10,
        "discount_expiration_days": 90,
        "acceptance_window_seconds": 1800,
        "fulfillment_window_seconds": 3600,
        "claim_window_seconds": 3600
      },
      "payment_profile": {
        "required_commitment_type": "LEGAL_SIGNATURE",
        "counter_escrow_amount": "0",
        "fulfillment_type": "DELIVERY",
        "allows_timed_release": true,
        "timed_release_delay": 3600
      }
    }
  }
}


⸻

10. Implementation Checklist (TxIP Side Only)

For the transaction-border-controller repo, this is what TxIP-00 implies:
	•	TxIP envelope struct / type with:
	•	txip_version, msg_id, session_id, message_type, tgp_phase, etc.
	•	JSON (de)serialization for TxIP envelopes
	•	HTTP POST /txip/v0/messages handler:
	•	Validate envelope, route CONTROL vs TGP vs ERROR
	•	WebSocket /txip/v0/ws handler:
	•	Accept connection, expect HELLO, reply WELCOME
	•	Handle HEARTBEAT
	•	Forward TGP messages into the routing core
	•	Error handler:
	•	Emit TxIP ERROR envelopes on protocol/transport problems
	•	Replay/idempotency cache keyed by msg_id
	•	Logging that logs:
	•	TxIP msg_id, session_id, message_type, tgp_phase, tgp_type
	•	Correlated order_id and on-chain tx hashes when present

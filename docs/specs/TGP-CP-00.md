📘 TGP-CP-00 v1.0 — Transaction Gateway Protocol: Client Profile

Version: 1.0 Draft
Status: Draft (internal)
Author: Ledger of Earth
Scope: Defines the required behavior of TGP Clients interacting with payment gateways, wallets, and settlement engines under TGP-00 v3.2.
Audience: Browser extension developers, wallet-module authors, embedded agents, automated runtimes.

⸻

0. Overview

A TGP Client is any runtime that interprets payment triggers, constructs
QUERY messages, executes gateway-issued Economic Envelopes, and submits signed transactions to a payment gateway or a blockchain.

The TGP Client sits between:
	•	Merchant / User environment (402-trigger, x402 metadata, Direct Pay)
	•	Gateway (TBC or any verification node)
	•	Wallet (standard EIP-1193 signer)
	•	Settlement Engine (on-chain contracts observed by the gateway)

A compliant Client:
	•	constructs QUERY messages according to TGP-00
	•	validates and obeys ACK messages
	•	forwards executable transactions to a wallet unchanged
	•	routes signed transactions as directed by the gateway
	•	listens for terminal SETTLE messages
	•	maintains ephemeral session context locally (never in Gateway)

The Client does not generate keys, modify wallets, or bypass gateway
authorization.

⸻

1. Client Responsibilities

A compliant TGP Client MUST:
	1.	Detect payment triggers (HTTP 402, x402, Direct Pay, agent intent).
	2.	Construct a well-formed QUERY.
	3.	Include optional routing metadata if needed.
	4.	Validate ACK messages.
	5.	Construct transactions verbatim from ACK.tx.
	6.	Forward unsigned transactions to a wallet for approval.
	7.	Route signed transactions according to ACK.routing.
	8.	Continue multi-verb flows until completion.
	9.	Receive and process terminal SETTLE messages.
	10.	Maintain minimal local session state (session_token, timestamps).

A Client MUST NOT:
	•	generate private keys
	•	intercept or modify wallet popups
	•	alter Economic Envelopes
	•	bypass gateway verification
	•	broadcast unsigned transactions
	•	embed transaction metadata into external logs

A Client MAY:
	•	render optional UI (“Protection Active”)
	•	expose a visible indicator of TGP activity
	•	allow “agent mode” automation with explicit user authorization
	•	keep local, non-sensitive logs

⸻

2. Trigger Conditions

A TGP Client MUST activate when one of the following occurs:

2.1 HTTP 402 “Payment Required” (Canonical)

Detected from:
	•	a merchant website
	•	an API-triggered checkout
	•	a native application

The Client extracts the payment profile and constructs a QUERY.

2.2 x402 Message (Optional Alternative)

The Client MAY treat x402 metadata as a payment trigger if present, but HTTP 402
remains the canonical mechanism.

2.3 Explicit User Command (Direct Pay)

User manually enters:
	•	amount
	•	merchant URL
	•	or scans QR to obtain payment profile

The Client constructs a QUERY identically to merchant-initiated flows.

2.4 Escrow Continuation

If ACK.status = “offer” or a multi-verb flow is in progress, the Client MUST
issue additional QUERY messages as required.

⸻

3. QUERY Construction (Client → Gateway)

The Client MUST construct a QUERY that conforms to TGP-00 v3.2.
A minimal QUERY:

{
  “type”: “QUERY”,
  “tgp_version”: “3.2”,
  “id”: “uuid”,
  “session_token”: “<opaque-or-null>”,
  “delegated_key”: “<public-key-or-null>”,
  “scope”: { },

  “transaction_area_id”: null,
  “path”: [],
  “next_gateway”: null,

  “intent”: {
    “verb”: “COMMIT”,
    “party”: “BUYER”,
    “mode”: “DIRECT”
  },

  “payment_profile”: “0xContract”,
  “amount”: “1000000”,
  “chain_id”: 369,
  “metadata”: { }
}

Normative Requirements

The Client MUST:
	•	include intent.verb
	•	use the gateway endpoint configured by the user
	•	use HTTPS only
	•	include routing metadata only when required by the environment
	•	include session_token and delegated_key when using delegated-session flows

The Client MUST NOT:
	•	include private keys
	•	include wallet seeds
	•	include signatures
	•	embed sensitive metadata

⸻

4. ACK Handling

A gateway responds with an ACK containing:
	•	status = “offer” — preview, not executable
	•	status = “allow” — executable Economic Envelope
	•	status = “deny” — authorization refusal
	•	status = “revise” — missing or incorrect fields

{
  “type”: “ACK”,
  “status”: “allow”,
  “id”: “uuid”,
  “intent”: { “verb”: “COMMIT” },
  “tx”: {
    “to”: “0xPaymentProfile”,
    “value”: “1000000”,
    “data”: “0x...”,
    “chain_id”: 369,
    “gas_limit”: 200000
  },
  “routing”: {
    “mode”: “direct”,
    “rpc_url”: “https://rpc.example”
  },
  “expires_at”: “2025-11-18T15:00:00Z”
}

The Client MUST:
	•	obey status
	•	obey routing instructions
	•	obey expires_at
	•	treat offer as informational only
	•	treat deny as final
	•	require a new QUERY for any parameter changes

The Client MUST NOT:
	•	modify calldata
	•	override chain_id
	•	change destination address
	•	retry automatically after a denial

⸻

5. Transaction Construction

When ACK.status = “allow”, the Client MUST:
	•	construct a transaction identical to ACK.tx
	•	include no additional calldata
	•	include no additional fields
	•	not adjust the gas parameters unless explicitly provided

Any deviation MUST cause the Client to generate a new QUERY.

⸻

6. Wallet Interaction (Signing Layer)

A Client MUST:
	•	invoke wallet APIs using standard methods:

ethereum.request({ method: “eth_sendTransaction”, params: [ tx ] })


	•	display the wallet’s native confirmation UI
	•	wait for explicit user approval
	•	treat the wallet as a blind signer

A Client MUST NOT:
	•	suppress wallet confirmation
	•	modify wallet provider objects
	•	intercept keystrokes, seeds, or popup windows

Wallets remain completely unaware of TGP.

⸻

7. Routing Signed Transactions

The Client MUST route signed transactions exactly as specified in ACK.routing.

7.1 Direct Mode

Send the signed transaction directly to the RPC endpoint:

POST <rpc_url>

7.2 Relay Mode

Send a relay payload to the Gateway:

{
  “id”: “uuid”,
  “signed_tx”: “0x...”
}

The Gateway handles RPC submission.

⸻

8. Verb Sequencing (Multi-Step Escrow)

For multi-verb flows (COMMIT → ACCEPT → CLAIM → etc.):
	1.	Client sends initial QUERY.
	2.	Gateway returns ACK(status=“offer”).
	3.	Client sends upgraded QUERY if needed.
	4.	Gateway returns ACK(status=“allow”).
	5.	Client signs & routes the transaction.
	6.	Upon state transition, Client MUST issue the next QUERY.

This continues until a final verb (e.g., CLAIM, WITHDRAW) completes.

⸻

9. Settlement Handling (SETTLE Messages)

After an authorized transaction is submitted, the Gateway observes the Settlement
Engine and emits a terminal SETTLE message:

{
  “type”: “SETTLE”,
  “id”: “uuid”,
  “result”: {
    “final_status”: “claimed”,
    “escrow_id”: “0xEscrow”
  },
  “timestamp”: “2025-11-18T15:00:05Z”
}

A Client MUST:
	•	listen for SETTLE
	•	finalize the user-visible transaction record
	•	not expect further messages
	•	not send additional QUERY messages for that lifecycle

⸻

10. Client State Tracking

Clients MUST maintain:
	•	session_token (ephemeral)
	•	delegated_key (optional)
	•	local timestamps
	•	last ACK
	•	pending verb state
	•	gateway reachability information

Clients MUST NOT persist:
	•	private keys
	•	seeds
	•	RPC credentials
	•	gateway internal metadata
	•	wallet-specific secrets

Session state is local-only and ephemeral.

⸻

11. Security & Privacy Requirements

A compliant TGP Client MUST:
	•	use HTTPS for all gateway communications
	•	validate certificates
	•	protect against replay attacks
	•	ensure SETTLE messages match known id values
	•	never broadcast unsigned transactions
	•	never share transaction metadata externally

A Client MUST NOT:
	•	modify wallet state
	•	override wallet security
	•	leak transaction metadata
	•	attempt to inspect sensitive DOM areas (in extension contexts)

⸻

12. Compliance Tests

A conforming Client MUST pass:
	1.	QUERY Construction Test
	2.	ACK Validation Test
	3.	Economic Envelope Execution Test
	4.	Wallet Interaction Test
	5.	Routing Correctness Test
	6.	Escrow Verb Sequencing Test
	7.	SETTLE Handling Test
	8.	Security Sandbox Test

Successful completion indicates the implementation is TGP-CP-00 compliant.

⸻

End of TGP-CP-00 v1.0.
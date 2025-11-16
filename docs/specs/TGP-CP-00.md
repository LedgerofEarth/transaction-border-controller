📘 TGP-CP-00 — Transaction Gateway Protocol: Client Profile

Version: 0.1 Draft
Status: Draft (internal)
Author: Ledger of Earth
Scope: Specifies how the TGP Client interacts with x402, the TBC, wallets, and on-chain payment profile contracts.
Purpose: Establish a standard execution model for TGP-aware clients such as browser extensions, headless agents, embedded runtimes, or wallet-native modules.

⸻

0. Overview

The TGP Client is the runtime component responsible for negotiating and executing transactions through a Transaction Border Controller (TBC) using the Transaction Gateway Protocol.

The TGP Client:
	•	interprets x402 payment_required
	•	constructs and sends TGP QUERY messages
	•	receives and obeys TGP ACK responses
	•	constructs blockchain transactions exactly as instructed
	•	forwards them to a wallet for signing
	•	routes signed transactions to RPC or TBC relay endpoints
	•	manages multi-verb escrow sequences
	•	maintains TGP session state

The TGP Client does not generate keys, modify wallets, or replace signing engines.

⸻

1. Responsibilities

A compliant TGP Client MUST:
	1.	Detect and parse x402 payment_required messages
	2.	Construct a TGP QUERY to a configured TBC endpoint
	3.	Validate TGP ACK responses
	4.	Construct transactions verbatim from ACK data
	5.	Forward the transaction to a wallet for signing (EIP-1193 or equivalent)
	6.	Route the signed transaction per ACK routing rules
	7.	Loop multi-verb escrow flows (commit → accept → fulfill → claim)
	8.	Track per-session state (locally)
	9.	Expose user-visible confirmation dialogs when required

A compliant TGP Client MUST NOT:
	•	generate private keys
	•	read seed phrases
	•	intercept wallet popups
	•	alter transaction calldata
	•	override destination addresses
	•	bypass TBC decision-making
	•	broadcast unsigned transactions

A Client MAY:
	•	render optional UI elements
	•	keep local logs
	•	allow “agent mode” automation with user authorization
	•	expose optional session indicators
	•	detect wallet presence flags
	•	integrate awareness of presence API (TGP-EX)

⸻

2. Trigger Conditions

A TGP Client MUST activate when one of the following occurs:

2.1 x402 “payment_required” Event

Received from:
	•	a website/dApp
	•	an AI agent
	•	a merchant API
	•	local application code

2.2 Explicit User Command

User initiates a payment through a dApp or agent UI.

2.3 Escrow Continuation

Returned ACK specifies a next verb requiring additional transactions.

⸻

3. TGP QUERY (Client → TBC)

A TGP Client MUST send a QUERY message to the TBC over HTTPS:

{
  "tgp_version": "0.1",
  "session_id": "<uuid-or-null>",
  "buyer_address": "<0x...>",
  "payment_profile": "<0xContract>",
  "chain_id": 369, 
  "amount": "1000000000000000000",
  "intent": { "verb": "commit" },
  "metadata": {
      "x402": {...}
  }
}

Required fields:

Field	Description
session_id	Null on first QUERY; TBC returns a new session if needed
buyer_address	Wallet address being used
payment_profile	Settlement gateway / escrow contract
chain_id	Target chain
amount	Proposed payment amount
intent.verb	Requested action: commit, pay, quote, etc.
metadata	x402 contents or merchant-provided fields

The Client MUST NOT include private keys or wallet secrets in the QUERY.

⸻

4. TGP ACK (TBC → Client)

A TGP Client MUST be able to parse and obey:

{
  "status": "allow",
  "session_id": "abcd-1234",
  "next_verb": "commit",
  "tx": {
    "to": "0xPaymentProfileContract",
    "data": "0xabcdef...",
    "value": "1000000000000000000",
    "chain_id": 369,
    "gas_limit": null,
    "gas_price": null
  },
  "routing": {
    "mode": "direct",
    "rpc_url": "https://rpc.pulsechain.com"
  },
  "timeouts": {
    "fulfillment_window": 30000,
    "session_expiry": 600000
  },
  "notes": "Commit authorized"
}

A Client MUST obey:
	•	status (allow/deny/revise)
	•	tx fields exactly as provided
	•	routing directives
	•	next_verb sequencing

A Client MUST NOT:
	•	modify calldata
	•	override chain_id
	•	alter the recipient address
	•	ignore deny or revise statuses

⸻

5. Transaction Construction

The Client MUST construct the transaction exactly matching the ACK:
	•	to
	•	value
	•	data
	•	chain_id

No field may be changed by the Client.

Any modifications MUST trigger a new QUERY.

⸻

6. Wallet Interaction (Signing Layer)

A TGP Client MUST:
	•	call the wallet using standard APIs (e.g., ethereum.request({ method: 'eth_sendTransaction', params: [...] }))
	•	display native wallet approval popup
	•	not bypass user approval

A Wallet:
	•	does not need TGP awareness
	•	must only sign what it sees
	•	remains a blind signer

⸻

7. Routing Signed Transactions

After signing, the Client MUST route the transaction according to the ACK:

7.1 Direct Mode

Send signed tx to the provided RPC endpoint.

7.2 Relay Mode

Send signed tx back to the TBC:

{
  "session_id": "...",
  "signed_tx": "0x..."
}

TBC relays to RPC.

⸻

8. Escrow Verb Sequencing

If next_verb is not final:
	•	Client MUST generate a new QUERY after the transaction reaches its state transition
	•	TBC returns next verb
	•	Loop continues until claim or success status

Example sequence:

commit → accept → fulfill → verify → claim


⸻

9. Session Tracking

Client MUST maintain:
	•	session_id
	•	timestamps
	•	whether TBC is reachable
	•	last ACK status
	•	next required verb

Client MUST NOT store:
	•	private keys
	•	wallet seed words
	•	sensitive chain metadata

⸻

10. Optional User Interface Elements

A Client MAY show:
	•	session status
	•	current verb
	•	TBC reachability
-* protection active* indicator

A Client MUST NOT misrepresent TGP guarantees or expose internal TBC routing data.

⸻

11. Security & Privacy Requirements

The Client MUST:
	•	use HTTPS for all TBC communications
	•	validate TBC certificates
	•	protect against replay attacks
	•	never store sensitive wallet data
	•	never broadcast unsigned transactions

The Client MUST NOT:
	•	attempt to modify wallet state
	•	override wallet provider objects
	•	inject code into wallet popups
	•	leak transaction metadata to third-party servers

⸻

12. Compliance Tests

A TGP Client MUST pass:
	1.	QUERY/ACK handshake test
	2.	Transaction construction correctness test
	3.	Wallet interaction test
	4.	Routing correctness test
	5.	Escrow sequencing test
	6.	Timeout and error recovery test

Passing these tests makes the Client TGP-CP-00 compliant.

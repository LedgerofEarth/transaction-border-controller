📗 TGP-EXT-00 v1.2 — Transaction Gateway Protocol: Extension Runtime

Version: 1.2
Status: Draft (internal)
Author: Ledger of Earth
Audience: Browser extension developers, wallet developers, agent-framework implementers
Scope: Defines the browser-resident runtime that implements TGP-CP-00 securely, safely, and compatibly with Chrome MV3, Firefox, Brave, Edge, and Safari.

Related Specifications:
- DKW-00 v0.5 — Delegated Key Wallet Specification
- TGP-00 v3.2 — Transaction Gateway Protocol
- TGP-EXT-ZK-00 — Zero-Knowledge Proof Envelope Specification

⸻

0. Overview

The TGP Extension Runtime (TGP-EXT) is the default browser implementation of the TGP Client described in TGP-CP-00. It enables any wallet—without modification—to participate in protected blockchain transactions mediated through a payment gateway such as a Transaction Border Controller (TBC).

The extension:
	•	Detects HTTP 402 Payment Required (canonical trigger)
	•	Optionally detects x402 metadata as a secondary trigger
	•	Constructs and sends TGP QUERY messages
	•	Receives and obeys TGP ACK responses
	•	Builds blockchain transactions exactly as instructed
	•	Hands transactions to the wallet for signing
	•	Routes signed transactions per ACK routing rules
	•	Tracks escrow state locally
	•	Listens for SETTLE notifications

The extension never generates private keys, modifies wallets, or intercepts wallet popups.

⸻

1. Architectural Model

The browser extension consists of four logical components:

1.1 Background Service Worker (MV3-Compliant)
	•	Implements QUERY → ACK loop
	•	Constructs Economic Envelope transactions
	•	Routes signed transactions
	•	Receives SETTLE and ERROR messages
	•	Maintains minimal, non-persistent escrow tracking

1.2 Content Script (Isolated World)
	•	Detects HTTP 402 and x402 payment-required signals
	•	Injects the TGP Presence API (window.tgp)
	•	Forwards permitted fields to the background worker
	•	DOES NOT read or manipulate sensitive DOM elements

1.3 UI Components
	•	Popup UI (settings, active escrow, WITHDRAW action)
	•	Badge indicator (stateful escrow visualization)
	•	Optional notifications

1.4 Local Storage

Stores only:
	•	TBC/Gateway endpoint
	•	Session metadata
	•	Active escrow tracking

MUST NOT store:
	•	Private keys
	•	Wallet seeds
	•	Signed transactions
	•	Sensitive merchant data

⸻

2. Permissions (Strict Minimum)

A compliant TGP-EXT extension MUST request only:

Permission	Purpose
storage	TBC endpoint & minimal metadata
activeTab	Detect 402 or x402 events
scripting	Inject Presence API object
notifications	Optional user alerts
host permissions	Only for user-entered TBC endpoint

Forbidden permissions:
	•	webRequestBlocking
	•	Clipboard read/write
	•	Password or credential access
	•	Wallet popup inspection or modification
	•	Browser-internal key/crypto API access

These requirements ensure compliance across all major extension marketplaces.

⸻

3. Event Flow

3.1 Standard Sequence
	1.	Trigger Detected
Content script detects HTTP 402 or x402 payment_required.
	2.	Forward Event
Content script → background worker (via messaging).
	3.	Construct QUERY
Background worker builds a valid TGP QUERY per TGP-CP-00.
	4.	Send to Gateway
QUERY → HTTPS → Gateway (TBC or other).
	5.	Receive ACK
Extension processes authorization or preview state.
	6.	Construct Transaction
Using ACK’s Economic Envelope (to, data, value, chain_id, gas).
	7.	Request Wallet Signature
ethereum.request({ method: “eth_sendTransaction”, … }).
	8.	Wallet Signs
Wallet shows standard popup; user approves.
	9.	Route Signed Transaction
	•	direct → RPC
	•	relay → TBC endpoint
	10.	Escrow Sequencing
If ACK defines a next verb, extension loops to step 3.

⸻

4. Gateway Communication Requirements

The extension MUST:
	•	Use HTTPS for QUERY and relay submission
	•	Validate TLS certificates
	•	Reject non-secure endpoints
	•	Use short-lived fetch() calls (MV3 requirement)
	•	NEVER open persistent or hidden background loops

Agent Mode (optional):
	•	MAY open a user-approved WebSocket
	•	MUST NOT open a WebSocket without explicit user action

The extension MUST NOT:
	•	Leak metadata to any endpoint except the configured Gateway
	•	Contact analytics or telemetry services
	•	Phone home

⸻

5. HTTP 402 & x402 Integration

The extension MUST support:
	•	HTTP 402 Payment Required (primary trigger)
	•	Optional x402 compatibility for legacy flows

Content script MUST:
	•	Listen for window.postMessage events
	•	Extract ONLY required payment fields
	•	Forward minimal metadata to the background worker

Content script MUST NOT:
	•	Parse confidential merchant DOM
	•	Read arbitrary DOM nodes
	•	Infer user intent outside the 402/x402 event

⸻

6. Transaction Construction Requirements

The extension MUST:
	•	Use Economic Envelope parameters verbatim
	•	Never override to, data, value, chain_id, or gas fields
	•	Follow routing directives exactly
	•	Refuse to construct a transaction if ACK is malformed

The extension MUST NOT:
	•	Broadcast unsigned transactions
	•	Bypass wallet UI
	•	Perform internal signing
	•	Inject or reorder calldata

Wallets remain blind signers.

⸻

7. Delegated Key Intent (DKI) — New in v1.2

The extension supports delegated signing via EIP-712 wallet authorization.
See DKW-00 v0.5 for full specification.

7.1 DKI Flow Summary

1. Merchant calls `window.tgp.requestPayment()`
2. Extension presents economic data to user
3. Extension requests EIP-712 signature from wallet (DKI)
4. User approves in wallet → delegation granted
5. Extension sends TGP_QUERY to TBC
6. TBC responds with ACK-ALLOWED
7. Extension submits delegated transaction

7.2 Wallet Interaction

The extension MUST:
- Present clear economic data before requesting signature
- Use standard `eth_signTypedData_v4` method
- Validate chain ID matches
- Respect user rejection without retry

The extension MUST NOT:
- Request delegation without user-initiated action
- Store wallet private keys
- Bypass wallet UI for delegation

7.3 Delegation Scope

DKI delegations are constrained by:
- Maximum value (wei)
- Specific merchant (or any)
- Specific chain ID
- Expiry timestamp (max 24 hours)
- Nonce (replay protection)

7.4 Stored Delegations

Delegations are stored in `chrome.storage.local`:
- Key: `cp_delegations`
- Max entries: 100
- Auto-expiry on: timeout, value exceeded, new delegation

7.5 Message Types

Background script handles:
- `DKI_INITIATE` — Start delegation flow
- `DKI_GET_STATE` — Query current state
- `DKI_SUBMIT_TX` — Submit with delegation
- `DKI_CHECK_DELEGATION` — Check existing
- `DKI_RESET` — Cancel flow

⸻

8. TGP Presence API (Wallet-Detected Signal)

The extension MUST expose a presence flag detectable by wallets.

8.1 window.tgp Injection

window.tgp = {
  version: "1.2",
  active: true,
  tbc: { reachable: true | false },
  dki: { supported: true }  // New in v1.2
};

8.2 Presence Event

document.dispatchEvent(
  new CustomEvent("tgp:present", {
    detail: { version: "1.2", reachable: true | false }
  })
);

Wallets MAY subscribe to detect TGP availability.

8.3 Security Constraints

Presence API MUST NOT expose:
	•	Gateway URL
	•	Session tokens
	•	Payment profiles
	•	Routing or path metadata
	•	Transaction calldata

⸻

9. Security Requirements

The extension MUST NOT:
	•	Request seed phrases
	•	Intercept or alter wallet popups
	•	Scrape passwords or sensitive DOM
	•	Capture RPC traffic
	•	Spoof transaction details

The extension MUST:
	•	Operate strictly as router + policy client
	•	Maintain transparency
	•	Be auditable and deterministic

⸻

10. Browser Compliance

Chrome MV3
	•	Service worker required
	•	No persistent background pages
	•	Script injection via isolated worlds

Firefox
	•	May allow background pages, but extension MUST emulate MV3 behavior

Safari
	•	Strict sandboxing
	•	Content script MUST avoid sensitive DOM reads

⸻

11. Compliance Tests

A compliant extension MUST pass:
	1.	Presence API test
	2.	402/x402 detection test
	3.	QUERY/ACK loop test
	4.	Transaction construction correctness
	5.	Wallet integration test
	6.	Routing correctness
	7.	Escrow sequencing test
	8.	Sandbox & isolation test

⸻

12. ERROR Handling (New in v1.1)

12.1 ERROR Notification

When receiving a TGP ERROR, the extension MUST:
	•	Display a visible notification
	•	Present error.code and error.message
	•	Provide actionable guidance
	•	Log to local diagnostics (optional)

It MUST NOT auto-retry or suppress the error.

12.2 Session Abort

Upon ERROR:
	•	Mark session as failed
	•	Disable pending actions
	•	Clear transient extension-side state

⸻

13. Escrow Monitoring (New in v1.1)

The extension maintains minimal local escrow state.

13.1 Escrow Record

Stored per active escrow:
	•	escrow_id
	•	state (PENDING, ACCEPTED, etc.)
	•	created_at
	•	ttl
	•	party_role
	•	next_verb

13.2 TTL Monitoring

The extension MUST:
	•	Compute time_remaining
	•	Emit warnings prior to timeout
	•	Update badge state

MUST NOT:
	•	Poll blockchain aggressively
	•	Trigger automatic withdrawal

13.3 SETTLE Handling

When a Gateway emits SETTLE:
	•	Escrow finalizes
	•	TTL monitoring stops
	•	UI updates to final state

⸻

14. WITHDRAW Eligibility & Initiation (New in v1.1)

14.1 L6 Eligibility Detection

WITHDRAW eligible when:
	•	Buyer: state = PENDING & TTL expired
	•	Seller: state = ACCEPTED & TTL expired
	•	Cooperative: both parties submit release intent (future optional)

14.2 User Notification

When eligible:
	•	Notify: “Withdrawal available”
	•	Update badge
	•	Enable WITHDRAW button in popup

14.3 WITHDRAW Action

Upon confirmation, extension MUST construct:

QUERY {
  type: “QUERY”,
  intent: { verb: “WITHDRAW”, party: “BUYER” | “SELLER” },
  escrow_id: “0xEscrow”,
  chain_id: …,
  payment_profile: “0x…”
}

ACK MUST be followed exactly.

⸻

15. Multi-Verb State Display (New in v1.1)

15.1 Badge States

Color	Meaning
Gray	Idle
Blue	PENDING
Yellow	ACCEPTED
Green	CLAIMED/RELEASED
Red	ERROR/REFUNDED

15.2 Popup Escrow Panel

Popup MUST show:
	•	Current escrow state
	•	Time remaining
	•	Next verb
	•	Actions (ACCEPT, CLAIM, WITHDRAW)
	•	Simple step history

Popup MUST NOT expose:
	•	Wallet addresses
	•	Routing metadata
	•	Merchant identifiers

⸻

End of TGP-EXT-00 v1.2
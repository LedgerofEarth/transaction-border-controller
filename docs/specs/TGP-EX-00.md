📗 TGP-EX-00 — Transaction Gateway Protocol: Browser Extension Runtime

Version: 0.1-draft
Status: Draft (internal)
Author: Ledger of Earth
Audience: Browser extension developers, wallet developers, agent-framework developers
Purpose: Define the browser-resident runtime that implements TGP-CP-00 securely, safely, and compatibly with Chrome MV3, Firefox, Brave, Edge, and Safari.

⸻

0. Overview

The TGP Extension Runtime (TGP-EX) is the default implementation of the TGP Client defined in TGP-CP-00. It allows any wallet—without modification—to participate in protected TGP/TBC-mediated transactions.

The extension:
	•	listens for x402 payment_required events
	•	constructs and sends TGP QUERY to the TBC
	•	receives and obeys TGP ACK
	•	constructs blockchain transactions exactly as instructed
	•	forwards transactions to wallets for signing
	•	routes signed transactions to RPC or TBC relay endpoints

The extension never handles private keys or intercepts wallet popups.

⸻

1. Architectural Model

The TGP Extension consists of:

1. Background Service Worker
	•	Implements QUERY/ACK communication
	•	Constructs transactions
	•	Handles routing
	•	Maintains minimal session state
	•	Event-driven (MV3 compliant)

2. Content Script (Isolated World)
	•	Detects x402 payment_required signals on dApp pages
	•	Injects TGP Presence API object (window.tgp)
	•	Listens/forwards events
	•	DOES NOT read or interact with sensitive DOM nodes

3. UI Components
	•	Popup for user settings (TBC URL, enable/disable TGP, logs)
	•	Optional badge (TGP Active indicator)

4. Local Storage
	•	Stores:
	•	session metadata
	•	TBC URL
	•	Never stores:
	•	private keys
	•	wallet seeds
	•	signed transactions

⸻

2. Permissions (Strict Minimum)

A compliant extension MUST request only:

Permission	Purpose
storage	TBC endpoint & session metadata
activeTab	Detect x402 events from page
scripting	Inject Presence API object
notifications	Optional user alerts
host permissions	Only for user-entered TBC endpoint

Forbidden permissions:
	•	webRequestBlocking (highly scrutinized)
	•	clipboardRead or clipboardWrite
	•	Any password/credential access
	•	Reading or modifying wallet popups
	•	Access to browser internal APIs related to keys

This ensures storefront approval across Chrome, Brave, Firefox, Safari.

⸻

3. Event Flow

3.1 Step-by-step sequence

1. x402 detected
Content script receives a payment_required x402 message from the page.

2. Message forwarded
Content script → background worker via extension messaging.

3. QUERY constructed
Extension creates a TGP QUERY using TGP-CP-00 format.

4. QUERY → TBC
Background worker sends HTTPS request to user-provided TBC endpoint.

5. ACK received
Extension receives TGP ACK with transaction instructions.

6. Construct transaction
Extension builds transaction exactly per ACK instructions (to, data, value, chain_id).

7. Request wallet signature
Extension triggers ethereum.request({method: 'eth_sendTransaction'...}) or equivalent.

8. Wallet signs normally
Wallet remains ignorant of TGP.
Only shows a standard transaction popup.

9. Route signed tx
Extension routes per ACK:
	•	direct → RPC
	•	relay → TBC endpoint

10. Escrow sequencing
If next_verb not terminal, the extension loops back to step 3.

⸻

4. TBC Communication Requirements

A TGP-EX-compliant extension MUST:
	•	use HTTPS
	•	validate certificates
	•	reject non-TLS endpoints
	•	use short-lived fetch() calls (no persistent background pages)

Optional (Agent Mode only):
	•	user-approved WebSocket to TBC

The extension MUST NOT:
	•	leak metadata to any server except the user’s TBC
	•	connect to third-party analytics
	•	phone home
	•	maintain long-running hidden loops (MV3 violation)

⸻

5. x402 Integration

The extension MUST support x402 event detection via:
	•	content script listening to window.postMessage
	•	detecting standard payment_required fields
	•	forwarding minimal fields to background worker

The extension MUST NOT:
	•	parse or modify confidential merchant content
	•	read arbitrary DOM content beyond x402 event payload

⸻

6. Transaction Construction Requirements

A TGP-EX MUST:
	•	use ACK transaction parameters verbatim
	•	not modify calldata or destination
	•	not override chain_id
	•	not inject extra fields

A TGP-EX MUST NOT:
	•	broadcast unsigned transactions
	•	bypass user wallet confirmations
	•	request private keys
	•	perform signing internally

Wallet is the signer.
Extension is the policy/router.

⸻

7. TGP Presence API (Wallet-Detected Signal)

(NEW — final version)

The extension MUST expose a “presence flag” detectable by wallets.

7.1 window.tgp Injection

Injected via isolated-world content script:

window.tgp = {
  version: "0.1",
  active: true,
  tbc: {
    reachable: true | false
  }
};

Wallets MAY read:

if (window.tgp?.active) {
    // enable TGP indicator
}

7.2 Presence Event

Extension MUST emit:

document.dispatchEvent(
  new CustomEvent("tgp:present", {
    detail: {
      version: "0.1",
      reachable: true | false
    }
  })
);

Wallets MAY subscribe:

document.addEventListener("tgp:present", (e) => {
  // Wallet knows TGP is active
});

7.3 Security Constraints

Presence API MUST NOT expose:
	•	TBC URL
	•	session IDs
	•	routing data
	•	merchant profiles
	•	x402 metadata
	•	any blockchain transaction data

It MAY expose only:
	•	active
	•	version
	•	TBC reachability boolean

⸻

8. Security Requirements

The TGP Extension MUST NOT:
	•	request seed phrases
	•	display misleading transaction details
	•	observe or modify wallet UI
	•	intercept popups
	•	monitor keystrokes
	•	inspect password fields
	•	scrape DOM
	•	capture wallet RPC traffic

The extension MUST:
	•	operate purely as a router + policy client
	•	keep all behavior transparent
	•	remain auditable

⸻

9. Browser Compliance

Chrome MV3
	•	Must use service_worker
	•	No persistent background scripts
	•	Script injection must use isolated worlds

Firefox
	•	Equivalent behavior allowed
	•	Background page may be permitted, but MUST mimic MV3 restrictions for portability

Safari/WKWebExtension
	•	Tightly sandboxed; extension must minimize permissions
	•	Content script MUST avoid sensitive DOM access

⸻

10. Compliance Tests

A TGP-EX implementation MUST pass:
	1.	Presence API test
	•	window.tgp exposed
	•	tgp:present event emitted
	2.	x402 detection test
	•	Content script passes payment_required reliably
	3.	QUERY/ACK loop test
	•	Proper handling of TBC responses
	4.	Transaction construction correctness
	5.	Wallet integration test
	•	Standard signing popup triggered
	6.	Routing correctness test
	•	RPC vs TBC relay modes
	7.	Escrow sequencing test
	8.	Security sandbox test
	•	No forbidden DOM access

⸻

End of Finalized TGP-EX-00 Draft

⸻

This spec is now polished, self-contained, and ready for:
	•	GitHub
	•	the protocol doc folder
	•	inclusion in the TGP-00 umbrella spec
	•	sharing with KD / wallet devs when appropriate
	•	internal engineering alignment

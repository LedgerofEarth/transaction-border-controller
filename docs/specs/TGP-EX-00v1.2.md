# 📗 TGP-EXT-00 v1.2 — Transaction Gateway Protocol: Extension Runtime

**Version:** 1.2  
**Status:** Draft (internal)  
**Author:** Ledger of Earth  
**Audience:** Browser extension developers, wallet developers, agent-framework implementers  
**Scope:** Defines the browser-resident runtime that implements TGP-CP-00 securely, safely, and compatibly with Chrome MV3, Firefox, Brave, Edge, and Safari.

——

## Version History

|Version|Date      |Changes                                                                                                                                                                                               |
|-——|-———|——————————————————————————————————————————————————————————————————|
|1.0    |2025-10-15|Initial specification                                                                                                                                                                                 |
|1.1    |2025-11-20|Added ERROR handling, escrow monitoring, WITHDRAW eligibility, multi-verb state display                                                                                                               |
|1.2    |2025-11-29|**Added Gateway-Missing Fallback Mode (Section 14A)**, wallet discovery standards (EIP-6963), delegated wallet execution, enhanced privacy notices, chain validation, error handling for fallback mode|

——

## 0. Overview

The TGP Extension Runtime (TGP-EXT) is the default browser implementation of the TGP Client described in TGP-CP-00. It enables any wallet—without modification—to participate in protected blockchain transactions mediated through a payment gateway such as a Transaction Border Controller (TBC).

The extension:

- Detects HTTP 402 Payment Required (canonical trigger)
- Optionally detects x402 metadata as a secondary trigger
- Constructs and sends TGP QUERY messages
- Receives and obeys TGP ACK responses
- Builds blockchain transactions exactly as instructed
- Hands transactions to the wallet for signing
- Routes signed transactions per ACK routing rules
- Tracks escrow state locally
- Listens for SETTLE notifications
- **Supports direct wallet execution when no gateway is configured (Gateway-Missing Fallback Mode)**

The extension never generates private keys, modifies wallets, or intercepts wallet popups.

——

## 1. Architectural Model

The browser extension consists of four logical components:

### 1.1 Background Service Worker (MV3-Compliant)

- Implements QUERY → ACK loop
- Constructs Economic Envelope transactions
- Routes signed transactions
- Receives SETTLE and ERROR messages
- Maintains minimal, non-persistent escrow tracking
- **Manages wallet discovery and direct settlement fallback**

### 1.2 Content Script (Isolated World)

- Detects HTTP 402 and x402 payment-required signals
- Injects the TGP Presence API (window.tgp)
- Forwards permitted fields to the background worker
- DOES NOT read or manipulate sensitive DOM elements

### 1.3 UI Components

- Popup UI (settings, active escrow, WITHDRAW action)
- Badge indicator (stateful escrow visualization)
- Optional notifications
- **Wallet connection management interface**
- **Privacy warning dialogs for fallback mode**

### 1.4 Local Storage

Stores only:

- TBC/Gateway endpoint
- Session metadata
- Active escrow tracking
- **Connected wallet configuration**
- **User preferences for chain switching and privacy**

MUST NOT store:

- Private keys
- Wallet seeds
- Signed transactions
- Sensitive merchant data

——

## 2. Permissions (Strict Minimum)

A compliant TGP-EXT extension MUST request only:

|Permission      |Purpose                           |
|-—————|-———————————|
|storage         |TBC endpoint & minimal metadata   |
|activeTab       |Detect 402 or x402 events         |
|scripting       |Inject Presence API object        |
|notifications   |Optional user alerts              |
|host permissions|Only for user-entered TBC endpoint|

Forbidden permissions:

- webRequestBlocking
- Clipboard read/write
- Password or credential access
- Wallet popup inspection or modification
- Browser-internal key/crypto API access

These requirements ensure compliance across all major extension marketplaces.

——

## 3. Event Flow

### 3.1 Standard Sequence (Gateway Mode)

1. **Trigger Detected**  
   Content script detects HTTP 402 or x402 payment_required.
1. **Forward Event**  
   Content script → background worker (via messaging).
1. **Construct QUERY**  
   Background worker builds a valid TGP QUERY per TGP-CP-00.
1. **Send to Gateway**  
   QUERY → HTTPS → Gateway (TBC or other).
1. **Receive ACK**  
   Extension processes authorization or preview state.
1. **Construct Transaction**  
   Using ACK’s Economic Envelope (to, data, value, chain_id, gas).
1. **Request Wallet Signature**  
   `ethereum.request({ method: “eth_sendTransaction”, … })`.
1. **Wallet Signs**  
   Wallet shows standard popup; user approves.
1. **Route Signed Transaction**

- `direct` → RPC
- `relay` → TBC endpoint

1. **Escrow Sequencing**  
   If ACK defines a next verb, extension loops to step 3.

### 3.2 Fallback Sequence (Gateway-Missing Mode)

See Section 14A for complete fallback mode specification.

——

## 4. Gateway Communication Requirements

The extension MUST:

- Use HTTPS for QUERY and relay submission
- Validate TLS certificates
- Reject non-secure endpoints
- Use short-lived fetch() calls (MV3 requirement)
- NEVER open persistent or hidden background loops

Agent Mode (optional):

- MAY open a user-approved WebSocket
- MUST NOT open a WebSocket without explicit user action

The extension MUST NOT:

- Leak metadata to any endpoint except the configured Gateway
- Contact analytics or telemetry services
- Phone home

——

## 5. HTTP 402 & x402 Integration

The extension MUST support:

- HTTP 402 Payment Required (primary trigger)
- Optional x402 compatibility for legacy flows

Content script MUST:

- Listen for window.postMessage events
- Extract ONLY required payment fields
- Forward minimal metadata to the background worker

Content script MUST NOT:

- Parse confidential merchant DOM
- Read arbitrary DOM nodes
- Infer user intent outside the 402/x402 event

——

## 6. Transaction Construction Requirements

The extension MUST:

- Use Economic Envelope parameters verbatim
- Never override to, data, value, chain_id, or gas fields
- Follow routing directives exactly
- Refuse to construct a transaction if ACK is malformed

The extension MUST NOT:

- Broadcast unsigned transactions
- Bypass wallet UI
- Perform internal signing
- Inject or reorder calldata

Wallets remain blind signers.

——

## 7. TGP Presence API (Wallet-Detected Signal)

The extension MUST expose a presence flag detectable by wallets.

### 7.1 window.tgp Injection

```javascript
window.tgp = {
  version: “1.2”,
  active: true,
  tbc: { reachable: true | false }
};
```

### 7.2 Presence Event

```javascript
document.dispatchEvent(
  new CustomEvent(“tgp:present”, {
    detail: { version: “1.2”, reachable: true | false }
  })
);
```

Wallets MAY subscribe to detect TGP availability.

### 7.3 Security Constraints

Presence API MUST NOT expose:

- Gateway URL
- Session tokens
- Payment profiles
- Routing or path metadata
- Transaction calldata

——

## 8. Security Requirements

The extension MUST NOT:

- Request seed phrases
- Intercept or alter wallet popups
- Scrape passwords or sensitive DOM
- Capture RPC traffic
- Spoof transaction details

The extension MUST:

- Operate strictly as router + policy client
- Maintain transparency
- Be auditable and deterministic

——

## 9. Browser Compliance

**Chrome MV3**

- Service worker required
- No persistent background pages
- Script injection via isolated worlds

**Firefox**

- May allow background pages, but extension MUST emulate MV3 behavior

**Safari**

- Strict sandboxing
- Content script MUST avoid sensitive DOM reads

——

## 10. Compliance Tests

A compliant extension MUST pass:

1. Presence API test
1. 402/x402 detection test
1. QUERY/ACK loop test
1. Transaction construction correctness
1. Wallet integration test
1. Routing correctness
1. Escrow sequencing test
1. Sandbox & isolation test
1. **Wallet discovery test (EIP-6963)**
1. **Fallback mode execution test**
1. **Chain switching test**

——

## 11. ERROR Handling

### 11.1 ERROR Notification

When receiving a TGP ERROR, the extension MUST:

- Display a visible notification
- Present error.code and error.message
- Provide actionable guidance
- Log to local diagnostics (optional)

It MUST NOT auto-retry or suppress the error.

### 11.2 Session Abort

Upon ERROR:

- Mark session as failed
- Disable pending actions
- Clear transient extension-side state

——

## 12. Escrow Monitoring

The extension maintains minimal local escrow state.

### 12.1 Escrow Record

Stored per active escrow:

- escrow_id
- state (PENDING, ACCEPTED, etc.)
- created_at
- ttl
- party_role
- next_verb

### 12.2 TTL Monitoring

The extension MUST:

- Compute time_remaining
- Emit warnings prior to timeout
- Update badge state

MUST NOT:

- Poll blockchain aggressively
- Trigger automatic withdrawal

### 12.3 SETTLE Handling

When a Gateway emits SETTLE:

- Escrow finalizes
- TTL monitoring stops
- UI updates to final state

——

## 13. WITHDRAW Eligibility & Initiation

### 13.1 L6 Eligibility Detection

WITHDRAW eligible when:

- Buyer: state = PENDING & TTL expired
- Seller: state = ACCEPTED & TTL expired
- Cooperative: both parties submit release intent (future optional)

### 13.2 User Notification

When eligible:

- Notify: “Withdrawal available”
- Update badge
- Enable WITHDRAW button in popup

### 13.3 WITHDRAW Action

Upon confirmation, extension MUST construct:

```json
QUERY {
  “type”: “QUERY”,
  “intent”: { “verb”: “WITHDRAW”, “party”: “BUYER” | “SELLER” },
  “escrow_id”: “0xEscrow”,
  “chain_id”: …,
  “payment_profile”: “0x…”
}
```

ACK MUST be followed exactly.

——

## 14. Multi-Verb State Display

### 14.1 Badge States

|Color |Meaning         |
|——|-—————|
|Gray  |Idle            |
|Blue  |PENDING         |
|Yellow|ACCEPTED        |
|Green |CLAIMED/RELEASED|
|Red   |ERROR/REFUNDED  |

### 14.2 Popup Escrow Panel

Popup MUST show:

- Current escrow state
- Time remaining
- Next verb
- Actions (ACCEPT, CLAIM, WITHDRAW)
- Simple step history

Popup MUST NOT expose:

- Wallet addresses
- Routing metadata
- Merchant identifiers

——

## 14A. Gateway-Missing Fallback Mode *(New in v1.2)*

This section defines extension behavior when a TGP-compliant Gateway (e.g., TBC) is not available, either because:

1. The user has not configured a gateway endpoint, AND
1. The merchant did not provide a gateway endpoint in the 402 or x402 metadata.

When both conditions are true, the extension MUST enter **Gateway-Missing Fallback Mode**.

——

### 14A.1 Conditions for Entry

The extension MUST enter fallback mode only when all of the following are true:

1. **No user-configured gateway**  
   `localStorage.tbc_endpoint` is null or unreachable.
1. **No merchant-provided gateway**  
   The 402/x402 metadata does not include `gateway`, `tbc`, `tgp_gateway`, or equivalent routing hints.
1. **User approval**  
   The extension MUST present a clear dialog stating:  
   *“This transaction will be sent directly to your wallet without TGP protections.”*

**Fallback mode MUST NOT activate automatically or silently.**

——

### 14A.2 Operational Semantics

When in fallback mode:

#### 14A.2.1 No TGP Message Construction

The extension MUST NOT construct a TGP QUERY.  
No QUERY, ACK, ERROR, or SETTLE messages SHOULD be used.

#### 14A.2.2 EIP-1193 Transaction Construction

The extension MUST construct an EIP-1193–compatible transaction request using either:

- Merchant-provided transaction parameters, OR
- User-supplied parameters from Direct Pay input UI.

#### 14A.2.3 Direct Wallet Routing

The extension MUST route the transaction request directly to the user’s selected wallet via:

```javascript
ethereum.request({
  method: “eth_sendTransaction”,
  params: [{
    to: “0x...”,
    value: “0x...”,
    data: “0x...”,
    chainId: “0x...”,
    gas: “0x...”
  }]
})
```

#### 14A.2.4 Shielded Mode Delegation

If the payment request originated from a SHIELDED intent:

**(a)** The extension MAY attempt to signal delegated execution via one of the following methods, in order of preference:

1. **EIP-TBD custom method (future):**
   
   ```javascript
   ethereum.request({
     method: “tgp_sendShieldedTransaction”,
     params: [{ to, value, data, chainId, shielded: true }]
   })
   ```
1. **Data field encoding (fallback):**  
   Include a TGP delegation marker in the transaction data field:
   
   ```
   data = keccak256(“TGP_DELEGATED”) + originalData
   ```
   
   Wallets MAY recognize this prefix for future ZK-NAT integration.
1. **Standard transaction (current):**  
   If neither method is supported, proceed with standard `eth_sendTransaction`. No delegation signal is sent.

**(b)** The extension MUST NOT require wallet support for TGP delegation. If the wallet does not recognize TGP signals, the transaction proceeds as a standard transfer.

**(c)** Privacy degradation notice:  
If delegation signaling fails, the extension MUST inform the user:  
*“Your wallet does not support shielded transactions. Proceeding with standard transfer (address visible).”*

#### 14A.2.5 No NAT Operations

The extension MUST NOT attempt NAT or ZK-NAT operations in fallback mode. Privacy is the responsibility of the delegated wallet.

#### 14A.2.6 Chain Validation

Before executing fallback transaction:

**(a)** The extension MUST validate that the user’s wallet is connected to the correct chain:

```javascript
if (wallet.chainId !== transaction.chainId) {
  // Prompt chain switch
}
```

**(b)** If chain mismatch detected, the extension MUST:

1. Attempt `wallet_switchEthereumChain`
1. If chain not added, attempt `wallet_addEthereumChain`
1. If user declines, abort transaction

**(c)** Supported chains:  
Fallback mode MUST support at minimum:

- Ethereum Mainnet (chainId: 1)
- Base (chainId: 8453)
- PulseChain (chainId: 369)

Additional chains MAY be supported via configuration.

**(d)** Chain configuration:  
The extension MUST maintain chain metadata for `wallet_addEthereumChain` calls, including:

- chainName
- nativeCurrency
- rpcUrls
- blockExplorerUrls

——

### 14A.3 Wallet Discovery Requirement

To support fallback mode, the extension MUST implement **Wallet Discovery** using the following standards:

#### (a) Primary discovery method: EIP-6963

The extension MUST listen for EIP-6963 wallet announcements:

```javascript
window.addEventListener(“eip6963:announceProvider”, (event) => {
  const { info, provider } = event.detail;
  // info.name = “MetaMask”, “Rabby”, “Rainbow”, etc.
  // provider = EIP-1193 provider interface
  registerWallet(info, provider);
});
```

And request provider list:

```javascript
window.dispatchEvent(new Event(“eip6963:requestProvider”));
```

#### (b) Fallback discovery method: window.ethereum

If no EIP-6963 providers are detected within 100ms, the extension MAY use legacy `window.ethereum` injection.

#### (c) User wallet selection

If multiple wallets are discovered, the extension MUST allow the user to select a preferred wallet. The selection MUST persist in extension storage.

#### (d) No wallet case

If no wallet is detected, the extension MUST display:  
*“No compatible wallet detected. Please install MetaMask, Rabby, or another EIP-1193 wallet.”*

**The extension MUST NOT scrape the DOM or fingerprint the browser to detect wallets.**

——

### 14A.4 User Consent Requirements

The fallback dialog MUST clearly state that:

#### (a) Protocol degradation:

*“This transaction bypasses TGP gateway protections. The following risks apply:”*

#### (b) Privacy exposure:

- Merchant will see your wallet address
- Merchant will see transaction amount
- Merchant will see your blockchain (Ethereum/Base/PulseChain)
- No address shielding or ZK-NAT will occur
- Transaction is permanently recorded on public blockchain

#### (c) Missing protections:

- No verification routing
- No deterministic settlement guarantees
- No escrow enforcement
- No automated dispute resolution

#### (d) Consent acknowledgment:

User MUST actively click **“I Understand, Proceed”** button.  
Dialog MUST NOT have a “Don’t show again” checkbox.

**Only after explicit confirmation MAY the extension initiate the raw transaction.**

——

### 14A.5 Non-Applicability of TGP Rules

In fallback mode:

- TGP-00 message structure does not apply.
- TGP-EXT escrow tracking MUST be disabled.
- Badge indicators MUST revert to Idle mode.
- No SETTLE notifications will occur.

This ensures the user is not misled into believing a TGP flow is active.

——

### 14A.6 Exit Conditions

The extension MUST exit fallback mode when either:

1. A valid TBC/gateway endpoint is configured by the user, OR
1. A merchant provides a valid gateway endpoint in a subsequent request, OR
1. The user navigates away or cancels the transaction.

**Fallback mode MUST NOT persist between sessions.**

——

### 14A.7 Error Handling in Fallback Mode

#### (a) Wallet rejection:

If `wallet.request()` throws or rejects:

- **User cancellation** → Silent return to idle state
- **Insufficient funds** → Display balance error with amount needed
- **Gas estimation failure** → Suggest manual gas input
- **Unknown error** → Log to console, show generic error message

#### (b) Network errors:

If RPC provider is unreachable:

- The extension SHOULD retry once with 2-second timeout
- If retry fails, display network error
- MUST NOT attempt fallback RPC (maintains decentralization)
- User MUST manually retry or configure different RPC

#### (c) Transaction monitoring:

After successful sendTransaction:

- Extension MUST store txHash in local transaction history
- Extension SHOULD poll for receipt (with backoff)
- Extension MAY display “Pending…” indicator
- Extension MUST NOT claim settlement finality until confirmed

#### (d) No automatic retries:

The extension MUST NOT automatically retry failed transactions. User MUST explicitly re-initiate payment.

——

**End of TGP-EXT-00 v1.2**

——

## Summary of v1.2 Changes

This version adds comprehensive support for **Gateway-Missing Fallback Mode**, enabling the extension to operate without a TGP gateway while maintaining transparency about degraded privacy and security guarantees.

**Key additions:**

- Section 14A: Complete fallback mode specification
- EIP-6963 wallet discovery standard
- Multi-chain support (ETH, Base, PulseChain)
- Enhanced privacy consent dialogs
- Chain switching automation
- Delegated execution signaling (future-compatible)
- Comprehensive error handling for direct wallet transactions

**Architectural impact:**

- Extension can now function end-to-end without TBC dependency
- Clear user communication about privacy trade-offs
- Future-proof for TGP-aware wallet integration
- Maintains trust-minimization principles​​​​​​​​​​​​​​​​
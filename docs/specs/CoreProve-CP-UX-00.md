
This is the user-facing flow inside the CoreProve browser extension, defining:
	•	the buyer commit UX
	•	the seller commit UX
	•	session wallet behavior
	•	ZK messaging UX
	•	TTL expiry and warnings
	•	receipt vault UI
	•	merchant-driven settlement flow
	•	final ACK integration with the TBC

The goal is to deliver a developer-ready UX spec that can be implemented in the extension popup and background scripts immediately, fully aligned with:
	•	TGP-00 v3.2
	•	TGP-EXT-00
	•	TGP-EXT-ZK-00
	•	CoreProve-ZK-01 v1.1
	•	SettlementContractTemplate v0.2.2
	•	ReceiptVault_2025_26
	•	TypeChain bindings now available

Minimal but complete.

⸻

📘 CoreProve Extension CP-UX Spec (v0.2 — Day 2)

0. Overview

The CoreProve Extension handles:
	•	Session wallet generation
	•	TGP QUERY → EXT routing
	•	BuyerCommit and SellerCommit ZK flows
	•	ZK proof generation (circuit invoked locally)
	•	Envelope packaging (ZKB01/ZKS01)
	•	Sending messages → TBC
	•	Handling TGP-ACK responses
	•	TTL expiration detection
	•	Receipt verification + vault view

The extension must remain:
	•	stateless with respect to identity
	•	ephemeral for every transaction
	•	non-invasive to user wallets
	•	fully privacy-preserving

⸻

1. Startup Screen

State: Idle

Elements:
	•	CoreProve hexagon logo
	•	Button: “Initialize Session”
	•	Short text:
CoreProve enables private, ZK-protected payments without revealing your main wallet.

Action (click):
	•	Generates session wallet (ephemeral EOA)
	•	Stores:
	•	sessionId = 0x...32B
	•	sessionWallet.privateKey (in extension memory only)
	•	nonce = 0
	•	Moves to “Listening for Merchant” screen

⸻

2. Listening for Merchant Request

State: Waiting for TGP-EXT trigger

UI:
	•	Spinner
	•	Text: “Waiting for merchant checkout…”
	•	Session details (not exposing the pkHash):
	•	✔ Session active
	•	✔ Wallet ephemeral
	•	Button: Cancel session

Trigger:

When the extension receives from merchant website:

chrome.runtime.sendMessage(TGP_QUERY_PAYMENT_REQUIRED)

→ Move to Buyer Confirm.

⸻

3. Buyer Commit Flow (ZKB01)

3.1 Buyer Confirmation Screen

Inputs from merchant JSON policy:
	•	amount
	•	asset (native or ERC-20)
	•	TTL
	•	merchantFeeRecipient
	•	settlement contract address

UI:

Title: Confirm Private Payment

Fields:
	•	Amount (big font)
	•	Asset symbol
	•	Merchant: (domain)
	•	TTL countdown: Expires in X:YY
	•	Privacy note:
Your main wallet is never used. A ZK proof will be generated locally.

Buttons:
	•	Pay Privately (Generate Proof)
	•	Cancel

⸻

3.2 ZK Proof Generation Screen

State: Busy

UI:
	•	Progress indicator
	•	Text:
Generating ZK proof…
This stays local and never touches the blockchain.

Internally:
	•	Use sessionWallet to sign private inputs
	•	Generate pkHash
	•	Build nullifier + epoch
	•	Build public inputs:
	•	amount
	•	assetId
	•	expiry = timestamp
	•	Create envelope:

{
  type: “ZKB01”,
  sessionId,
  proof: {a,b,c},
  publicInputs: {nullifier, amount, assetId, expiry},
  identity: {pkHash, sessionId, chainId},
  nonce
}

→ increments nonce

Move to: Sending Payment.

⸻

3.3 Sending Payment (to TBC)

UI:
	•	Spinner
	•	Text: “Submitting encrypted payment to CoreProve Gateway…”

Action:

chrome.runtime.sendMessage({
    type: “TGP_EXT_ZK”,
    envelope: ZKB01
})

Await ACK from TBC:
	•	ACK_BUYER_COMMIT_ACCEPTED
	•	or error

On success → “Payment Accepted”.

On failure → “Payment Failed.”

⸻

3.4 Buyer Payment Accepted

UI:
	•	Green check
	•	Amount
	•	Settlement contract
	•	Note:
Continue to merchant to complete checkout.

Buttons:
	•	OK
	•	View Transaction Details (optional, shows tx hash if non-private mode)

Return to idle or merchant page.

⸻

4. Seller Commit Flow (ZKS01)

Triggered by merchant portal action:

chrome.runtime.sendMessage({ type: “TGP_QUERY_SELLER_COMMIT” })

4.1 Seller Confirmation Screen

Fields:
	•	Order ID
	•	Fulfillment Hash (hash of order details)
	•	TTL countdown
	•	Buyer committed? (status from TBC)

Buttons:
	•	Commit to Fulfill (Generate ZK Proof)
	•	Reject

4.2 ZK Proof Generation (seller)

Generate:
	•	pkHash
	•	nullifier
	•	timestamp
	•	fulfilHash

Envelope:

{
  type: “ZKS01”,
  sessionId,
  proof,
  publicInputs: { nullifier, fulfilHash, expiry },
  identity: { pkHash, sessionId, chainId },
  nonce
}

Send to TBC → show result.

⸻

5. TTL Expiration UX

TTL countdown displayed on buyer and seller screens:

Expires in HH:MM

When TTL < 30 seconds:
	•	Warning banner: “Expiring Soon”

If expired before commit:
	•	Red banner: “Time Window Expired — Restart Required”
	•	Disable commit buttons

If expired after commit:
	•	Leave commit alone (settle can still fail if TTL invalid)
	•	Show warning: “Waiting for settlement (expired)”

⸻

6. Receipt Vault UX

After settlement, the TBC notifies extension:

TGP_ACK_RECEIPT_ANCHORED
{
   receiptId,
   sessionId,
   orderId,
   settlementContract
}

The extension shows:

6.1 Receipt Anchored Screen

UI:
	•	NFT icon (non-transferable)
	•	Text:
Your private settlement is complete.
	•	Receipt ID: #XXXX
	•	View on chain: link to explorer
	•	Merchant name
	•	Asset + amount

Button:
	•	Done

⸻

7. Session Wallet UX

Session wallet metadata accessible via Developer / Advanced panel:
	•	sessionId
	•	pkHash
	•	ephemeral address
	•	NEVER show private key
	•	Button: Discard Session (wipes memory)

After settlement or cancellation:

delete sessionWallet
delete sessionId
delete nonce

Extension returns to “Initialize Session”.

⸻

8. Error UX

Display standard messages:
	•	“Merchant Policy Invalid”
	•	“Payment Rejected by Gateway”
	•	“ZK Proof Generation Failed”
	•	“Nullifier Previously Used”
	•	“TTL Expired”
	•	“Settlement Contract Error”

All error dialogs include:
	•	Retry
	•	Cancel

Retries regenerate a fresh ZK proof + new nullifier.

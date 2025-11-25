Spec is aligned with:
	•	MerchantContractFactory v0.3
	•	SettlementContractTemplate v0.2.2
	•	ReceiptVault_2025_26
	•	TGP-00 v3.2
	•	CoreProve-00 v0.9
	•	CoreProve-ZK-01 v1.1

It defines every screen, state, and workflow needed for the merchant-facing web portal…placing in TBC repo for test alignment…will move to coreprove repo and clean this up after testing.

⸻

📘 CoreProve Merchant Portal UX Spec (v0.1 — Day 2)

0. Overview

The Merchant Portal is the merchant-facing GUI used to:
	•	configure payment policies
	•	deploy merchant instances
	•	manage assets
	•	manage TTLs
	•	review receipts and settlements
	•	generate JSON policies consumable by CoreProve Extension
	•	inspect settlement contract status
	•	revoke or authorize assets
	•	monitor TBC fees + ZK fees

The portal interacts with:
	•	MerchantContractFactory
	•	ReceiptVault
	•	SettlementContractTemplate (deployed instance)
	•	TBC (for runtime API queries & confirmations)

⸻

1. Login / Connection Screen

Goal: Establish merchant identity by connecting their EOA for merchantAdmin.

UI:
	•	Branding: CoreProve hexagon + “Merchant Portal”
	•	Button: Connect Wallet
	•	Supported: MetaMask / Coinbase Wallet / WalletConnect
	•	Text: “Your connected wallet becomes the merchant admin.”

After connect → show:

Connected wallet: 0xAbc…123

Proceed to:

Merchant Dashboard

⸻

2. Merchant Dashboard

Primary blocks:

A. My Merchant Contracts
	•	List of deployed SettlementContracts
	•	For each:
	•	Version
	•	Address (copyable)
	•	Default asset
	•	TTL
	•	Status: Active / Deprecated
	•	Button: View Contract

B. Deploy New Merchant Contract

Button → opens Deployment Wizard

C. Settlement Receipts
	•	Count of total receipts minted
	•	View receipts in vault
	•	Button: Open ReceiptVault Explorer

D. Factory Information
	•	Active Template Version
	•	Factory address
	•	Vault address

⸻

3. Deployment Wizard

Main merchant workflow.

Flow:

Step 1 — Select Template Version

UI:
	•	Dropdown: Available Template Versions (from registry)
	•	For each version:
	•	Status: Experimental / Stable / Deprecated
	•	Code hash
	•	Template address

Rules:
	•	Deprecated versions: disabled
	•	Experimental: warning banner

Button: Next

⸻

Step 2 — Configure Payment Parameters

Fields:
	•	TTL Seconds
	•	Input box
	•	Description: max lifetime for buyer/seller commits
	•	Fee Recipients
	•	TBC fee (address)
	•	ZK fee (address)
	•	Merchant revenue recipient (address)
	•	Default Asset
	•	Dropdown:
	•	“Native (PLS)”
	•	“ERC20 — choose token”

If ERC20 chosen, detect:
	•	Symbol
	•	Decimals

Button: Next

⸻

Step 3 — Asset Configuration (Advanced)

UI:
	•	Allowed Assets List
	•	Native asset toggles
	•	ERC20 tokens with symbol + decimals
	•	“Add Asset” button opens modal

Modal fields:
	•	Token address
	•	Symbol
	•	Decimals
	•	Toggle: allow/disallow

Button: Save Asset

⸻

Step 4 — Review Deployment Summary

Shows:
	•	Template version
	•	Runtime bytecode hash
	•	TTL
	•	Fee recipients
	•	Default asset
	•	Connected wallet (merchantAdmin)

Button: Deploy with CREATE2

⸻

Step 5 — Deployment Transaction

UI:
	•	Spinner
	•	“Deploying deterministic contract using CREATE2…”
	•	Show the salt explicitly (auto-generated using factory rule)

On success:
	•	Show merchant contract address
	•	Auto-add to dashboard
	•	Button: Proceed to Configure Policy JSON

⸻

4. SettlementContract Detail View

Selecting a merchant contract from the dashboard opens:

Section A — Contract Summary
	•	address
	•	template version
	•	TTL
	•	default asset
	•	fee recipients
	•	creation date
	•	code hash
	•	authorized in vault: yes/no

Button: Revoke Settlement Authorization (optional)

⸻

Section B — Asset Registry Management

UI table:

Asset	Symbol	Decimals	Allowed	Type	Actions
0x0	native	n/a	✔	native	toggle
0xToken	USDC	6	✔	ERC20	remove

Actions:
	•	Toggle native asset allowed
	•	Remove ERC20 token
	•	Add new ERC20 token

Each action → one-click transaction modal.

⸻

Section C — Active Commit Status

Query via TBC:
	•	Buyer commits
	•	Seller commits
	•	Settlement-ready pairs

Show summary:

Open Buyer Commits: X
Open Seller Commits: Y
Ready for Settlement: Z

(Slightly future-facing, but aligned with the TGP/TBC pipeline.)

⸻

Section D — Settlement Actions

A merchant can optionally call settle() manually (allowed actor: merchantAdmin).

UI:
	•	Enter Session ID
	•	Enter Order ID
	•	Button: Trigger Settlement

On optional ZK-enabled future version: show ZK settle flows.

⸻

5. Policy JSON Builder (Main Deliverable)

This is critical.
This defines the JSON that merchant websites embed to trigger buyer commits.

Screen title: Generate Checkout Policy

⸻

Step A — Required Fields (auto-filled from contract)
	•	settlementContractAddress
	•	merchantFeeRecipient
	•	tbcFeeRecipient
	•	zkFeeRecipient
	•	defaultAsset
	•	ttlSeconds
	•	version

All pulled from live contract state.

⸻

Step B — Checkout Details (merchant provides):
	•	Product name
	•	Price
	•	Currency (based on default asset)
	•	Order metadata (string → turned into hash)

UI generates:

orderHash = keccak256(JSON.stringify(metadata))


⸻

Step C — Generate JSON

The final policy file =

{
  “version”: “coreprove-policy-1.0”,
  “settlementContract”: “0x...”,
  “tbcFeeRecipient”: “0x...”,
  “zkFeeRecipient”: “0x...”,
  “merchantFeeRecipient”: “0x...”,
  “ttlSeconds”: 300,
  “defaultAsset”: “0x...”,
  “order”: {
    “name”: “Product”,
    “price”: “100000000”,
    “orderHash”: “0x...”,
    “notes”: “optional”,
    “timestamp”: 1732578123
  }
}

Button: Copy Policy JSON
Button: Download as policy.json

This file is embedded in merchant websites:

<script id=“coreprove-policy” type=“application/json”>
   {...policy json...}
</script>


⸻

6. ReceiptVault Explorer

UI shows:
	•	receiptId
	•	orderId
	•	merchantContract
	•	amount
	•	asset
	•	timestamp

Clickable rows → show details.

Merchant can filter:
	•	by date
	•	by customer
	•	by asset
	•	by orderId

⸻

7. TBC Connectivity Panel (Advanced)

Merchant can see:
	•	TBC endpoint
	•	TGP handshake logs
	•	Buyer commits incoming
	•	Seller commits incoming

Not needed for MVP, but spec includes it for investor demo.

⸻

8. Error UX

Common merchant errors:
	•	INVALID_TEMPLATE
	•	TEMPLATE_DEPRECATED
	•	INVALID_ASSET
	•	NOT_MERCHANT_ADMIN
	•	INVALID_POLICY
	•	INVALID_DEPLOY_SALT
	•	CREATE2_FAILED
	•	RECEIPT_NOT_AUTHORIZED

Show modal with:
	•	error code
	•	what happened
	•	suggested action


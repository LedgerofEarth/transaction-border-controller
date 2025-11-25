🌐 Transaction Border Controller (TBC)

Private, Safe, NATted Payments for Buyers and Merchants

The Transaction Border Controller (TBC) is a transaction firewall that lets buyers and merchants transact securely without ever exposing their real wallet addresses, financial history, or operational infrastructure.

TBC provides transaction NAT — Network Address Translation — but for blockchain wallets.

Just like IP NAT hides internal infrastructure behind a carrier-grade gateway, TBC hides buyer and merchant wallets behind a policy-controlled transaction gateway.

⸻

🔒 What “Transaction NAT” Means

🛒 For Buyers
	•	Your real wallet address is never exposed to the merchant.
	•	The extension uses a delegated session key or policy key.
	•	Merchants never see:
	•	your main wallet
	•	your transaction history
	•	your token holdings
	•	prior or future activity

Your main wallet stays private — permanently.

When a buyer authorizes a purchase:
	1.	The TGP Client constructs a QUERY.
	2.	The TBC returns an Economic Envelope.
	3.	The wallet blindly signs the pre-constructed transaction.
	4.	The settlement contract receives the funds.
	5.	The merchant only sees escrow state, not the buyer’s wallet.

This is payment NAT for consumers.

⸻

🏬 For Merchants

Merchants also gain NAT-level protection:
	•	Their treasury address is never exposed to buyers.
	•	Every payment flows through the CoreProve settlement contract, not the merchant’s hot wallet.
	•	Settlement contracts act as isolated escrow endpoints.

Buyers never know:
	•	the merchant’s internal wallet structure
	•	which accounts hold operational funds
	•	routing between merchant business units

Attack surface is dramatically reduced.

Just like a web server behind NAT:
	•	the merchant’s wallets cannot be DDoS’d
	•	cannot be probed
	•	cannot be target-profiled

All a buyer sees is the merchant’s on-chain payment profile (a contract, not a wallet).

⸻

🧩 Why Businesses Care

Merchants today are hesitant to accept crypto because:

❌ Their wallets get doxxed

Once a buyer pays a merchant, the merchant’s entire financial history becomes visible.

❌ They must operate hot wallets

Hot wallets are dangerous and operationally expensive.

❌ Every payment exposes infrastructure

Treasury flows, employee payroll wallets, vendor payments — all traceable.

❌ Multi-step settlement flows are fragile

Current Web3 wallets are not built to handle accept/fulfill/claim workflows.

⸻

TBC solves all of this.

⸻

🔐 What TBC Delivers

1. Privacy Without Mixing

No mixers, no tumblers, no regulatory risk.
Just NAT-style indirection using a smart contract–driven settlement system.

Merchants and buyers only see what the settlement contract reveals.

⸻

2. Composable Settlement Governance

Multi-step escrow flows enforced by protocol:

commit → accept → fulfill → claim → settle

This works for:
	•	local delivery
	•	digital goods
	•	subscriptions
	•	staged services
	•	agent-driven automation

⸻

3. Wallet-Safe Checkout

TBC never sees:
	•	private keys
	•	seed phrases
	•	signatures
	•	wallet internals

The wallet remains a blind signer, exactly as today — but safer.

⸻

4. Zero Custody Risk

Funds are held in merchant-specific settlement contracts, with:
	•	no admin keys
	•	no upgrade keys
	•	no backdoors
	•	no privileged users

These contracts are constrained custodians:
They hold funds but cannot be abused.

⸻

5. NAT Across Jurisdictions

The TGP routing layer allows transaction flows across multiple gateways.

Each gateway can:
	•	apply local compliance policy
	•	append jurisdiction metadata
	•	add required fees

This lets merchants operate in:
	•	multiple states
	•	multiple countries
	•	federated environments

All while keeping their internal wallet infrastructure private.

⸻

⚙ How the NAT Layer Works

Buyer Wallet
   |
   | (blind signing)
   v
Buyer NAT (TBC)
   |
   | Economic Envelope
   v
CoreProve Settlement Contract ←→ Merchant NAT (TBC)
                                     |
                                     v
                              Merchant Treasury

The buyer and merchant can operate behind their own NAT layers.

Neither party learns the other’s true wallet.

Settlement happens in a neutral zone (CoreProve contract).


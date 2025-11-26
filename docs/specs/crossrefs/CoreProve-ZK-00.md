📘 CoreProve-ZK-01 (Corrected Edition)

Zero-Knowledge Integration Specification

Version: 1.1-corrected
Status: Integration-Ready
Company: Ledger of Earth, LLC

All conceptual mistakes resolved.
All diagrams updated.
All flows corrected to reflect:
	•	Extension generates ZK proofs
	•	TBC verifies ZK proofs
	•	Settlement contract receives only public outputs
	•	ZK is never sent to the blockchain
	•	ReceiptVault stores only proof hashes

⸻

0. Purpose

This document defines the Zero-Knowledge (ZK) integration model for CoreProve.

It specifies:
	•	what must be proven
	•	where proofs are generated
	•	where proofs are verified
	•	how nullifiers, timestamps, and pk_hashes flow into settlement contracts
	•	how proofs anchor into Receipt NFTs
	•	how proofs remain verifiable decades into the future

This spec defines ZK statements and data interfaces, not circuit math.

⸻

1. Correct Architectural Position

ZK proofs are:
	•	generated in the Extension
	•	verified inside the TBC
	•	never uploaded to the settlement contract
	•	reduced to public inputs + nullifiers for the contract

Correct CoreProve ZK pipeline:

EXTENSION
    │
    │  (Generate ZK proof: ZKB-01, ZKS-01, ZKM-01)
    ▼
TGP-EXT
    │
    │  (Deliver proof & public inputs to TBC)
    ▼
TBC
    │
    │  (Verify SNARK off-chain)
    │
    ▼
TBC-ZK Verification Layer
    │
    │  (Extract public inputs)
    │  (Bind nullifier)
    │  (Bind timestamps & pkHash)
    ▼
Settlement Instruction Builder
    │
    │  (ABI: BuyerZKProof, SellerZKProof)
    ▼
CoreProve Settlement Contract (on-chain)
    │
    ▼
ReceiptVault (anchors proof hashes & public inputs)

Key correction:
The blockchain sees only public circuit outputs, not the ZK proof.

⸻

2. ZK Goals (Unchanged, but clarified)

Privacy

Buyers/sellers never expose root wallets, signatures, or identity.

Integrity

Proofs ensure the right actor performed the right action.

Replay Prevention

Nullifiers bind proofs to single-use actions.

Auditability

ReceiptVault stores verifiable hashes, not proofs.

Cost Efficiency

All SNARK verification happens off-chain in the TBC.

⸻

3. Proof Types (Clarified)

Code	Name	Actor	Purpose
ZKB-01	Buyer Deposit Proof	Buyer	Prove deposit + root/session linkage
ZKS-01	Seller Fulfillment Proof	Seller	Prove seller authorized fulfillment
ZKM-01	Merchant Policy Integrity	Merchant	Prove policy & code integrity

Important correction:
All proofs are generated in the Extension, not the TBC.

⸻

4. Merchant ZK Flags (Clarified)

Merchant payment profile dictates:

requireZKBuyerDeposit: true|false
requireZKSellerFulfillment: true|false
requireZKPolicyIntegrity: true|false

TBC enforces these flags in the settlement path.

⸻

5. Verification Model (Corrected)

5.1 Off-Chain Verification (Primary)

The TBC verifies:
	•	the SNARK
	•	nullifier freshness
	•	pkHash correctness
	•	timestamp window
	•	merchant ZK policy

NO ZK verifier exists on-chain in v0.2.1.

5.2 On-Chain Verification (Fallback)

If a merchant requires it (future version):
	•	Groth16 verifier may be plugged in
	•	Settlement contract may call verifier contract

This is optional.

⸻

6. TGP-EXT ZK Delivery (Corrected)

TGP-EXT sends:

zk_proof      : raw SNARK proof bytes
zk_inputs     : public inputs for SNARK
zk_type       : ZKB01 | ZKS01 | ZKM01
zk_nullifier  : random oracle nullifier
zk_timestamp  : timestamp bound inside proof
session_pubkey: extension ephemeral pubkey

TBC verifies SNARK and then rewrites into the minimal structure required by contract.

⸻

7. Proof Statements (Corrected + Expanded)

7.1 ZKB-01 — Buyer Deposit Proof

Buyer proves:
	1.	They control the session wallet.
	2.	Session is linked to buyer root wallet (ZK-NAT).
	3.	Deposit amount matches amount committed.
	4.	Deposit signature matches session wallet.
	5.	Nullifier is unique.
	6.	Timestamp is fresh.

Contract sees only:
	•	amount
	•	pkHash
	•	nullifier
	•	timestamp

⸻

7.2 ZKS-01 — Seller Fulfillment Proof

Seller proves:
	1.	They control merchant’s signing key.
	2.	They authorized fulfillment for this specific order.
	3.	Nullifier is unique.
	4.	Timestamp is fresh.

Contract sees only:
	•	pkHash
	•	nullifier
	•	timestamp
	•	orderHash

⸻

7.3 ZKM-01 — Merchant Policy Integrity Proof (Optional)

Merchant proves:
	1.	Policy contract bytecode matches canonical hash.
	2.	Chain id is correct.
	3.	They authorized the policy.

Contract sees nothing (merchant proofs do not go on-chain).
Only the TBC enforces this.

⸻

8. Receipt Anchoring (Corrected)

ReceiptVault stores:
	•	proof_buyer_hash
	•	proof_seller_hash
	•	proof_policy_hash
	•	public_inputs_hash
	•	timestamp
	•	settlement metadata

Receipts never store full proofs.

Receipts never reveal buyer or seller identity.

Receipts never reveal witness data.

Hashing:

proof_hash = SHA256(zk_proof_bytes)
inputs_hash = SHA256(public_inputs_bytes)


⸻

9. Archival Model (Corrected)

On-Chain

Only proof hashes + public input hashes.

Off-Chain (Entire Proof)
	•	extension device
	•	merchant servers
	•	ReceiptVault (optional)
	•	IPFS/Filecoin

Reproducibility

Given the NFT’s public inputs + the witness retained by the user,
the proof can be regenerated even decades later.

⸻

10. Circuit Systems (No change, but clarified)
	•	Groth16 for on-chain
	•	PLONK for off-chain
	•	Recursive SNARK future

⸻

11. Integration with TBC (Corrected)

Precise flow:

Extension (produce proof)
    → TGP (deliver proof)
        → TBC (verify)
            → rewrite to public inputs
                → settlement tx
                    → CoreProve contract
                        → ReceiptVault

If verification fails → REJECT: ZK_INVALID_PROOF.

⸻

12. Integration with SettlementContract v0.2.1 (Corrected)

Contract accepts:

Buyer

BuyerZKProof {
    bytes32 pkHash;
    bytes32 nullifier;
    uint256 amount;
    uint256 timestamp;
}

Seller

SellerZKProof {
    bytes32 pkHash;
    bytes32 nullifier;
    uint256 timestamp;
    bytes32 orderHash;
}

These are not SNARK proofs —
they are the public input field outputs of TBC’s ZK verification layer.

⸻

13. Updated Security Considerations (Corrected)
	•	Proofs MUST NOT go on-chain.
	•	Contract MUST rely on nullifiers + timestamps, not proofs.
	•	TBC MUST enforce ZK verification.
	•	Extensions MUST generate proofs.
	•	ReceiptVault MUST not reveal witness.
	•	Nullifiers MUST be collision-resistant.

⸻

END OF DOCUMENT
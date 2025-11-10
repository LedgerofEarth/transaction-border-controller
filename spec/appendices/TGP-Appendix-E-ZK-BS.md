Appendix E – ZKX-01 Combined Buyer–Seller Settlement Circuit (Informative / Normative Hybrid)

H.1 Purpose

ZKX-01 merges the buyer’s ownership proof (ZKB-01) and the seller’s fulfillment proof (ZKS-01) into one zero-knowledge attestation.
It produces a single verifiable proof that both parties:
	1.	satisfied their respective obligations under a specific policy_hash and trace_id, and
	2.	never revealed sensitive address or fulfillment data to intermediate gateways.

It serves as the canonical privacy-preserving replacement for traditional dual-signature receipts.

⸻

H.2 Scope & Use Cases
	•	Atomic private commerce – one proof replaces multiple signatures.
	•	Multi-hop settlement – gateways need only check one compact proof.
	•	Cross-jurisdiction audits – auditors can confirm compliance without user PII.
	•	AI-agent transactions – machine-to-machine exchange of data/services under TGP.

⸻

H.3 Conceptual Flow

Buyer → Prover → Seller  → Prover → Gateway → TGP SETTLE
      (ZKB-01)             (ZKS-01)
           \______________________/  
                Aggregated → ZKX-01

The Prover (or an aggregator) fuses both proofs into one composable circuit before SETTLE.

⸻

H.4 Circuit Overview

Goal: prove jointly that
	•	the buyer controls the funded address in txHash; and
	•	the seller fulfilled the corresponding commitment_hash and attachments_hash,
under the same policy_hash and trace_id.

Public Inputs

Field	Description
chainId	Settlement chain identifier.
txHash	Transaction reference or receipt root.
commitment_hash	CoreProver commitment being fulfilled.
attachments_hash	Hash of fulfillment artifacts.
policy_hash	Binds both sub-proofs to one TGP policy.
trace_id	Session identifier (TIB §11).

Private Inputs

Field	Description
sk_buyer	Buyer’s spend secret.
sk_seller	Seller’s signing secret.
salt_buyer, salt_seller	Randomizers for unlinkability.
log_root	Seller’s internal fulfillment log Merkle root.
credentialSecrets?	Optional selective-disclosure credentials.

Constraints
	1.	Verify buyer inclusion (per ZKB-01).
	2.	Verify seller signature & attachments (per ZKS-01).
	3.	Enforce shared policy_hash and trace_id.
	4.	Combine both sub-proof roots into
proof_root = Poseidon(proof_root_buyer, proof_root_seller, policy_hash, trace_id).

Outputs
	•	proof_root – unified settlement root.
	•	proof_bytes – serialized aggregated ZK proof (Groth16/Plonk/Halo2).
	•	verifier_signature? – optional gateway attestation.

⸻

H.5 Reference Pseudo-Code

PublicInputs:
  chainId, txHash, commitment_hash, attachments_hash, policy_hash, trace_id
PrivateInputs:
  sk_buyer, sk_seller, salts, log_root, credentialSecrets?

Computation:
  addr = DeriveAddress(sk_buyer)
  pk_seller = PublicKey(sk_seller)
  assert VerifyMerkleInclusion(txHash, addr)
  assert VerifySellerSign(pk_seller, commitment_hash || attachments_hash)
  assert Hash(policy) == policy_hash
  assert MerkleCheck(log_root, attachments_hash)
Outputs:
  proof_root = Poseidon(addr, pk_seller, commitment_hash, attachments_hash, policy_hash, trace_id)


⸻

H.6 Verifier Interface

function verifySettlementProof(
    bytes calldata proof_bytes,
    bytes32 proof_root,
    bytes32 policy_hash,
    bytes32 commitment_hash,
    bytes32 txHash
) external view returns (bool valid);

Any Prover advertising ZK_ATTESTATION | ESCROW | FULFILLMENT MAY implement this interface.

⸻

H.7 Integration with TGP Lifecycle

Stage	Proof Interaction	Outcome
LOCKED → PROOF	Buyer & Seller contribute sub-proofs to aggregator	proof_root computed
PROOF → SETTLE	Gateway verifies ZKX-01 via Prover	Atomic validation
SETTLE → CLOSED	Verified proof logged in TDR	Final non-repudiation


⸻

H.8 JSON Example in TGP Message

{
  “tgp_version”: “0.1”,
  “message_type”: “SETTLE”,
  “sender”: “gateway-b.example”,
  “zkx_settlement_proof”: {
    “system”: “halo2”,
    “proof_root”: “0xabc9...fe7”,
    “policy_hash”: “0xd15c...3aa”,
    “commitment_hash”: “0x7af1...e02”,
    “txHash”: “0x6de4...991”,
    “proof_bytes”: “base64:AAAA1111...”,
    “verifier”: “0xProverZKX01”
  }
}


⸻

H.9 Security Properties

Property	Guarantee
Atomicity	Both buyer & seller proven under same policy.
Anonymity	Neither address nor internal logs revealed.
Non-repudiation	Single verifiable proof replaces dual signatures.
Replay Resistance	Bound to trace_id and policy_hash.
Auditability	Gateways log proof_root, policy_hash, commitment_hash, txHash.


⸻

H.10 Supported Aggregation Modes

Mode	Description
On-chain Aggregation	Both sub-proofs submitted to Prover, combined via recursive Halo2.
Off-chain Aggregation	Aggregator service produces proof_bytes, Prover verifies succinctly.
Deferred Aggregation	Gateways store buyer & seller proofs separately, aggregate later for audit.


⸻

H.11 Extensibility

Future ZKX profiles may introduce:
	•	Multi-party proofs (n-of-m participants).
	•	Layer-2 batch aggregation for micropayments.
	•	Post-quantum proof systems (STARKs).
	•	Selective-disclosure audit proofs (ZKX-02) revealing only compliance attributes.

⸻

(end of Appendix H – ZKX-01 Combined Buyer–Seller Settlement Circuit)

⸻

✅ With Appendix H, the TGP-00 spec now supports a complete privacy-preserving settlement stack:
	•	ZKB-01 Buyer Proof → ownership without exposure.
	•	ZKS-01 Seller Proof → fulfillment without disclosure.
	•	ZKX-01 Combined Proof → atomic, auditable settlement in one verification.
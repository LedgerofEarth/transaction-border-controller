Appendix G – ZKR-01: Zero-Knowledge Proof-of-Settlement Receipts

(Normative / Informative Hybrid)

J.1 Purpose

ZKR-01 defines a standard receipt format and verification method for recording the final state of any TGP transaction settled through ZK-based proofs (ZKB-01 / ZKS-01 / ZKX-01) and validated via an Aggregator (ZKA-01).
It provides:
	•	Audit-grade verifiability of settlement outcomes.
	•	Privacy preservation, ensuring no sensitive fields (addresses, attachments, or internal logs) are exposed.
	•	Cross-domain interoperability so that gateways, regulators, and users can confirm transaction finality with one receipt hash.

⸻

J.2 Conceptual Overview

Every TGP transaction that completes SETTLE with a verified ZK proof MUST emit or store a ZK-Receipt.
Each receipt binds the three cryptographic pillars of TGP:

Layer	Evidence	From
L8 (Value)	Settlement tx / proof_root	ZKX-01 or CoreProver
L9 (Identity)	Aggregator / Registry attestation	ZKA-01
L10 (Policy)	policy_hash + trace_id	Gateway / TIB

Together they yield a single deterministic commitment:

receipt_hash = H( proof_root || manifest_hash || policy_hash || trace_id )


⸻

J.3 Receipt Structure

{
  “zkr_receipt”: {
    “version”: “1.0”,
    “trace_id”: “uuid-v4”,
    “policy_hash”: “0xd15c...3aa”,
    “proof_root”: “0x9f3b...fe7”,
    “aggregator_id”: “agg.us.zk”,
    “manifest_hash”: “0xabc...789”,
    “verifier_contract”: “0xVerifier123”,
    “chain_id”: 369,
    “timestamp”: “2025-11-08T19:22:00Z”,
    “settlement_txid”: “0x6de4...991”,
    “result”: “SETTLED”,
    “audit_level”: “summary|full”,
    “receipt_hash”: “0xdef...555”,
    “aggregator_signature”: “base64(ed25519(sig over receipt_hash))”
  }
}

Optional fields:
	•	proof_system: “halo2”, “plonk”, etc.
	•	jurisdiction: “US”, “EU”, etc.
	•	fee_paid: numeric USD or token equivalent.

⸻

J.4 On-Chain vs Off-Chain Anchoring

Mode	Description
On-chain anchor	Receipt hash emitted as event by CoreProver or Aggregator contract. Auditors verify inclusion via block header proofs.
Off-chain anchor	Receipt stored in append-only registry or IPFS; periodic Merkle root anchored on-chain.
Hybrid	Off-chain bulk receipts + batched Merkle root on-chain (recommended).


⸻

J.5 Verification Procedure

Any verifier / auditor / gateway MAY verify a receipt by:
	1.	Recompute receipt_hash from fields (proof_root, manifest_hash, policy_hash, trace_id).
	2.	Check Aggregator signature using public key from registry manifest (§ I.8).
	3.	Confirm proof_root validity via ZKX-01 verifier contract.
	4.	Confirm manifest_hash still active in registry Merkle root.
	5.	Ensure timestamp within valid SETTLE window per TGP policy.

If all pass, the settlement is cryptographically proven even if underlying data remain hidden.

⸻

J.6 Receipt Attestation Hierarchy

CoreProver  ─┐
             │  produces proof_root
Aggregator  ─┼─→ signs receipt_hash
Registry    ─┘   logs manifest_hash
Auditor     ─►   verifies + timestamps

Multiple signers MAY co-sign a receipt using threshold or multi-sig schemes (t-of-n auditors).

⸻

J.7 Audit Integration

Auditors and regulators can reconstruct proofs of compliance by collecting only:

receipt_hash, manifest_hash, policy_hash, trace_id, timestamp

and verifying signatures—no access to private data required.

Gateways SHOULD retain ZKR receipts for N_years ≥ 7 for regulated markets.

⸻

J.8 Security & Privacy Properties

Property	Guarantee
Integrity	receipt_hash binds proof, policy, and registry attestations.
Non-repudiation	aggregator_signature makes denial of settlement infeasible.
Anonymity	buyer/seller data omitted; only proof_root revealed.
Replay protection	trace_id + timestamp prevent re-submission.
Audit resilience	receipts verifiable offline via registry manifests.


⸻

J.9 Inter-Registry Validation

If multiple registries coexist:
	•	Each publishes Merkle root of accepted receipts.
	•	Cross-registry notaries can co-sign aggregated receipt bundles (receipt_root).
	•	A ZKR-AGG message MAY broadcast these roots to peers:

{
  “message_type”: “ADVERT.ZKR”,
  “registry_id”: “reg.global.zk”,
  “receipt_root”: “0x55aa...e3f”,
  “count”: 100000,
  “timestamp”: “2025-11-08T20:00:00Z”
}


⸻

J.10 Extensibility

Future revisions may include:
	•	ZKR-02: Recursive receipts aggregating many settlements.
	•	ZKR-03: Post-quantum signature envelopes.
	•	Selective-disclosure audit proofs revealing only compliance attributes (jurisdiction, fee).
	•	Decentralized notarization using DIDComm attestations for off-chain participants.

⸻

J.11 Relationship to Other Appendices

Appendix	Role
C (ZKB-01)	Supplies buyer control proof.
D (ZKS-01)	Supplies seller fulfillment proof.
E (ZKX-01)	Combines proofs into unified settlement.
F (ZKA-01)	Provides registry and signature standards.
G (ZKR-01)	Records and audits verified settlements.


⸻

J.12 Summary

ZKR-01 establishes that zero-knowledge ≠ zero accountability.
It ensures every private, policy-bound settlement within TGP produces a publicly verifiable receipt—the cornerstone of a ZK audit trail for decentralized commerce.
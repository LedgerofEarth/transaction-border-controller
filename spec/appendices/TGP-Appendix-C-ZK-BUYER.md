Appendix C – ZKB-01: ZK Buyer-Proof Circuit (Informative / Normative Hybrid)

F.1 Purpose

The ZK Buyer-Proof circuit allows a buyer or its agent to prove ownership or control of the destination address used in a settlement—without revealing the address itself to upstream gateways or sellers.
It supports privacy-preserving confirmation that the funds came from a legitimate wallet while keeping identity and wallet topology hidden.

ZKB-01 complements the CoreProver model (§ E) and is compatible with any Prover declaring the ZK_ATTESTATION capability.

⸻

F.2 Scope & Use Cases
	•	Private purchase or subscription confirmations where revealing a wallet address could link identities.
	•	Proof-of-payment assertions in multi-hop or federated TGP paths.
	•	Reputation or compliance proofs derived from shielded transaction histories.

The proof MAY be attached to:
	•	the PROOF or SETTLE message in TGP, or
	•	the fulfillment phase of a CoreProver commitment.

⸻

F.3 Circuit Overview

Goal: Verify that the buyer controls the address which actually received (or sent) a settlement output in a known transaction, without disclosing that address.

Inputs:

Category	Field	Visibility	Description
Public	chainId	public	Chain on which the settlement occurred.
Public	txHash / receiptRoot	public	Transaction reference.
Public	credited_output_commitment	public	Commitment or Merkle root of the credited output.
Public	policy_hash	public	Hash binding proof to the current TGP policy (§ 12.3).
Private	sk_buyer	private	Buyer’s private key or spend secret.
Private	salt	private	Random nonce for unlinkability.
Private	credentialSecrets?	private	Optional selective-disclosure credentials (e.g., VC).

Constraints:
	1.	addr = f(sk_buyer) using the appropriate address derivation for the target chain.
	2.	credited_output_commitment ↔ addr proven by inclusion in txHash (Merkle path / receipt parsing gadget).
	3.	policy_hash ensures the proof is valid only for the specific TGP transaction and policy context.
	4.	Optional: credential verifies policy membership (AML_OK, KYC_TIER2, etc.).
	5.	Proof binds to session_id or trace_id from the TGP TIB (§ 11).

Outputs:
	•	proof_root – commitment that binds all public inputs.
	•	proof_bytes – verifier-ready proof (Groth16 / Plonk / Halo2).
	•	verifier_signature (optional) – attestation by an off-chain verifier or gateway.

⸻

F.4 Reference Pseudo-Code

PublicInputs:
  chainId, txHash, credited_output_commitment, policy_hash
PrivateInputs:
  sk_buyer, salt, credentialSecrets?

Computation:
  addr = DeriveAddress(sk_buyer)
  assert VerifyMerkleInclusion(txHash, addr, credited_output_commitment)
  assert Hash(policy) == policy_hash
  if credentialSecrets:
       assert VerifyCredential(credentialSecrets)

Outputs:
  proof_root = Poseidon(chainId, txHash, policy_hash, credited_output_commitment)

The resulting proof is bound to the current TGP transaction context and is non-transferable.

⸻

F.5 Verifier Integration with TGP

A Prover implementing ZK_ATTESTATION exposes:

function verifyBuyerProof(
    bytes calldata proof_bytes,
    bytes32 proof_root,
    bytes32 policy_hash
) external view returns (bool valid);

Gateways or CoreProvers MAY call this to validate proofs before accepting a PreAuth or Commitment.

When successful, gateways SHOULD log the tuple:

zk_proof_hash = keccak256(proof_bytes)
zk_policy_hash = policy_hash
zk_verifier = prover_id

into the TDR.

⸻

F.6 Security Properties

Property	Guarantee
Unlinkability	Fresh salt prevents correlation across transactions.
Non-repudiation	Proof bound to policy_hash and trace_id.
Selective disclosure	Optional credentials can reveal attributes without identity.
Replay protection	policy_hash and trace_id tie proof to one session.


⸻

F.7 Supported Proof Systems

System	Notes
Groth16	Compact proofs (≈ 128 B); requires trusted setup.
Plonk	Universal setup, good verifier performance.
Halo2	Recursive proof composition for multi-hop attestations.

Implementations SHOULD include the proof system identifier in metadata:

“zk”: { “system”: “halo2”, “curve”: “bn254”, “proof_root”: “0x...” }


⸻

F.8 Integration with CoreProver Workflow

Stage	Purpose	Action
PreAuth	Buyer may attach ZK proof proving wallet control.	openPosition includes proof_bytes.
Commitment	Seller verifies proof via Prover or gateway before accepting.	verifyBuyerProof() called.
Fulfillment	Proof (if provided) archived with receipt and attachments.	attachmentsHash updated.

If verification fails, the gateway MUST reject the SELECT or Commitment with ERROR.ZK_INVALID.

⸻

F.9 JSON Example in TGP Message

{
  “tgp_version”: “0.1”,
  “message_type”: “PROOF”,
  “sender”: “gateway-a.example”,
  “zk_proof”: {
    “system”: “halo2”,
    “proof_root”: “0x9a4c...77f”,
    “policy_hash”: “0xd15c...3aa”,
    “proof_bytes”: “base64:ABCD...”,
    “verifier”: “0xProverZK01”
  }
}


⸻

F.10 Compliance and Privacy Notes
	•	ZKB-01 proofs are pseudonymous; they confirm control, not identity.
	•	Jurisdictions requiring identity disclosure MUST use selective-disclosure credentials inside the proof.
	•	Gateways MUST treat the absence of a ZK proof as “no additional privacy assurances,” not as failure.

⸻

F.11 Extensibility

Future versions MAY include:
	•	multi-hop recursive proofs (ZKB-02),
	•	post-quantum circuits,
	•	cross-ledger attestation gadgets for non-EVM chains,
	•	integration with decentralized identity wallets (DIDs + SD-JWT).
# ZKB-01: Zero Knowledge Buyer-Control Proof Circuit (Draft)

**Purpose:**  
Allow the buyer to **hide their wallet address** while proving they control the destination address (or output) that received value in a settlement transaction, and bind that proof to:

- a specific **settlement transaction** (L8),  
- a specific **Economic Envelope** (EE, TGP-01),  
- a specific **policy_root** / TA context (TGP-TAI), and  
- a specific **TGP session/request**.

This proof is intended to be logged into the **Proof-of-Settlement (PoS)** object defined in **TGP-01** and referenced in TDRs.

—

## Public Inputs

The verifier (gateway / TBC / Prove Router) sees:

- `chain_id` — settlement chain identifier (e.g. `369` for PulseChain).  
- `settlement_txid` — hash of the on-chain settlement transaction whose output should be controlled by the buyer.  
- `credited_output_commitment` — a hash/commitment to the output or address that received funds (e.g. commitment over `{asset, amount, blinded_addr}` or a Merkle leaf).  
- `amount` — settled amount associated with this output (in smallest unit: wei, sats, etc.).  
- `asset_id` — asset identifier (e.g. `evm:ETH:1` or `evm:USDC:369`).  
- `economic_envelope_hash` — `keccak256(canonical-serialize(EE))` as defined in TGP-01.  
- `policy_root` — Merkle root of the TA policy corpus (from TGP-TAI / TAO).  
- `policy_hash` — deterministic hash of the evaluated policy (from TGP-00 §12.3).  
- `tgp_path` — T-Path string (e.g. `tai:7abf92c6>tai:8bde4411`).  
- `session_id` or `request_id` — unique TGP identifier for this negotiation/settlement.  

These public inputs ensure the proof is **bound to this specific settlement + envelope + policy + path**, and cannot be replayed for another deal.

—

## Private Inputs (Witness)

Held only by the buyer:

- `sk_buyer` — secret key or seed controlling the true destination wallet.  
- `addr_buyer` — derived address (could be recomputed from `sk_buyer` or included as a witness, depending on circuit design).  
- `merkle_path` or `receipt_witness` — minimal data needed to prove inclusion of the credited output in `settlement_txid` (SPV proof, receipt fields, or log index path).  
- `salt` — random scalar used to blind/commit the address or output.  
- `credential_secrets?` — secrets from selective-disclosure credentials (e.g., age > 18, KYC tier), if required by policy.  

—

## Constraints

The circuit MUST enforce:

1. **Key-to-Address Binding**

   ```text
   addr_buyer = f(sk_buyer)

Where f is the chain-specific address derivation function (e.g., addr = keccak256(pubkey)[12:] for EVM).
The circuit either:
	•	recomputes addr_buyer from sk_buyer, or
	•	verifies a signature from addr_buyer and treats it as a witness.

	2.	Output Ownership
Prove that addr_buyer is the beneficiary of the credited output:
	•	Using merkle_path / receipt_witness, the circuit verifies that settlement_txid includes an output or log entry which:
	•	matches the public amount and asset_id, and
	•	is committed in credited_output_commitment in such a way that:

credited_output_commitment = H(asset_id || amount || blind(addr_buyer, salt))


	•	The circuit checks the Merkle inclusion or receipt structure proving that this committed output is part of the transaction identified by settlement_txid on chain_id.

	3.	Policy & Envelope Binding
The circuit MUST bind the proof to the policy and envelope used in TGP:

H_policy = H(policy_root || policy_hash || tgp_path)
H_env    = economic_envelope_hash

The proof enforces that:
	•	policy_root and policy_hash are consistent with a valid TA policy under which this settlement is permitted (logic can be partial or via external verifier), and
	•	economic_envelope_hash equals the hash of the EE agreed in SELECT / SETTLE.
At minimum, the circuit MUST include policy_root, policy_hash, tgp_path, and economic_envelope_hash as public inputs and constrain them into a combined binding (e.g., H_bind = H(H_policy || H_env || session_id)).

	4.	Session / Request Binding
The circuit MUST include session_id (or request_id) as a public input and include it in the binding hash:

H_bind = H(policy_root || policy_hash || tgp_path || economic_envelope_hash || session_id)

This ensures the proof cannot be replayed across different TGP sessions or envelopes.

	5.	Optional Credential Constraints
If the policy requires attributes like “age ≥ 18”, “KYC tier ≥ P1”, etc.:
	•	The circuit takes credential_secrets as witness.
	•	Verifies signature / MAC from issuer on a credential commitment.
	•	Enforces required inequalities or set membership.
	•	Does NOT reveal the underlying raw attributes; only the satisfaction of constraints is encoded implicitly in the proof.
This aligns with TGP-00’s L9/L10 identity and policy layers.

⸻

Outputs

The circuit outputs:
	•	proof_bytes — backend-specific proof object (Groth16/Plonk/Halo2).
	•	proof_root — a compact hash or commitment of key public inputs (often H_bind), suitable for:
	•	embedding in TDR logs,
	•	embedding in PoS object under e.g. buyer_control_proof,
	•	verifying cross-system that a given settlement was bound to this proof instance.

Example usage in PoS (TGP-01):

“buyer_control_proof”: {
  “scheme”: “zkb-01”,
  “proof_root”: “0x1234…abcd”,
  “proof_bytes”: “0x…”,            // optional or off-chain
  “chain_id”: 369,
  “settlement_txid”: “0xabc…def”
}

Gateways MAY store only proof_root and off-load full proof_bytes to separate storage or on-demand verification systems.

⸻

Backend Support

The circuit SHOULD be implementable over:
	•	Groth16 — minimal proof size, requires trusted setup per circuit.
	•	Plonk — universal setup, better for evolving logic.
	•	Halo2 — recursive proof-friendly, suitable for aggregating buyer proofs with other settlement proofs.

The proof MUST be verifiable in a deterministic, constant-time fashion by gateways that only know:
	•	the settlement tx,
	•	the Economic Envelope hash,
	•	the policyRoot and policyHash,
	•	the TGP path and session id.

⸻

Summary

ZKB-01 provides a buyer control proof that:
	•	hides the buyer’s wallet address,
	•	proves ownership of the credited settlement output,
	•	binds that ownership to:
	•	a specific settlement transaction,
	•	a specific Economic Envelope,
	•	a specific policyRoot / TAI path,
	•	a specific TGP session,
	•	and can be carried as part of the Proof-of-Settlement and logged into TDRs.

It is compatible by design with the TGP-00, TGP-TAI, and TGP-01 (Economic Envelope + Prove Protocol Logic) specifications.
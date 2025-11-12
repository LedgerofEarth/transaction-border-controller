Appendix D – ZKS-01: ZK Seller-Delivery Proof Circuit (Draft)

**Purpose:**  
Allow the seller to prove, in zero-knowledge, that they **delivered value into the agreed escrow/swap contract** (or directly to the counterparty) while:

- optionally hiding their **primary wallet address** (using a deterministic but temporary address),  
- binding the delivery to:
  - a specific **settlement transaction** (L8),  
  - a specific **Economic Envelope** (EE, TGP-01),  
  - a specific **policy_root / TAI path** (TGP-TAI),  
  - a specific **TGP session/request**,  
- and producing a compact proof that can live inside the **Proof-of-Settlement (PoS)** object.

This circuit is mainly intended for **coordinated swap / escrow flows** where both buyer and seller transact through a **PrivateProver escrow contract** (or functionally similar settlement contract).

—

## Public Inputs

The verifier (gateway / TBC / escrow contract verifier) sees:

- `chain_id` — settlement chain identifier (e.g. `369` for PulseChain).  
- `settlement_txid` — hash of the **seller-side delivery transaction**, e.g. a call to a PrivateProver escrow contract or direct transfer.  
- `debited_input_commitment` — commitment representing the value debited from the seller’s control (e.g. commitment over `{asset_id, amount, blinded_seller_addr}`).  
- `amount` — delivered amount (smallest unit: wei, sats, etc.).  
- `asset_id` — asset identifier (e.g. `evm:ETH:1`, `evm:USDC:369`, NFT id).  
- `escrow_contract` — address or identifier of the PrivateProver escrow/swap contract used.  
- `swap_id` or `escrow_id` — unique identifier linking this delivery to a particular swap/settlement instance.  
- `economic_envelope_hash` — `keccak256(canonical-serialize(EE))` as per TGP-01.  
- `policy_root` — Merkle root of TA policy corpus (from TGP-TAI / TAO).  
- `policy_hash` — deterministic hash of evaluated policy (TGP-00 §12.3).  
- `tgp_path` — T-Path string, e.g. `tai:7abf92c6>tai:8bde4411`.  
- `session_id` or `request_id` — TGP identifier for this negotiation/settlement.

These public inputs make the proof specific to **this seller’s delivery into this escrow/swap for this session**, not something you can replay elsewhere.

—

## Private Inputs (Witness)

Held only by the seller:

- `sk_seller` — secret key or seed of the **seller’s delivery address** (which may be a temporary, deterministic child of a primary wallet).  
- `addr_seller` — derived address that actually sent funds in `settlement_txid`.  
- `merkle_path` or `receipt_witness` — path/witness showing that the debited input or “from” field in `settlement_txid` corresponds to `addr_seller` and `amount`/`asset_id`.  
- `salt` — randomness for blinding `addr_seller` inside the commitment.  
- `credential_secrets?` — optional secrets for policy-driven constraints (e.g., “seller is licensed provider X”), if required.

—

## Constraints

The circuit MUST enforce:

### 1. Seller Key-to-Address Binding

```text
addr_seller = f(sk_seller)

Where f is the appropriate address derivation for the settlement chain (e.g. EVM public-key → address mapping).
	•	Either recompute addr_seller from sk_seller inside the circuit,
	•	Or verify a signature proving control of addr_seller and treat the address as a witness, depending on performance constraints.

2. Debited Value Ownership & Delivery

Prove that value actually left a seller-controlled address and went where it was supposed to go (escrow or buyer):
	•	Use merkle_path / receipt_witness to verify that settlement_txid contains:
	•	an input, log, or event debiting amount of asset_id from addr_seller, and
	•	a corresponding transfer to:
	•	escrow_contract (for swap/escrow flows), or
	•	a specific buyer / destination address for direct delivery flows.
	•	The circuit enforces that this debit is committed in the public debited_input_commitment:

debited_input_commitment = H(asset_id || amount || blind(addr_seller, salt))


	•	The circuit also checks that escrow_contract and swap_id (if provided) match what appears in the transaction’s call data or log fields, tying this debit to the correct escrow instance.

3. Policy & Envelope Binding

Bind the proof to the same policy + envelope context as the buyer proof:

H_policy = H(policy_root || policy_hash || tgp_path)
H_env    = economic_envelope_hash

Then bind them with the session identifier:

H_bind = H(H_policy || H_env || session_id || swap_id)

The circuit MUST:
	•	Treat policy_root, policy_hash, tgp_path, economic_envelope_hash, session_id, and swap_id as public inputs.
	•	Constrain them into H_bind so the proof is cryptographically tied to the exact same policy + envelope + session + swap as the rest of the settlement.

(Full policy evaluation may be done outside the circuit; the circuit just binds to the policy_hash and policy_root used by TGP.)

4. Session / Request Binding

session_id (or request_id) MUST be included as a public input and used in the binding hash (H_bind above).

This prevents the same delivery proof from being re-attached to a different TGP session or Economic Envelope.

5. Optional Credential Constraints

If seller-side policy requires extra conditions (e.g., “seller is a registered merchant”, “seller is part of a whitelisted TA cohort”):
	•	The circuit consumes credential_secrets as witness,
	•	Verifies issuer signature / MAC on a credential commitment,
	•	Checks required predicates (e.g. attribute in allowed set, license not expired),
	•	Without revealing the underlying secret attributes.

This plugs into TGP-00’s L9 identity / L10 policy story.

⸻

Outputs

The circuit outputs:
	•	proof_bytes — backend-specific ZK proof (Groth16/Plonk/Halo2).
	•	proof_root — a hash/commitment over the key public inputs (typically H_bind), used as a durable handle.

Example embedding in PoS (TGP-01):

“seller_delivery_proof”: {
  “scheme”: “zks-01”,
  “proof_root”: “0xabcd…1234”,
  “proof_bytes”: “0x…”,        // optional or external storage
  “chain_id”: 369,
  “settlement_txid”: “0xabc…def”,
  “escrow_contract”: “0xEscrow…”,
  “swap_id”: “swap-77f91c”
}

Gateways MAY store only proof_root and a reference to where proof_bytes can be fetched for deep audit.

⸻

Backend Support

As with ZKB-01, the circuit SHOULD be implementable on:
	•	Groth16 — smallest proofs, trusted setup per circuit.
	•	Plonk — universal setup, easier to evolve the circuit as policy logic grows.
	•	Halo2 — friendly to recursive aggregation (e.g., aggregating ZKB-01 + ZKS-01 + Prove Protocol Logic proofs into one).

Verification MUST be deterministic and constant-time given:
	•	settlement_txid,
	•	debited_input_commitment,
	•	economic_envelope_hash,
	•	policy_root, policy_hash,
	•	tgp_path, session_id, swap_id.

⸻

Summary

ZKS-01 is the seller-side mirror to ZKB-01 for escrow/swap flows:
	•	Proves the seller actually delivered value out of their control into the agreed escrow/swap or destination.
	•	Optionally hides the seller’s primary wallet behind a derived, temporary address.
	•	Binds that delivery to:
	•	a specific settlement transaction,
	•	the agreed Economic Envelope,
	•	the TA policyRoot and T-Path,
	•	a specific TGP session and swap ID.
	•	Produces a compact commitment (proof_root) suitable for PoS and TDR logging.

In “plain” payment flows where the seller’s address must be known upfront and no escrow is used, ZKS-01 is generally not necessary—but for PrivateProver escrow contracts and coordinated swaps, it becomes the symmetric ZK guarantee to the buyer-side ZKB-01 proof.
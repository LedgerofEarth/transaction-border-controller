# TGP-01: Proof-of-Settlement and Economic Envelope Extension.

**Spec Family:** TGP-00  
**Version:** 0.1-draft  
**Status:** Draft  
**Date:** 2025-11-08  
**Maintainer:** Ledger of Earth  
**Contact:** tgp@ledgerofearth.org  

—

## 1. Purpose and Scope

TGP-01 extends **TGP-00** to define:

- **Proof-of-Settlement (PoS)** — verifiable settlement receipts bound to on-chain transactions.  
- **Economic Envelope (EE)** — a deterministic structure describing price, fees, and policy bindings.  
- **Prove Protocol Logic (PPL)** — an optional, transparent proof-of-action process allowing atomic swaps with seller set protocol fees.  
- Validation rules linking **Transaction Areas (TAIs)**, **policyRoots**, and **economic behavior** for compliant routing.

This specification is **chain-neutral** and **non-custodial**.  Gateways never hold user funds; all proofs derive from cryptographic and ledger events.

—

## 2. Normative References

- **TGP-00** — Transaction Gateway Protocol Core  
- **TGP-TAI** — Appendix A (Transaction Areas & Paths)  
- **x402 Signaling Spec** — Ledger of Earth  
- **EIP-3009 / EIP-2612** — Permit-based transfer standards  

—

## 3. Terminology

| Term | Meaning |
|——|-———|
| EE | Economic Envelope |
| PPL | Prove Protocol Logic |
| PoS | Proof of Settlement |
| TAI | Transaction Area Identifier |
| TG | Transaction Gateway |

—

## 4. Economic Envelope (EE) — Normative

### 4.1 Schema

```json
{
  “ee_version”: “1.0”,
  “price”: {
    “amount”: “10000.00”,
    “asset”: “evm:ETH:1”,
    “payer”: “buyer”,
    “payee”: “seller”
  },
  “buyer_fee”: {
    “amount”: “50.00”,
    “asset”: “evm:ETH:1”,
    “payer”: “buyer”,
    “payee”: “seller”,
    “protocol_fee”: {
      “ratio”: 0.20,
      “dest”: “tgp://proverouter”,
      “mode”: “auto_prove_burn”,
      “asset”: “prove”
    }
  },
  “routing_fee”: {
    “amount”: “10.00”,
    “asset”: “evm:ETH:1”,
    “payer”: “buyer”,
    “payee”: “tbc_operator”,
    “policy_ref”: “policy://carrierA/us-ca/routing-fee-v1”
  },
  “timestamp”: “2025-11-08T12:00:00Z”,
  “policy_root”: “0x9341…aa23”,
  “sig”: “ed25519(signature over canonical JSON)”
}

4.2 Rules
	•	price MUST exist.
	•	buyer_fee and routing_fee MAY exist.
	•	protocol_fee.dest identifies a Prove Router endpoint (TAI or URI).
	•	protocol_fee.mode=“auto_prove_burn” signals activation of Prove Protocol Logic.
	•	Monetary values MUST be canonical string decimals.

⸻

5. Proof-of-Settlement (PoS) — Normative

{
  “pos_version”: “1.0”,
  “request_id”: “req-123”,
  “settlement_txid”: “0xabc…def”,
  “settlement_chain”: “pulsechain”,
  “tgp_path”: “tai:7abf92c6>tai:8bde4411”,
  “economic_envelope_hash”: “0x9fbc…e201”,
  “receipt_hash”: “0x7a2d…”,
  “prove_proof”: {
    “txid”: “0xdead…beef”,
    “amount”: “1.234”,
    “asset”: “prove”,
    “mode”: “auto_prove_burn”
  },
  “tdr_hash”: “sha256-of-tdr”,
  “signature”: “ed25519(sig)”
}

	•	prove_proof MUST appear if the EE includes protocol_fee.mode=auto_prove_burn.
	•	The proof MAY originate on a different chain but MUST include a verifiable tx hash and block height.
	•	receipt_hash binds to the L8/L9/L10 triplet from TGP-00 §14.

⸻

6. Prove Protocol Logic (PPL)

6.1 Purpose

Implements a transparent, automated proof event (burn or lock) for protocol fees defined in Economic Envelopes.

6.2 Operation
	1.	Gateway computes
prove_amount = buyer_fee.amount × protocol_fee.ratio.
	2.	Gateway or designated Prove Router swaps fee asset → prove asset.
	3.	The prove asset is irreversibly burned (e.g., sent to 0x000…dead).
	4.	Resulting tx hash + amount + height logged into PoS and TDR.
	5.	prove_proof returned in SETTLE or PROOF message.

6.3 Compliance & Limits
	•	TAs MAY cap protocol_fee.ratio via policyRoot.
	•	Prove Routers MUST publish on-chain proofs.
	•	Gateways MUST NOT custody user funds; all actions occur via designated protocol accounts.

⸻

7. Integration with TGP-00 Messages

Message	Added Field	Description
QUERY	economic_envelope	Optional initial price/fee intent
ADVERT	economic_envelope	Advertised fee capabilities
SELECT	economic_envelope	Negotiated final EE
SETTLE	proof_of_settlement	Attached PoS object with prove_proof

economicEnvelopeHash = keccak256(canonical-serialize(EE))
and SHOULD be propagated in the x402 tgpContext block.

⸻

8. TAI and Economic Policy Alignment
	•	Each TA policyRoot MAY include fee_constraints (max routing bps, max protocol ratio, permitted assets).
	•	Gateways MUST reject Economic Envelopes that violate TA constraints.
	•	The TAI of a Prove Router MUST be publicly advertised and auditable.

⸻

9. Security Considerations
	•	Replay: PoS objects bound to unique request_id and economicEnvelopeHash.
	•	Integrity: EE and PoS MUST be signed by initiator or gateway.
	•	Transparency: All prove transactions are on-chain and auditable.
	•	Privacy: EE amounts MAY be encrypted with HPKE and revealed only to authorized participants.

⸻

10. Example Flow with Prove Protocol Logic

1. Buyer → TG-A: QUERY (EE: price + buyer_fee + protocol_fee 20%)
2. TG-A ↔ TG-B: ADVERT (includes TAI, policyRoot, fee limits)
3. Buyer SELECT path; TG-B LOCKED
4. Provider delivers service via x402
5. TG-B executes prove-burn of 0.2 × buyer_fee = 1 prove token
6. TG-B emits PoS with prove_proof.txid → chain registry
7. TG-A and Buyer verify burn via receipt and PoS


⸻

11. TDR Extensions

TDR records MUST include:

ee_hash,pos_hash,prove_txid,prove_amount,prove_asset,protocol_fee_ratio

These fields enable end-to-end auditing of economic flows without custodial risk.

⸻

12. Revision History

Version	Date	Change
0.1-draft	2025-11-08	Initial draft introducing Economic Envelope, Proof-of-Settlement, and Prove Protocol Logic aligned with TAIs and policyRoots.


⸻

13. Informative Note

This extension generalizes EIP-1559-style fee burns as verifiable proofs of activity within TGP-routed transactions.
The Prove Protocol Logic demonstrates economic finality without requiring trust or custody, linking value flow to auditable proofs across Transaction Areas.
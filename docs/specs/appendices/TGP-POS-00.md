# TGP-01: Proof-of-Settlement (PoS) Extensions
**Status:** Draft  
**Depends on:** TGP-00 (base), TxIP-00 (transport), TDR-00 (logging)  
**Non-goals:** Introducing a token, custody, or chain-specific constraints.

## 1. Motivation
Enable tokenless, cross-domain settlement using verifiable proofs (fiat + crypto) with optional buyer-address privacy via zero-knowledge proofs. This lets TBCs replace exchange/escrow trust while keeping compliance and policy enforcement at the edge.

## 2. Roles
- **Buyer Agent (BA)** – creates fiat payment & ZK buyer-control proof.
- **Seller Agent (SA)** – executes crypto transfer, supplies on-chain proof.
- **PrivateProver (PP)** – validates proofs, emits Proof-of-Settlement.
- **TBC** – policy gate, routing, fee metering, TDR anchoring, optional on-chain verifier call.

## 3. Capability Discovery
JSON example:
```json
{
  “tgp_caps”: {
    “pos_v1”: true,
    “pos_zk_buyer_privacy”: [“groth16”,”plonk”,”halo2”],
    “pos_fiat_attestors”: [“bank_api_v2”,”open_banking_uk”,”plaid_vX”],
    “pos_chain_attestors”: [“evm_tx_receipt”,”light_client_merkle”,”oracle_sig”]
  }
}
```

## 4. Message Types
Includes PROOF_INIT, PROOF_PART, PROOF_VERIFY, PROOF_ACK.

Refer to the detailed examples in the main conversation.

## 5. ZK Buyer-Privacy Claim
Proves control of an address without revealing it. Uses circuits defined in ZKB-01.

## 6. Fiat and Crypto Proof Claims
Defines proof formats and attestor expectations.

## 7. Policy & Compliance Hooks
policy_ref indicates rule set; TBC applies policy before accepting proofs.


# TGP-01: Economic Envelope and Fee Flow (RFC-Style Specification)

## 7.1 Economic Envelope and Fee Flow

### 7.1 Overview

The Transaction Gateway Protocol (TGP) defines an optional **Economic Envelope** that encapsulates the monetary and fee-related metadata for cross-domain settlements.  
The envelope allows endpoints to negotiate pricing and fees while retaining non-custodial settlement semantics.  
Three distinct value components are supported:

| Field | Description | Notes |
|-——|—————|-——|
| **P** | *Price of goods or services* | Peer-negotiated amount transferred end-to-end. |
| **Bf** | *Buyer Fee* | Seller-defined fee paid by buyer; may contain Protocol Fee sub-field (`Fp`). |
| **Fp** | *Protocol Fee* | Deterministic fraction of `Bf` diverted to the protocol BurnRouter for MRPROVE buy/burn. |
| **Rf** | *Routing Service Fee* | Payment to TBC operators for policy enforcement and session routing. |

Implementations MUST treat these values as **metadata** only and MUST NOT take custody of user assets.  
Settlement MUST remain peer-to-peer.

—

### 7.2 Flow Diagram

```text
Buyer ──► Seller
  │          ▲
  │          │
  ├─ P  (price of good/service)
  ├─ Bf (buyer fee → seller)
  │     └─ Fp = α·Bf  → BurnRouter (buy+burn)
  └─ Rf (routing service fee → TBC operator)

	1.	P flows directly from buyer to seller.
	2.	Bf is paid by buyer to seller; protocol logic MUST divert Fp = α × Bf to the BurnRouter contract.
	3.	Rf is a separate network-service payment to TBC operators.
	4.	All fees MUST be visible to both endpoints and MUST be immutable once signed within the session.

⸻

7.3 Economic Envelope Header Format

Each TGP frame MAY include an economic_envelope object encoded as structured JSON.
Routers and gateways MUST forward the object unchanged if they do not process fees.

{
  “tgp_version”: “1.0”,
  “tx_id”: “uuid-or-hash”,
  “domain”: “tgp://carrierA/us-ca”,
  “economic_envelope”: {
    “price”: {
      “amount”: “1000000000000000000”,
      “asset”: “evm:ETH:1”,
      “payer”: “buyer”,
      “payee”: “seller”
    },
    “buyer_fee”: {
      “amount”: “5000000000000000”,
      “asset”: “evm:ETH:1”,
      “payer”: “buyer”,
      “payee”: “seller”,
      “protocol_fee”: {
        “ratio”: “0.2”,
        “dest”: “tgp://burnrouter”,
        “mode”: “auto_buy_burn”
      }
    },
    “routing_fee”: {
      “amount”: “1000000000000000”,
      “asset”: “evm:ETH:1”,
      “payer”: “buyer”,
      “payee”: “tbc_operator”,
      “policy_ref”: “policy://carrierA/us-ca/routing-fee-v1”
    }
  }
}

Field Requirements

Field	Requirement
price	MUST specify amount, asset, payer, and payee. Used for policy checks only.
buyer_fee	MUST be explicitly set by the seller and MUST be visible to both parties.
protocol_fee	MUST define ratio or amount. The protocol MUST route this value to the BurnRouter endpoint designated by dest.
routing_fee	MUST identify the TBC operator receiving the fee. Routers MAY append policy_ref for jurisdictional context.

Routers MUST verify digital signatures covering the entire economic_envelope before executing settlement logic.

⸻

7.4 BurnRouter Interaction (Informative)

The BurnRouter contract MUST implement the following behavior:
	1.	Receive Fp in base asset from settlement contract.
	2.	Swap Fp for MRPROVE tokens using a pre-configured DEX route.
	3.	Send the purchased MRPROVE to a designated burn address (e.g., 0x0000…dead).
	4.	Emit BurnExecuted(amount, tx_id, origin_domain, timestamp) for auditability.

Implementations SHOULD batch swaps to minimize gas and MEV exposure.
Governance parameters (e.g., α ratio) SHOULD be time-locked and MUST NOT be mutable per transaction.

⸻

7.5 Integration with x402 and TxIP
	•	x402 Layer: The economic envelope MAY be embedded in the payment_terms field for application-level visibility.
	•	TxIP Layer: TBCs MUST include the envelope within session_params.economic_envelope.
	•	TGP Layer: Routers MUST read and validate the envelope for fee routing and policy enforcement.

Example frame:

{
  “type”: “TGP_FRAME”,
  “msg_type”: “PROOF_INIT”,
  “tx_id”: “123e4567-e89b-12d3-a456-426614174000”,
  “economic_envelope”: { /* defined above */ },
  “claims”: [
    “fiat.payment.sent”,
    “crypto.transfer.executed”,
    “buyer.wallet.controlled(zk)”
  ],
  “policy_ref”: “policy://carrierA/us-ca/standard-v1”
}


⸻

7.6 Security and Compliance Considerations
	•	Implementations MUST ensure that economic_envelope data is cryptographically bound to the session signature set.
	•	Routers and TBC operators MUST log all fee processing events for audit.
	•	The protocol MUST NOT pool user funds or redistribute profits; all fee handling is deterministic and transparent.
	•	Regulated operators MAY apply local compliance policies via policy_ref.

⸻

7.7 Normative Keywords

The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHOULD”, “SHOULD NOT”, and “MAY” in this document are to be interpreted as described in [RFC 2119].


## 8. TDR Anchoring
Each PoS produces a TDR line, hashed and optionally on-chain anchored.

## 9. Batching (“zkp2p roll-up”)
Allows the TBC to roll up N PoS results and post batch anchors.

## 10. Backward Compatibility
Agents without pos_v1 simply ignore the PROOF_* frames.

## 11. Security
Session-bound proofs, attestor thresholds, and optional mTLS.
This spec will plug directly into:
	•	TGP-00 v3.2
	•	CoreProve-ZK-01
	•	CoreProve-Settlement v0.2.1
	•	ReceiptVault 2025–26

It defines the entire message shape for browser extensions that submit ZK proofs to the TBC.

You will need this for:
	•	MCP agents
	•	extension implementation
	•	merchant portal compatibility
	•	interop testing
	•	TBC validation logic
	•	settlement path instrumentation

This is the missing piece to begin wiring everything together.

⸻

📘 TGP-EXT-ZK-00 — Zero-Knowledge Proof Envelope Specification

Version: 0.1

Status: Draft (Internal)

Applies To: CoreProve-00, TGP-00 v3.2, TGP-CP-00, CoreProve-ZK-01

⸻

0. Purpose

This document defines the ZK proof envelope used by browser extensions and agents to submit cryptographic proofs through the Transaction Gateway Protocol (TGP), enabling buyers and sellers to transact privately through CoreProve’s dual-escrow settlement model.

The TGP-EXT ZK envelope:
	•	standardizes how ZK proofs are encoded
	•	defines which public inputs must be present
	•	separates raw proofs from rewritten settlement-safe structures
	•	ensures compatibility with the TBC verification layer
	•	ensures safe passage to CoreProve settlement contracts

This spec defines message shape, not circuit math.

⸻

1. Architectural Position

ZK proofs move ONE TIME ONLY from:

Extension → TGP-EXT → TBC → (ZK Verify) → Settlement Instruction → Contract

Correct flow:
	1.	EXTENSION generates ZK proof.
	2.	TGP-EXT wraps proof into standard envelope.
	3.	TBC verifies SNARK off-chain.
	4.	TBC extracts public inputs.
	5.	TBC rewrites proof into contract-safe ZK structs.
	6.	SettlementContract receives no proof, only outputs.

⸻

2. ZK Message Types

There are three types of ZK proofs:

zk_type	From	Description
ZKB01	Buyer → TBC	Proves deposit + session link
ZKS01	Seller → TBC	Proves fulfillment auth
ZKM01	Merchant → TBC	Proves policy integrity

All three share a common envelope, but have different fields in zk_inputs.

⸻

3. Base Envelope Structure

This is the raw message the extension sends through TGP-EXT:

{
  “tgp_message_type”: “TGP_ZK_PROOF”,
  “zk_type”: “ZKB01 | ZKS01 | ZKM01”,
  “zk_proof”: “base64url(<proof_bytes>)”,
  “zk_inputs”: {
    “...”: “type-specific”
  },
  “zk_nullifier”: “0x<32-byte>”,
  “zk_timestamp”: 1698012337,
  “session_pubkey”: “0x<compressed_pubkey>”,
  “device_commitment”: “0x<hash>”,
  “proof_version”: “1”,
  “session_id”: “0x<32-byte>”,
  “order_id”: “0x<32-byte>”,
  “profile_hash”: “0x<32-byte>”,
  “chain_id”: 369
}

Required fields:

Field	Purpose
zk_proof	Raw SNARK proof, never goes on-chain
zk_inputs	Public inputs binding session, pkHash, amount, timestamp
zk_nullifier	Prevents replay of proof
zk_timestamp	Enforces buyer/seller TTL
session_pubkey	Identifies ephemeral session
device_commitment	Anti-theft, anti-malware
proof_version	Enables circuit upgrades
session_id	Required for dual commitment
chain_id	Required for deterministic proving


⸻

4. Type-Specific zk_inputs Schemas

4.1 ZKB-01 — Buyer Deposit Proof

Buyer proves:
	•	session wallet ownership
	•	buyer→root linkage
	•	deposit correctness
	•	nullifier freshness
	•	timestamp freshness

Schema:

“zk_inputs”: {
  “escrow_address”: “0x<20-byte>”,
  “amount”: “string (uint256)”,
  “pk_hash”: “0x<32-byte>”,
  “nullifier”: “0x<32-byte>”,
  “timestamp”: “string (uint256)”,
  “session_pubkey”: “0x<33-byte>”,
  “deposit_tx_hash”: “0x<32-byte>”,
  “chain_id”: 369
}

Settlement contract receives only:

pkHash, nullifier, timestamp, amount


⸻

4.2 ZKS-01 — Seller Fulfillment Proof

Seller proves:
	•	merchant key ownership
	•	fulfillment authorization
	•	nullifier freshness
	•	timestamp freshness

Schema:

“zk_inputs”: {
  “order_hash”: “0x<32-byte>”,
  “pk_hash”: “0x<32-byte>”,
  “nullifier”: “0x<32-byte>”,
  “timestamp”: “string (uint256)”,
  “session_pubkey”: “0x<33-byte>”,
  “chain_id”: 369
}

Settlement contract receives:

pkHash, nullifier, timestamp, orderHash


⸻

4.3 ZKM-01 — Merchant Policy Integrity Proof

(TBC-only: never reaches contract)

Merchant proves:
	•	correctness of policy bytecode
	•	correctness of policy_hash
	•	correctness of chain_id
	•	optional merchant key auth

Schema:

“zk_inputs”: {
  “policy_address”: “0x<20-byte>”,
  “policy_hash”: “0x<32-byte>”,
  “bytecode_hash”: “0x<32-byte>”,
  “timestamp”: “string (uint256)”,
  “nullifier”: “0x<32-byte>”,
  “chain_id”: 369
}

This is consumed by the TBC only.

⸻

5. Unified TGP-EXT Message

This is the full TGP frame:

{
  “type”: “TGP_ZK_PROOF”,
  “payload”: {
    “zk_type”: “ZKB01 | ZKS01 | ZKM01”,
    “zk_proof”: “base64url(...)”,
    “zk_inputs”: { ... },
    “zk_nullifier”: “0x...”,
    “zk_timestamp”: 1698012337,
    “session_pubkey”: “0x...”,
    “device_commitment”: “0x...”,
    “proof_version”: 1,
    “session_id”: “0x...”,
    “order_id”: “0x...”,
    “profile_hash”: “0x...”,
    “chain_id”: 369
  }
}


⸻

6. TBC Rewrite Rules (Critical)

After ZK verification, the TBC rewrites the message into the contract-safe form:

{
  “buyer”: {
    “pkHash”: “0x...”,
    “nullifier”: “0x...”,
    “timestamp”: “...”,
    “amount”: “...”
  },
  “seller”: {
    “pkHash”: “0x...”,
    “nullifier”: “0x...”,
    “timestamp”: “...”,
    “orderHash”: “0x...”
  }
}

No proof is forwarded.
No witness is forwarded.
No private data is forwarded.

This is what the SettlementContract receives.

⸻

7. Error Conditions

Condition	Error
invalid proof	ZK_INVALID_PROOF
timestamp expired	ZK_EXPIRED_PROOF
nullifier reused	ZK_REPLAY
pkHash mismatch	ZK_PK_MISMATCH
wrong circuit version	ZK_UNSUPPORTED_VERSION


⸻

8. Security Notes
	•	Full proofs never hit chain
	•	Nullifiers are mandatory
	•	Timestamp binding enforces TTL
	•	Session_pubkey prevents “replay by malware”
	•	Proof_version prevents old circuit reuse
# CoreProver-ZK-00  
### Zero-Knowledge Proof Architecture for CoreProver Settlement  
**Version:** 0.1-draft  
**Status:** Informational  
**Authors:** David Bigge, Shannon Jolin  
**Company:** Ledger of Earth, LLC  
**Applies to:** TBC-00, TGP-00, TBC-SEC-00, CoreProver-Escrow-00

—

# 0. Purpose

This document defines the **Zero-Knowledge (ZK) proof architecture** for the
CoreProver settlement system.  
It describes:

- what must be proven via SNARKs,  
- where proofs are verified,  
- how proofs integrate with TBC routing and policy flow,  
- how proofs interact with CoreProver escrow contracts,  
- how proof hashes are stored in Receipt NFTs, and  
- how full proofs are archived for long-term compliance.

This document **does not** define the underlying circuit math.  
It defines the **statements** that must be proven and the **interfaces** required
for verification.

—

# 1. Architectural Positioning

SNARKs exist **between**:

- TBC-SEC-00 (security & policy decision tree)  
and  
- CoreProver-Escrow-00 (on-chain settlement)

The ZK layer performs **post-security, pre-settlement** verification.

TBC-00 → TGP-00 → TBC-SEC-00 → CoreProver-ZK-00 → CoreProver-Escrow-00

This preserves:

- layered architecture  
- TBC routing independence  
- minimal coupling to cryptographic systems  
- optional merchant-level ZK requirements

—

# 2. Goals

ZK proofs in CoreProver must provide:

### 2.1 Privacy  
Users transact using session wallets without exposing root wallets.

### 2.2 Integrity  
Merchants and sellers cannot falsify fulfillment or contract data.

### 2.3 Auditability  
Receipt NFTs must remain verifiable decades into the future.

### 2.4 Flexibility  
Merchants choose when ZK proofs are required.

### 2.5 Cost-Efficiency  
SNARK verification occurs off-chain in the TBC whenever possible.

—

# 3. Proof Types

CoreProver uses **three** SNARK-based proofs.

| Proof | Name | Actor | Purpose |
|——|——|-——|———|
| ZKB-01 | Buyer Deposit Proof | Buyer | Proves deposit + session→root linkage |
| ZKS-01 | Seller Fulfillment Proof | Seller | Proves fulfillment or counter-escrow |
| ZKM-01 | Merchant Policy Integrity Proof | Merchant | Proves correctness of contract & policy |

All proofs are **statement-level primitives**.  
Their internal circuit structure is out-of-scope for this document.

—

# 4. Merchant Policy Flags

ZK requirements are configured per merchant within the Payment Profile:

requireZKBuyerDeposit: true | false
requireZKSellerFulfillment: true | false
requireZKPolicyIntegrity: true | false

Defaults:

- Buyer Deposit Proof: OPTIONAL  
- Seller Fulfillment Proof: OPTIONAL  
- Policy Integrity Proof: OPTIONAL

Merchants operating under higher security, compliance, or luxury-goods
conditions may require any or all of these proofs.

—

# 5. Verification Model

## 5.1 TBC-First Verification (Fast Path)

All SNARK proofs are verified **off-chain** by the TBC whenever possible.

Reasons:

- fast response  
- low cost  
- reduced gas burden  
- no chain congestion  
- flexible circuit upgrade path

## 5.2 Smart-Contract Verification (Fallback Path)

If required by merchant policy or audit conditions:

- CoreProver escrow contracts can verify the SNARK on-chain  
- only Groth16 verifiers MUST exist on-chain  
- PLONK/other circuits MAY be accepted via external verifier contracts

—

# 6. Proof Delivery Format

Proofs are delivered inside TGP messages via:

zk_proof: base64url(SNARK proof bytes)
zk_inputs: array of public inputs
zk_type: ZKB01 | ZKS01 | ZKM01
zk_vk_id: verifier key identifier

The TBC extracts, verifies, and attaches settlement-level proof metadata.

—

# 7. Proof Statements (Informal)

Below are the required statements.  
Circuit math is out-of-scope; statements must be provable by a SNARK.

—

## 7.1 ZKB-01 — Buyer Deposit Proof

The buyer must prove:

1. A deposit of `<amount>` was made to the escrow contract.  
2. The deposit originated from a **session wallet** whose private key is known.  
3. The session wallet is **cryptographically linked** to a buyer **root wallet**  
   (ZK-NAT linkage).  
4. No identifying data is revealed in the process.

**Public Inputs:**

- escrow_address  
- amount  
- session_pubkey  
- deposit_tx_hash  
- chain_id  

**Private Witness:**

- root_privkey  
- session_privkey  
- linkage_nonce  
- deposit_signature  

—

## 7.2 ZKS-01 — Seller Fulfillment Proof

Two fulfillment paths exist:

### A. Seller Signature
Prove knowledge of seller private key used to sign fulfillment hash.

### B. Counter-Escrow (optional)
Prove posting of counter-escrow as required by merchant profile.

**Public Inputs:**

- fulfillment_hash  
- seller_pubkey  
- order_id  

**Private Witness:**

- seller_privkey  
- counter_escrow_witness (if used)

—

## 7.3 ZKM-01 — Merchant Policy Integrity Proof

Merchant must prove:

1. The policy contract address is correct.  
2. The contract bytecode matches the advertised `policy_hash`.  
3. The chain ID matches the configured settlement chain.  
4. No tampering occurred.

**Public Inputs:**

- policy_contract_address  
- policy_hash  
- chain_id  

**Private Witness:**

- merchant_privkey (optional)  
- contract_bytecode_hash (if required)

—

# 8. Receipt NFT Integration

Receipt NFTs MUST store:

buyer_proof_hash
seller_proof_hash
merchant_proof_hash
public_inputs_hash
fulfillment_profile_id
policy_hash

NFTs MUST NOT store:

- full proofs  
- private witness elements  
- signature material  
- circuit structure  
- proving keys

Hashing algorithm:

proof_hash = SHA256(zk_proof_bytes)

—

# 9. Proof Archival Model

CoreProver uses a **hybrid archival model**:

### 9.1 On-chain (Receipt NFT)
Stores only **hashes** of proofs and public inputs.

### 9.2 Receipt Vault (optional)
Stores **full proofs** for merchants requiring compliance retention.

Vault storage MAY include:

- IPFS  
- merchant-hosted storage  
- CoreProver cluster storage  
- end-user local storage (browser extension)  

### 9.3 Deterministic Re-Generation
Any proof MAY be recreated using:

- NFT’s public_inputs  
- buyer/seller/merchant retained witness  
- same circuit version

This ensures multi-decade audit resilience even if archives disappear.

—

# 10. Circuit Systems

CoreProver supports a **hybrid circuit strategy**:

### 10.1 Groth16 (Primary)
- cheap verification on PulseChain  
- small proofs  
- required for on-chain verifiers

### 10.2 PLONK (Optional)
- universal setup  
- flexible circuit upgrades  
- off-chain verifier only (for now)

Future versions MAY introduce recursive verification.

—

# 11. Integration with TBC-SEC-00

ZK proofs are validated **after** TBC security layers approve the session.

Flow:

1. TBC-SEC approves session  
2. ZK proofs are evaluated  
3. TBC emits settlement instruction  
4. Escrow contract processes withdrawal/deposit  

Failure to validate ZK proofs MUST result in:

REJECT: ZK_INVALID_PROOF

—

# 12. Integration with CoreProver-Escrow-00

Escrow contract MUST accept proof arguments:

deposit(amount, session_pubkey, zk_proof, zk_inputs)
withdraw(order_id, fulfillment_hash, zk_proof_seller, zk_proof_merchant)

Contracts MUST:

- verify Groth16 proofs when required  
- forward events to Receipt Vault  
- mint Receipt NFTs after settlement  

—

# 13. Security Considerations

- Proof hashes MUST be collision-resistant  
- Witness data MUST never leave buyer/seller/merchant domain  
- TBC MUST sandbox verifier failures  
- NFT metadata MUST NOT reveal private data  
- Loss of off-chain proof archives MUST NOT break verification  
- Session-wallet usage MUST always be encouraged over root wallets  

—

# Appendix A — Diagrams

## A.1 ZK Layer Position in Architecture

TBC-00
|
TGP-00
|
TBC-SEC-00
|
CoreProver-ZK-00   <— this document
|
CoreProver-Escrow-00
|
Receipt Vault / NFTs

## A.2 High-Level ZK Flow

User → TBC → ZK-Verify → TGP → CoreProver → Receipt NFT

## A.3 Proof Lifecycle

       Buyer                 Seller                 Merchant
         |                      |                       |
     ZKB-01                  ZKS-01                  ZKM-01
         |                      |                       |
         +——————-———→ TBC Verifier ←——————————————-—————+
                            |
                      Settlement OK
                            |
               CoreProver Escrow Contract
                            |
                    Receipt NFT Minted
                            |
                Proof Hashes Stored On-Chain

—

# END OF FILE


1. QUERY (Extension → TBC)
   ├── TGP-00: QUERY message
   └── Merchant, amount, session info

2. ACK (TBC → Extension)  
   ├── TGP-00: ACK message
   └── allow/deny/revise + policy

3. ZK PROOF GENERATION (Extension)
   ├── TGP-EXT-ZK-00: Generate ZKB01
   └── Buyer deposit proof

4. TGP_ZK_PROOF (Extension → TBC)
   ├── TGP-EXT-ZK-00: Envelope
   └── zk_proof, zk_inputs, nullifier

5. TBC VERIFICATION
   ├── Verify SNARK off-chain
   ├── Check nullifier unused
   ├── Check timestamp fresh
   └── Rewrite to contract-safe form

6. BUYER_COMMIT (TBC → Chain)
   ├── CoreProve-00: buyerCommit()
   └── pkHash, nullifier, amount

7. SELLER_COMMIT (Merchant → Chain)
   ├── CoreProve-00: sellerCommit()
   └── pkHash, nullifier, orderHash

8. SETTLE (TBC → Extension)
   ├── TGP-00: SETTLE message
   └── final_status, escrow_id

9. RECEIPT (Chain → ReceiptVault)
   ├── CoreProve-00: mint()
   └── Wallet-unlinkable receipt
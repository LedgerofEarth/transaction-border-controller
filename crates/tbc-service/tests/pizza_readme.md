# 🍕 CoreProver v0.3 Pizza Delivery Simulation
A Demonstration of Tokenless Settlement, Triple Clocks, and State Transitions

---

## Purpose
The pizza delivery simulation is the canonical demonstration of CoreProver’s escrow engine and the Transaction Border Controller (TBC). It shows:

- Dual-commitment escrow  
- Timed windows (accept → fulfill → claim)  
- Triple-clock timestamping (mono/unix/iso)  
- Late delivery handling & automatic discount  
- Receipt generation  
- Fully verifiable settlement flows  
- Seller-claim and buyer-withdraw paths  
- Typed errors & fault injection  
- TBC-compatible settlement triggers  

---

## Actors
- **Buyer** – commits funds upfront  
- **Seller** – pizza shop  
- **Escrow contract** – chain-level enforcement mechanism  
- **CoreProver Engine** – off-chain verifier  
- **TBC** – orchestrator for QUERY/OFFER/SETTLE  

---

## Timelines & Windows

### Standard Pizza Profile
| Phase | Duration | State Change |
|-------|----------|--------------|
| Acceptance Window | 5 min | BuyerCommitted → SellerAccepted |
| Fulfillment Window | 60 min | SellerAccepted → SellerFulfilled or FulfillmentExpired |
| Claim Window | 24 hrs | SellerFulfilled → SellerClaimed |

Late discount applies if fulfillment occurs after 3600s.

---

## State Machine
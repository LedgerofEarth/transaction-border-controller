# CoreProve + TBC — 7‑Day Sprint Plan
### Version: 1.0  
### Author: David Bigge  
### Purpose: Product Accountability & Execution Timeline  
### Audience: Shannon (Product Lead)

—

## Overview  
This 7‑day plan defines the core deliverables needed to bring the CoreProve + TBC MVP to a full end‑to‑end testnet demonstration.  
Each day includes clear goals, deliverables, responsibilities, and risk buffers.

Shannon’s role:  
- External accountability  
- Daily check‑ins  
- Scope clarity  
- Decision unblocking  
- Timeline enforcement  

—

# 🟦 Day 1 — Delegation System Foundation

### **Core Goal:**  
Formalize the delegation system that enables the TBC to broadcast blockchain transactions on behalf of a user via a session wallet.

### **Deliverables:**  
- CoreProve-Delegation-00.md (v0.1 draft)  
- EIP‑712 typed delegation structure  
- Domain separator definitions  
- Session wallet lifecycle rules  
- Delegation signature verification logic (Rust + Solidity stubs)  
- TGP integration reference  

### **David:**  
- Validate structure  
- Run manual signing tests in MetaMask/Rabby  
- Approve final delegation schema  

### **AI Assistants:**  
- Generate the full spec  
- Produce code stubs for Rust + Solidity  
- Generate test vectors  

### **Risk Buffer:**  
Delegation is foundational; complexity spikes possible.

—

# 🟧 Day 2 — CoreProve Settlement Contract v0.1

### **Core Goal:**  
Deploy first working Solidity version of the CoreProve escrow system on testnet.

### **Deliverables:**  
- `deposit()` + `withdraw()` with argument validation  
- Gas buffer reconciliation logic  
- `merchantNet ≥ 0` enforcement  
- Receipt NFT mint stub  
- Event structures  
- Storage layout  
- Deployment addresses on testnet  

### **David:**  
- Deploy contracts  
- Verify events on block explorer  
- Integrate contract addresses into TBC config  

### **AI Assistants:**  
- Generate contract code + tests  
- Optimize storage packing  
- Generate receipt NFT minimal implementation  

### **Risk Buffer:**  
Smart contract toolchains can be volatile.

—

# 🟩 Day 3 — TBC Routing Engine (Multi‑Chain)

### **Core Goal:**  
Implement multi‑chain routing with Path 1–4 fully functional.

### **Deliverables:**  
- Parallel gas estimation with per‑chain timeouts  
- Gas reservation call  
- Path 1 chain scoring logic  
- Delegation verifier in Rust  
- TGP message builder for settlement  
- End‑to‑end mock flow (no contract calls yet)  

### **David:**  
- Configure RPC endpoints  
- Run integration tests  
- Verify TDR logs  

### **AI Assistants:**  
- Write Rust routing modules  
- Implement state transitions  
- Add detailed error reporting  

### **Risk Buffer:**  
RPC variability across chains.

—

# 🟨 Day 4 — CoreProve Browser Extension v0.2

### **Core Goal:**  
User can sign delegation, approve PaymentIntent, and forward to TBC.

### **Deliverables:**  
- Delegation signing popup  
- PaymentIntent approval popup  
- Chain selection UI  
- Session wallet generation  
- Storage of delegation signature  
- TBC API communication  

### **David:**  
- Build + archive extension in Xcode  
- Install and test popups  
- Validate signing flow end‑to‑end  

### **AI Assistants:**  
- Generate UI logic + manifest  
- Provide compatibility patches  
- Produce popup templates  

### **Risk Buffer:**  
Browser extension permissions and signing flow quirks.

—

# 🟪 Day 5 — Full Testnet Roundtrip

### **Core Goal:**  
Achieve the first full on‑chain CoreProve transaction through TBC.

### **Flow:**  
1. Merchant → PaymentIntent  
2. User → Delegation  
3. Extension → TBC  
4. TBC → Path selection  
5. Contract → `deposit()`  
6. Contract → `withdraw()`  
7. Receipt NFT minted  
8. Vault logs proof metadata  

### **Deliverables:**  
- Working settlement on testnet  
- Valid events in explorer  
- Receipt NFT visible  
- Vault hash log  

### **David:**  
- Record demo  
- Validate correctness  
- Save transaction artifacts  

### **AI Assistants:**  
- Patch integration errors  
- Reconcile calldata formats  
- Fix routing issues  

### **Risk Buffer:**  
ABI mismatches and calldata encoding bugs.

—

# 🟫 Day 6 — ZK Proof Integration (Stubs Flow)

### **Core Goal:**  
Set up ZK plumbing so proofs flow through the system.

### **Deliverables:**  
- ZKB‑01 circuit skeleton  
- ZKS‑01 circuit skeleton  
- ZKM‑01 circuit skeleton  
- Witness structures  
- Public input packing  
- Verifier contract stub  
- TBC proof input generation  

### **David:**  
- Install proving backends  
- Run sample circuit compile  
- Validate witness encoding  

### **AI Assistants:**  
- Generate circuits  
- Generate Solidity verifiers  
- Create Rust verification harness  

### **Risk Buffer:**  
ZK toolchains can break unexpectedly.

—

# 🟩 Day 7 — Merchant Portal + Multi‑Chain Demo

### **Core Goal:**  
Merchant configuration + live multi‑chain settlement demo.

### **Deliverables:**  
- Merchant fee + policy config UI  
- Path 1 multi-chain routing working live  
- End‑to‑end demo video  
- v0.5.3 spec update from discoveries  
- Final review with Shannon  

### **David:**  
- Run live demo  
- Clean up scripts  
- Prepare YC demo variant  

### **AI Assistants:**  
- Polish UI  
- Fix REST discrepancies  
- Add policy propagation  

### **Risk Buffer:**  
Cross‑chain timing delays, logging drift.

—

# 🧩 Summary

By the end of 7 days we will have:

- Fully operational delegation system  
- Working CoreProve escrow contract on testnet  
- TBC routing engine with multi-chain support  
- Browser extension v0.2 signing flow  
- Live end‑to‑end settlement  
- Receipt NFT minting  
- ZK proof flow working (stubbed)  
- Merchant config portal  
- Demo-ready multi-chain payment

Shannon’s job:  
- Daily check-ins  
- Red flag identification  
- Scope containment  
- Demo polish oversight  
- Founder morale stabilization (optional)

—

# END OF DOCUMENT

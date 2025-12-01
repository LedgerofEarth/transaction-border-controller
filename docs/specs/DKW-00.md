# DKW-00 v0.5 — Delegated Key Wallet Specification

> **Status**: Implemented (Wave-7.6)  
> **Last Updated**: December 1, 2025  
> **Depends On**: TGP-00 v3.2, TGP-EXT-00 v1.1, EIP-712

---

## 1. Overview

The Delegated Key Wallet (DKW) specification defines how the CoreProve Extension obtains temporary signing authority from the user's external wallet. This enables the extension to:

1. **Sign TGP messages** on behalf of the user
2. **Submit delegated transactions** through the TBC or directly to chain
3. **Maintain session continuity** without repeated wallet prompts

### 1.1 Key Types

| Key Type | Abbreviation | Purpose | Lifetime |
|----------|--------------|---------|----------|
| Primary Key | PK | User's main wallet key | Permanent |
| Delegated Key for Intent | DKI | Signs payment intents | Session (max 24h) |
| Delegated Key for Settlement | DKS | Signs settlement actions | Per-transaction |

---

## 2. Message Flow

```
┌──────────┐    ┌────────────┐    ┌─────────────┐    ┌─────────┐    ┌──────────┐
│ Merchant │    │User Wallet │    │  CP-EXT     │    │   TBC   │    │  Chain   │
│   Page   │    │   (DKW)    │    │ (Extension) │    │         │    │ Contract │
└────┬─────┘    └─────┬──────┘    └──────┬──────┘    └────┬────┘    └────┬─────┘
     │                │                  │                │              │
     │ 1. requestPayment()               │                │              │
     │ ─────────────────────────────────►│                │              │
     │                │                  │                │              │
     │                │ 2. DKI Request   │                │              │
     │                │   (EIP-712 w/    │                │              │
     │                │    Economic Data)│                │              │
     │                │◄─────────────────│                │              │
     │                │                  │                │              │
     │                │ 3. DKI Response  │                │              │
     │                │   (Signed)       │                │              │
     │                │─────────────────►│                │              │
     │                │                  │                │              │
     │                │                  │ 4. TGP_QUERY   │              │
     │                │                  │ ──────────────►│              │
     │                │                  │                │              │
     │                │                  │ 5. TGP_ACK     │              │
     │                │                  │   (ALLOWED)    │              │
     │                │                  │◄───────────────│              │
     │                │                  │                │              │
     │    PATH A: TBC-Relayed           │                │              │
     │                │                  │ 6a. Delegated  │              │
     │                │                  │     TX Submit  │              │
     │                │                  │ ──────────────►│ 7a. Submit  │
     │                │                  │                │ ────────────►│
     │                │                  │                │              │
     │    PATH B: Direct (skip 4-5)      │                │              │
     │                │                  │ 4b. EIP-712 Direct Submit     │
     │                │                  │ ────────────────────────────►│
```

### 2.1 Path Selection

| Path | Via | Use Case |
|------|-----|----------|
| **A** | TBC | Gas relaying, L1-L5 verification, ZK rewriting |
| **B** | Direct | TBC unavailable, user preference, lower latency |

---

## 3. EIP-712 Typed Data Structures

### 3.1 Domain Separator

```typescript
const DKI_DOMAIN = {
  name: "CoreProve DKI",
  version: "1",
  chainId: <dynamic>,
  verifyingContract: "0x0000000000000000000000000000000000000000"
};
```

### 3.2 DKI Message Type

```typescript
interface DKIMessage {
  sessionKey: string;        // Compressed secp256k1 pubkey (hex)
  expiry: number;            // Unix timestamp
  maxValue: string;          // Max spend in wei
  merchant: string;          // Merchant address (0x0 = any)
  settlementContract: string; // Contract address
  chainId: number;           // Target chain
  nonce: string;             // 32-byte hex
  description: string;       // Human-readable (max 256 chars)
}
```

### 3.3 EIP-712 Types Definition

```typescript
const DKI_TYPES = {
  EIP712Domain: [
    { name: "name", type: "string" },
    { name: "version", type: "string" },
    { name: "chainId", type: "uint256" },
    { name: "verifyingContract", type: "address" },
  ],
  DKIMessage: [
    { name: "sessionKey", type: "bytes" },
    { name: "expiry", type: "uint256" },
    { name: "maxValue", type: "uint256" },
    { name: "merchant", type: "address" },
    { name: "settlementContract", type: "address" },
    { name: "chainId", type: "uint256" },
    { name: "nonce", type: "bytes32" },
    { name: "description", type: "string" },
  ],
};
```

---

## 4. Economic Data Envelope

The extension presents economic data to the user for review before requesting signature:

```typescript
interface DKIEconomicData {
  merchantName: string;       // "Pizza Palace"
  merchantAddress: string;    // 0x...
  orderId: string;            // Order reference
  amount: string;             // Amount in wei
  tokenSymbol: string;        // "ETH", "PLS", "USDC"
  tokenAddress: string;       // Token contract (0x0 for native)
  displayAmount: string;      // "1.5 ETH"
  fiatEquivalent?: string;    // "~$2,500 USD"
  profileType: string;        // Payment profile
  escrowTimeout: number;      // Seconds
  chainName: string;          // "PulseChain"
  chainId: number;            // 369
  settlementContract: string; // Contract address
  memo?: string;              // Merchant memo
}
```

---

## 5. DKI Request/Response Protocol

### 5.1 Request (Extension → Wallet)

```typescript
interface DKIRequest {
  type: "DKI_REQUEST";
  requestId: string;              // UUID
  domain: EIP712Domain;
  message: DKIMessage;
  economicData: DKIEconomicData;  // For UI display
  sessionPublicKey: string;       // Extension's session key
}
```

### 5.2 Response (Wallet → Extension)

```typescript
interface DKIResponse {
  type: "DKI_RESPONSE";
  requestId: string;
  approved: boolean;
  signature?: string;       // EIP-712 signature (if approved)
  signerAddress?: string;   // Wallet address that signed
  error?: string;           // Error message (if rejected)
}
```

---

## 6. Stored Delegation

After successful signing, the delegation is stored:

```typescript
interface StoredDelegation {
  id: string;                 // Request ID
  dki: DKIMessage;            // The signed message
  signature: string;          // EIP-712 signature
  signerAddress: string;      // Wallet address
  createdAt: number;          // Unix timestamp
  valueSpent: string;         // Total spent under this delegation
  txCount: number;            // Transactions signed
  active: boolean;            // Still valid
}
```

### 6.1 Storage Key

```
chrome.storage.local: "cp_delegations"
```

### 6.2 Delegation Limits

- Max stored delegations: 100
- Max delegation lifetime: 24 hours
- Auto-deactivation on: expiry, value limit exceeded, new delegation for same merchant/chain

---

## 7. Validation Rules

### 7.1 DKI Message Validation

```typescript
function validateDKIMessage(msg: DKIMessage): ValidationResult {
  // Session key: compressed pubkey (66 hex chars with 0x)
  if (!/^0x[0-9a-fA-F]{66}$/.test(msg.sessionKey)) {
    return error("Invalid sessionKey format");
  }
  
  // Expiry: future, max 24h
  const now = Math.floor(Date.now() / 1000);
  if (msg.expiry <= now) return error("Expiry must be in future");
  if (msg.expiry > now + 86400) return error("Expiry max 24h");
  
  // maxValue: positive
  if (BigInt(msg.maxValue) <= 0n) return error("maxValue must be positive");
  
  // Addresses: valid format
  if (!/^0x[0-9a-fA-F]{40}$/.test(msg.merchant)) return error("Invalid merchant");
  if (!/^0x[0-9a-fA-F]{40}$/.test(msg.settlementContract)) return error("Invalid contract");
  
  // Chain ID: positive
  if (msg.chainId <= 0) return error("Invalid chainId");
  
  // Nonce: 32 bytes hex
  if (!/^0x[0-9a-fA-F]{64}$/.test(msg.nonce)) return error("Invalid nonce");
  
  // Description: max length
  if (msg.description.length > 256) return error("Description too long");
  
  return { valid: true };
}
```

### 7.2 Stored Delegation Validation

```typescript
function validateStoredDelegation(
  delegation: StoredDelegation,
  requiredValue: string,
  requiredMerchant: string
): ValidationResult {
  if (!delegation.active) return error("Delegation inactive");
  
  const now = Math.floor(Date.now() / 1000);
  if (delegation.dki.expiry <= now) return error("Delegation expired");
  
  // Check value limit
  const totalSpent = BigInt(delegation.valueSpent) + BigInt(requiredValue);
  if (totalSpent > BigInt(delegation.dki.maxValue)) {
    return error("Would exceed value limit");
  }
  
  // Check merchant (0x0 = any)
  const ANY = "0x0000000000000000000000000000000000000000";
  if (delegation.dki.merchant !== ANY && 
      delegation.dki.merchant.toLowerCase() !== requiredMerchant.toLowerCase()) {
    return error("Merchant not authorized");
  }
  
  return { valid: true };
}
```

---

## 8. Flow State Machine

### 8.1 States

```typescript
type DKIFlowState = 
  | "IDLE"
  | "CONNECTING_WALLET"
  | "AWAITING_USER_APPROVAL"
  | "DELEGATION_RECEIVED"
  | "SENDING_QUERY"
  | "AWAITING_ACK"
  | "ACK_RECEIVED"
  | "SUBMITTING_TX"
  | "COMPLETE"
  | "ERROR"
  | "DIRECT_MODE";
```

### 8.2 State Transitions

```
IDLE ──────────────────► CONNECTING_WALLET
                              │
                              ▼
                    AWAITING_USER_APPROVAL
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
           ERROR      DELEGATION_RECEIVED  (user rejects)
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        DIRECT_MODE    SENDING_QUERY       ERROR
              │               │
              │               ▼
              │        AWAITING_ACK
              │               │
              │               ▼
              │        ACK_RECEIVED
              │               │
              └───────┬───────┘
                      ▼
               SUBMITTING_TX
                      │
                      ▼
                  COMPLETE
```

---

## 9. Implementation Files

### 9.1 Extension (coreprove-client)

| File | Purpose |
|------|---------|
| `src/types/dki.ts` | EIP-712 typed data structures |
| `src/wallet/walletConnector.ts` | EIP-1193 provider interaction |
| `src/wallet/dkiFlowManager.ts` | State machine orchestration |
| `src/popup/dkiApprovalModal.ts` | Approval UI component |
| `src/background/background.ts` | Message handlers |

### 9.2 Background Message Types

```typescript
// Initiate DKI flow
{ type: "DKI_INITIATE", economicData: DKIEconomicData, options?: { expirySeconds?: number } }

// Get current flow state
{ type: "DKI_GET_STATE" }

// Submit delegated transaction
{ type: "DKI_SUBMIT_TX", delegationId: string, txData: TxData }

// Check for existing delegation
{ type: "DKI_CHECK_DELEGATION", merchant: string, chainId: number }

// Reset flow
{ type: "DKI_RESET" }
```

### 9.3 Popup Message Types

```typescript
// Approval requested (background → popup)
{ type: "DKI_APPROVAL_REQUESTED", requestId: string, economicData: DKIEconomicData }

// Delegation received
{ type: "DKI_DELEGATION_RECEIVED", requestId: string, delegationId: string }

// ACK allowed from TBC
{ type: "DKI_ACK_ALLOWED", sessionId: string, ackData: any }

// Transaction submitted
{ type: "DKI_TX_SUBMITTED", delegationId: string, signedTx: string }
```

---

## 10. Security Considerations

### 10.1 Delegation Scope

- **Merchant-scoped**: Delegation can be limited to specific merchant address
- **Value-limited**: Maximum spend enforced per delegation
- **Time-bounded**: 24-hour maximum lifetime
- **Chain-specific**: Delegation valid only for specified chainId

### 10.2 Session Key Security

- Generated using `@noble/secp256k1` in extension
- Stored encrypted in `chrome.storage.local`
- Device-bound key derivation (extension ID + install key)
- Public key verification after decrypt

### 10.3 Replay Protection

- Unique nonce per DKI request (32 bytes random)
- Expiry timestamp prevents indefinite validity
- Transaction count tracking prevents over-use

### 10.4 Trust Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                    TRUSTED BOUNDARY                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │   Wallet    │◄──►│  Extension  │◄──►│    TBC      │      │
│  │  (EIP-712)  │    │   (DKI)     │    │  (L1-L5)    │      │
│  └─────────────┘    └─────────────┘    └─────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   UNTRUSTED BOUNDARY                         │
│  ┌─────────────┐    ┌─────────────┐                         │
│  │  Merchant   │    │   Chain     │                         │
│  │   Page      │    │  (Verify)   │                         │
│  └─────────────┘    └─────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 11. Wallet Compatibility

### 11.1 Supported Wallets

| Wallet | Detection | EIP-712 |
|--------|-----------|---------|
| MetaMask | `ethereum.isMetaMask` | ✅ `eth_signTypedData_v4` |
| Rabby | `ethereum.isRabby` | ✅ `eth_signTypedData_v4` |
| Coinbase | `ethereum.isCoinbaseWallet` | ✅ `eth_signTypedData_v4` |
| Brave | `ethereum.isBraveWallet` | ✅ `eth_signTypedData_v4` |

### 11.2 EIP-6963 Support

Extension supports EIP-6963 provider discovery for multi-wallet environments:

```typescript
// Listen for provider announcements
window.addEventListener("eip6963:announceProvider", (event) => {
  const { info, provider } = event.detail;
  wallets.push({ info, provider });
});

// Request providers
window.dispatchEvent(new Event("eip6963:requestProvider"));
```

---

## 12. Error Codes

| Code | Description |
|------|-------------|
| `DKI_NO_WALLET` | No EIP-1193 wallet detected |
| `DKI_REJECTED` | User rejected signature request |
| `DKI_WRONG_CHAIN` | Wallet on wrong chain |
| `DKI_EXPIRED` | Delegation has expired |
| `DKI_VALUE_EXCEEDED` | Would exceed maxValue |
| `DKI_MERCHANT_UNAUTHORIZED` | Merchant not in delegation scope |
| `DKI_INVALID_SIGNATURE` | Signature verification failed |
| `DKI_STORAGE_FAILED` | Failed to store delegation |

---

## 13. Future Extensions

### 13.1 EIP-1271 Contract Wallets

For smart contract wallets (Safe, Argent), signature verification via:

```solidity
function isValidSignature(bytes32 _hash, bytes memory _signature) 
  external view returns (bytes4);
```

### 13.2 DKS (Delegated Key for Settlement)

Per-transaction delegation for settlement actions (claim, release, refund) with tighter scope and shorter lifetime.

### 13.3 Multi-Chain Delegation

Single delegation valid across multiple chains with chain-specific verification.

---

## Appendix A: Full Type Definitions

See: `coreprove-client/src/types/dki.ts`

## Appendix B: Flow Manager Implementation

See: `coreprove-client/src/wallet/dkiFlowManager.ts`

## Appendix C: Wallet Connector Implementation

See: `coreprove-client/src/wallet/walletConnector.ts`


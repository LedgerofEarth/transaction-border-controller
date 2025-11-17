# CoreProver v0.3 Verification Checklist

**Date:** November 14, 2025  
**Purpose:** Verify complete compliance with CoreProver v0.3 canonical assumptions  
**Status:** Use this checklist before deployment

---

## Overview

This document provides a comprehensive verification checklist to ensure your implementation fully complies with CoreProver v0.3 requirements. Every item must be checked before considering the migration complete.

---

## Rule 1: Triple-Clock Model ✓

### Required: NO Instant/Duration/SystemTime

- [ ] Search codebase for `std::time::Instant` - ZERO results
- [ ] Search codebase for `std::time::Duration` - ZERO results
- [ ] Search codebase for `std::time::SystemTime` - ZERO results
- [ ] Search codebase for `.elapsed()` - ZERO results
- [ ] Search codebase for `.duration_since` - ZERO results

**Verification Command:**
```bash
grep -r "std::time::" crates/ --include="*.rs" | grep -v "UNIX_EPOCH"
# Should return NO results (except UNIX_EPOCH is OK in TimestampProvider impl)
```

### Required: Triple-Clock Fields

- [ ] All timestamps have `_mono: u64` field
- [ ] All timestamps have `_unix: u64` field
- [ ] All timestamps have `_iso: String` field
- [ ] TripleTimestamp struct used consistently
- [ ] TimestampProvider trait implemented

**Verification Test:**
```rust
#[test]
fn verify_triple_clock() {
    let ts = TripleTimestamp::new(1000, 1731600000, "2024-11-14T12:00:00Z".to_string());
    assert_eq!(ts.mono, 1000);
    assert_eq!(ts.unix, 1731600000);
    assert!(ts.iso.contains("T"));
    assert!(ts.validate_iso().is_ok());
}
```

### Required: Engine-Provided Timestamps

- [ ] SessionManager has TimestampProvider parameter
- [ ] HTTP handlers get timestamps from session_manager.now()
- [ ] WebSocket handlers get timestamps from session_manager.now()
- [ ] NO handlers compute their own timestamps
- [ ] All TxipEnvelope::new() calls include TripleTimestamp

**Verification Test:**
```rust
#[test]
fn verify_engine_timestamps() {
    let provider = Arc::new(TestTimestampProvider::new(1000, 1731600000));
    let manager = SessionManager::new(Default::default(), provider);
    
    let ts = manager.now();
    assert_eq!(ts.mono, 1000);
    assert_eq!(ts.unix, 1731600000);
}
```

---

## Rule 2: TXID Provenance ✓

### Required: Buyer TXIDs

- [ ] buyer_commit_txid present in all escrow records
- [ ] buyer_commit_txid validated (0x prefix, 66 chars, hex)
- [ ] buyer_chain_id stored
- [ ] buyer_withdraw_txid optional but validated if present

**Verification Test:**
```rust
#[test]
fn verify_buyer_txids() {
    let buyer = BuyerTxIds::new(
        1,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    );
    
    assert_eq!(buyer.chain_id, 1);
    assert!(!buyer.commit_txid.is_empty());
}
```

### Required: Seller TXIDs

- [ ] seller_accept_txid present in all accepted escrows
- [ ] seller_fulfill_txid present in all fulfilled escrows
- [ ] seller_block_height stored with fulfill
- [ ] seller_claim_txid OR seller_refund_txid (exactly one)
- [ ] Validation enforces exactly one settlement TXID

**Verification Test:**
```rust
#[test]
fn verify_seller_txids() {
    let seller = SellerTxIds::new(
        369,
        "0xaaaa...".to_string(),
        "0xbbbb...".to_string(),
        12345,
    )
    .with_claim("0xcccc...".to_string());
    
    assert!(seller.validate_settlement().is_ok());
    assert!(seller.was_paid());
}
```

### Required: TXID Validation

- [ ] TxRef::validate_txid() used on all TXIDs
- [ ] 0x prefix enforced
- [ ] 66 character length enforced
- [ ] Hex characters only
- [ ] Validation errors return descriptive messages

**Verification Test:**
```rust
#[test]
fn verify_txid_validation() {
    let valid = TxRef::new(1, "0x1234...".to_string(), 12345);
    assert!(valid.validate_txid().is_ok());
    
    let invalid = TxRef::new(1, "not-a-txid".to_string(), 12345);
    assert!(invalid.validate_txid().is_err());
}
```

---

## Rule 3: Canonical Receipt Format ✓

### Required: All 17 Fields Present

- [ ] session_id
- [ ] order_amount
- [ ] fulfillment_mono
- [ ] fulfillment_unix
- [ ] fulfillment_iso
- [ ] settlement_mono
- [ ] settlement_unix
- [ ] settlement_iso
- [ ] discount_pct
- [ ] discount_expiration_unix
- [ ] buyer_chain_id
- [ ] buyer_commit_txid
- [ ] seller_chain_id
- [ ] seller_accept_txid
- [ ] seller_fulfill_txid
- [ ] seller_block_height
- [ ] seller_claim_txid OR seller_refund_txid

**Verification Test:**
```rust
#[test]
fn verify_receipt_fields() {
    let ts = TripleTimestamp::new(1000, 1731600000, "2024-11-14T12:00:00Z".to_string());
    
    let receipt = CoreProverReceipt::new(
        "sess-123".to_string(),
        30000000,
        ts.clone(),
        ts,
        0, 0,
        1, "0x1234...".to_string(),
        369, "0xaaaa...".to_string(), "0xbbbb...".to_string(),
        12345,
    )
    .with_seller_claim("0xcccc...".to_string());
    
    assert!(receipt.validate().is_ok());
    assert_eq!(receipt.fulfillment_mono, 1000);
    assert_eq!(receipt.seller_block_height, 12345);
}
```

### Required: Receipt Validation

- [ ] CoreProverReceipt::validate() called before storage
- [ ] Validation checks all required fields
- [ ] Validation enforces settlement outcome
- [ ] Empty strings rejected
- [ ] Zero amounts rejected (where inappropriate)

**Verification Test:**
```rust
#[test]
fn verify_receipt_validation() {
    let mut receipt = CoreProverReceipt::new(...);
    
    // Before adding settlement - should fail
    assert!(receipt.validate().is_err());
    
    // After adding settlement - should pass
    receipt = receipt.with_seller_claim("0xcccc...".to_string());
    assert!(receipt.validate().is_ok());
}
```

---

## Rule 4: No Legacy Structs ✓

### Banned: Old Code Patterns

- [ ] NO old Receipt structs
- [ ] NO old Escrow structs with Duration
- [ ] NO old Session structs with SystemTime
- [ ] NO old timestamp handling
- [ ] Clean slate v0.3 implementation

**Verification Command:**
```bash
# Search for banned patterns
grep -r "Duration" crates/tbc-gateway/src/txip/ --include="*.rs"
grep -r "SystemTime" crates/tbc-gateway/src/txip/ --include="*.rs"
grep -r "Instant" crates/tbc-gateway/src/txip/ --include="*.rs"
# All should return NO results
```

---

## Rule 5: JSON Serialization ✓

### Required: Derive Macros

- [ ] All structs derive Serialize
- [ ] All structs derive Deserialize
- [ ] TXIDs serialize as String
- [ ] chain_id serializes as u64
- [ ] Timestamps serialize correctly

**Verification Test:**
```rust
#[test]
fn verify_serialization() {
    let ts = TripleTimestamp::new(1000, 1731600000, "2024-11-14T12:00:00Z".to_string());
    let json = serde_json::to_string(&ts).unwrap();
    
    let deserialized: TripleTimestamp = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.mono, 1000);
    assert_eq!(deserialized.unix, 1731600000);
}
```

### Required: Field Formats

- [ ] txids: String with 0x prefix
- [ ] chain_id: u64 number
- [ ] timestamps: u64 for mono/unix, String for iso
- [ ] amounts: u128 or u64
- [ ] block_height: u64

**Verification Test:**
```rust
#[test]
fn verify_json_format() {
    let buyer = BuyerTxIds::new(1, "0x1234...".to_string());
    let json = serde_json::to_value(&buyer).unwrap();
    
    assert_eq!(json["chain_id"].as_u64().unwrap(), 1);
    assert!(json["commit_txid"].as_str().unwrap().starts_with("0x"));
}
```

---

## Rule 6: TGP-00 Compatibility ✓

### Required: Transport Features

- [ ] TxIP transports CoreProver sessions
- [ ] session_id in all messages
- [ ] origin_chain_id in TxIP envelope
- [ ] TGP phase preserved
- [ ] Event type preserved

**Verification Test:**
```rust
#[test]
fn verify_tgp_transport() {
    let ts = TripleTimestamp::new(1000, 1731600000, "2024-11-14T12:00:00Z".to_string());
    
    let envelope = TxipEnvelope::tgp(
        "msg-123".to_string(),
        "sess-456".to_string(),
        Direction::ClientToTbc,
        Role::BuyerAgent,
        TgpPhase::Query,
        ts,
        json!({"phase": "QUERY"}),
    )
    .with_origin_chain(369);
    
    assert_eq!(envelope.origin_chain_id, Some(369));
    assert_eq!(envelope.session_id, "sess-456");
}
```

### Required: Layer Separation

- [ ] NO timing logic in TxIP layer
- [ ] NO economic decisions in TxIP layer
- [ ] NO policy enforcement in TxIP layer
- [ ] TxIP only transports messages
- [ ] Timestamps passed through, not computed

---

## Rule 7: Multi-File Output ✓

### Required: File Organization

- [ ] timestamp_types_v03.rs exists
- [ ] blockchain_types_v03.rs exists
- [ ] coreprover_types_v03.rs exists
- [ ] txip_types_v03.rs exists
- [ ] txip_session_v03.rs exists
- [ ] txip_http_handler_v03.rs exists
- [ ] txip_websocket_handler_v03.rs exists
- [ ] txip_mod_v03.rs exists
- [ ] All files are ASCII-only
- [ ] All files compile independently

**Verification Command:**
```bash
# Check all files exist
ls crates/tbc-gateway/src/txip/*_v03.rs

# Check ASCII compliance
for f in crates/tbc-gateway/src/txip/*_v03.rs; do
  grep -P '[^\x00-\x7F]' "$f" && echo "$f has non-ASCII" || echo "$f is clean"
done

# Check compilation
cargo check --lib
```

---

## Integration Tests ✓

### Test Suite Completeness

- [ ] Triple-clock timestamp test
- [ ] TXID validation test
- [ ] Receipt format test
- [ ] Escrow state machine test
- [ ] Session management test
- [ ] HTTP handler test
- [ ] WebSocket handler test
- [ ] Serialization roundtrip test
- [ ] Provenance validation test
- [ ] Deadline checking test

**Run All Tests:**
```bash
cargo test --lib txip
cargo test --test integration_tests
```

---

## Performance Verification ✓

### No Performance Regressions

- [ ] Session lookup O(1)
- [ ] Message deduplication O(1)
- [ ] TXID validation O(n) where n=66
- [ ] Timeout checking O(1)
- [ ] No blocking operations in handlers

**Benchmark Command:**
```bash
cargo bench --bench txip_benchmarks
```

---

## Security Verification ✓

### Security Checklist

- [ ] NO unsafe code blocks
- [ ] Thread-safe session management (Arc<RwLock>)
- [ ] Idempotency enforced
- [ ] Input validation on all TXIDs
- [ ] Timestamp validation
- [ ] Session timeout enforcement
- [ ] NO SQL injection vectors (if using DB)
- [ ] NO command injection vectors

**Security Scan:**
```bash
cargo audit
cargo clippy -- -D warnings
```

---

## Documentation Verification ✓

### Required Documentation

- [ ] TxIP-00.md specification present
- [ ] TGP-00.md specification present
- [ ] MIGRATION_TO_V03.md present
- [ ] V03_VERIFICATION.md present (this file)
- [ ] README with quick start
- [ ] API documentation in code
- [ ] Example server documented

**Documentation Check:**
```bash
cargo doc --no-deps --open
```

---

## Final Deployment Checklist ✓

### Before Production

- [ ] All verification items above checked
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] No security audit findings
- [ ] Documentation complete
- [ ] Example server runs successfully
- [ ] Integration tests pass
- [ ] Load tests pass (if available)
- [ ] Monitoring configured
- [ ] Logging configured

### Production Readiness

- [ ] TimestampProvider production implementation
- [ ] TLS configured for HTTP/WS
- [ ] Authentication implemented
- [ ] Rate limiting configured
- [ ] Session timeout tuned
- [ ] Error handling comprehensive
- [ ] Graceful shutdown implemented
- [ ] Health checks configured

---

## Compliance Score

**Total Items:** 100+  
**Required for v0.3:** All items must be checked

**Status Tracking:**
```
[ ] Rule 1: Triple-Clock Model (14 items)
[ ] Rule 2: TXID Provenance (15 items)
[ ] Rule 3: Receipt Format (12 items)
[ ] Rule 4: No Legacy (5 items)
[ ] Rule 5: Serialization (8 items)
[ ] Rule 6: TGP Compatibility (10 items)
[ ] Rule 7: File Organization (10 items)
[ ] Integration Tests (10 items)
[ ] Performance (5 items)
[ ] Security (8 items)
[ ] Documentation (7 items)
[ ] Deployment (12 items)
```

---

## Verification Commands Summary

```bash
# 1. Check for banned patterns
grep -r "std::time::" crates/tbc-gateway/src/txip/ --include="*.rs" | grep -v "UNIX_EPOCH"

# 2. Check ASCII compliance
for f in crates/tbc-gateway/src/txip/*_v03.rs; do
  grep -P '[^\x00-\x7F]' "$f" && echo "$f has non-ASCII" || echo "$f is clean"
done

# 3. Run all tests
cargo test --lib txip
cargo test --test integration_tests

# 4. Security checks
cargo audit
cargo clippy -- -D warnings

# 5. Documentation
cargo doc --no-deps

# 6. Build check
cargo check --workspace
cargo build --release
```

---

## Sign-Off

**Verified By:** _________________  
**Date:** _________________  
**v0.3 Compliant:** YES / NO  
**Ready for Production:** YES / NO

**Notes:**
_________________________________
_________________________________
_________________________________

---

**Document Version:** 1.0  
**Last Updated:** November 14, 2025  
**Status:** Final verification checklist

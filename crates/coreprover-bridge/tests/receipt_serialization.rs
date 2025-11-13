//coreprover-bridge/src/tests/receipt_serialization
use coreprover_bridge::types::receipt::Receipt;
use ethers::types::{Address, H256};
use serde_json;

#[test]
fn test_receipt_serialization_roundtrip() {
    let buyer_address: Address = "0x0000000000000000000000000000000000000001".parse().unwrap();
    let seller_address: Address = "0x0000000000000000000000000000000000000002".parse().unwrap();

    let receipt = Receipt {
        receipt_id: H256::zero(),
        order_id: H256::zero().into(),
        buyer: buyer_address,
        seller: seller_address,
        buyer_amount: 1000,
        seller_amount: 0,
        timestamp: 1234567890,
        policy_hash: H256::zero(),
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let decoded: Receipt = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded, receipt);
}
use coreprover_zk::{ZkBuyerInput, ZkSellerInput};

#[test]
fn test_zk_buyer_input_serialization() {
    let input = ZkBuyerInput {
        buyer_address: [0u8; 20],
        salt: [1u8; 32],
        order_id: [2u8; 32],
    };

    let circuit = input.to_circuit_format(); // assuming this exists
    assert_eq!(circuit.len(), 3); // adjust based on struct
}

#[test]
fn test_zk_seller_commitment_generation() {
    let input = ZkSellerInput::default(); // placeholder
    let hash = input.compute_commitment(); // assuming this method
    assert_eq!(hash.len(), 32); // Should be a valid output
}
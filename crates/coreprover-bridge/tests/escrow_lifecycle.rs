use coreprover_bridge::client::EscrowClient;
use coreprover_bridge::types::{Escrow, Receipt};
use ethers::types::{Address, H256};
use std::sync::Arc;
use ethers::providers::{Http, Provider};
use std::str::FromStr;

/// Simulates the buyer-only purchase escrow lifecycle
#[tokio::test]
async fn test_purchase_escrow_lifecycle() {
    let buyer = Address::from_str("0x0000000000000000000000000000000000000B01").unwrap();
    let seller = Address::from_str("0x0000000000000000000000000000000000005E11").unwrap();
    let contract = Address::from_str("0x000000000000000000000000000000000000C0DE").unwrap();

    let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
    let client = EscrowClient::new(provider.clone(), contract, buyer);

    let receipt = client.create_purchase_receipt(seller, 420);
    assert_eq!(receipt.buyer_amount, 420);
    assert_eq!(receipt.seller_amount, 0);
    assert_eq!(receipt.buyer, buyer);
    assert_eq!(receipt.seller, seller);
    assert_ne!(receipt.receipt_id, H256::zero());

    assert!(client.verify_receipt(&receipt));

    // Simulate matching escrow from receipt
    let escrow = client.get_escrow(receipt.receipt_id.into()).await.unwrap();
    assert_eq!(escrow.buyer, buyer);
    assert_eq!(escrow.seller, seller);
    assert_eq!(escrow.buyer_amount, 420.into());
    assert_eq!(escrow.seller_amount, 0.into());
}

/// Simulates dual-funded swap escrow lifecycle
#[tokio::test]
async fn test_swap_escrow_lifecycle() {
    let buyer = Address::from_str("0x000000000000000000000000000000000000BEEF").unwrap();
    let seller = Address::from_str("0x000000000000000000000000000000000000F00D").unwrap();
    let contract = Address::from_str("0x000000000000000000000000000000000000C0DE").unwrap();

    let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
    let client = EscrowClient::new(provider.clone(), contract, buyer);

    let receipt = client.create_swap_receipt(seller, 1000, 777);
    assert_eq!(receipt.buyer_amount, 1000);
    assert_eq!(receipt.seller_amount, 777);
    assert_eq!(receipt.buyer, buyer);
    assert_eq!(receipt.seller, seller);

    assert!(client.verify_receipt(&receipt));

    let escrow = client.get_escrow(receipt.receipt_id.into()).await.unwrap();
    assert_eq!(escrow.buyer_amount, 1000.into());
    assert_eq!(escrow.seller_amount, 777.into());
    assert_eq!(escrow.order_id, receipt.order_id);
}
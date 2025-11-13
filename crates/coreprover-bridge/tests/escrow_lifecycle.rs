use coreprover_bridge::client::EscrowClient;
use coreprover_bridge::types::{Escrow, Receipt};
use ethers::types::{Address, H256, U256};
use std::sync::Arc;
use ethers::providers::{Http, Provider};
use std::str::FromStr;
use coreprover_bridge::types::{EscrowMode, EscrowState};

/// Simulates dual-funded swap escrow lifecycle
#[tokio::test]
async fn test_swap_escrow_lifecycle() {
let buyer = "0x000000000000000000000000000000000000BEEF".parse().unwrap();
let seller = "0x000000000000000000000000000000000000F00D".parse().unwrap();
let contract = "0x000000000000000000000000000000000000C0DE".parse().unwrap();

let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
let client = EscrowClient::new(provider.clone(), contract, buyer);

let receipt = client.create_swap_receipt(seller, 100, 200); // 🧠 asymmetric amounts
assert_eq!(receipt.buyer_amount, 100);
assert_eq!(receipt.seller_amount, 200);
assert_eq!(receipt.buyer, buyer);
assert_eq!(receipt.seller, seller);
assert!(client.verify_receipt(&receipt));

// Convert receipt_id to H256 for comparison
let receipt_id_h256 = H256::from(receipt.receipt_id.0);
let escrow = client.get_escrow(receipt_id_h256).await.unwrap();

assert_eq!(escrow.buyer_amount, U256::from(100u64));
assert_eq!(escrow.seller_amount, U256::from(200u64));
assert_eq!(escrow.order_id, receipt_id_h256);
assert_eq!(escrow.mode, EscrowMode::Swap);
assert_eq!(escrow.state, EscrowState::BuyerCommitted); // assuming buyer initiates

}

#[tokio::test]
async fn test_purchase_escrow_lifecycle() {
let buyer = Address::from_str("0x0000000000000000000000000000000000000B01").unwrap();
let seller = Address::from_str("0x0000000000000000000000000000000000005E11").unwrap();
let contract = Address::from_str("0x000000000000000000000000000000000000C0DE").unwrap();

let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
let client = EscrowClient::new(provider.clone(), contract, buyer);

let receipt = client.create_purchase_receipt(seller, 42); // 🧠 unique amount
assert_eq!(receipt.buyer_amount, 42);
assert_eq!(receipt.seller_amount, 0);
assert_eq!(receipt.buyer, buyer);
assert_eq!(receipt.seller, seller);
assert!(client.verify_receipt(&receipt));

// Convert receipt_id to H256 for comparison
let receipt_id_h256 = H256::from(receipt.receipt_id.0);
let escrow = client.get_escrow(receipt_id_h256).await.unwrap();

assert_eq!(escrow.buyer_amount, U256::from(42u64));
assert_eq!(escrow.seller_amount, U256::from(0u64));
assert_eq!(escrow.buyer, buyer);
assert_eq!(escrow.seller, seller);
assert_eq!(escrow.mode, EscrowMode::Purchase);
assert_eq!(escrow.state, EscrowState::BuyerCommitted);

}

#[tokio::test]
async fn test_purchase_receipt_lifecycle() {
let buyer = "0x0000000000000000000000000000000000000B01".parse::<Address>().unwrap();
let seller = "0x0000000000000000000000000000000000005E11".parse::<Address>().unwrap();
let contract = "0x000000000000000000000000000000000000C0DE".parse::<Address>().unwrap();
let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());

let client = EscrowClient::new(provider.clone(), contract, buyer);
let receipt = client.create_purchase_receipt(seller, 5555);

// Convert receipt_id to H256 for comparison
let receipt_id_h256 = H256::from(receipt.receipt_id.0);
let escrow = client.get_escrow(receipt_id_h256).await.unwrap();

assert_eq!(receipt_id_h256, escrow.order_id);
assert_eq!(escrow.buyer_amount, U256::from(5555u64));
assert_eq!(escrow.seller_amount, U256::from(0u64));
assert_eq!(escrow.mode, EscrowMode::Purchase);
assert_eq!(escrow.state, EscrowState::BuyerCommitted);

}

#[tokio::test]
async fn test_swap_receipt_lifecycle() {
let buyer = "0x0000000000000000000000000000000000000B01".parse::<Address>().unwrap();
let seller = "0x0000000000000000000000000000000000005E11".parse::<Address>().unwrap();
let contract = "0x000000000000000000000000000000000000C0DE".parse::<Address>().unwrap();
let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());

let client = EscrowClient::new(provider.clone(), contract, buyer);
let receipt = client.create_swap_receipt(seller, 123, 456); // 🧠 uniquely trackable

// Convert receipt_id to H256 for comparison
let receipt_id_h256 = H256::from(receipt.receipt_id.0);
let escrow = client.get_escrow(receipt_id_h256).await.unwrap();

assert_eq!(receipt_id_h256, escrow.order_id);
assert_eq!(escrow.buyer_amount, U256::from(123u64));
assert_eq!(escrow.seller_amount, U256::from(456u64));
assert_eq!(escrow.mode, EscrowMode::Swap);
assert_eq!(escrow.state, EscrowState::BuyerCommitted); // until seller confirms

}
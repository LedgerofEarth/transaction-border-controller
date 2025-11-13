use coreprover_bridge::types::receipt::Receipt;
use coreprover_bridge::client::EscrowClient;
use ethers::providers::{Http, Provider};
use std::sync::Arc;
use ethers::types::Address;

#[tokio::test]
async fn test_create_escrow_client() {
    let dummy_address = "0x0000000000000000000000000000000000000000"
    .parse::<Address>()
    .unwrap();
    let dummy_provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
    let _client = EscrowClient::new(dummy_provider, dummy_address, Address::zero());
      // No panic = pass
}
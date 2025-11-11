# CoreProver Repository Layout for TBC Integration

Here’s a comprehensive repository structure that integrates CoreProver into your existing TBC (Transaction Border Controller) Rust codebase:

```
transaction-border-controller/
├── Cargo.toml                           # Updated workspace config
├── README.md
├── LICENSE
│
├── crates/
│   ├── tbc-core/                        # Existing TBC core
│   ├── tbc-gateway/                     # Existing TGP implementation
│   │
│   ├── coreprover-contracts/           # NEW: Smart contract layer
│   │   ├── Cargo.toml
│   │   ├── foundry.toml
│   │   ├── remappings.txt
│   │   ├── src/
│   │   │   ├── CoreProverEscrow.sol
│   │   │   ├── ReceiptVault.sol
│   │   │   ├── PaymentProfileRegistry.sol
│   │   │   ├── DisputeResolver.sol          # Optional
│   │   │   ├── interfaces/
│   │   │   │   ├── ICoreProverEscrow.sol
│   │   │   │   ├── IReceiptVault.sol
│   │   │   │   ├── IPaymentProfileRegistry.sol
│   │   │   │   └── IDisputeResolver.sol
│   │   │   ├── libraries/
│   │   │   │   ├── EscrowState.sol
│   │   │   │   ├── SignatureVerifier.sol
│   │   │   │   └── PaymentCalculator.sol
│   │   │   └── mocks/                       # For testing
│   │   │       ├── MockERC20.sol
│   │   │       └── MockPriceOracle.sol
│   │   ├── test/
│   │   │   ├── CoreProverEscrow.t.sol
│   │   │   ├── ReceiptVault.t.sol
│   │   │   ├── PaymentProfileRegistry.t.sol
│   │   │   ├── integration/
│   │   │   │   ├── FullSettlement.t.sol
│   │   │   │   ├── TimedRelease.t.sol
│   │   │   │   └── MultiAsset.t.sol
│   │   │   └── fuzzing/
│   │   │       ├── EscrowFuzz.t.sol
│   │   │       └── SignatureFuzz.t.sol
│   │   ├── script/
│   │   │   ├── Deploy.s.sol
│   │   │   ├── DeployMultiChain.s.sol
│   │   │   └── ConfigureProfiles.s.sol
│   │   └── docs/
│   │       ├── architecture.md
│   │       ├── security-analysis.md
│   │       └── gas-optimization.md
│   │
│   ├── coreprover-bridge/              # NEW: Rust ↔ Solidity bridge
│   │   ├── Cargo.toml
│   │   ├── build.rs                         # Generate bindings
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── contract_bindings/           # Auto-generated
│   │   │   │   ├── mod.rs
│   │   │   │   ├── core_prover_escrow.rs
│   │   │   │   ├── receipt_vault.rs
│   │   │   │   └── payment_profile_registry.rs
│   │   │   ├── client/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── escrow_client.rs         # High-level API
│   │   │   │   ├── vault_client.rs
│   │   │   │   └── multi_chain_client.rs
│   │   │   ├── types/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── escrow.rs                # Rust structs matching Solidity
│   │   │   │   ├── payment_profile.rs
│   │   │   │   ├── legal_signature.rs
│   │   │   │   └── receipt.rs
│   │   │   ├── events/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── listener.rs              # Event stream processing
│   │   │   │   └── indexer.rs               # Local event cache
│   │   │   └── utils/
│   │   │       ├── mod.rs
│   │   │       ├── signature.rs             # ECDSA signing helpers
│   │   │       └── encoding.rs              # ABI encoding utils
│   │   └── tests/
│   │       ├── integration_tests.rs
│   │       └── fixtures/
│   │           └── contract_addresses.json
│   │
│   ├── coreprover-service/             # NEW: Settlement service
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── api/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── escrow_api.rs            # REST/gRPC endpoints
│   │   │   │   ├── receipt_api.rs
│   │   │   │   └── profile_api.rs
│   │   │   ├── settlement/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── engine.rs                # Core settlement logic
│   │   │   │   ├── validator.rs             # Pre-flight checks
│   │   │   │   ├── executor.rs              # Transaction execution
│   │   │   │   └── monitor.rs               # Event monitoring
│   │   │   ├── profiles/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── manager.rs               # Profile CRUD
│   │   │   │   ├── templates.rs             # Common profile templates
│   │   │   │   └── validator.rs             # Profile validation
│   │   │   ├── state/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── escrow_state.rs          # In-memory state cache
│   │   │   │   └── sync.rs                  # Chain sync logic
│   │   │   ├── workers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── timeout_worker.rs        # Handle expirations
│   │   │   │   ├── release_worker.rs        # Timed releases
│   │   │   │   └── indexer_worker.rs        # Event indexing
│   │   │   └── config.rs
│   │   ├── config/
│   │   │   ├── default.toml
│   │   │   ├── pulsechain.toml
│   │   │   └── base.toml
│   │   └── tests/
│   │       ├── api_tests.rs
│   │       └── settlement_tests.rs
│   │
│   ├── coreprover-zk/                  # NEW: Zero-knowledge components
│   │   ├── Cargo.toml
│   │   ├── circuits/                        # Circom circuits
│   │   │   ├── package.json
│   │   │   ├── circuits/
│   │   │   │   ├── receipt_ownership.circom
│   │   │   │   ├── ephemeral_wallet.circom
│   │   │   │   └── batch_proof.circom
│   │   │   ├── build/                       # Compiled circuits
│   │   │   └── test/
│   │   │       └── circuit_tests.js
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── prover/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── receipt_prover.rs        # Generate proofs
│   │   │   │   └── batch_prover.rs
│   │   │   ├── verifier/
│   │   │   │   ├── mod.rs
│   │   │   │   └── on_chain_verifier.rs     # Interact with Solidity verifier
│   │   │   ├── keys/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── generator.rs             # Key derivation
│   │   │   │   └── manager.rs               # Secure key storage
│   │   │   └── utils/
│   │   │       ├── mod.rs
│   │   │       └── poseidon.rs              # Hash functions
│   │   └── tests/
│   │       ├── proof_generation_tests.rs
│   │       └── verification_tests.rs
│   │
│   ├── coreprover-cli/                 # NEW: CLI tool for operators
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── deploy.rs                # Deploy contracts
│   │   │   │   ├── escrow.rs                # Manage escrows
│   │   │   │   ├── profile.rs               # Manage profiles
│   │   │   │   ├── receipt.rs               # Query receipts
│   │   │   │   └── monitor.rs               # Monitor chain state
│   │   │   └── ui/
│   │   │       ├── mod.rs
│   │   │       └── table_renderer.rs
│   │   └── tests/
│   │       └── cli_tests.rs
│   │
│   └── coreprover-sdk/                 # NEW: Developer SDK
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── builder/
│       │   │   ├── mod.rs
│       │   │   ├── escrow_builder.rs        # Fluent API
│       │   │   └── profile_builder.rs
│       │   ├── client/
│       │   │   ├── mod.rs
│       │   │   └── unified_client.rs        # Simple high-level API
│       │   └── examples/
│       │       ├── basic_escrow.rs
│       │       ├── digital_goods.rs
│       │       ├── physical_shipping.rs
│       │       └── subscription.rs
│       ├── examples/
│       │   ├── pizza_delivery.rs
│       │   ├── saas_license.rs
│       │   └── marketplace.rs
│       └── tests/
│           └── sdk_integration_tests.rs
│
├── docs/
│   ├── specs/
│   │   ├── coreprover.md                    # Updated spec
│   │   ├── payment-profiles.md
│   │   ├── zk-privacy.md
│   │   └── multi-chain.md
│   ├── guides/
│   │   ├── quickstart.md
│   │   ├── seller-integration.md
│   │   ├── buyer-privacy.md
│   │   └── deployment.md
│   ├── api/
│   │   ├── rest-api.md
│   │   ├── grpc-api.md
│   │   └── contract-abi.md
│   └── architecture/
│       ├── overview.md
│       ├── settlement-flow.md
│       ├── security-model.md
│       └── diagrams/
│           ├── dual-commitment.svg
│           ├── state-machine.svg
│           └── multi-chain.svg
│
├── scripts/
│   ├── setup-dev.sh                         # Local dev environment
│   ├── deploy-testnet.sh
│   ├── deploy-mainnet.sh
│   ├── generate-bindings.sh                 # Regenerate Rust bindings
│   └── run-tests.sh
│
├── docker/
│   ├── Dockerfile.service
│   ├── Dockerfile.indexer
│   ├── docker-compose.yml
│   └── docker-compose.dev.yml
│
└── .github/
    └── workflows/
        ├── contracts-ci.yml                 # Foundry tests
        ├── rust-ci.yml                      # Cargo tests
        ├── integration-ci.yml               # Full stack tests
        └── deploy-testnet.yml
```

——

## 📦 Updated Root `Cargo.toml`

```toml
[workspace]
members = [
    “crates/tbc-core”,
    “crates/tbc-gateway”,
    “crates/coreprover-bridge”,
    “crates/coreprover-service”,
    “crates/coreprover-zk”,
    “crates/coreprover-cli”,
    “crates/coreprover-sdk”,
]

resolver = “2”

[workspace.package]
version = “0.2.0”
edition = “2021”
license = “COMMERCIAL”
authors = [“Ledger of Earth”]

[workspace.dependencies]
# Existing TBC dependencies
tbc-core = { path = “crates/tbc-core” }
tbc-gateway = { path = “crates/tbc-gateway” }

# New CoreProver dependencies
coreprover-bridge = { path = “crates/coreprover-bridge” }
coreprover-service = { path = “crates/coreprover-service” }
coreprover-zk = { path = “crates/coreprover-zk” }
coreprover-sdk = { path = “crates/coreprover-sdk” }

# Ethereum & Blockchain
ethers = { version = “2.0”, features = [“abigen”, “ws”] }
alloy-primitives = “0.7”
alloy-sol-types = “0.7”
foundry-compilers = “0.3”

# Async & Networking
tokio = { version = “1.35”, features = [“full”] }
tokio-stream = “0.1”
axum = { version = “0.7”, features = [“ws”] }
tower = “0.4”
tower-http = { version = “0.5”, features = [“cors”, “trace”] }

# Serialization
serde = { version = “1.0”, features = [“derive”] }
serde_json = “1.0”
toml = “0.8”

# Database & Storage
sqlx = { version = “0.7”, features = [“runtime-tokio-rustls”, “postgres”, “macros”] }
redis = { version = “0.24”, features = [“tokio-comp”, “connection-manager”] }

# Cryptography
sha3 = “0.10”
k256 = { version = “0.13”, features = [“ecdsa”] }
rand = “0.8”

# Zero-Knowledge (placeholder for future ZK libs)
ark-bn254 = “0.4”
ark-groth16 = “0.4”

# Tracing & Observability
tracing = “0.1”
tracing-subscriber = { version = “0.3”, features = [“env-filter”, “json”] }

# Error Handling
anyhow = “1.0”
thiserror = “1.0”

# CLI
clap = { version = “4.4”, features = [“derive”, “cargo”] }
indicatif = “0.17”
colored = “2.1”

# Testing
proptest = “1.4”
mockito = “1.2”
```

——

## 🔧 Key Crate Details

### 1. `coreprover-contracts/Cargo.toml`

```toml
[package]
name = “coreprover-contracts”
version.workspace = true
edition.workspace = true

[build-dependencies]
foundry-compilers = “0.3”

[dev-dependencies]
# Foundry uses its own test framework
```

### 2. `coreprover-bridge/Cargo.toml`

```toml
[package]
name = “coreprover-bridge”
version.workspace = true
edition.workspace = true

[dependencies]
ethers.workspace = true
alloy-primitives.workspace = true
alloy-sol-types.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true

[build-dependencies]
foundry-compilers = “0.3”

[dev-dependencies]
tokio = { workspace = true, features = [“test-util”] }
mockito.workspace = true
```

### 3. `coreprover-service/Cargo.toml`

```toml
[package]
name = “coreprover-service”
version.workspace = true
edition.workspace = true

[[bin]]
name = “coreprover-service”
path = “src/main.rs”

[dependencies]
coreprover-bridge.workspace = true
tbc-core.workspace = true

ethers.workspace = true
tokio.workspace = true
axum.workspace = true
tower.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
toml.workspace = true
sqlx.workspace = true
redis.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
```

### 4. `coreprover-zk/Cargo.toml`

```toml
[package]
name = “coreprover-zk”
version.workspace = true
edition.workspace = true

[dependencies]
ark-bn254.workspace = true
ark-groth16.workspace = true
serde.workspace = true
serde_json.workspace = true
sha3.workspace = true
k256.workspace = true
rand.workspace = true
anyhow.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
```

### 5. `coreprover-cli/Cargo.toml`

```toml
[package]
name = “coreprover-cli”
version.workspace = true
edition.workspace = true

[[bin]]
name = “coreprover”
path = “src/main.rs”

[dependencies]
coreprover-bridge.workspace = true
coreprover-service.workspace = true

clap.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
indicatif.workspace = true
colored.workspace = true
anyhow.workspace = true
```

### 6. `coreprover-sdk/Cargo.toml`

```toml
[package]
name = “coreprover-sdk”
version.workspace = true
edition.workspace = true

[dependencies]
coreprover-bridge.workspace = true

ethers.workspace = true
tokio.workspace = true
serde.workspace = true
anyhow.workspace = true
thiserror.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = [“test-util”] }
```

——

## 🧪 Test Structure

### Foundry Tests (Solidity)

```bash
cd crates/coreprover-contracts

# Run all tests
forge test

# Run with gas reports
forge test —gas-report

# Run specific test
forge test —match-test testBothCommitted

# Run fuzzing
forge test —match-contract EscrowFuzz

# Coverage
forge coverage
```

### Rust Tests

```bash
# Test all crates
cargo test —workspace

# Test specific crate
cargo test -p coreprover-bridge

# Integration tests only
cargo test —test integration_tests

# With logging
RUST_LOG=debug cargo test

# Watch mode
cargo watch -x “test —workspace”
```

### Full Stack Integration Tests

```bash
# Start local chain + services
./scripts/setup-dev.sh

# Run integration tests
cargo test —test full_stack_integration — —test-threads=1
```

——

## 🚀 Development Workflow Scripts

### `scripts/setup-dev.sh`

```bash
#!/bin/bash
set -e

echo “🔧 Setting up CoreProver development environment...”

# Start local Ethereum node (Anvil from Foundry)
anvil —port 8545 —chain-id 31337 &
ANVIL_PID=$!

sleep 2

# Deploy contracts
cd crates/coreprover-contracts
forge build
forge script script/Deploy.s.sol —rpc-url http://localhost:8545 —broadcast

# Generate Rust bindings
cd ../..
./scripts/generate-bindings.sh

# Start Redis for caching
docker run -d -p 6379:6379 redis:alpine

# Start Postgres for indexing
docker run -d -p 5432:5432 \
  -e POSTGRES_PASSWORD=dev \
  -e POSTGRES_DB=coreprover \
  postgres:15-alpine

# Run migrations
cd crates/coreprover-service
sqlx migrate run

echo “✅ Development environment ready!”
echo “Anvil PID: $ANVIL_PID”
```

### `scripts/generate-bindings.sh`

```bash
#!/bin/bash
set -e

echo “🔨 Generating Rust bindings from Solidity contracts...”

cd crates/coreprover-contracts

# Ensure contracts are compiled
forge build

# Generate bindings
cd ../coreprover-bridge

cat > build.rs << ‘EOF’
use ethers::contract::Abigen;

fn main() {
    let contracts = [
        (“CoreProverEscrow”, “../coreprover-contracts/out/CoreProverEscrow.sol/CoreProverEscrow.json”),
        (“ReceiptVault”, “../coreprover-contracts/out/ReceiptVault.sol/ReceiptVault.json”),
        (“PaymentProfileRegistry”, “../coreprover-contracts/out/PaymentProfileRegistry.sol/PaymentProfileRegistry.json”),
    ];

    for (name, path) in contracts {
        Abigen::new(name, path)
            .unwrap()
            .generate()
            .unwrap()
            .write_to_file(format!(“src/contract_bindings/{}.rs”, name.to_lowercase()))
            .unwrap();
    }
}
EOF

cargo build

echo “✅ Bindings generated in crates/coreprover-bridge/src/contract_bindings/“
```

——

## 🐳 Docker Configuration

### `docker/docker-compose.dev.yml`

```yaml
version: ‘3.9’

services:
  anvil:
    image: ghcr.io/foundry-rs/foundry:latest
    command: anvil —host 0.0.0.0 —chain-id 31337
    ports:
      - “8545:8545”

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_PASSWORD: dev
      POSTGRES_DB: coreprover
    ports:
      - “5432:5432”
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:alpine
    ports:
      - “6379:6379”

  coreprover-service:
    build:
      context: ..
      dockerfile: docker/Dockerfile.service
    depends_on:
      - anvil
      - postgres
      - redis
    environment:
      DATABASE_URL: postgres://postgres:dev@postgres:5432/coreprover
      REDIS_URL: redis://redis:6379
      ETH_RPC_URL: http://anvil:8545
    ports:
      - “8080:8080”
    volumes:
      - ../config:/app/config

volumes:
  postgres_data:
```

——

## 📋 CI/CD Workflow

### `.github/workflows/integration-ci.yml`

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: coreprover_test
        options: >-
          —health-cmd pg_isready
          —health-interval 10s
          —health-timeout 5s
          —health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:alpine
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Foundry
        uses: foundry-rs/foundry-toolchain@v1
      
      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - name: Build Solidity contracts
        working-directory: crates/coreprover-contracts
        run: forge build
      
      - name: Run Solidity tests
        working-directory: crates/coreprover-contracts
        run: forge test -vvv
      
      - name: Generate Rust bindings
        run: ./scripts/generate-bindings.sh
      
      - name: Run Rust tests
        run: cargo test —workspace —all-features
      
      - name: Start Anvil
        run: anvil —port 8545 &
      
      - name: Deploy contracts
        working-directory: crates/coreprover-contracts
        run: |
          forge script script/Deploy.s.sol \
            —rpc-url http://localhost:8545 \
            —broadcast
      
      - name: Run integration tests
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/coreprover_test
          REDIS_URL: redis://localhost:6379
          ETH_RPC_URL: http://localhost:8545
        run: cargo test —test ‘*’ — —test-threads=1
```

——

## 📚 Example File Contents

### `crates/coreprover-bridge/src/lib.rs`

```rust
//! CoreProver Bridge
//! 
//! High-level Rust interface to CoreProver smart contracts

pub mod contract_bindings;
pub mod client;
pub mod types;
pub mod events;
pub mod utils;

pub use client::{EscrowClient, VaultClient};
pub use types::{Escrow, PaymentProfile, LegalSignature, Receipt};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_escrow_flow() {
        // Integration test placeholder
    }
}
```

### `crates/coreprover-service/src/main.rs`

```rust
use anyhow::Result;
use coreprover_service::{config::Config, api, settlement, workers};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var(“RUST_LOG”).unwrap_or_else(|_| “info”.into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_file(“config/default.toml”)?;

    // Start workers
    let timeout_worker = workers::timeout_worker::start(&config).await?;
    let release_worker = workers::release_worker::start(&config).await?;
    let indexer_worker = workers::indexer_worker::start(&config).await?;

    // Start API server
    let api_server = api::serve(config).await?;

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!(“Received shutdown signal”);
        }
    }

    Ok(())
}
```

### `crates/coreprover-cli/src/main.rs`

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = “coreprover”)]
#[command(about = “CoreProver CLI - Manage escrows and settlements”)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy contracts to a network
    Deploy {
        #[arg(short, long)]
        network: String,
    },
    /// Create a new escrow
    Escrow {
        #[command(subcommand)]
        action: EscrowCommands,
    },
    /// Manage payment profiles
    Profile {
        #[command(subcommand)]
        action: ProfileCommands,
    },
    /// Monitor chain state
    Monitor {
        #[arg(short, long)]
        chain: String,
    },
}

#[derive(Subcommand)]
enum EscrowCommands {
    Create,
    Commit,
    Claim,
    Query,
}

#[derive(Subcommand)]
enum ProfileCommands {
    Create,
    List,
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy { network } => {
            println!(“🚀 Deploying to {}...”, network);
            // Implementation
        }
        Commands::Escrow { action } => {
            // Implementation
        }
        Commands::Profile { action } => {
            // Implementation
        }
        Commands::Monitor { chain } => {
            // Implementation
        }
    }

    Ok(())
}
```

——

## 🎯 Next Steps

1. **Initialize the structure:**
   
   ```bash
   # Create new crates
   cd crates
   cargo new coreprover-bridge —lib
   cargo new coreprover-service —bin
   cargo new coreprover-zk —lib
   cargo new coreprover-cli —bin
   cargo new coreprover-sdk —lib
   
   # Initialize Foundry project
   mkdir coreprover-contracts
   cd coreprover-contracts
   forge init —no-git
   ```
1. **Set up contracts:**
   
   ```bash
   cd crates/coreprover-contracts
   # Copy Solidity files from spec
   forge build
   forge test
   ```
1. **Generate bindings:**
   
   ```bash
   ./scripts/generate-bindings.sh
   ```
1. **Implement core types in Rust:**
   Start with `coreprover-bridge/src/types/` matching Solidity structs
1. **Build settlement service:**
   Implement `coreprover-service` with event monitoring


// ============================================================================
// CoreProver v0.3 Test Harness
// File: tests/harness/context.rs
//
// TestContext is the hybrid orchestrator that ties together:
//   * Multi-chain support (HashMap of engines + chains)
//   * Single-chain driver pattern (primary EngineDriver)
//   * TimeController (deterministic clock)
//   * MockChain (deterministic TXID + block height)
//   * ModelChecker (invariant validation)
//   * Profile factories (scenario-specific configurations)
//
// This creates a full testing runtime supporting both:
//   - Complex multi-chain cross-settlement flows
//   - Standard single-chain scenario testing
//
// Architecture:
//   Layer 3 of integration (from reconciliation report):
//     - Layer 1: types.rs (bridge types)
//     - Layer 2: engine_driver.rs (operation wrapper)
//     - Layer 3: context.rs (test coordination) ← THIS FILE
//
// ============================================================================

#![allow(dead_code)]

use std::collections::HashMap;

use super::engine_driver::{EngineDriver, DriverConfig};
use super::engine_factory::EngineFactory;
use super::mock_chain::{MockChain, MockChainId};
use super::time::TimeController;
use super::model_checker::ModelChecker;
use super::types::*;

use coreprover_service::engine::CoreProverEngine;
use coreprover_types_v03::{
    CoreProverReceipt, EscrowState, PaymentProfile, TimingProfile,
};

use std::time::Duration;

// ============================================================================
// TestConfig - Configuration for test context
// ============================================================================

#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Primary chain ID for single-chain scenarios
    pub primary_chain_id: u64,
    
    /// Block interval in seconds
    pub block_interval_secs: u64,
    
    /// Genesis timestamp (unix)
    pub genesis_unix: u64,
    
    /// Enable invariant checking
    pub enable_invariants: bool,
    
    /// Enable detailed tracing
    pub enable_tracing: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            primary_chain_id: 1,         // Ethereum mainnet
            block_interval_secs: 12,     // Ethereum block time
            genesis_unix: 1700000000,    // Nov 2023
            enable_invariants: true,
            enable_tracing: true,
        }
    }
}

impl TestConfig {
    /// Create driver config from test config
    pub fn driver_config(&self) -> DriverConfig {
        DriverConfig {
            chain_id: self.primary_chain_id,
            block_interval_secs: self.block_interval_secs,
            genesis_unix: self.genesis_unix,
            default_profile: PaymentProfile::default(),
        }
    }
}

// ============================================================================
// ValidationError - Errors from invariant checking
// ============================================================================

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidationError: {}", self.message)
    }
}

impl std::error::Error for ValidationError {}

// ============================================================================
// TestContext - Hybrid multi-chain + single-driver orchestrator
// ============================================================================

pub struct TestContext {
    // =========================================================================
    // Single-Chain Driver Pattern (from reconciliation report)
    // =========================================================================
    
    /// Primary engine driver for standard single-chain scenarios
    pub driver: EngineDriver,
    
    /// Model checker for invariant validation
    pub model_checker: ModelChecker,
    
    /// Test configuration
    pub config: TestConfig,
    
    // =========================================================================
    // Multi-Chain Orchestration (original design)
    // =========================================================================
    
    /// Global deterministic time controller (shared for all chains)
    time: TimeController,

    /// One engine per chain (multi-chain support)
    engines: HashMap<MockChainId, CoreProverEngine>,

    /// Mock chains for TXID + block height simulation
    chains: HashMap<MockChainId, MockChain>,

    /// Session + order counters for reproducible test IDs
    session_counter: u64,
    order_counter: u64,
}

impl TestContext {
    // =========================================================================
    // Construction
    // =========================================================================
    
    pub fn new(config: TestConfig) -> Self {
        let time = TimeController::new(config.genesis_unix, config.block_interval_secs);
        
        let driver = EngineDriver::new(config.driver_config());
        
        let model_checker = if config.enable_invariants {
            ModelChecker::new_with_v03_rules()
        } else {
            ModelChecker::new()
        };
        
        Self {
            driver,
            model_checker,
            config,
            time,
            engines: HashMap::new(),
            chains: HashMap::new(),
            session_counter: 1,
            order_counter: 1,
        }
    }
    
    pub fn default() -> Self {
        Self::new(TestConfig::default())
    }

    // =========================================================================
    // Invariant Checking
    // =========================================================================
    
    pub fn check_invariants(&self) -> Result<(), ValidationError> {
        if !self.config.enable_invariants {
            return Ok(());
        }
        
        self.model_checker.check(&self.driver)
            .map_err(|e| ValidationError::new(e))
    }
    
    pub fn validate_state_transition(
        &self,
        before: EscrowState,
        after: EscrowState,
    ) -> Result<(), ValidationError> {
        self.model_checker.validate_transition(before, after)
            .map_err(|e| ValidationError::new(e))
    }

    // =========================================================================
    // Profile Factories
    // =========================================================================
    
    pub fn pizza_delivery_profile(&self) -> PaymentProfile {
        PaymentProfile {
            timing: TimingProfile {
                acceptance_window_secs: 300,
                fulfillment_window_secs: 3600,
                claim_window_secs: 86400,
            },
            enables_late_discount: true,
            late_discount_pct: 15,
            discount_expiration_days: 7,
            allows_timed_release: true,
        }
    }
    
    pub fn digital_goods_profile(&self) -> PaymentProfile {
        PaymentProfile {
            timing: TimingProfile {
                acceptance_window_secs: 60,
                fulfillment_window_secs: 300,
                claim_window_secs: 3600,
            },
            enables_late_discount: false,
            late_discount_pct: 0,
            discount_expiration_days: 0,
            allows_timed_release: true,
        }
    }
    
    pub fn swap_profile(&self) -> PaymentProfile {
        PaymentProfile {
            timing: TimingProfile {
                acceptance_window_secs: 600,
                fulfillment_window_secs: 1800,
                claim_window_secs: 7200,
            },
            enables_late_discount: false,
            late_discount_pct: 0,
            discount_expiration_days: 0,
            allows_timed_release: false,
        }
    }
    
    pub fn physical_goods_profile(&self) -> PaymentProfile {
        PaymentProfile {
            timing: TimingProfile {
                acceptance_window_secs: 3600,
                fulfillment_window_secs: 259200,
                claim_window_secs: 604800,
            },
            enables_late_discount: true,
            late_discount_pct: 10,
            discount_expiration_days: 14,
            allows_timed_release: true,
        }
    }
    
    pub fn service_profile(&self) -> PaymentProfile {
        PaymentProfile {
            timing: TimingProfile {
                acceptance_window_secs: 1800,
                fulfillment_window_secs: 86400,
                claim_window_secs: 172800,
            },
            enables_late_discount: false,
            late_discount_pct: 0,
            discount_expiration_days: 0,
            allows_timed_release: true,
        }
    }
    
    pub fn profile_by_name(&self, name: &str) -> PaymentProfile {
        match name {
            "pizza" | "pizza_delivery" => self.pizza_delivery_profile(),
            "digital" | "digital_goods" => self.digital_goods_profile(),
            "swap" | "atomic_swap" => self.swap_profile(),
            "physical" | "physical_goods" => self.physical_goods_profile(),
            "service" => self.service_profile(),
            _ => PaymentProfile::default(),
        }
    }

    // =========================================================================
    // Multi-Chain Support
    // =========================================================================

    pub fn chain(&mut self, chain_id: MockChainId) -> MockChainId {
        if !self.chains.contains_key(&chain_id) {
            self.chains.insert(chain_id, MockChain::new(chain_id));
            self.engines.insert(
                chain_id,
                EngineFactory::build(
                    chain_id,
                    self.config.block_interval_secs,
                    self.time.unix_now(),
                ),
            );
        }
        chain_id
    }

    // =========================================================================
    // Time Advancement
    // =========================================================================

    pub fn advance_time(&mut self, secs: u64) {
        let duration = Duration::from_secs(secs);
        
        self.driver.advance_time(duration);
        self.time.advance(secs);

        for engine in self.engines.values_mut() {
            engine.advance_time(secs);
        }
        for chain in self.chains.values_mut() {
            chain.advance_time(secs);
        }
    }
    
    pub fn advance_blocks(&mut self, blocks: u64) {
        let secs = blocks * self.config.block_interval_secs;
        self.advance_time(secs);
    }

    // =========================================================================
    // Session + Order Generation
    // =========================================================================

    pub fn next_session_id(&mut self, label: &str) -> HarnessSessionId {
        let s = HarnessSessionId::new(label, self.session_counter);
        self.session_counter += 1;
        s
    }

    pub fn next_order_id(&mut self, label: &str) -> HarnessOrderId {
        let o = HarnessOrderId::new(label, self.order_counter);
        self.order_counter += 1;
        o
    }

    // =========================================================================
    // Multi-Chain Operations
    // =========================================================================

    pub fn commit(
        &mut self,
        buyer_chain: MockChainId,
        seller_chain: MockChainId,
        amount: u64,
        label: &str,
    ) -> HarnessOrderId {
        self.chain(buyer_chain);
        self.chain(seller_chain);

        let order = self.next_order_id(label);

        let commit_tx = {
            let chain = self.chains.get_mut(&buyer_chain).unwrap();
            chain.generate_txid()
        };

        let oid_bytes = order.to_bytes().unwrap();

        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let profile = self.pizza_delivery_profile();
        let _ = engine.buyer_commit(
            format!("buyer@chain{}", buyer_chain.0),
            format!("seller@chain{}", seller_chain.0),
            amount,
            profile,
            buyer_chain.0,
            commit_tx.txid.clone(),
        );

        order
    }

    pub fn accept(&mut self, order: &HarnessOrderId, seller_chain: MockChainId) {
        let tx = {
            let chain = self.chains.get_mut(&seller_chain).unwrap();
            chain.generate_txid()
        };

        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let _ = engine.seller_accept(&oid, tx.txid);
    }

    pub fn fulfill(&mut self, order: &HarnessOrderId, seller_chain: MockChainId) {
        let tx = {
            let chain = self.chains.get_mut(&seller_chain).unwrap();
            chain.generate_txid()
        };

        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let _ = engine.seller_fulfill(&oid, tx.txid);
    }

    pub fn claim(&mut self, order: &HarnessOrderId, seller_chain: MockChainId) {
        let tx = {
            let chain = self.chains.get_mut(&seller_chain).unwrap();
            chain.generate_txid()
        };

        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let _ = engine.seller_claim(&oid, tx.txid);
    }

    pub fn refund(&mut self, order: &HarnessOrderId, seller_chain: MockChainId) {
        let tx = {
            let chain = self.chains.get_mut(&seller_chain).unwrap();
            chain.generate_txid()
        };

        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let _ = engine.seller_refund(&oid, tx.txid);
    }

    pub fn withdraw(&mut self, order: &HarnessOrderId, seller_chain: MockChainId) {
        let tx = {
            let chain = self.chains.get_mut(&seller_chain).unwrap();
            chain.generate_txid()
        };

        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get_mut(&seller_chain).unwrap();
        let _ = engine.buyer_withdraw(&oid, Some(tx.txid));
    }

    pub fn receipt(
        &self,
        order: &HarnessOrderId,
        seller_chain: MockChainId,
    ) -> Option<CoreProverReceipt> {
        let oid = order.to_bytes().unwrap();
        let engine = self.engines.get(&seller_chain)?;
        engine.get_receipt(&oid).map(|m| m.to_public_receipt())
    }

    // =========================================================================
    // State Queries
    // =========================================================================

    pub fn current_timestamp(&self) -> HarnessTimestamp {
        self.driver.current_time()
    }
    
    pub fn current_block(&self) -> u64 {
        self.driver.current_block()
    }
    
    pub fn current_mono(&self) -> u64 {
        self.time.current_mono()
    }
    
    pub fn current_unix(&self) -> u64 {
        self.time.current_unix()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_initialization() {
        let config = TestConfig::default();
        let ctx = TestContext::new(config);
        
        assert_eq!(ctx.current_block(), 1);
        assert_eq!(ctx.current_mono(), 0);
    }

    #[test]
    fn profile_factories() {
        let ctx = TestContext::default();
        
        let pizza = ctx.pizza_delivery_profile();
        assert_eq!(pizza.timing.fulfillment_window_secs, 3600);
        assert!(pizza.enables_late_discount);
        
        let digital = ctx.digital_goods_profile();
        assert_eq!(digital.timing.fulfillment_window_secs, 300);
        assert!(!digital.enables_late_discount);
        
        let swap = ctx.swap_profile();
        assert!(!swap.allows_timed_release);
    }

    #[test]
    fn time_synchronization() {
        let mut ctx = TestContext::default();
        
        ctx.advance_time(600);
        
        assert_eq!(ctx.current_mono(), 600);
        assert_eq!(ctx.driver.current_time().mono, 600);
    }

    #[test]
    fn multi_chain_basic_flow() {
        let mut ctx = TestContext::default();

        let buyer_chain = ctx.chain(MockChainId(1));
        let seller_chain = ctx.chain(MockChainId(369));

        let order = ctx.commit(buyer_chain, seller_chain, 1000, "pizza");

        ctx.advance_time(5);
        ctx.accept(&order, seller_chain);

        ctx.advance_time(10);
        ctx.fulfill(&order, seller_chain);

        ctx.advance_time(20);
        ctx.claim(&order, seller_chain);

        let receipt = ctx.receipt(&order, seller_chain)
            .expect("receipt must exist");

        assert_eq!(receipt.order_amount, 1000);
    }

    #[test]
    fn single_chain_via_driver() {
        let mut ctx = TestContext::default();
        
        let params = CommitParams::new(
            "buyer".into(),
            "seller".into(),
            1000,
        ).with_profile(ctx.pizza_delivery_profile());
        
        let order_id = ctx.driver.buyer_commit(params)
            .expect("commit should succeed");
        
        ctx.advance_time(300);
        
        ctx.driver.seller_accept(order_id.clone(), None)
            .expect("accept should succeed");
        
        let state = ctx.driver.get_state(&order_id)
            .expect("state query should succeed");
        
        assert_eq!(state, EscrowState::SellerAccepted);
    }

    #[test]
    fn invariant_checking() {
        let config = TestConfig {
            enable_invariants: true,
            ..Default::default()
        };
        let ctx = TestContext::new(config);
        
        ctx.check_invariants().expect("invariants should pass");
    }

    #[test]
    fn state_transition_validation() {
        let ctx = TestContext::default();
        
        ctx.validate_state_transition(
            EscrowState::BuyerCommitted,
            EscrowState::SellerAccepted,
        ).expect("valid transition should pass");
        
        let result = ctx.validate_state_transition(
            EscrowState::BuyerCommitted,
            EscrowState::SellerClaimed,
        );
        
        assert!(result.is_err(), "invalid transition should fail");
    }
}
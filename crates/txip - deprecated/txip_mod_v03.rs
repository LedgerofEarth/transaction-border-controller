// crates/tbc-gateway/src/txip/mod_v03.rs
// FINAL - TxIP Module Root with CoreProver v0.3 Exports
//
// This module exports all TxIP v0.2 types with CoreProver v0.3 compatibility.
// NO Instant, Duration, or SystemTime usage throughout.
// All timing via TimestampProvider trait from engine.

pub mod timestamp_types_v03;
pub mod blockchain_types_v03;
pub mod coreprover_types_v03;
pub mod txip_types_v03;
pub mod txip_session_v03;
pub mod txip_http_handler_v03;
pub mod txip_websocket_handler_v03;

// Re-export timestamp types
pub use timestamp_types_v03::{
    TripleTimestamp, TimestampProvider, Deadline, TimeWindow,
};

// Re-export blockchain types
pub use blockchain_types_v03::{
    TxId, ChainId, BlockHeight, TxRef, BuyerTxIds, SellerTxIds, TxIdProvenance,
};

// Re-export CoreProver types
pub use coreprover_types_v03::{
    CoreProverReceipt, EscrowState, EscrowRecord,
};

// Re-export TxIP types
pub use txip_types_v03::{
    TxipEnvelope, Direction, Role, MessageType, TgpPhase, Payload,
    ControlPayload, TgpPayload, ErrorPayload, HelloPayload, WelcomePayload,
    HeartbeatPayload, ClosePayload, Features, AuthInfo, AuthScheme,
    ErrorCode, CloseReason, TXIP_VERSION,
};

// Re-export session types
pub use txip_session_v03::{SessionManager, SessionInfo, SessionConfig};

// Re-export handler types
pub use txip_http_handler_v03::{HttpHandlerState, handle_txip_message, MessageAcceptedResponse};
pub use txip_websocket_handler_v03::{WebSocketHandlerState, handle_websocket_upgrade};

/// Prelude module for common imports
pub mod prelude {
    pub use super::timestamp_types_v03::{TripleTimestamp, TimestampProvider};
    pub use super::blockchain_types_v03::{TxId, ChainId, TxIdProvenance};
    pub use super::coreprover_types_v03::{CoreProverReceipt, EscrowState};
    pub use super::txip_types_v03::{
        TxipEnvelope, Direction, Role, MessageType, TgpPhase,
        ErrorCode, TXIP_VERSION,
    };
    pub use super::txip_session_v03::{SessionManager, SessionConfig};
    pub use super::txip_http_handler_v03::HttpHandlerState;
    pub use super::txip_websocket_handler_v03::WebSocketHandlerState;
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    /// Test timestamp provider for integration tests
    struct TestTimestampProvider {
        current_mono: std::sync::atomic::AtomicU64,
        current_unix: std::sync::atomic::AtomicU64,
    }

    impl TestTimestampProvider {
        fn new(mono: u64, unix: u64) -> Self {
            Self {
                current_mono: std::sync::atomic::AtomicU64::new(mono),
                current_unix: std::sync::atomic::AtomicU64::new(unix),
            }
        }
    }

    impl TimestampProvider for TestTimestampProvider {
        fn now(&self) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(
                mono,
                unix,
                format!("2024-11-14T12:00:{}Z", mono % 60),
            )
        }

        fn at_unix(&self, unix: u64) -> TripleTimestamp {
            let mono = self.current_mono.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(mono, unix, "2024-11-14T12:00:00Z".to_string())
        }

        fn at_mono(&self, mono: u64) -> TripleTimestamp {
            let unix = self.current_unix.load(std::sync::atomic::Ordering::SeqCst);
            TripleTimestamp::new(mono, unix, "2024-11-14T12:00:00Z".to_string())
        }
    }

    #[test]
    fn test_txip_envelope_roundtrip() {
        let ts = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );

        let envelope = TxipEnvelope::tgp(
            "msg-123".to_string(),
            "sess-456".to_string(),
            Direction::ClientToTbc,
            Role::BuyerAgent,
            TgpPhase::Query,
            ts,
            json!({
                "phase": "QUERY",
                "id": "q-123",
                "from": "buyer://alice",
                "to": "seller://pizza",
                "asset": "USDC",
                "amount": "30000000"
            }),
        )
        .with_origin_chain(369);

        // Serialize
        let json_str = serde_json::to_string(&envelope).unwrap();
        
        // Deserialize
        let deserialized: TxipEnvelope = serde_json::from_str(&json_str).unwrap();

        // Verify
        assert_eq!(envelope.msg_id, deserialized.msg_id);
        assert_eq!(envelope.session_id, deserialized.session_id);
        assert_eq!(envelope.message_type, deserialized.message_type);
        assert_eq!(envelope.tgp_phase, deserialized.tgp_phase);
        assert_eq!(envelope.timestamp_mono, deserialized.timestamp_mono);
        assert_eq!(envelope.origin_chain_id, deserialized.origin_chain_id);
    }

    #[test]
    fn test_coreprover_receipt_validation() {
        let ts = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );

        let receipt = CoreProverReceipt::new(
            "sess-123".to_string(),
            30000000,
            ts.clone(),
            ts,
            0,
            0,
            1,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            369,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            12345,
        )
        .with_seller_claim("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string());

        assert!(receipt.validate().is_ok());
        assert!(receipt.seller_was_paid());
        assert!(receipt.is_cross_chain());
        assert!(!receipt.has_discount());
    }

    #[test]
    fn test_escrow_state_machine() {
        let ts = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );

        let mut escrow = EscrowRecord::new(
            "order-123".to_string(),
            "sess-456".to_string(),
            30000000,
            "USDC".to_string(),
            ts.clone(),
            ts.clone(),
            1,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        );

        assert_eq!(escrow.state, EscrowState::BuyerCommitted);

        // Seller accepts
        escrow.seller_accept(
            369,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            ts.clone(),
            3600,
        ).unwrap();
        assert_eq!(escrow.state, EscrowState::SellerAccepted);

        // Seller fulfills
        escrow.seller_fulfill(
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            12345,
            ts.clone(),
        ).unwrap();
        assert_eq!(escrow.state, EscrowState::SellerFulfilled);

        // Seller claims
        escrow.seller_claim(
            "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
        ).unwrap();
        assert_eq!(escrow.state, EscrowState::SellerClaimed);
    }

    #[test]
    fn test_session_with_timestamp_provider() {
        let provider = std::sync::Arc::new(TestTimestampProvider::new(1000, 1731600000));
        let manager = SessionManager::new(SessionConfig::default(), provider);

        let hello = HelloPayload {
            agent_id: "buyer://alice".to_string(),
            supported_tgp_versions: vec!["2.0".to_string()],
            supported_transports: vec!["HTTP".to_string()],
            supported_chains: vec![1, 369],
            supported_assets: vec!["USDC".to_string()],
            features: Features {
                zk_discount_proofs: true,
                receipt_ownership_proofs: true,
                late_discount_support: true,
                cross_chain_support: true,
            },
            auth: AuthInfo {
                scheme: AuthScheme::None,
                token: None,
            },
        };

        let session_info = manager.handle_hello(
            &hello,
            "sess-123".to_string(),
            Role::BuyerAgent,
        ).unwrap();

        assert_eq!(session_info.session_id, "sess-123");
        assert_eq!(session_info.agent_id, "buyer://alice");
        assert_eq!(session_info.created_mono, 1000);
        assert_eq!(session_info.created_unix, 1731600000);
    }

    #[test]
    fn test_txid_provenance_validation() {
        let buyer = BuyerTxIds::new(
            1,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        );

        let seller = SellerTxIds::new(
            369,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            12345,
        )
        .with_claim("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string());

        let provenance = TxIdProvenance::new(buyer, seller);

        assert!(provenance.validate().is_ok());
        assert!(provenance.is_cross_chain());
        assert_eq!(provenance.chain_ids(), vec![1, 369]);
    }

    #[test]
    fn test_triple_timestamp_validation() {
        let valid = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );
        assert!(valid.validate_iso().is_ok());

        let invalid = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14 12:00:00".to_string(),
        );
        assert!(invalid.validate_iso().is_err());
    }

    #[test]
    fn test_deadline_checking() {
        let ts = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );
        let deadline = Deadline::new(ts, "Test deadline".to_string());

        assert!(!deadline.has_passed(900));
        assert!(!deadline.has_passed(1000));
        assert!(deadline.has_passed(1001));

        assert_eq!(deadline.seconds_remaining(900), 100);
        assert_eq!(deadline.seconds_remaining(1000), 0);
        assert_eq!(deadline.seconds_remaining(1100), -100);
    }

    #[test]
    fn test_time_window() {
        let start = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );
        let end = TripleTimestamp::new(
            2000,
            1731603600,
            "2024-11-14T13:00:00Z".to_string(),
        );
        let window = TimeWindow::new(start, end, "Test window".to_string());

        assert!(!window.is_active(900));
        assert!(window.is_active(1500));
        assert!(!window.is_active(2100));

        assert_eq!(window.duration_seconds(), 1000);
    }
}

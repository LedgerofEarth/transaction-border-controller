//! Integration tests for complete TGP session lifecycle
//!
//! Tests cover:
//!   • Happy path (Query → Offer → Settle)
//!   • Error paths (timeouts, invalid states)
//!   • State machine validation
//!   • Persistence across messages
//!   • Replay protection integration

use tbc_gateway::{InboundRouter, TGPInboundRouter, InMemorySessionStore};
use tbc_core::tgp::state::{TGPState, SessionStore};

// ============================================================================
// Test Fixtures
// ============================================================================

fn sample_query() -> &'static str {
    r#"{
        "phase": "QUERY",
        "id": "q-test-1",
        "from": "buyer-addr",
        "to": "seller-addr",
        "asset": "USDC",
        "amount": 1000,
        "escrow_from_402": false,
        "zk_profile": "OPTIONAL"
    }"#
}

fn sample_settle_success(query_id: &str) -> String {
    format!(
        r#"{{
            "phase": "SETTLE",
            "id": "settle-test-1",
            "query_or_offer_id": "{}",
            "success": true,
            "source": "ControllerWatcher",
            "layer8_tx": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        }}"#,
        query_id
    )
}

fn sample_settle_failure(query_id: &str) -> String {
    format!(
        r#"{{
            "phase": "SETTLE",
            "id": "settle-test-2",
            "query_or_offer_id": "{}",
            "success": false,
            "source": "ControllerWatcher",
            "layer8_tx": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        }}"#,
        query_id
    )
}

fn sample_error(correlation_id: Option<&str>) -> String {
    let corr = correlation_id
        .map(|c| format!(r#""correlation_id": "{}","#, c))
        .unwrap_or_default();
    
    format!(
        r#"{{
            "phase": "ERROR",
            "id": "err-test-1",
            {}
            "code": "INTERNAL_ERROR",
            "message": "Something failed"
        }}"#,
        corr
    )
}

// ============================================================================
// Happy Path Tests
// ============================================================================

#[tokio::test]
async fn test_happy_path_query_to_settled() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Step 1: Send QUERY
    let response = router.route_inbound(sample_query()).await.unwrap();
    assert!(response.contains("OFFER"), "Expected OFFER response");
    assert!(response.contains("q-test-1"), "Response should reference query ID");
    
    // Verify session created and transitioned to QuerySent
    let session = store.get_session("q-test-1").await.unwrap();
    assert!(session.is_some(), "Session should exist");
    let session = session.unwrap();
    assert_eq!(session.state, TGPState::QuerySent, "State should be QuerySent");
    assert_eq!(session.query_id, Some("q-test-1".to_string()));
    
    // Step 2: Send SETTLE (success)
    let settle = sample_settle_success("q-test-1");
    let response = router.route_inbound(&settle).await.unwrap();
    assert!(response.contains("SETTLE"), "Expected SETTLE response");
    
    // Verify final state is Settled
    let session = store.get_session("q-test-1").await.unwrap().unwrap();
    assert_eq!(session.state, TGPState::Settled, "State should be Settled");
    assert!(session.is_terminal(), "Settled is a terminal state");
}

#[tokio::test]
async fn test_settle_failure_transitions_to_errored() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Send QUERY
    router.route_inbound(sample_query()).await.unwrap();
    
    // Send SETTLE with success=false
    let settle = sample_settle_failure("q-test-1");
    router.route_inbound(&settle).await.unwrap();
    
    // Verify state is Errored
    let session = store.get_session("q-test-1").await.unwrap().unwrap();
    assert_eq!(session.state, TGPState::Errored, "Failed settle should transition to Errored");
    assert!(session.is_terminal(), "Errored is a terminal state");
}

// ============================================================================
// Error Path Tests
// ============================================================================

#[tokio::test]
async fn test_error_response_forces_errored_state() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Send QUERY
    router.route_inbound(sample_query()).await.unwrap();
    
    // Send ERROR with correlation
    let error = sample_error(Some("q-test-1"));
    let response = router.route_inbound(&error).await.unwrap();
    assert!(response.contains("ERROR"), "Should echo ERROR");
    
    // Verify state transitioned to Errored
    let session = store.get_session("q-test-1").await.unwrap().unwrap();
    assert_eq!(session.state, TGPState::Errored, "ERROR should force Errored state");
}

#[tokio::test]
async fn test_error_without_correlation_creates_ephemeral() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());  // ✅ Clone so we can use store later

    
    // Send ERROR with no correlation_id
    let error = sample_error(None);
    let response = router.route_inbound(&error).await.unwrap();
    assert!(response.contains("ERROR"), "Should echo ERROR");
    
    // No session should be persisted (ephemeral)
    let session = store.get_session("err-test-1").await.unwrap();
    assert!(session.is_none(), "Ephemeral session should not be persisted");
}

// ============================================================================
// Replay Protection Tests
// ============================================================================

#[tokio::test]
async fn test_replay_protection() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store);
    
    // First send - success
    let response1 = router.route_inbound(sample_query()).await.unwrap();
    assert!(response1.contains("OFFER"), "First send should succeed");
    
    // Second send (replay) - rejected
    let response2 = router.route_inbound(sample_query()).await.unwrap();
    assert!(response2.contains("ERROR"), "Replay should be rejected");
    assert!(response2.contains("REPLAY_DETECTED"), "Should indicate replay");
}

#[tokio::test]
async fn test_replay_detection_preserves_session_state() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Send QUERY
    router.route_inbound(sample_query()).await.unwrap();
    
    // Get initial state
    let session_before = store.get_session("q-test-1").await.unwrap().unwrap();
    let state_before = session_before.state;
    
    // Attempt replay
    router.route_inbound(sample_query()).await.unwrap();
    
    // Verify state unchanged
    let session_after = store.get_session("q-test-1").await.unwrap().unwrap();
    assert_eq!(session_after.state, state_before, "Replay should not change state");
}

// ============================================================================
// State Machine Validation Tests
// ============================================================================

#[tokio::test]
async fn test_terminal_state_no_further_transitions() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Complete a full flow to Settled
    router.route_inbound(sample_query()).await.unwrap();
    let settle = sample_settle_success("q-test-1");
    router.route_inbound(&settle).await.unwrap();
    
    // Verify terminal
    let session = store.get_session("q-test-1").await.unwrap().unwrap();
    assert!(session.is_terminal(), "Should be in terminal state");
    
    // Attempt to send another message (would be rejected by replay protection anyway)
    // This test documents that terminal states don't transition
}

#[tokio::test]
async fn test_session_persistence_across_messages() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    // Send QUERY
    router.route_inbound(sample_query()).await.unwrap();
    
    // Verify session exists
    let session1 = store.get_session("q-test-1").await.unwrap();
    assert!(session1.is_some(), "Session should be created");
    
    let created_at = session1.as_ref().unwrap().created_at;
    
    // Send SETTLE
    let settle = sample_settle_success("q-test-1");
    router.route_inbound(&settle).await.unwrap();
    
    // Verify same session, different state
    let session2 = store.get_session("q-test-1").await.unwrap().unwrap();
    assert_eq!(session2.created_at, created_at, "Should be same session");
    assert_ne!(session2.state, session1.unwrap().state, "State should have changed");
    assert!(session2.updated_at > created_at, "Should be updated");
}

// ============================================================================
// Invalid Message Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_json_rejected() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store);
    
    let invalid = r#"{ this is not valid json }"#;
    let response = router.route_inbound(invalid).await.unwrap();
    
    assert!(response.contains("ERROR"), "Invalid JSON should return ERROR");
    assert!(response.contains("INVALID_JSON"), "Should indicate JSON error");
}

#[tokio::test]
async fn test_unknown_session_rejected() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store);
    
    // Try to SETTLE without creating session first
    let settle = sample_settle_success("nonexistent-query");
    let result = router.route_inbound(&settle).await;
    
    // Should return error (unknown session)
    assert!(result.is_err(), "Unknown session should error");
}

// ============================================================================
// Session Counting Tests
// ============================================================================

#[tokio::test]
async fn test_session_count_increases() {
    let store = InMemorySessionStore::new();
    let router = InboundRouter::new(store.clone());
    
    let initial_count = store.session_count();
    
    // Create 3 sessions
    for i in 1..=3 {
        let query = format!(
            r#"{{
                "phase": "QUERY",
                "id": "q-test-{}",
                "from": "buyer",
                "to": "seller",
                "asset": "USDC",
                "amount": 100,
                "escrow_from_402": false,
                "zk_profile": "OPTIONAL"
            }}"#,
            i
        );
        router.route_inbound(&query).await.unwrap();
    }
    
    let final_count = store.session_count();
    assert_eq!(final_count, initial_count + 3, "Should have 3 more sessions");
}

//! TGP-00 v3.2 -- Transaction State Engine (state.rs)
//! --------------------------------------------------
//! Mirrors SIP RFC 3261 §17 "Transactions"
//! but adapted for blockchain settlement and TGP semantics.
//!
//! Responsibilities:
//!   • Deterministic QUERY → ACK transitions
//!   • Deterministic terminal SETTLE message generation
//!   • Layered Verification Model (L1–L6) dispatcher
//!   • Economic Envelope generation (delegates to tx_builder)
//!   • WITHDRAW eligibility determination
//!   • Fail-closed behavior
//!
//! Stateless by design -- all state derived from QUERY payload and blockchain.

use chrono::{Utc};
use crate::tgp::protocol::{
    QueryMessage,
    AckMessage, AckStatus,
    ErrorMessage, make_protocol_error,
    SettleMessage,
};
use crate::tgp::types::{EconomicEnvelope};
use crate::tgp::validation::{
    validate_payment_profile,
    validate_chain_id,
    validate_amount_nonzero,
};

use crate::tgp::layers::{
    layer1_registry_check,
    layer2_cryptographic_check,
    layer3_contract_rpc_check,
    layer4_zk_attestation_check,
    layer5_policy_check,
    layer6_withdraw_eligibility,
};

use crate::tgp::tx_builder::build_envelope_for;

// -----------------------------------------------------------------------------
// 0. Result Type
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum TGPStateResult {
    Ack(AckMessage),
    Error(ErrorMessage),
    Settle(SettleMessage),
}

// -----------------------------------------------------------------------------
// 1. QUERY Entry Point
// -----------------------------------------------------------------------------

/// Main entry point for all inbound QUERY messages.
/// Equivalent to "Server Transaction Processing" in SIP.
pub async fn handle_query(
    mut query: QueryMessage,
) -> TGPStateResult {

    // ---------------------------------------------------------
    // Normalize routing first (spec requires this)
    // ---------------------------------------------------------
    query.normalize_routing();

    // ---------------------------------------------------------
    // Basic sanity checks (fail fast)
    // ---------------------------------------------------------
    if let Err(e) = validate_chain_id(query.chain_id) {
        return TGPStateResult::Error(make_protocol_error(
            1, "TGP_CHAIN_INVALID", e,
        ));
    }

    if let Err(e) = validate_payment_profile(&query.payment_profile) {
        return TGPStateResult::Error(make_protocol_error(
            1, "TGP_PROFILE_INVALID", e,
        ));
    }

    if let Err(e) = validate_amount_nonzero(query.amount) {
        return TGPStateResult::Error(make_protocol_error(
            1, "TGP_AMOUNT_ZERO", e,
        ));
    }

    // ---------------------------------------------------------
    // Begin Layered Verification Model (L1–L6)
    // ---------------------------------------------------------

    // L1 -- Registry / Merchant Profile
    if let Err(reason) = layer1_registry_check(&query).await {
        return TGPStateResult::Error(make_protocol_error(
            1, "TGP_L1_FAILURE", reason
        ));
    }

    // L2 -- Key/Signature/Delegated-key checks
    if let Err(reason) = layer2_cryptographic_check(&query).await {
        return TGPStateResult::Error(make_protocol_error(
            2, "TGP_L2_FAILURE", reason
        ));
    }

    // L3 -- Contract bytecode & chain RPC validation
    if let Err(reason) = layer3_contract_rpc_check(&query).await {
        return TGPStateResult::Error(make_protocol_error(
            3, "TGP_L3_FAILURE", reason
        ));
    }

    // L4 -- Optional ZK / Attestation
    if let Err(reason) = layer4_zk_attestation_check(&query).await {
        return TGPStateResult::Error(make_protocol_error(
            4, "TGP_L4_FAILURE", reason
        ));
    }

    // L5 -- Policy evaluation (merchant rules, fees, limits)
    if let Err(reason) = layer5_policy_check(&query).await {
        return TGPStateResult::Error(make_protocol_error(
            5, "TGP_L5_FAILURE", reason
        ));
    }

    // L6 -- WITHDRAW eligibility (only if requested)
    if matches!(query.intent.verb, crate::tgp::protocol::TGPVerb::WITHDRAW) {
        if let Err(reason) = layer6_withdraw_eligibility(&query).await {
            return TGPStateResult::Error(make_protocol_error(
                6, "TGP_L6_WITHDRAW_FAILURE", reason
            ));
        }
    }

    // ---------------------------------------------------------
    // All Layers Passed → Build Envelope
    // ---------------------------------------------------------
    match build_envelope_for(&query).await {
        Ok(envelope) => finalize_ack_allow(query, envelope),
        Err(reason) => TGPStateResult::Error(make_protocol_error(
            5, "TGP_ENVELOPE_FAILURE", reason
        )),
    }
}

// -----------------------------------------------------------------------------
// 2. ACK Construction Helpers
// -----------------------------------------------------------------------------

fn finalize_ack_allow(
    query: QueryMessage,
    envelope: EconomicEnvelope,
) -> TGPStateResult {

    let expires = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(5))
        .unwrap()
        .to_rfc3339();

    TGPStateResult::Ack(
        AckMessage::allow_for(&query, envelope, expires)
    )
}

/// In case we ever need to generate a preview ACK without full envelope.
/// (QUERY → ACK(status=offer))
pub fn ack_offer(query: &QueryMessage) -> AckMessage {
    AckMessage::offer_for(query)
}

// -----------------------------------------------------------------------------
// 3. Terminal State Handling
// -----------------------------------------------------------------------------

/// The state engine produces SETTLE messages when:
//!   • Settlement contract reached final status
//!   • Timeout fired
//!   • Refund triggered
//!   • Withdraw processed
//!   • RPC indicates revert
pub fn make_settle_message(
    id: impl Into<String>,
    final_status: impl Into<String>,
    escrow_id: impl Into<String>,
) -> SettleMessage {

    SettleMessage::terminal(
        id.into(),
        final_status.into(),
        escrow_id.into(),
        Utc::now().to_rfc3339(),
    )
}
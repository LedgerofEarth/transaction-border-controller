//! QUERY Handler
//!
//! Receives:  QUERY
//! Returns:   OFFER  (or ERROR)
//!
//! Controller responsibilities here:
//!   • validate QUERY
//!   • apply jurisdiction + policy rules
//!   • determine escrow path vs direct path
//!   • synthesize OFFER message
//!   • bind session to queried seller/asset

use anyhow::Result;
use tbc_core::tgp::{
    protocol::{QueryMessage, OfferMessage, TGPMessage, make_protocol_error},
    types::{ZkProfile, EconomicEnvelope},
    state::TGPSession,
    validation::*,
};
use crate::logging::*;

/// Handle inbound QUERY
pub async fn handle_inbound_query(
    meta: &crate::TGPMetadata,
    session: &TGPSession,
    q: QueryMessage,
) -> Result<TGPMessage> {

    log_handler("QUERY");

    // ------------------------------------------
    // 1. Structural validation (already done by router)
    // ------------------------------------------

    // ------------------------------------------
    // 2. Policy-level enforcement (placeholder hooks)
    // ------------------------------------------
    if !policy_allow_asset(&q.asset) {
        let err = make_protocol_error(
            Some(q.id.clone()),
            "UNSUPPORTED_ASSET",
            format!("Asset {} not supported", q.asset),
        );
        return Ok(TGPMessage::Error(err));
    }

    // ------------------------------------------
    // 3. Determine settlement method
    // ------------------------------------------
    let escrow_required = q.zk_profile.requires_escrow();

    let coreprover_contract = if escrow_required {
        // Production logic would pick contract by jurisdiction + chain
        Some("0x000000000000000000000000000000000000dead".to_string())
    } else {
        None
    };

    // ------------------------------------------
    // 4. Build OFFER
    // ------------------------------------------
    let offer = OfferMessage {
        id: format!("offer-{}", meta.msg_id),
        query_id: q.id.clone(),
        asset: q.asset.clone(),
        amount: q.amount,
        coreprover_contract,
        session_id: Some(session.id.clone()),
        zk_required: escrow_required,
        economic_envelope: EconomicEnvelope::new(50),
    };

    // Validate before returning (defensive)
    if let Err(e) = offer.validate() {
        let err = make_protocol_error(
            Some(q.id.clone()),
            "INVALID_OFFER",
            format!("Controller produced invalid OFFER: {}", e),
        );
        return Ok(TGPMessage::Error(err));
    }

    // ------------------------------------------
    // 5. Return OFFER
    // ------------------------------------------
    Ok(TGPMessage::Offer(offer))
}


// -----------------------------------------------------
// Placeholder policy checks
// -----------------------------------------------------

fn policy_allow_asset(asset: &str) -> bool {
    // Later: jurisdiction + chain + business rules
    asset == "USDC" || asset == "ETH" || asset == "USDT"
}
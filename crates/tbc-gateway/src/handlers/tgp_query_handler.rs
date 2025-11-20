//! QUERY Handler (Pure Function)
//!
//! Receives:  QUERY
//! Returns:   OFFER  (or ERROR)
//!
//! Responsibilities:
//!   • validate QUERY semantics beyond structural validation
//!   • determine recommended settlement path (escrow vs direct)
//!   • synthesize OFFER message
//!   • provide economic envelope & zk_required guidance
//!   • no session mutation (SIP-style purity)
//!
//! NOTE: Session transitions occur in the Router, not in handlers.

use anyhow::Result;

use tbc_core::{
    protocol::{QueryMessage, OfferMessage, TGPMessage, make_protocol_error},
    tgp::{
        codec_tx::TGPMetadata,
        state::TGPSession,
        types::{EconomicEnvelope, ZkProfile},
    },
};

use crate::logging::*;

/// Handle inbound QUERY → returns OFFER
pub async fn handle_inbound_query(
    meta: &TGPMetadata,
    _session: &TGPSession,
    q: QueryMessage,
) -> Result<TGPMessage> 
{
    log_handler("QUERY");

    // ----------------------------------------------------------
    // 1. Application-layer checks (controller policy stub)
    // ----------------------------------------------------------
    if !asset_supported(&q.asset) {
        let err = make_protocol_error(
            Some(q.id.clone()),
            "UNSUPPORTED_ASSET",
            format!("Asset {} not supported", q.asset),
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 2. Determine settlement path (escrow vs direct)
    // ----------------------------------------------------------
    let escrow_required = matches!(q.zk_profile, ZkProfile::Required);

    // Placeholder contract selection logic
    let coreprover_contract = if escrow_required {
        Some("0x000000000000000000000000000000000000F00D".to_string())
    } else {
        None
    };

    // ----------------------------------------------------------
    // 3. Construct OFFER
    // ----------------------------------------------------------
    let offer = OfferMessage {
        id: format!("offer-{}", meta.msg_id),
        query_id: q.id.clone(),
        asset: q.asset.clone(),
        amount: q.amount,
        coreprover_contract,
        session_id: Some(meta.msg_id.clone()),   // Router enforces canonical mapping
        zk_required: escrow_required,
        economic_envelope: EconomicEnvelope::new(50),
    };

    // Validate OFFER defensively
    if let Err(e) = offer.validate() {
        let err = make_protocol_error(
            Some(q.id.clone()),
            "INVALID_OFFER",
            format!("Generated OFFER invalid: {}", e),
        );
        return Ok(TGPMessage::Error(err));
    }

    // ----------------------------------------------------------
    // 4. Return OFFER
    // ----------------------------------------------------------
    Ok(TGPMessage::Offer(offer))
}

// ----------------------------------------------------------
// Internal placeholder policy hooks
// ----------------------------------------------------------
fn asset_supported(asset: &str) -> bool {
    matches!(asset, "USDC" | "USDT" | "ETH" | "DAI")
}
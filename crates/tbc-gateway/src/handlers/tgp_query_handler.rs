//! tgp_query_handlers.rs -- TGP-00 v3.2 QUERY Handler
//! -------------------------------------------------
//! In TGP v3.2, QUERY is the only client → gateway message.
//! Gateways MUST:
//!   • Perform layered verification (L1–L6)
//!   • Return ACK(status="offer") when preliminary checks pass
//!   • Return ACK(status="allow") with an Economic Envelope after full verification
//!   • Return ERROR on any failure
//!
//! No session state is created or persisted.

use anyhow::{anyhow, Result};

use tbc_core::{
    codec_tx::TGPMetadata,
    protocol::{TGPMessage, make_protocol_error},
    tgp::query::QueryMessage,
    tgp::ack::{AckMessage, AckStatus},
    tgp::economic::EconomicEnvelope,
};

// Logging
use crate::logging::{log_handler, log_info, log_err};


/// ---------------------------------------------------------------------------
/// QUERY Handler -- Stateless, Deterministic
/// ---------------------------------------------------------------------------
pub async fn handle_inbound_query(
    meta: &TGPMetadata,
    q: QueryMessage,
) -> Result<TGPMessage>
{
    log_handler("QUERY");

    // ================================================================
    // 1. STRUCTURAL VALIDATION
    // ================================================================
    if let Err(e) = q.validate() {
        let err = make_protocol_error(Some(q.id.clone()), "TGP_INVALID_QUERY", e);
        log_err(&err);
        return Ok(TGPMessage::Error(err));
    }

    log_info!(
        target: "tgp.query",
        {
            "id": q.id.clone(),
            "verb": q.intent.verb.clone(),
            "party": q.intent.party.clone(),
            "chain": q.chain_id,
            "profile": q.payment_profile.clone()
        },
        "Inbound QUERY parsed"
    );

    // ================================================================
    // 2. LAYERED VERIFICATION (L1 → L6)
    // ================================================================
    //
    // For now these are stubs. They MUST be implemented
    // in `tgp_core::layers::*` as pure deterministic checks.
    //
    // Any failure → return an ERROR.
    //

    if let Err(e) = verify_l1_registry(&q).await {
        return error_layer(&q, 1, e);
    }

    if let Err(e) = verify_l2_crypto(&q).await {
        return error_layer(&q, 2, e);
    }

    if let Err(e) = verify_l3_contracts(&q).await {
        return error_layer(&q, 3, e);
    }

    if let Err(e) = verify_l4_zk(&q).await {
        return error_layer(&q, 4, e);
    }

    if let Err(e) = verify_l5_policy(&q).await {
        return error_layer(&q, 5, e);
    }

    if q.intent.verb == "WITHDRAW" {
        if let Err(e) = verify_l6_withdraw_eligibility(&q).await {
            return error_layer(&q, 6, e);
        }
    }

    // ================================================================
    // 3. PREVIEW PHASE -- ACK(status="offer")
    // ================================================================
    //
    // TGP v3.2 defines a deterministic preview phase. 
    // This MUST be returned before the executable envelope is produced.
    //
    let preview = AckMessage::offer(&q);

    log_info!(
        target: "tgp.query",
        {
            "id": q.id.clone(),
            "status": "offer"
        },
        "Returning ACK(status=offer)"
    );

    // Client MAY submit same QUERY again to obtain allow+envelope.
    if q.intent.mode == "DIRECT" {
        // Direct mode always requires a second QUERY for allow.
        return Ok(TGPMessage::Ack(preview));
    }

    // Shielded mode MAY jump directly to allow after preview.
    // In v3.2 we retain preview semantics but can optionally fall through.
    // Fallthrough continues to allow-phase processing.

    // ================================================================
    // 4. ECONOMIC ENVELOPE CONSTRUCTION (for ACK=allow)
    // ================================================================
    let envelope = match build_economic_envelope(&q).await {
        Ok(env) => env,
        Err(e) => {
            return error_layer(&q, 5, format!("Failed to build economic envelope: {}", e));
        }
    };

    let allow = AckMessage::allow(&q, envelope);

    log_info!(
        target: "tgp.query",
        {
            "id": q.id.clone(),
            "status": "allow"
        },
        "Returning ACK(status=allow) with Economic Envelope"
    );

    Ok(TGPMessage::Ack(allow))
}



/// =======================================================================
/// LAYER HELPERS (L1 → L6)
/// =======================================================================
/// These are stubs. Replace with real implementations.

async fn verify_l1_registry(_q: &QueryMessage) -> Result<()> { Ok(()) }
async fn verify_l2_crypto(_q: &QueryMessage) -> Result<()> { Ok(()) }
async fn verify_l3_contracts(_q: &QueryMessage) -> Result<()> { Ok(()) }
async fn verify_l4_zk(_q: &QueryMessage) -> Result<()> { Ok(()) }
async fn verify_l5_policy(_q: &QueryMessage) -> Result<()> { Ok(()) }
async fn verify_l6_withdraw_eligibility(_q: &QueryMessage) -> Result<()> { Ok(()) }


/// =======================================================================
/// ECONOMIC ENVELOPE BUILDER (placeholder)
/// =======================================================================
async fn build_economic_envelope(q: &QueryMessage) -> Result<EconomicEnvelope> {
    Ok(EconomicEnvelope {
        to: q.payment_profile.clone(),
        value: q.amount,
        data: "0x".into(),
        chain_id: q.chain_id,
        gas_limit: 250000,
        rpc_url: Some("https://rpc.example".into()),
        tbc_endpoint: Some("https://gateway.example".into()),
        expires_at: None,
        fees_bps: 0,
    })
}


/// =======================================================================
/// ERROR BUILDER
/// =======================================================================
fn error_layer(q: &QueryMessage, layer: u8, msg: impl Into<String>)
    -> Result<TGPMessage>
{
    let err = make_protocol_error(
        Some(q.id.clone()),
        format!("TGP_L{}_FAILURE", layer),
        msg.into()
    );

    log_err(&err);

    Ok(TGPMessage::Error(err))
}
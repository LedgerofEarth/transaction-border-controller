use anyhow::Result;
use crate::router::{InboundRouter, TGPInboundRouter};

/// Dispatches a WS JSON string → TGP router → encoded output.
/// 
/// SECURITY: Uses the same InboundRouter as HTTP path.
/// Per TGP-TBC-SEC-00, all verification layers (L1-L6) are evaluated.
/// Fail-closed: any error results in rejection.
pub async fn route_ws_message(json: &str) -> Result<String> {
    // Use the same stateless router as HTTP endpoint
    // This ensures identical security verification for both transports
    let router = InboundRouter::new();
    
    // Route through full verification pipeline
    // InboundRouter handles: classify → validate → replay check → dispatch → encode
    router.route_inbound(json).await
}
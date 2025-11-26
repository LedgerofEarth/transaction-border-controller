// crates/tbc-gateway/src/txip/txip_types_v03.rs
// FINAL - TxIP v0.2 Envelope Types with CoreProver v0.3 Compatibility
//
// This module defines TxIP message envelopes that transport TGP messages
// with proper triple-clock timestamps and TXID provenance tracking.
//
// NO Instant, Duration, or SystemTime usage - all timestamps from engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::blockchain_types_v03::ChainId;
use super::timestamp_types_v03::TripleTimestamp;

/// TxIP protocol version
pub const TXIP_VERSION: &str = "0.2";

/// TxIP message envelope with v0.3 compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxipEnvelope {
    /// Protocol version
    pub txip_version: String,
    
    /// Unique message identifier (UUID v4 recommended)
    pub msg_id: String,
    
    /// Session identifier tying related messages together
    pub session_id: String,
    
    /// Message direction hint
    pub direction: Direction,
    
    /// Sender's role
    pub role: Role,
    
    /// Message timestamp (triple-clock from engine)
    pub timestamp_mono: u64,
    pub timestamp_unix: u64,
    pub timestamp_iso: String,
    
    /// Type of message
    pub message_type: MessageType,
    
    /// TGP phase if applicable
    pub tgp_phase: TgpPhase,
    
    /// Specific TGP event type if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tgp_type: Option<String>,
    
    /// Origin chain ID for blockchain-aware routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_chain_id: Option<ChainId>,
    
    /// Message payload
    pub payload: Payload,
}

/// Message direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    ClientToTbc,
    TbcToClient,
    TbcToTbc,
}

/// Sender role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    BuyerAgent,
    SellerAgent,
    Tbc,
    Watcher,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Control,
    Tgp,
    Error,
}

/// TGP phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TgpPhase {
    Query,
    Offer,
    Settle,
    Event,
    None,
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Control(ControlPayload),
    Tgp(TgpPayload),
    Error(ErrorPayload),
}

/// Control message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "control_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ControlPayload {
    Hello(HelloPayload),
    Welcome(WelcomePayload),
    Heartbeat(HeartbeatPayload),
    Close(ClosePayload),
}

/// HELLO control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    pub agent_id: String,
    pub supported_tgp_versions: Vec<String>,
    pub supported_transports: Vec<String>,
    pub supported_chains: Vec<ChainId>,
    pub supported_assets: Vec<String>,
    pub features: Features,
    pub auth: AuthInfo,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    pub zk_discount_proofs: bool,
    pub receipt_ownership_proofs: bool,
    pub late_discount_support: bool,
    pub cross_chain_support: bool,
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub scheme: AuthScheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Authentication scheme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthScheme {
    BearerJwt,
    ApiKey,
    Mtls,
    None,
}

/// WELCOME control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomePayload {
    pub tbc_id: String,
    pub session_id: String,
    pub negotiated_tgp_version: String,
    pub negotiated_chains: Vec<ChainId>,
    pub negotiated_features: Features,
    pub heartbeat_interval_sec: u64,
}

/// HEARTBEAT control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub seq: u64,
}

/// CLOSE control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosePayload {
    pub reason: CloseReason,
}

/// Close reason
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    IdleTimeout,
    ClientShutdown,
    ProtocolError,
    Other,
}

/// TGP message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgpPayload {
    /// Raw TGP message as defined in TGP-00
    pub tgp: Value,
}

/// Error message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub error_code: ErrorCode,
    pub http_status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_msg_id: Option<String>,
    pub details: String,
    pub retryable: bool,
}

/// TxIP error codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    TxipInvalidEnvelope,
    TxipUnsupportedVersion,
    TxipUnauthenticated,
    TxipUnauthorized,
    TxipInternalError,
    TxipRateLimited,
    TxipUpstreamUnavailable,
    TxipMalformedTgpPayload,
}

impl TxipEnvelope {
    /// Create a new TxIP envelope with engine-provided timestamp
    pub fn new(
        msg_id: String,
        session_id: String,
        direction: Direction,
        role: Role,
        message_type: MessageType,
        tgp_phase: TgpPhase,
        timestamp: TripleTimestamp,
        payload: Payload,
    ) -> Self {
        Self {
            txip_version: TXIP_VERSION.to_string(),
            msg_id,
            session_id,
            direction,
            role,
            timestamp_mono: timestamp.mono,
            timestamp_unix: timestamp.unix,
            timestamp_iso: timestamp.iso,
            message_type,
            tgp_phase,
            tgp_type: None,
            origin_chain_id: None,
            payload,
        }
    }

    /// Create a TGP message envelope
    pub fn tgp(
        msg_id: String,
        session_id: String,
        direction: Direction,
        role: Role,
        tgp_phase: TgpPhase,
        timestamp: TripleTimestamp,
        tgp_message: Value,
    ) -> Self {
        Self::new(
            msg_id,
            session_id,
            direction,
            role,
            MessageType::Tgp,
            tgp_phase,
            timestamp,
            Payload::Tgp(TgpPayload { tgp: tgp_message }),
        )
    }

    /// Create an error envelope
    pub fn error(
        msg_id: String,
        session_id: String,
        direction: Direction,
        error_code: ErrorCode,
        http_status: u16,
        related_msg_id: Option<String>,
        details: String,
        retryable: bool,
        timestamp: TripleTimestamp,
    ) -> Self {
        Self::new(
            msg_id,
            session_id,
            direction,
            Role::Tbc,
            MessageType::Error,
            TgpPhase::None,
            timestamp,
            Payload::Error(ErrorPayload {
                error_code,
                http_status,
                related_msg_id,
                details,
                retryable,
            }),
        )
    }

    /// Create a WELCOME control message
    pub fn welcome(
        msg_id: String,
        session_id: String,
        tbc_id: String,
        negotiated_tgp_version: String,
        negotiated_chains: Vec<ChainId>,
        negotiated_features: Features,
        heartbeat_interval_sec: u64,
        timestamp: TripleTimestamp,
    ) -> Self {
        Self::new(
            msg_id,
            session_id,
            Direction::TbcToClient,
            Role::Tbc,
            MessageType::Control,
            TgpPhase::None,
            timestamp,
            Payload::Control(ControlPayload::Welcome(WelcomePayload {
                tbc_id,
                session_id,
                negotiated_tgp_version,
                negotiated_chains,
                negotiated_features,
                heartbeat_interval_sec,
            })),
        )
    }

    /// Add origin chain ID for blockchain-aware routing
    pub fn with_origin_chain(mut self, chain_id: ChainId) -> Self {
        self.origin_chain_id = Some(chain_id);
        self
    }

    /// Add TGP event type
    pub fn with_tgp_type(mut self, tgp_type: String) -> Self {
        self.tgp_type = Some(tgp_type);
        self
    }

    /// Get timestamp as TripleTimestamp
    pub fn timestamp(&self) -> TripleTimestamp {
        TripleTimestamp::new(
            self.timestamp_mono,
            self.timestamp_unix,
            self.timestamp_iso.clone(),
        )
    }

    /// Validate envelope structure
    pub fn validate(&self) -> Result<(), String> {
        if self.txip_version != TXIP_VERSION {
            return Err(format!("Unsupported TxIP version: {}", self.txip_version));
        }

        if self.msg_id.is_empty() {
            return Err("msg_id cannot be empty".to_string());
        }

        if self.session_id.is_empty() {
            return Err("session_id cannot be empty".to_string());
        }

        // Validate timestamp ISO format
        let ts = self.timestamp();
        ts.validate_iso()?;

        // Validate message_type matches payload
        match (&self.message_type, &self.payload) {
            (MessageType::Control, Payload::Control(_)) => Ok(()),
            (MessageType::Tgp, Payload::Tgp(_)) => Ok(()),
            (MessageType::Error, Payload::Error(_)) => Ok(()),
            _ => Err("message_type does not match payload".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_timestamp() -> TripleTimestamp {
        TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_envelope_creation() {
        let ts = create_test_timestamp();
        let envelope = TxipEnvelope::tgp(
            "msg-123".to_string(),
            "sess-456".to_string(),
            Direction::ClientToTbc,
            Role::BuyerAgent,
            TgpPhase::Query,
            ts,
            json!({"phase": "QUERY", "id": "q-123"}),
        );

        assert_eq!(envelope.msg_id, "msg-123");
        assert_eq!(envelope.timestamp_mono, 1000);
        assert_eq!(envelope.timestamp_unix, 1731600000);
    }

    #[test]
    fn test_envelope_with_chain_id() {
        let ts = create_test_timestamp();
        let envelope = TxipEnvelope::tgp(
            "msg-123".to_string(),
            "sess-456".to_string(),
            Direction::ClientToTbc,
            Role::BuyerAgent,
            TgpPhase::Query,
            ts,
            json!({"phase": "QUERY"}),
        )
        .with_origin_chain(369);

        assert_eq!(envelope.origin_chain_id, Some(369));
    }

    #[test]
    fn test_envelope_validation() {
        let ts = create_test_timestamp();
        let valid = TxipEnvelope::tgp(
            "msg-123".to_string(),
            "sess-456".to_string(),
            Direction::ClientToTbc,
            Role::BuyerAgent,
            TgpPhase::Query,
            ts,
            json!({"phase": "QUERY"}),
        );

        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_error_envelope() {
        let ts = create_test_timestamp();
        let error = TxipEnvelope::error(
            "err-123".to_string(),
            "sess-456".to_string(),
            Direction::TbcToClient,
            ErrorCode::TxipInvalidEnvelope,
            400,
            Some("msg-789".to_string()),
            "Test error".to_string(),
            false,
            ts,
        );

        assert_eq!(error.message_type, MessageType::Error);
        assert!(error.validate().is_ok());
    }
}

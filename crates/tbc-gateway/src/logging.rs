//! Unified logging module for TBC Gateway
//!
//! This module implements telecom-grade structured logging for all TGP
//! operations. It supports:
//!   • JSON logging mode (TBC_LOG_FORMAT=json)
//!   • ANSI-safe color output for terminals
//!   • RFC3339 timestamps with microseconds (SIP-style diagnostics)
//!   • State-transition visualization (Observer pattern)
//!   • Full TGP metadata logging hooks
//!   • Router-level RX/TX logs
//!
//! Design notes:
//!   • SIP RFC3261 encourages splitting parsing, transaction,
//!     and transport layers -- but logging remains centralized.
//!   • Following that guidance, TBC centralizes all visibility here,
//!     while keeping business logic clean.

use ansi_term::Colour::*;
use chrono::Utc;
use serde_json::json;
use std::env;

// ============================================================================
// Logging Macros
// ============================================================================

/// Emit error-level log with structured fields
#[macro_export]
macro_rules! log_error {
    (target: $target:expr, $fields:tt, $msg:expr) => {
        $crate::logging::error($msg, serde_json::json!($fields));
    };
}

/// Emit info-level log with structured fields
#[macro_export]
macro_rules! log_info {
    (target: $target:expr, $fields:tt, $msg:expr) => {
        $crate::logging::info($msg, serde_json::json!($fields));
    };
}

/// Whether JSON mode is enabled (TBC_LOG_FORMAT=json)
fn json_mode() -> bool {
    env::var("TBC_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false)
}

/// RFC3339 timestamp w/ microseconds (SIP-grade granularity)
fn ts() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Emit a unified log entry
pub fn log_event(level: &str, msg: &str, fields: serde_json::Value) {
    if json_mode() {
        let payload = json!({
            "ts": ts(),
            "level": level,
            "msg": msg,
            "fields": fields
        });
        println!("{}", payload.to_string());
        return;
    }

    // ANSI fallback mode
    let lvl = match level {
        "ERROR" => Red.paint("ERROR").to_string(),
        "WARN"  => Yellow.paint("WARN").to_string(),
        "INFO"  => Cyan.paint("INFO").to_string(),
        "DEBUG" => Purple.paint("DEBUG").to_string(),
        "TRACE" => White.paint("TRACE").to_string(),
        _ => level.to_string(),
    };

    println!("[{}] {} | {}", ts(), lvl, msg);
}

/// Convenience wrappers
pub fn info(msg: &str, fields: serde_json::Value) {
    log_event("INFO", msg, fields);
}

pub fn warn(msg: &str, fields: serde_json::Value) {
    log_event("WARN", msg, fields);
}

pub fn error(msg: &str, fields: serde_json::Value) {
    log_event("ERROR", msg, fields);
}

pub fn debug(msg: &str, fields: serde_json::Value) {
    log_event("DEBUG", msg, fields);
}

pub fn trace(msg: &str, fields: serde_json::Value) {
    log_event("TRACE", msg, fields);
}

// =============================================================================
// RX / TX Logging
// =============================================================================

/// Log raw inbound JSON (pre-parse)
pub fn log_rx(json: &str) {
    trace(
        "Inbound JSON",
        json!({ "payload": json })
    );
}

/// Log outbound response JSON
pub fn log_tx(json: &str) {
    trace(
        "Outbound JSON",
        json!({ "payload": json })
    );
}

/// Log protocol-level error before it is encoded
pub fn log_err(err: &tbc_core::tgp::protocol::ErrorMessage) {
    error(
        "Protocol Error",
        json!({
            "id": err.id,
            "code": err.code,
            "message": err.message,
            "correlation_id": err.correlation_id
        })
    );
}

// =============================================================================
// Session & Handler Logging
// =============================================================================

/// Log session creation event
pub fn log_session_created(session: &tbc_core::tgp::state::TGPSession) {
    info(
        "session-created",
        json!({
            "session_id": session.session_id,
            "state": format!("{:?}", session.state),
            "created_at": session.created_at,
        })
    );
}

/// Log handler entry point
pub fn log_handler(phase: &str) {
    debug(
        "handler-entry",
        json!({ "phase": phase })
    );
}

/// Create tracing span for TGP handler
///
/// Requires tracing crate in Cargo.toml
pub fn tgp_span(session_id: &str, phase: &str) -> tracing::Span {
    tracing::info_span!(
        "tgp-handler",
        session_id = %session_id,
        phase = %phase
    )
}

// =============================================================================
// State Transition Logging (Observer Pattern)
// =============================================================================

/// Log state transitions (TGPSession)
///
/// Called by state.rs via an observer.
/// This keeps `state.rs` clean and side-effect-free.
///
/// Colors:
///   • from-state = Green
///   • to-state   = Blue
///   • action tag = Cyan
pub fn log_state_transition(
    session_id: &str,
    from: &str,
    to: &str,
) {
    if json_mode() {
        info(
            "state-change",
            json!({
                "session_id": session_id,
                "from": from,
                "to": to
            }),
        );
        return;
    }

    let msg = format!(
        "{} {} → {}",
        Cyan.paint("state-change"),
        Green.paint(from),
        Blue.paint(to)
    );

    info(
        &msg,
        json!({
            "session_id": session_id,
            "from": from,
            "to": to
        }),
    );
}

/// Dump full SSO (state storage object) for debugging
pub fn trace_sso(session_id: &str, sso_json: serde_json::Value) {
    trace(
        "SSO dump",
        json!({
            "session_id": session_id,
            "sso": sso_json
        }),
    );
}

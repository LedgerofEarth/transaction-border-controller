//! Unified logging module for TBC Gateway
//!
//! Provides:
//!   • JSON logging (TBC_LOG_FORMAT=json)
//!   • ANSI color output for terminals
//!   • RFC3339 timestamps w/ microseconds
//!   • RX/TX logs
//!   • Session transition logs (observer)
//!   • TGP handler logs

// ============================================================================
// Imports
// ============================================================================

use ansi_term::Colour::{Red, Yellow, Cyan, Purple, White, Green, Blue};
use chrono::{Utc, SecondsFormat};
use serde_json::json;
use std::env;

use tbc_core::protocol::ErrorMessage;

// ============================================================================
// Logging Macros
// ============================================================================

#[macro_export]
macro_rules! log_error {
    (target: $target:expr, $fields:tt, $msg:expr) => {
        $crate::logging::error($msg, serde_json::json!($fields));
    };
}

#[macro_export]
macro_rules! log_info {
    (target: $target:expr, $fields:tt, $msg:expr) => {
        $crate::logging::info($msg, serde_json::json!($fields));
    };
}

// ============================================================================
// JSON Mode Detection
// ============================================================================

fn json_mode() -> bool {
    env::var("TBC_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false)
}

// ============================================================================
// Timestamp
// ============================================================================

fn ts() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)
}

// ============================================================================
// Core Log Event
// ============================================================================

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

// ============================================================================
// Convenience Wrappers
// ============================================================================

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

// ============================================================================
// RX / TX Logging
// ============================================================================

pub fn log_rx(json_raw: &str) {
    trace("Inbound JSON", json!({ "payload": json_raw }));
}

pub fn log_tx(json_raw: &str) {
    trace("Outbound JSON", json!({ "payload": json_raw }));
}

pub fn log_err(err: &ErrorMessage) {
    error(
        "Protocol Error",
        json!({
            "id": err.id,
            "code": err.code,
            "message": err.message,
            "layer_failed": err.layer_failed
        })
    );
}

// ============================================================================
// Session & Handler Logging
// ============================================================================

// TODO: Re-enable when TGPSession is available
// pub fn log_session_created(session: &TGPSession) {
//     info(
//         "session-created",
//         json!({
//             "session_id": session.session_id,
//             "state": format!("{:?}", session.state),
//             "created_at": session.created_at,
//         })
//     );
// }

pub fn log_handler(phase: &str) {
    debug("handler-entry", json!({ "phase": phase }));
}

pub fn tgp_span(session_id: &str, phase: &str) -> tracing::Span {
    tracing::info_span!(
        "tgp-handler",
        session_id = %session_id,
        phase = %phase
    )
}

// ============================================================================
// State Transition Logging (Observer Pattern)
// ============================================================================

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

pub fn trace_sso(session_id: &str, sso_json: serde_json::Value) {
    trace(
        "SSO dump",
        json!({
            "session_id": session_id,
            "sso": sso_json
        }),
    );
}
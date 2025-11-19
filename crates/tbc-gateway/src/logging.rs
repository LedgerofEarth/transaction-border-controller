//! Unified logging module for TBC Gateway
//!
//! Features:
//! - JSON logging mode (`TBC_LOG_FORMAT=json`)
//! - ANSI-safe colored output
//! - tracing::span integration
//! - Structured fields for TGP message lifecycle
//! - Telecom-grade timestamping

use ansi_term::Colour::*;
use chrono::Utc;
use serde_json::json;
use std::env;
use tracing::{event, span, Level};

/// Determine if JSON mode is enabled
fn json_mode() -> bool {
    env::var("TBC_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false)
}

/// Telecom-grade timestamp (RFC3339 + microseconds)
fn ts() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Emit a log entry
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

    // ANSI-colored fallback
    let line = match level {
        "ERROR" => Red.paint(msg).to_string(),
        "WARN"  => Yellow.paint(msg).to_string(),
        "INFO"  => Cyan.paint(msg).to_string(),
        "DEBUG" => Purple.paint(msg).to_string(),
        "TRACE" => White.paint(msg).to_string(),
        _ => msg.to_string(),
    };

    println!("[{}] {} | {}", ts(), level, line);
}

/// Logging helpers
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

/// Create a tracing span for TGP operations
pub fn tgp_span(session: &str, phase: &str) -> tracing::Span {
    span!(
        Level::INFO,
        "tgp",
        session = session,
        phase = phase
    )
}

/// Colored state transition logging
pub fn log_state_transition(
    session_id: &str,
    from: &str,
    to: &str,
) {
    let colored = format!(
        "{} {} → {}",
        Cyan.paint("state-change"),
        Green.paint(from),
        Blue.paint(to)
    );

    info(
        &colored,
        json!({
            "session_id": session_id,
            "from": from,
            "to": to
        }),
    );
}

/// TRACE: dump full SSO
pub fn trace_sso(session_id: &str, sso_json: serde_json::Value) {
    trace(
        "SSO dump",
        json!({
            "session_id": session_id,
            "sso": sso_json
        }),
    );
}
// crates/tbc-gateway/src/txip/timestamp_types_v03.rs
// FINAL - CoreProver v0.3 Triple-Clock Timestamp Types
//
// This module defines the canonical triple-clock timestamp model:
// - monotonic clock (mono): u64 seconds since arbitrary epoch
// - unix timestamp (unix): u64 seconds since Unix epoch
// - ISO8601 timestamp (iso): String in RFC3339 format
//
// NO Instant, Duration, or SystemTime allowed.
// All timestamps must be provided by the engine.

use serde::{Deserialize, Serialize};

/// Triple-clock timestamp representation
/// All three clocks must be synchronized and provided by the engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TripleTimestamp {
    /// Monotonic clock in seconds (arbitrary epoch, never goes backwards)
    pub mono: u64,
    
    /// Unix timestamp in seconds (seconds since 1970-01-01T00:00:00Z)
    pub unix: u64,
    
    /// ISO8601 timestamp string (RFC3339 format)
    pub iso: String,
}

impl TripleTimestamp {
    /// Create a new triple timestamp
    /// 
    /// # Arguments
    /// * `mono` - Monotonic seconds
    /// * `unix` - Unix seconds
    /// * `iso` - ISO8601 string
    pub fn new(mono: u64, unix: u64, iso: String) -> Self {
        Self { mono, unix, iso }
    }

    /// Validate that the ISO string is well-formed
    pub fn validate_iso(&self) -> Result<(), String> {
        // Basic validation: must contain 'T' and end with 'Z' or timezone
        if !self.iso.contains('T') {
            return Err("ISO timestamp must contain 'T' separator".to_string());
        }
        
        if !self.iso.ends_with('Z') && !self.iso.contains('+') && !self.iso.contains('-') {
            return Err("ISO timestamp must have timezone (Z or +/-)".to_string());
        }
        
        Ok(())
    }
}

/// Timestamp provider trait
/// Must be implemented by the engine to provide synchronized triple-clock values
pub trait TimestampProvider {
    /// Get current triple timestamp from engine
    fn now(&self) -> TripleTimestamp;
    
    /// Get triple timestamp at specific unix time
    fn at_unix(&self, unix: u64) -> TripleTimestamp;
    
    /// Get triple timestamp at specific monotonic time
    fn at_mono(&self, mono: u64) -> TripleTimestamp;
}

/// Deadline representation with triple-clock
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deadline {
    /// When the deadline occurs
    pub timestamp: TripleTimestamp,
    
    /// Human-readable description
    pub description: String,
}

impl Deadline {
    /// Create a new deadline
    pub fn new(timestamp: TripleTimestamp, description: String) -> Self {
        Self {
            timestamp,
            description,
        }
    }

    /// Check if deadline has passed (based on monotonic clock)
    pub fn has_passed(&self, current_mono: u64) -> bool {
        current_mono > self.timestamp.mono
    }

    /// Check if deadline has passed (based on unix time)
    pub fn has_passed_unix(&self, current_unix: u64) -> bool {
        current_unix > self.timestamp.unix
    }

    /// Seconds remaining until deadline (monotonic)
    pub fn seconds_remaining(&self, current_mono: u64) -> i64 {
        (self.timestamp.mono as i64) - (current_mono as i64)
    }
}

/// Time window with start and end
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeWindow {
    /// Window start time
    pub start: TripleTimestamp,
    
    /// Window end time
    pub end: TripleTimestamp,
    
    /// Window description
    pub description: String,
}

impl TimeWindow {
    /// Create a new time window
    pub fn new(start: TripleTimestamp, end: TripleTimestamp, description: String) -> Self {
        Self {
            start,
            end,
            description,
        }
    }

    /// Check if current time is within window (monotonic)
    pub fn is_active(&self, current_mono: u64) -> bool {
        current_mono >= self.start.mono && current_mono <= self.end.mono
    }

    /// Check if window has started (monotonic)
    pub fn has_started(&self, current_mono: u64) -> bool {
        current_mono >= self.start.mono
    }

    /// Check if window has ended (monotonic)
    pub fn has_ended(&self, current_mono: u64) -> bool {
        current_mono > self.end.mono
    }

    /// Duration of window in seconds
    pub fn duration_seconds(&self) -> u64 {
        self.end.mono.saturating_sub(self.start.mono)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_timestamp() -> TripleTimestamp {
        TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        )
    }

    #[test]
    fn test_triple_timestamp_creation() {
        let ts = create_test_timestamp();
        assert_eq!(ts.mono, 1000);
        assert_eq!(ts.unix, 1731600000);
        assert_eq!(ts.iso, "2024-11-14T12:00:00Z");
    }

    #[test]
    fn test_iso_validation() {
        let valid = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );
        assert!(valid.validate_iso().is_ok());

        let invalid_no_t = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14 12:00:00Z".to_string(),
        );
        assert!(invalid_no_t.validate_iso().is_err());

        let invalid_no_tz = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00".to_string(),
        );
        assert!(invalid_no_tz.validate_iso().is_err());
    }

    #[test]
    fn test_deadline() {
        let ts = create_test_timestamp();
        let deadline = Deadline::new(ts, "Test deadline".to_string());

        // Before deadline
        assert!(!deadline.has_passed(900));
        assert_eq!(deadline.seconds_remaining(900), 100);

        // At deadline
        assert!(!deadline.has_passed(1000));
        assert_eq!(deadline.seconds_remaining(1000), 0);

        // After deadline
        assert!(deadline.has_passed(1100));
        assert_eq!(deadline.seconds_remaining(1100), -100);
    }

    #[test]
    fn test_time_window() {
        let start = TripleTimestamp::new(
            1000,
            1731600000,
            "2024-11-14T12:00:00Z".to_string(),
        );
        let end = TripleTimestamp::new(
            2000,
            1731603600,
            "2024-11-14T13:00:00Z".to_string(),
        );
        let window = TimeWindow::new(start, end, "Test window".to_string());

        // Before window
        assert!(!window.has_started(900));
        assert!(!window.is_active(900));
        assert!(!window.has_ended(900));

        // During window
        assert!(window.has_started(1500));
        assert!(window.is_active(1500));
        assert!(!window.has_ended(1500));

        // After window
        assert!(window.has_started(2100));
        assert!(!window.is_active(2100));
        assert!(window.has_ended(2100));

        // Duration
        assert_eq!(window.duration_seconds(), 1000);
    }
}

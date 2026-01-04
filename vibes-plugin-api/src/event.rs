//! FFI-safe event types for plugin callbacks.
//!
//! These types use simple primitives and JSON serialization to safely cross
//! the host-plugin boundary without TypeId mismatches.

use serde::{Deserialize, Serialize};

/// A raw event suitable for FFI across the host-plugin boundary.
///
/// This is a simplified, serialization-friendly view of `StoredEvent` that
/// avoids TypeId issues when crossing dynamic library boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    /// UUID v7 event ID as raw bytes (16 bytes).
    pub event_id: [u8; 16],

    /// Unix timestamp in milliseconds when the event was stored.
    pub timestamp_ms: u64,

    /// Session ID if the event is session-scoped.
    pub session_id: Option<String>,

    /// Event type name (e.g., "SessionCreated", "TextDelta", "ToolResult").
    pub event_type: String,

    /// JSON-serialized event payload for type-specific data.
    pub payload: String,
}

impl RawEvent {
    /// Create a new RawEvent.
    #[must_use]
    pub fn new(
        event_id: [u8; 16],
        timestamp_ms: u64,
        session_id: Option<String>,
        event_type: String,
        payload: String,
    ) -> Self {
        Self {
            event_id,
            timestamp_ms,
            session_id,
            event_type,
            payload,
        }
    }

    /// Get the event ID as a UUID string.
    #[must_use]
    pub fn event_id_string(&self) -> String {
        uuid::Uuid::from_bytes(self.event_id).to_string()
    }
}

/// A result from plugin assessment processing.
///
/// Plugins return these from `on_event()` to signal assessment outcomes.
/// The host deserializes the payload and writes to the assessment log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAssessmentResult {
    /// Unique event identifier (UUIDv7 string).
    ///
    /// This is typically the ID of the event that triggered the assessment,
    /// enabling time-ordering and pagination in the UI.
    pub event_id: String,

    /// Type of assessment result: "lightweight", "checkpoint", "session_end".
    pub result_type: String,

    /// Session ID this result applies to.
    pub session_id: String,

    /// JSON-serialized assessment-specific payload.
    pub payload: String,
}

/// Query parameters for retrieving assessment results from plugins.
///
/// Used by the host to request assessment data from plugins without
/// needing to know about plugin-specific storage types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentQuery {
    /// Filter by session ID (None = all sessions).
    pub session_id: Option<String>,

    /// Filter by result types (empty = all types).
    pub result_types: Vec<String>,

    /// Maximum number of results to return.
    pub limit: usize,

    /// Pagination cursor (event ID to start after).
    pub after_event_id: Option<String>,

    /// Sort order (true = newest first).
    pub newest_first: bool,
}

impl AssessmentQuery {
    /// Create a new query with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_id: None,
            result_types: vec![],
            limit: 100,
            after_event_id: None,
            newest_first: true,
        }
    }

    /// Filter by session ID.
    #[must_use]
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set result limit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set pagination cursor.
    #[must_use]
    pub fn after(mut self, event_id: impl Into<String>) -> Self {
        self.after_event_id = Some(event_id.into());
        self
    }

    /// Filter by result type (can be called multiple times).
    #[must_use]
    pub fn with_type(mut self, result_type: impl Into<String>) -> Self {
        self.result_types.push(result_type.into());
        self
    }

    /// Set sort order (true = newest first).
    #[must_use]
    pub fn newest_first(mut self, newest: bool) -> Self {
        self.newest_first = newest;
        self
    }
}

/// Response containing assessment results and pagination metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentQueryResponse {
    /// The assessment results.
    pub results: Vec<PluginAssessmentResult>,

    /// Event ID of the oldest result (for pagination).
    pub oldest_event_id: Option<String>,

    /// Whether more results are available.
    pub has_more: bool,
}

impl PluginAssessmentResult {
    /// Create a lightweight assessment result (per-message pattern detection).
    #[must_use]
    pub fn lightweight(
        event_id: impl Into<String>,
        session_id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            result_type: "lightweight".to_string(),
            session_id: session_id.into(),
            payload: payload.into(),
        }
    }

    /// Create a checkpoint assessment result (periodic summaries).
    #[must_use]
    pub fn checkpoint(
        event_id: impl Into<String>,
        session_id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            result_type: "checkpoint".to_string(),
            session_id: session_id.into(),
            payload: payload.into(),
        }
    }

    /// Create a session end assessment result (full session analysis).
    #[must_use]
    pub fn session_end(
        event_id: impl Into<String>,
        session_id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            result_type: "session_end".to_string(),
            session_id: session_id.into(),
            payload: payload.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_event_creation() {
        let event_id = [0u8; 16];
        let event = RawEvent::new(
            event_id,
            1234567890,
            Some("session-1".to_string()),
            "TextDelta".to_string(),
            r#"{"text": "hello"}"#.to_string(),
        );

        assert_eq!(event.timestamp_ms, 1234567890);
        assert_eq!(event.session_id, Some("session-1".to_string()));
        assert_eq!(event.event_type, "TextDelta");
    }

    #[test]
    fn test_raw_event_id_string() {
        let event_id = [
            0x01, 0x93, 0xf7, 0x8d, 0x8f, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
        ];
        let event = RawEvent::new(event_id, 0, None, "Test".to_string(), "{}".to_string());

        // Should produce a valid UUID string
        let uuid_str = event.event_id_string();
        assert!(uuid::Uuid::parse_str(&uuid_str).is_ok());
    }

    #[test]
    fn test_plugin_assessment_result_types() {
        let lightweight = PluginAssessmentResult::lightweight("evt-1", "s1", "{}");
        assert_eq!(lightweight.event_id, "evt-1");
        assert_eq!(lightweight.result_type, "lightweight");
        assert_eq!(lightweight.session_id, "s1");

        let checkpoint = PluginAssessmentResult::checkpoint("evt-2", "s2", "{}");
        assert_eq!(checkpoint.event_id, "evt-2");
        assert_eq!(checkpoint.result_type, "checkpoint");

        let session_end = PluginAssessmentResult::session_end("evt-3", "s3", "{}");
        assert_eq!(session_end.event_id, "evt-3");
        assert_eq!(session_end.result_type, "session_end");
    }

    #[test]
    fn test_raw_event_serialization() {
        let event = RawEvent::new(
            [1u8; 16],
            999,
            Some("sess".to_string()),
            "Test".to_string(),
            "{}".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let parsed: RawEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, event.event_id);
        assert_eq!(parsed.timestamp_ms, event.timestamp_ms);
    }

    #[test]
    fn test_plugin_assessment_result_serialization() {
        let result = PluginAssessmentResult::lightweight("evt-1", "s1", r#"{"score": 0.5}"#);

        let json = serde_json::to_string(&result).unwrap();
        let parsed: PluginAssessmentResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, "evt-1");
        assert_eq!(parsed.result_type, "lightweight");
        assert_eq!(parsed.session_id, "s1");
        assert_eq!(parsed.payload, r#"{"score": 0.5}"#);
    }
}

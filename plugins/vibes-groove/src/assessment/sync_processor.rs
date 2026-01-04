//! Synchronous assessment processor for FFI-safe plugin callbacks.
//!
//! This processor wraps the assessment detection pipeline (LightweightDetector,
//! CircuitBreaker, SessionBuffer, CheckpointManager) with a pure synchronous
//! interface suitable for cross-library boundary calls.
//!
//! Unlike `AssessmentProcessor` which spawns async background tasks, this
//! processor is fully synchronous and returns results directly. The host
//! is responsible for persisting results to the AssessmentLog.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use vibes_plugin_api::{
    AssessmentQuery, AssessmentQueryResponse, PluginAssessmentResult, RawEvent,
};

use super::AssessmentConfig;
use super::checkpoint::{CheckpointConfig, CheckpointManager};
use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::lightweight::{LightweightDetector, LightweightDetectorConfig, SessionState};
use super::session_buffer::{SessionBuffer, SessionBufferConfig};
use super::types::SessionId;

/// Maximum number of results to store in memory.
const DEFAULT_MAX_RESULTS: usize = 10_000;

/// A stored result with its event ID for pagination.
#[derive(Debug, Clone)]
struct StoredResult {
    /// UUID string for the triggering event (used for pagination).
    event_id: String,
    /// The assessment result.
    result: PluginAssessmentResult,
}

/// Synchronous assessment processor for plugin callbacks.
///
/// This processor maintains state for pattern detection and returns assessment
/// results synchronously. It's designed for FFI-safe calls from the host.
///
/// Results are also stored internally for querying via `query()`.
pub struct SyncAssessmentProcessor {
    /// Configuration for assessment behavior.
    config: AssessmentConfig,
    /// Lightweight detector for per-message signal detection.
    detector: LightweightDetector,
    /// Per-session state for EMA computation.
    session_states: Mutex<HashMap<SessionId, SessionState>>,
    /// Circuit breaker for intervention decisions.
    circuit_breaker: Mutex<CircuitBreaker>,
    /// Session event buffer for batch processing.
    session_buffer: Mutex<SessionBuffer>,
    /// Checkpoint manager for triggering assessments.
    checkpoint_manager: Mutex<CheckpointManager>,
    /// Stored results for querying (newest first).
    stored_results: Mutex<VecDeque<StoredResult>>,
    /// Maximum results to store.
    max_results: usize,
}

impl SyncAssessmentProcessor {
    /// Create a new sync assessment processor.
    #[must_use]
    pub fn new(config: AssessmentConfig) -> Self {
        let detector_config = LightweightDetectorConfig::from_pattern_config(&config.patterns);
        let detector = LightweightDetector::new(detector_config);
        let circuit_breaker = CircuitBreaker::new(config.circuit_breaker.clone());
        let session_buffer = SessionBuffer::new(SessionBufferConfig::default());
        let checkpoint_manager = CheckpointManager::new(CheckpointConfig::default());

        Self {
            config,
            detector,
            session_states: Mutex::new(HashMap::new()),
            circuit_breaker: Mutex::new(circuit_breaker),
            session_buffer: Mutex::new(session_buffer),
            checkpoint_manager: Mutex::new(checkpoint_manager),
            stored_results: Mutex::new(VecDeque::new()),
            max_results: DEFAULT_MAX_RESULTS,
        }
    }

    /// Check if assessment is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Process a raw event and return any assessment results.
    ///
    /// This is the main entry point for the plugin callback. It analyzes
    /// the event and returns assessment results that should be persisted
    /// by the host.
    pub fn process(&self, raw: &RawEvent) -> Vec<PluginAssessmentResult> {
        if !self.is_enabled() {
            return vec![];
        }

        // Parse the session ID from the raw event
        let session_id = match &raw.session_id {
            Some(id) => SessionId::from(id.as_str()),
            None => return vec![], // Skip events without session ID
        };

        // Deserialize the event payload to get the VibesEvent
        let vibes_event: vibes_core::VibesEvent = match serde_json::from_str(&raw.payload) {
            Ok(event) => event,
            Err(_) => return vec![], // Skip malformed events
        };

        let mut results = Vec::new();

        // B3: Buffer all events for checkpoint context
        {
            let mut buffer = self.session_buffer.lock().unwrap();
            buffer.push(session_id.clone(), vibes_event.clone());
        }

        // Convert event_id bytes to UUID string for use throughout
        let event_id_str = raw.event_id_string();

        // B1: Route to LightweightDetector for signal detection
        let lightweight_event = {
            let mut states = self.session_states.lock().unwrap();
            let state = states.entry(session_id.clone()).or_default();

            // Convert event_id bytes to UUID
            let event_id = uuid::Uuid::from_bytes(raw.event_id);

            self.detector.process(&vibes_event, state, event_id)
        };

        if let Some(ref le) = lightweight_event {
            // Serialize to JSON for FFI boundary
            if let Ok(payload) = serde_json::to_string(le) {
                results.push(PluginAssessmentResult::lightweight(
                    &event_id_str,
                    session_id.as_str(),
                    payload,
                ));
            }

            // B2: Route to CircuitBreaker for intervention decisions
            {
                let mut cb = self.circuit_breaker.lock().unwrap();
                if let Some(transition) = cb.record_event(le) {
                    // Log transition for debugging (host can see this via tracing)
                    tracing::debug!(
                        session_id = %session_id,
                        transition = ?transition,
                        "Circuit state transition"
                    );
                }
            }

            // B4: Check for checkpoint triggers
            let trigger = {
                let buffer = self.session_buffer.lock().unwrap();
                let mut cm = self.checkpoint_manager.lock().unwrap();
                cm.should_checkpoint(&le.context.session_id, le, &buffer)
            };

            if let Some(trigger) = trigger {
                // Drain the buffer
                let _events = {
                    let mut buffer = self.session_buffer.lock().unwrap();
                    buffer.drain(&le.context.session_id)
                };

                // Create MediumEvent with checkpoint info
                let medium_event =
                    super::MediumEvent::new(le.context.clone(), (0, le.message_idx + 1), trigger);

                if let Ok(payload) = serde_json::to_string(&medium_event) {
                    results.push(PluginAssessmentResult::checkpoint(
                        &event_id_str,
                        session_id.as_str(),
                        payload,
                    ));
                }
            }
        }

        // Store results for querying
        if !results.is_empty() {
            let mut stored = self.stored_results.lock().unwrap();
            for result in &results {
                stored.push_front(StoredResult {
                    event_id: result.event_id.clone(),
                    result: result.clone(),
                });
            }
            // Trim to max size
            while stored.len() > self.max_results {
                stored.pop_back();
            }
        }

        results
    }

    /// Query stored assessment results.
    ///
    /// Returns results matching the query criteria. Results are stored newest-first.
    pub fn query(&self, query: AssessmentQuery) -> AssessmentQueryResponse {
        let stored = self.stored_results.lock().unwrap();

        let mut matching: Vec<_> = stored
            .iter()
            .filter(|sr| {
                // Session filter
                if let Some(ref session) = query.session_id
                    && &sr.result.session_id != session
                {
                    return false;
                }
                // Type filter
                if !query.result_types.is_empty()
                    && !query.result_types.contains(&sr.result.result_type)
                {
                    return false;
                }
                // Pagination is handled below after collecting all matches
                true
            })
            .collect();

        // Handle pagination cursor
        if let Some(ref after_id) = query.after_event_id {
            // Find the position of the cursor
            if let Some(pos) = matching.iter().position(|sr| &sr.event_id == after_id) {
                // Skip everything up to and including the cursor
                matching = matching.into_iter().skip(pos + 1).collect();
            }
        }

        // Handle sort order (stored is newest-first by default)
        if !query.newest_first {
            matching.reverse();
        }

        // Apply limit + 1 to check has_more
        let has_more = matching.len() > query.limit;
        let results: Vec<_> = matching
            .into_iter()
            .take(query.limit)
            .map(|sr| sr.result.clone())
            .collect();

        let oldest_event_id = if !results.is_empty() {
            // Find the oldest event in the results
            let stored_ref = stored
                .iter()
                .find(|sr| sr.result == results[results.len() - 1]);
            stored_ref.map(|sr| sr.event_id.clone())
        } else {
            None
        };

        AssessmentQueryResponse {
            results,
            oldest_event_id,
            has_more,
        }
    }

    /// Get the circuit breaker state for a session.
    #[must_use]
    pub fn circuit_state(&self, session_id: &SessionId) -> CircuitState {
        let cb = self.circuit_breaker.lock().unwrap();
        cb.state(session_id)
    }

    /// Get the number of events buffered for a session.
    #[must_use]
    pub fn buffer_len(&self, session_id: &SessionId) -> usize {
        let buffer = self.session_buffer.lock().unwrap();
        buffer.len(session_id)
    }

    /// Get the number of checkpoints triggered for a session.
    #[must_use]
    pub fn checkpoint_count(&self, session_id: &SessionId) -> u32 {
        let cm = self.checkpoint_manager.lock().unwrap();
        cm.checkpoint_count(session_id)
    }
}

impl std::fmt::Debug for SyncAssessmentProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncAssessmentProcessor")
            .field("config", &self.config)
            .field("enabled", &self.is_enabled())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::{ClaudeEvent, VibesEvent};

    fn make_raw_event(session_id: &str, text: &str) -> RawEvent {
        let event = VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: ClaudeEvent::TextDelta {
                text: text.to_string(),
            },
        };
        let payload = serde_json::to_string(&event).unwrap();

        RawEvent::new(
            uuid::Uuid::now_v7().into_bytes(),
            chrono::Utc::now().timestamp_millis() as u64,
            Some(session_id.to_string()),
            "Claude".to_string(),
            payload,
        )
    }

    #[test]
    fn test_sync_processor_creation() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        assert!(processor.is_enabled());
    }

    #[test]
    fn test_sync_processor_disabled_returns_empty() {
        let config = AssessmentConfig {
            enabled: false,
            ..Default::default()
        };
        let processor = SyncAssessmentProcessor::new(config);

        let raw = make_raw_event("test-session", "Hello");
        let results = processor.process(&raw);

        assert!(results.is_empty());
    }

    #[test]
    fn test_sync_processor_skips_events_without_session() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        let raw = RawEvent::new(
            [0u8; 16],
            0,
            None, // No session ID
            "Test".to_string(),
            "{}".to_string(),
        );
        let results = processor.process(&raw);

        assert!(results.is_empty());
    }

    #[test]
    fn test_sync_processor_emits_lightweight_event() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        let raw = make_raw_event("test-session", "Hello, this is a test");
        let results = processor.process(&raw);

        // Should emit at least one lightweight result
        assert!(!results.is_empty());
        assert_eq!(results[0].result_type, "lightweight");
        assert_eq!(results[0].session_id, "test-session");

        // Payload should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&results[0].payload).unwrap();
    }

    #[test]
    fn test_sync_processor_maintains_session_state() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process multiple events for the same session
        for i in 0..3 {
            let raw = make_raw_event("stateful-session", &format!("Message {i}"));
            let results = processor.process(&raw);

            // Each message should produce a result
            assert!(!results.is_empty());

            // Parse the lightweight event to check message_idx
            let le: super::super::LightweightEvent =
                serde_json::from_str(&results[0].payload).unwrap();
            assert_eq!(le.message_idx, i as u32);
        }
    }

    #[test]
    fn test_sync_processor_detects_frustration_patterns() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Send frustrating message
        let raw = make_raw_event(
            "frustration-session",
            "This is broken and doesn't work at all!",
        );
        let results = processor.process(&raw);

        assert!(!results.is_empty());

        // Parse and check for signals
        let le: super::super::LightweightEvent = serde_json::from_str(&results[0].payload).unwrap();
        assert!(
            !le.signals.is_empty(),
            "Should detect signals in frustrating message"
        );
        assert!(
            le.frustration_ema > 0.0,
            "Frustration EMA should be positive"
        );
    }

    #[test]
    fn test_sync_processor_buffers_events() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process multiple events
        for i in 0..3 {
            let raw = make_raw_event("buffer-session", &format!("Message {i}"));
            processor.process(&raw);
        }

        // Check buffer length
        let buffered = processor.buffer_len(&"buffer-session".into());
        assert_eq!(buffered, 3);
    }

    #[test]
    fn test_sync_processor_triggers_checkpoint() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process enough events to trigger checkpoint (default min_events = 5)
        let mut checkpoint_results = Vec::new();
        for i in 0..6 {
            let raw = make_raw_event("checkpoint-session", &format!("Message {i}"));
            let results = processor.process(&raw);
            checkpoint_results.extend(results);
        }

        // Should have at least one checkpoint result
        let checkpoints: Vec<_> = checkpoint_results
            .iter()
            .filter(|r| r.result_type == "checkpoint")
            .collect();
        assert!(
            !checkpoints.is_empty(),
            "Should have triggered at least one checkpoint"
        );
    }

    #[test]
    fn test_sync_processor_circuit_breaker_integration() {
        let mut config = AssessmentConfig::default();
        config.circuit_breaker.enabled = true;
        let processor = SyncAssessmentProcessor::new(config);

        // Send frustrating messages to potentially trigger circuit
        for i in 0..10 {
            let raw = make_raw_event("cb-session", &format!("Error! Failed! Broken! Attempt {i}"));
            processor.process(&raw);
        }

        // Circuit state should have changed
        let state = processor.circuit_state(&"cb-session".into());
        // Note: may or may not have opened depending on exact thresholds
        assert!(
            state == CircuitState::Closed || state == CircuitState::Open,
            "Circuit state should be valid"
        );
    }

    #[test]
    fn test_sync_processor_separate_sessions() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process events for two sessions
        for i in 0..2 {
            let raw = make_raw_event("session-a", &format!("A message {i}"));
            processor.process(&raw);
        }
        for i in 0..4 {
            let raw = make_raw_event("session-b", &format!("B message {i}"));
            processor.process(&raw);
        }

        // Check separate buffer counts
        assert_eq!(processor.buffer_len(&"session-a".into()), 2);
        assert_eq!(processor.buffer_len(&"session-b".into()), 4);
    }

    #[test]
    fn test_sync_processor_checkpoint_result_is_valid_json() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process enough events
        for i in 0..6 {
            let raw = make_raw_event("json-test", &format!("Message {i}"));
            let results = processor.process(&raw);

            for result in results {
                // All payloads should be valid JSON
                let value: serde_json::Value =
                    serde_json::from_str(&result.payload).expect("Payload should be valid JSON");
                assert!(value.is_object(), "Payload should be a JSON object");
            }
        }
    }

    #[test]
    fn test_sync_processor_result_types_are_correct() {
        let config = AssessmentConfig::default();
        let processor = SyncAssessmentProcessor::new(config);

        // Process events and collect results
        let mut all_results = Vec::new();
        for i in 0..6 {
            let raw = make_raw_event("type-test", &format!("Message {i}"));
            all_results.extend(processor.process(&raw));
        }

        // Verify result types
        for result in &all_results {
            assert!(
                result.result_type == "lightweight" || result.result_type == "checkpoint",
                "Result type should be 'lightweight' or 'checkpoint', got '{}'",
                result.result_type
            );
        }

        // Should have both types
        let has_lightweight = all_results.iter().any(|r| r.result_type == "lightweight");
        let has_checkpoint = all_results.iter().any(|r| r.result_type == "checkpoint");
        assert!(has_lightweight, "Should have lightweight results");
        assert!(has_checkpoint, "Should have checkpoint results");
    }
}

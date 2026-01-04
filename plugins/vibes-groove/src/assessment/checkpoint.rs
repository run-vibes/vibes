//! Checkpoint manager for assessment triggers.
//!
//! The `CheckpointManager` determines when to trigger an assessment checkpoint
//! based on various conditions:
//!
//! - **Pattern match**: Specific signal patterns detected (e.g., repeated errors)
//! - **Time interval**: Periodic checkpoints for regular assessment
//! - **Threshold exceeded**: EMA or other metrics crossing configured limits
//!
//! ## Checkpoint Flow
//!
//! ```text
//! LightweightEvent → CheckpointManager.should_checkpoint() → Option<CheckpointTrigger>
//!                         │
//!                         ├─ Check EMA thresholds
//!                         ├─ Check time since last checkpoint
//!                         └─ Check specific signal patterns
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::session_buffer::SessionBuffer;
use super::types::{LightweightEvent, LightweightSignal, SessionId};

/// Default checkpoint interval in seconds.
const DEFAULT_CHECKPOINT_INTERVAL_SECS: u64 = 300; // 5 minutes

/// Default frustration threshold to trigger checkpoint.
const DEFAULT_FRUSTRATION_THRESHOLD: f64 = 0.7;

/// Default minimum events before checkpoint is considered.
const DEFAULT_MIN_EVENTS: usize = 5;

/// Configuration for the checkpoint manager.
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Whether checkpointing is enabled.
    pub enabled: bool,
    /// Minimum interval between checkpoints (seconds).
    pub interval_seconds: u64,
    /// Frustration EMA threshold to trigger checkpoint.
    pub frustration_threshold: f64,
    /// Minimum events in buffer before checkpoint is considered.
    pub min_events: usize,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: DEFAULT_CHECKPOINT_INTERVAL_SECS,
            frustration_threshold: DEFAULT_FRUSTRATION_THRESHOLD,
            min_events: DEFAULT_MIN_EVENTS,
        }
    }
}

/// Reason why a checkpoint was triggered.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointTrigger {
    /// Triggered by specific pattern detection.
    PatternMatch {
        /// The pattern that matched.
        pattern: String,
    },
    /// Triggered by time interval expiring.
    TimeInterval,
    /// Triggered by metric threshold being exceeded.
    ThresholdExceeded {
        /// Name of the metric that exceeded.
        metric: String,
        /// The value that exceeded the threshold.
        value: f64,
    },
}

/// Per-session checkpoint tracking.
#[derive(Debug)]
struct SessionCheckpointState {
    /// When the last checkpoint occurred (None = never checkpointed yet).
    last_checkpoint: Option<Instant>,
    /// Number of checkpoints triggered.
    checkpoint_count: u32,
}

impl SessionCheckpointState {
    fn new() -> Self {
        Self {
            last_checkpoint: None, // No checkpoint yet - allows immediate first trigger
            checkpoint_count: 0,
        }
    }
}

/// Manager for checkpoint triggers.
///
/// Determines when to trigger assessment checkpoints based on
/// signal patterns, time intervals, and metric thresholds.
pub struct CheckpointManager {
    config: CheckpointConfig,
    sessions: HashMap<SessionId, SessionCheckpointState>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager with the given configuration.
    pub fn new(config: CheckpointConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
        }
    }

    /// Check if a checkpoint should be triggered.
    ///
    /// Evaluates the detector output and buffer state to determine if
    /// a checkpoint trigger condition is met.
    ///
    /// Returns `Some(CheckpointTrigger)` if a checkpoint should occur,
    /// or `None` if no trigger condition is met.
    pub fn should_checkpoint(
        &mut self,
        session_id: &SessionId,
        detector_output: &LightweightEvent,
        buffer: &SessionBuffer,
    ) -> Option<CheckpointTrigger> {
        if !self.config.enabled {
            return None;
        }

        // Get or create session state
        let session = self
            .sessions
            .entry(session_id.clone())
            .or_insert_with(SessionCheckpointState::new);

        // Check minimum events requirement
        let event_count = buffer.len(session_id);
        if event_count < self.config.min_events {
            return None;
        }

        // Check time interval trigger
        let interval = Duration::from_secs(self.config.interval_seconds);
        let interval_exceeded = match session.last_checkpoint {
            None => true, // Never checkpointed - trigger immediately
            Some(last) => last.elapsed() >= interval,
        };
        if interval_exceeded {
            session.last_checkpoint = Some(Instant::now());
            session.checkpoint_count += 1;
            return Some(CheckpointTrigger::TimeInterval);
        }

        // Check threshold trigger (frustration EMA)
        if detector_output.frustration_ema >= self.config.frustration_threshold {
            session.last_checkpoint = Some(Instant::now());
            session.checkpoint_count += 1;
            return Some(CheckpointTrigger::ThresholdExceeded {
                metric: "frustration_ema".to_string(),
                value: detector_output.frustration_ema,
            });
        }

        // Check pattern match trigger (multiple tool failures)
        let tool_failure_count = detector_output
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::ToolFailure { .. }))
            .count();

        if tool_failure_count >= 2 {
            session.last_checkpoint = Some(Instant::now());
            session.checkpoint_count += 1;
            return Some(CheckpointTrigger::PatternMatch {
                pattern: format!("{} tool failures", tool_failure_count),
            });
        }

        None
    }

    /// Record that a checkpoint occurred (for external triggers).
    pub fn record_checkpoint(&mut self, session_id: &SessionId) {
        let session = self
            .sessions
            .entry(session_id.clone())
            .or_insert_with(SessionCheckpointState::new);
        session.last_checkpoint = Some(Instant::now());
        session.checkpoint_count += 1;
    }

    /// Get the number of checkpoints for a session.
    pub fn checkpoint_count(&self, session_id: &SessionId) -> u32 {
        self.sessions
            .get(session_id)
            .map(|s| s.checkpoint_count)
            .unwrap_or(0)
    }

    /// Time since last checkpoint for a session.
    ///
    /// Returns `None` if the session doesn't exist or has never checkpointed.
    pub fn time_since_checkpoint(&self, session_id: &SessionId) -> Option<Duration> {
        self.sessions
            .get(session_id)
            .and_then(|s| s.last_checkpoint.map(|last| last.elapsed()))
    }

    /// Remove session state.
    pub fn remove_session(&mut self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new(CheckpointConfig::default())
    }
}

impl std::fmt::Debug for CheckpointManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointManager")
            .field("enabled", &self.config.enabled)
            .field("interval_seconds", &self.config.interval_seconds)
            .field("frustration_threshold", &self.config.frustration_threshold)
            .field("session_count", &self.sessions.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::types::AssessmentContext;

    fn make_lightweight_event(
        session_id: &str,
        signals: Vec<LightweightSignal>,
        frustration_ema: f64,
    ) -> LightweightEvent {
        LightweightEvent {
            context: AssessmentContext::new(session_id),
            message_idx: 0,
            signals,
            frustration_ema,
            success_ema: 0.5,
            triggering_event_id: uuid::Uuid::now_v7(),
        }
    }

    fn setup_buffer_with_events(session_id: &SessionId, count: usize) -> SessionBuffer {
        use crate::assessment::session_buffer::SessionBufferConfig;

        let mut buffer = SessionBuffer::new(SessionBufferConfig::default());
        for i in 0..count {
            buffer.push(
                session_id.clone(),
                vibes_core::VibesEvent::UserInput {
                    session_id: session_id.as_str().to_string(),
                    content: format!("Event {}", i),
                    source: vibes_core::InputSource::Unknown,
                },
            );
        }
        buffer
    }

    #[test]
    fn test_checkpoint_on_pattern_match() {
        let config = CheckpointConfig {
            enabled: true,
            interval_seconds: 3600,     // High to avoid interval trigger
            frustration_threshold: 0.9, // High to avoid threshold trigger
            min_events: 1,
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");
        let buffer = setup_buffer_with_events(&session_id, 5);

        // Record initial checkpoint to avoid interval trigger on first call
        manager.record_checkpoint(&session_id);

        // Event with multiple tool failures should trigger pattern match
        let signals = vec![
            LightweightSignal::ToolFailure {
                tool_name: "Bash".to_string(),
            },
            LightweightSignal::ToolFailure {
                tool_name: "Read".to_string(),
            },
        ];
        let event = make_lightweight_event("test-session", signals, 0.3);

        let result = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(matches!(
            result,
            Some(CheckpointTrigger::PatternMatch { .. })
        ));
        assert_eq!(manager.checkpoint_count(&session_id), 2); // 1 from record + 1 from pattern
    }

    #[test]
    fn test_checkpoint_on_time_interval() {
        let config = CheckpointConfig {
            enabled: true,
            interval_seconds: 0,        // Immediate trigger
            frustration_threshold: 0.9, // High to avoid threshold trigger
            min_events: 1,
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");
        let buffer = setup_buffer_with_events(&session_id, 5);

        // Any event should trigger due to zero interval
        let event = make_lightweight_event("test-session", vec![], 0.3);

        let result = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(matches!(result, Some(CheckpointTrigger::TimeInterval)));
    }

    #[test]
    fn test_checkpoint_on_threshold() {
        let config = CheckpointConfig {
            enabled: true,
            interval_seconds: 3600, // High to avoid interval trigger
            frustration_threshold: 0.5,
            min_events: 1,
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");
        let buffer = setup_buffer_with_events(&session_id, 5);

        // Record initial checkpoint to avoid interval trigger on first call
        manager.record_checkpoint(&session_id);

        // High frustration EMA should trigger threshold
        let event = make_lightweight_event("test-session", vec![], 0.7);

        let result = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(matches!(
            result,
            Some(CheckpointTrigger::ThresholdExceeded { .. })
        ));

        if let Some(CheckpointTrigger::ThresholdExceeded { metric, value }) = result {
            assert_eq!(metric, "frustration_ema");
            assert!((value - 0.7).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_checkpoint_requires_min_events() {
        let config = CheckpointConfig {
            enabled: true,
            interval_seconds: 0,        // Would normally trigger
            frustration_threshold: 0.1, // Would normally trigger
            min_events: 10,
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");

        // Only 5 events - below minimum
        let buffer = setup_buffer_with_events(&session_id, 5);
        let event = make_lightweight_event("test-session", vec![], 0.8);

        let result = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(result.is_none());
    }

    #[test]
    fn test_checkpoint_disabled() {
        let config = CheckpointConfig {
            enabled: false,
            ..Default::default()
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");
        let buffer = setup_buffer_with_events(&session_id, 100);
        let event = make_lightweight_event("test-session", vec![], 0.9);

        let result = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(result.is_none());
    }

    #[test]
    fn test_checkpoint_interval_respects_cooldown() {
        let config = CheckpointConfig {
            enabled: true,
            interval_seconds: 1,         // 1 second interval
            frustration_threshold: 0.99, // High to avoid threshold trigger
            min_events: 1,
        };
        let mut manager = CheckpointManager::new(config);
        let session_id = SessionId::from("test-session");
        let buffer = setup_buffer_with_events(&session_id, 5);
        let event = make_lightweight_event("test-session", vec![], 0.3);

        // First check triggers interval
        let result1 = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(matches!(result1, Some(CheckpointTrigger::TimeInterval)));

        // Immediate second check should not trigger (cooldown)
        let result2 = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(result2.is_none());

        // After waiting, should trigger again
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let result3 = manager.should_checkpoint(&session_id, &event, &buffer);
        assert!(matches!(result3, Some(CheckpointTrigger::TimeInterval)));
    }

    #[test]
    fn test_checkpoint_config_defaults() {
        let config = CheckpointConfig::default();

        assert!(config.enabled);
        assert_eq!(config.interval_seconds, DEFAULT_CHECKPOINT_INTERVAL_SECS);
        assert!(
            (config.frustration_threshold - DEFAULT_FRUSTRATION_THRESHOLD).abs() < f64::EPSILON
        );
        assert_eq!(config.min_events, DEFAULT_MIN_EVENTS);
    }

    #[test]
    fn test_checkpoint_manager_record_checkpoint() {
        let mut manager = CheckpointManager::default();
        let session_id = SessionId::from("test-session");

        assert_eq!(manager.checkpoint_count(&session_id), 0);

        manager.record_checkpoint(&session_id);
        assert_eq!(manager.checkpoint_count(&session_id), 1);

        manager.record_checkpoint(&session_id);
        assert_eq!(manager.checkpoint_count(&session_id), 2);
    }

    #[test]
    fn test_checkpoint_manager_remove_session() {
        let mut manager = CheckpointManager::default();
        let session_id = SessionId::from("test-session");

        manager.record_checkpoint(&session_id);
        assert_eq!(manager.checkpoint_count(&session_id), 1);

        manager.remove_session(&session_id);
        assert_eq!(manager.checkpoint_count(&session_id), 0);
        assert!(manager.time_since_checkpoint(&session_id).is_none());
    }
}

//! Lightweight signal detector for per-message assessment.
//!
//! The `LightweightDetector` processes each message in real-time (<10ms latency),
//! detecting patterns and computing exponential moving averages (EMAs) for
//! frustration and success indicators.

use regex::Regex;
use tracing::trace;
use uuid::Uuid;
use vibes_core::hooks::HookEvent;
use vibes_core::{ClaudeEvent, VibesEvent};

use super::config::PatternConfig;
use super::types::{AssessmentContext, LightweightEvent, LightweightSignal, SessionId};

/// Default EMA alpha (smoothing factor).
/// Lower values = smoother, higher values = more responsive.
const DEFAULT_EMA_ALPHA: f64 = 0.2;

/// Configuration for the lightweight detector.
#[derive(Debug, Clone)]
pub struct LightweightDetectorConfig {
    /// Compiled negative patterns.
    negative_patterns: Vec<CompiledPattern>,
    /// Compiled positive patterns.
    positive_patterns: Vec<CompiledPattern>,
    /// EMA smoothing factor (0.0 to 1.0).
    ema_alpha: f64,
}

/// A compiled regex pattern with its source string.
#[derive(Debug, Clone)]
struct CompiledPattern {
    source: String,
    regex: Regex,
}

impl CompiledPattern {
    fn new(pattern: &str) -> Option<Self> {
        Regex::new(pattern).ok().map(|regex| Self {
            source: pattern.to_string(),
            regex,
        })
    }

    fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}

impl LightweightDetectorConfig {
    /// Create a new configuration from pattern config.
    pub fn from_pattern_config(patterns: &PatternConfig) -> Self {
        let negative_patterns: Vec<_> = patterns
            .negative
            .iter()
            .filter_map(|p| CompiledPattern::new(p))
            .collect();

        let positive_patterns: Vec<_> = patterns
            .positive
            .iter()
            .filter_map(|p| CompiledPattern::new(p))
            .collect();

        Self {
            negative_patterns,
            positive_patterns,
            ema_alpha: DEFAULT_EMA_ALPHA,
        }
    }

    /// Create default configuration with common patterns.
    pub fn with_default_patterns() -> Self {
        let patterns = PatternConfig {
            negative: vec![
                r"(?i)\berror\b".to_string(),
                r"(?i)\bfailed\b".to_string(),
                r"(?i)\bfrustrat".to_string(),
                r"(?i)\bconfus".to_string(),
                r"(?i)\bwrong\b".to_string(),
                r"(?i)\bbroken\b".to_string(),
                r"(?i)\bnot\s+work".to_string(),
                r"(?i)\bdoesn't\s+work".to_string(),
                r"(?i)\bcan't\b".to_string(),
                r"(?i)\bwon't\b".to_string(),
            ],
            positive: vec![
                r"(?i)\bthank".to_string(),
                r"(?i)\bperfect\b".to_string(),
                r"(?i)\bexcellent\b".to_string(),
                r"(?i)\bgreat\b".to_string(),
                r"(?i)\bworks?\b".to_string(),
                r"(?i)\bsuccess".to_string(),
                r"(?i)\bcomplete".to_string(),
                r"(?i)\bdone\b".to_string(),
            ],
        };

        Self::from_pattern_config(&patterns)
    }

    /// Set the EMA alpha (smoothing factor).
    #[must_use]
    pub fn with_ema_alpha(mut self, alpha: f64) -> Self {
        self.ema_alpha = alpha.clamp(0.0, 1.0);
        self
    }
}

impl Default for LightweightDetectorConfig {
    fn default() -> Self {
        Self::with_default_patterns()
    }
}

/// State tracked per session for EMA computation.
#[derive(Debug, Clone, Default)]
pub struct SessionState {
    /// Current frustration EMA.
    pub frustration_ema: f64,
    /// Current success EMA.
    pub success_ema: f64,
    /// Message index (0-based).
    pub message_idx: u32,
}

impl SessionState {
    /// Create a new session state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update EMA with a new value.
    fn update_ema(current: f64, new_value: f64, alpha: f64) -> f64 {
        alpha * new_value + (1.0 - alpha) * current
    }
}

/// Lightweight detector for per-message signal detection.
///
/// Processes VibesEvents and emits LightweightEvents containing:
/// - Detected signals (pattern matches, tool failures, etc.)
/// - Exponential moving averages for frustration and success
pub struct LightweightDetector {
    config: LightweightDetectorConfig,
}

impl LightweightDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: LightweightDetectorConfig) -> Self {
        Self { config }
    }

    /// Create a detector with default patterns.
    pub fn with_default_patterns() -> Self {
        Self::new(LightweightDetectorConfig::default())
    }

    /// Process a VibesEvent and optionally emit a LightweightEvent.
    ///
    /// Returns `Some(LightweightEvent)` if the event is relevant for assessment,
    /// or `None` if the event should be ignored.
    ///
    /// # Arguments
    ///
    /// * `event` - The VibesEvent to process
    /// * `state` - Mutable session state for EMA tracking
    /// * `triggering_event_id` - The ID of the StoredEvent that triggered this assessment
    pub fn process(
        &self,
        event: &VibesEvent,
        state: &mut SessionState,
        triggering_event_id: Uuid,
    ) -> Option<LightweightEvent> {
        // Extract text content and session ID from the event
        let (session_id, text, signals) = self.extract_event_data(event)?;

        // Detect patterns in text
        let mut all_signals = self.detect_patterns(&text);
        all_signals.extend(signals);

        // Calculate signal values for EMA
        let frustration_value = self.calculate_frustration_value(&all_signals);
        let success_value = self.calculate_success_value(&all_signals);

        // Update EMAs
        state.frustration_ema = SessionState::update_ema(
            state.frustration_ema,
            frustration_value,
            self.config.ema_alpha,
        );
        state.success_ema =
            SessionState::update_ema(state.success_ema, success_value, self.config.ema_alpha);

        // Create lightweight event
        let context = AssessmentContext::new(session_id);
        let event = LightweightEvent {
            context,
            message_idx: state.message_idx,
            signals: all_signals,
            frustration_ema: state.frustration_ema,
            success_ema: state.success_ema,
            triggering_event_id,
        };

        // Increment message index for next event
        state.message_idx += 1;

        trace!(
            message_idx = event.message_idx,
            frustration_ema = event.frustration_ema,
            success_ema = event.success_ema,
            signal_count = event.signals.len(),
            "LightweightDetector emitting event"
        );

        Some(event)
    }

    /// Extract relevant data from a VibesEvent.
    ///
    /// Returns (session_id, text_content, additional_signals) or None if event is irrelevant.
    fn extract_event_data(
        &self,
        event: &VibesEvent,
    ) -> Option<(SessionId, String, Vec<LightweightSignal>)> {
        match event {
            // User input - check for patterns (at VibesEvent level, not ClaudeEvent)
            VibesEvent::UserInput {
                session_id,
                content,
                ..
            } => Some((
                SessionId::from(session_id.as_str()),
                content.clone(),
                vec![],
            )),

            // Text output from Claude - check for patterns
            VibesEvent::Claude {
                session_id,
                event: ClaudeEvent::TextDelta { text },
            } => Some((SessionId::from(session_id.as_str()), text.clone(), vec![])),

            // Tool result - check for failures
            // Note: ToolResult has { id, output, is_error }, not { tool, success }
            VibesEvent::Claude {
                session_id,
                event: ClaudeEvent::ToolResult { id, is_error, .. },
            } => {
                let signals = if *is_error {
                    vec![LightweightSignal::ToolFailure {
                        tool_name: id.clone(), // Use id as identifier (no tool name in result)
                    }]
                } else {
                    vec![]
                };
                Some((SessionId::from(session_id.as_str()), String::new(), signals))
            }

            // Error events
            VibesEvent::Claude {
                session_id,
                event: ClaudeEvent::Error { message, .. },
            } => {
                let signals = vec![LightweightSignal::Negative {
                    pattern: "error_event".to_string(),
                    confidence: 1.0,
                }];
                Some((
                    SessionId::from(session_id.as_str()),
                    message.clone(),
                    signals,
                ))
            }

            // Hook events from Claude Code - process like their equivalent VibesEvent types
            VibesEvent::Hook {
                event: HookEvent::UserPromptSubmit(data),
                ..
            } => data.session_id.as_ref().map(|sid| {
                (
                    SessionId::from(sid.as_str()),
                    data.prompt.clone().unwrap_or_default(),
                    vec![],
                )
            }),

            // Hook tool results - check for failures
            VibesEvent::Hook {
                event: HookEvent::PostToolUse(data),
                ..
            } => {
                let session_id = data.session_id.as_ref()?;
                // Check for failure by looking at tool_response.interrupted or stderr content
                let is_failure = data
                    .tool_response
                    .get("interrupted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || data
                        .tool_response
                        .get("stderr")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                let signals = if is_failure {
                    vec![LightweightSignal::ToolFailure {
                        tool_name: data.tool_name.clone(),
                    }]
                } else {
                    vec![]
                };
                Some((SessionId::from(session_id.as_str()), String::new(), signals))
            }

            // Ignore other event types for lightweight detection
            _ => None,
        }
    }

    /// Detect patterns in text and return signals.
    fn detect_patterns(&self, text: &str) -> Vec<LightweightSignal> {
        let mut signals = Vec::new();

        // Check negative patterns
        for pattern in &self.config.negative_patterns {
            if pattern.is_match(text) {
                signals.push(LightweightSignal::Negative {
                    pattern: pattern.source.clone(),
                    confidence: 0.8, // Pattern matches have moderate confidence
                });
            }
        }

        // Check positive patterns
        for pattern in &self.config.positive_patterns {
            if pattern.is_match(text) {
                signals.push(LightweightSignal::Positive {
                    pattern: pattern.source.clone(),
                    confidence: 0.8,
                });
            }
        }

        signals
    }

    /// Calculate frustration value from signals (0.0 to 1.0).
    fn calculate_frustration_value(&self, signals: &[LightweightSignal]) -> f64 {
        let negative_count = signals
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    LightweightSignal::Negative { .. } | LightweightSignal::ToolFailure { .. }
                )
            })
            .count();

        if negative_count == 0 {
            0.0
        } else {
            // Cap at 1.0, but scale based on number of signals
            (negative_count as f64 * 0.3).min(1.0)
        }
    }

    /// Calculate success value from signals (0.0 to 1.0).
    fn calculate_success_value(&self, signals: &[LightweightSignal]) -> f64 {
        let positive_count = signals
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    LightweightSignal::Positive { .. }
                        | LightweightSignal::BuildStatus { passed: true }
                )
            })
            .count();

        if positive_count == 0 {
            0.0
        } else {
            (positive_count as f64 * 0.3).min(1.0)
        }
    }
}

impl std::fmt::Debug for LightweightDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LightweightDetector")
            .field(
                "negative_pattern_count",
                &self.config.negative_patterns.len(),
            )
            .field(
                "positive_pattern_count",
                &self.config.positive_patterns.len(),
            )
            .field("ema_alpha", &self.config.ema_alpha)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::InputSource;
    use vibes_core::hooks::{PostToolUseData, UserPromptSubmitData};

    /// Generate a test triggering event ID.
    fn test_event_id() -> Uuid {
        Uuid::now_v7()
    }

    fn make_user_input(session_id: &str, content: &str) -> VibesEvent {
        // UserInput is a top-level VibesEvent variant, not nested under ClaudeEvent
        VibesEvent::UserInput {
            session_id: session_id.to_string(),
            content: content.to_string(),
            source: InputSource::Unknown,
        }
    }

    fn make_text_delta(session_id: &str, text: &str) -> VibesEvent {
        VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: ClaudeEvent::TextDelta {
                text: text.to_string(),
            },
        }
    }

    fn make_tool_result(session_id: &str, tool_id: &str, is_error: bool) -> VibesEvent {
        // ToolResult has { id, output, is_error }, not { tool, success, error }
        VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: ClaudeEvent::ToolResult {
                id: tool_id.to_string(),
                output: if is_error {
                    "Tool failed".to_string()
                } else {
                    "Success".to_string()
                },
                is_error,
            },
        }
    }

    #[test]
    fn test_detector_matches_error_patterns() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Test negative pattern matching
        let event = make_user_input("sess-1", "This is broken and doesn't work");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // Should detect negative patterns
        let negative_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::Negative { .. }))
            .collect();
        assert!(
            !negative_signals.is_empty(),
            "Should detect negative patterns"
        );
    }

    #[test]
    fn test_detector_matches_success_patterns() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Test positive pattern matching
        let event = make_user_input("sess-1", "Thank you, that's perfect!");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // Should detect positive patterns
        let positive_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::Positive { .. }))
            .collect();
        assert!(
            !positive_signals.is_empty(),
            "Should detect positive patterns"
        );
    }

    #[test]
    fn test_detector_detects_tool_failure() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Test tool failure detection (is_error=true means failure)
        let event = make_tool_result("sess-1", "Bash", true);
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // Should detect tool failure
        let tool_failure_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::ToolFailure { .. }))
            .collect();
        assert_eq!(tool_failure_signals.len(), 1, "Should detect tool failure");
    }

    #[test]
    fn test_detector_ema_computation() {
        let detector =
            LightweightDetector::new(LightweightDetectorConfig::default().with_ema_alpha(0.5));
        let mut state = SessionState::new();

        // Initial state
        assert_eq!(state.frustration_ema, 0.0);
        assert_eq!(state.success_ema, 0.0);

        // First event with negative patterns
        let event = make_user_input("sess-1", "This is broken!");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // EMA should increase from 0
        assert!(
            result.frustration_ema > 0.0,
            "Frustration EMA should increase"
        );
        let ema_after_first = state.frustration_ema;

        // Second event without negative patterns
        let event = make_user_input("sess-1", "Just a normal message");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // EMA should decay towards 0 (neutral message has 0 frustration contribution)
        // With alpha=0.5: new_ema = 0.5 * 0.0 + 0.5 * prev_ema = prev_ema/2
        assert!(
            result.frustration_ema < ema_after_first,
            "EMA should decay: {} < {}",
            result.frustration_ema,
            ema_after_first
        );

        // Verify message index increments
        assert_eq!(result.message_idx, 1);
    }

    #[test]
    fn test_detector_ema_decay() {
        // Test that EMA decays over time without signals
        let detector =
            LightweightDetector::new(LightweightDetectorConfig::default().with_ema_alpha(0.3));
        let mut state = SessionState::new();

        // Start with a frustrating event
        let event = make_user_input("sess-1", "Error! This is broken!");
        let _ = detector.process(&event, &mut state, test_event_id());
        let initial_ema = state.frustration_ema;

        // Process neutral events - EMA should decay
        for i in 0..5 {
            let event = make_user_input("sess-1", &format!("Neutral message {i}"));
            let _ = detector.process(&event, &mut state, test_event_id());
        }

        assert!(
            state.frustration_ema < initial_ema,
            "EMA should decay: {} < {}",
            state.frustration_ema,
            initial_ema
        );
    }

    #[test]
    fn test_detector_emits_lightweight_events() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Process multiple events
        let events = vec![
            make_user_input("sess-1", "First message"),
            make_text_delta("sess-1", "Claude response"),
            make_user_input("sess-1", "Second message"),
        ];

        for event in events {
            let result = detector.process(&event, &mut state, test_event_id());
            assert!(
                result.is_some(),
                "Should emit event for relevant VibesEvent"
            );
        }

        // Message index should have incremented
        assert_eq!(state.message_idx, 3);
    }

    #[test]
    fn test_detector_ignores_irrelevant_events() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Events that should be ignored
        let event = VibesEvent::SessionCreated {
            session_id: "sess-1".to_string(),
            name: None,
        };

        let result = detector.process(&event, &mut state, test_event_id());
        assert!(result.is_none(), "Should ignore SessionCreated event");

        // State should not change
        assert_eq!(state.message_idx, 0);
    }

    #[test]
    fn test_detector_session_state_isolation() {
        let detector = LightweightDetector::with_default_patterns();

        // Two separate session states
        let mut state1 = SessionState::new();
        let state2 = SessionState::new(); // No mut needed - never modified

        // Process event for session 1
        let event = make_user_input("sess-1", "This is broken!");
        let _ = detector.process(&event, &mut state1, test_event_id());

        // Session 1 should have frustration, session 2 should not
        assert!(state1.frustration_ema > 0.0);
        assert_eq!(state2.frustration_ema, 0.0);
    }

    #[test]
    fn test_detector_config_builder() {
        let config = LightweightDetectorConfig::default().with_ema_alpha(0.5);
        assert!((config.ema_alpha - 0.5).abs() < f64::EPSILON);

        // Alpha should be clamped
        let config = LightweightDetectorConfig::default().with_ema_alpha(2.0);
        assert!((config.ema_alpha - 1.0).abs() < f64::EPSILON);

        let config = LightweightDetectorConfig::default().with_ema_alpha(-0.5);
        assert!((config.ema_alpha - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_detector_custom_patterns() {
        let patterns = PatternConfig {
            negative: vec![r"(?i)\bcustom_error\b".to_string()],
            positive: vec![r"(?i)\bcustom_success\b".to_string()],
        };

        let config = LightweightDetectorConfig::from_pattern_config(&patterns);
        let detector = LightweightDetector::new(config);
        let mut state = SessionState::new();

        // Should match custom negative pattern
        let event = make_user_input("sess-1", "custom_error occurred");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        let has_negative = result
            .signals
            .iter()
            .any(|s| matches!(s, LightweightSignal::Negative { .. }));
        assert!(has_negative, "Should match custom negative pattern");

        // Should match custom positive pattern
        let event = make_user_input("sess-1", "custom_success achieved");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        let has_positive = result
            .signals
            .iter()
            .any(|s| matches!(s, LightweightSignal::Positive { .. }));
        assert!(has_positive, "Should match custom positive pattern");
    }

    #[test]
    fn test_detector_mixed_signals() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Message with both positive and negative patterns
        let event = make_user_input("sess-1", "Thanks for the help but it's still broken");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        let has_positive = result
            .signals
            .iter()
            .any(|s| matches!(s, LightweightSignal::Positive { .. }));
        let has_negative = result
            .signals
            .iter()
            .any(|s| matches!(s, LightweightSignal::Negative { .. }));

        assert!(has_positive, "Should detect positive pattern");
        assert!(has_negative, "Should detect negative pattern");
    }

    #[test]
    fn test_successful_tool_does_not_add_signal() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // is_error=false means successful tool execution
        let event = make_tool_result("sess-1", "Bash", false);
        let result = detector
            .process(&event, &mut state, test_event_id())
            .unwrap();

        // Successful tool should not add a tool failure signal
        let tool_failures: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::ToolFailure { .. }))
            .collect();
        assert!(
            tool_failures.is_empty(),
            "Successful tool should not add failure signal"
        );
    }

    // === Hook Event Tests ===
    // These test that hook events from Claude Code are processed like their
    // equivalent VibesEvent types, allowing assessment of hook-captured sessions.

    fn make_hook_user_prompt(session_id: &str, prompt: &str) -> VibesEvent {
        VibesEvent::Hook {
            session_id: Some(session_id.to_string()),
            event: HookEvent::UserPromptSubmit(UserPromptSubmitData {
                session_id: Some(session_id.to_string()),
                transcript_path: None,
                cwd: None,
                hook_event_name: Some("UserPromptSubmit".to_string()),
                prompt: Some(prompt.to_string()),
            }),
        }
    }

    fn make_hook_tool_result(session_id: &str, tool_name: &str, success: bool) -> VibesEvent {
        use serde_json::json;
        VibesEvent::Hook {
            session_id: Some(session_id.to_string()),
            event: HookEvent::PostToolUse(PostToolUseData {
                session_id: Some(session_id.to_string()),
                transcript_path: None,
                cwd: None,
                permission_mode: None,
                hook_event_name: Some("PostToolUse".to_string()),
                tool_name: tool_name.to_string(),
                tool_input: None,
                tool_response: json!({
                    "stdout": if success { "Success" } else { "" },
                    "stderr": if success { "" } else { "Error: tool failed" },
                    "interrupted": !success,
                    "isImage": false
                }),
                tool_use_id: None,
            }),
        }
    }

    #[test]
    fn test_hook_user_prompt_processed_like_user_input() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Hook UserPromptSubmit should be processed like UserInput
        let event = make_hook_user_prompt("sess-1", "This is broken and doesn't work");
        let result = detector.process(&event, &mut state, test_event_id());

        // Should return Some (not None) - hook events should be processed
        assert!(
            result.is_some(),
            "Hook UserPromptSubmit should be processed, not ignored"
        );

        let result = result.unwrap();
        // Should detect negative patterns in the prompt
        let negative_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::Negative { .. }))
            .collect();
        assert!(
            !negative_signals.is_empty(),
            "Should detect negative patterns in hook prompt"
        );
    }

    #[test]
    fn test_hook_user_prompt_updates_ema() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Process a frustrating prompt via hook
        let event = make_hook_user_prompt("sess-1", "Error! This is broken!");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .expect("Hook event should be processed");

        // EMA should increase
        assert!(
            result.frustration_ema > 0.0,
            "Frustration EMA should increase from hook event"
        );
        assert_eq!(result.message_idx, 0, "Message index should be set");
    }

    #[test]
    fn test_hook_tool_failure_detected() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Hook PostToolUse with success=false should generate ToolFailure signal
        let event = make_hook_tool_result("sess-1", "Bash", false);
        let result = detector.process(&event, &mut state, test_event_id());

        assert!(
            result.is_some(),
            "Hook PostToolUse should be processed, not ignored"
        );

        let result = result.unwrap();
        let tool_failures: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::ToolFailure { .. }))
            .collect();
        assert_eq!(
            tool_failures.len(),
            1,
            "Should detect tool failure from hook event"
        );
    }

    #[test]
    fn test_hook_tool_success_no_failure_signal() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Hook PostToolUse with success=true should NOT generate ToolFailure signal
        let event = make_hook_tool_result("sess-1", "Bash", true);
        let result = detector.process(&event, &mut state, test_event_id());

        assert!(
            result.is_some(),
            "Hook PostToolUse should be processed, not ignored"
        );

        let result = result.unwrap();
        let tool_failures: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::ToolFailure { .. }))
            .collect();
        assert!(
            tool_failures.is_empty(),
            "Successful hook tool should not add failure signal"
        );
    }

    #[test]
    fn test_hook_positive_patterns_detected() {
        let detector = LightweightDetector::with_default_patterns();
        let mut state = SessionState::new();

        // Hook UserPromptSubmit with positive patterns
        let event = make_hook_user_prompt("sess-1", "Thank you, that's perfect!");
        let result = detector
            .process(&event, &mut state, test_event_id())
            .expect("Hook event should be processed");

        let positive_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s, LightweightSignal::Positive { .. }))
            .collect();
        assert!(
            !positive_signals.is_empty(),
            "Should detect positive patterns in hook prompt"
        );
    }
}

//! Circuit breaker for assessment intervention decisions.
//!
//! # Assessment Circuit Breaker vs. Classic Circuit Breaker
//!
//! This implementation differs significantly from the classic circuit breaker pattern
//! (e.g., Hystrix, resilience4j) used in distributed systems:
//!
//! | Aspect | Classic Pattern | Assessment Pattern |
//! |--------|-----------------|-------------------|
//! | **Purpose** | Protect failing services from cascade failures | Prevent user "assessment fatigue" from too many suggestions |
//! | **Trigger** | Service call failures (timeouts, errors) | Accumulated frustration signals from session events |
//! | **Open means** | Stop calling the service | Intervention was triggered, now in cooldown |
//! | **Closed means** | Service is healthy, allow calls | Session is progressing normally, watching for issues |
//! | **Half-Open** | Test if service recovered with probe requests | Test if session recovered after intervention |
//! | **Scope** | Per-service or per-endpoint | Per-session |
//!
//! ## Design Rationale
//!
//! In classic circuit breakers, "Open" is a **protective** state that stops traffic.
//! Here, "Open" means we **just intervened** and need to wait (cooldown) before
//! potentially intervening again. This prevents bombarding users with suggestions.
//!
//! The flow is:
//! 1. **Closed**: Monitor session for frustration signals
//! 2. **Opened**: Threshold exceeded → trigger intervention → enter cooldown
//! 3. **HalfOpen**: Cooldown expired → test if user situation improved
//! 4. **Closed**: Success signals detected → back to normal monitoring
//!
//! ## State Machine
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │                                                                  │
//! │   ┌────────┐    threshold    ┌──────┐   cooldown   ┌──────────┐ │
//! │   │ Closed │ ───exceeded───► │ Open │ ──expired──► │ HalfOpen │ │
//! │   └───┬────┘                 └──────┘              └────┬─────┘ │
//! │       │                          ▲                      │       │
//! │       │                          │                      │       │
//! │       │                      failure                 success    │
//! │       │                          │                      │       │
//! │       └◄─────────────────────────┴──────────────────────┘       │
//! │                           (reset to closed)                      │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Configuration
//!
//! - `cooldown_seconds`: How long to wait after an intervention before testing again
//! - `max_interventions_per_session`: Cap on total interventions to prevent fatigue

use std::time::{Duration, Instant};

use super::config::CircuitBreakerConfig;
use super::types::{LightweightEvent, LightweightSignal, SessionId};

/// State of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - monitoring for issues.
    Closed,
    /// Threshold exceeded - intervention triggered, now in cooldown.
    Open,
    /// Cooldown expired - testing if session has recovered.
    HalfOpen,
}

impl Default for CircuitState {
    fn default() -> Self {
        Self::Closed
    }
}

/// Transition event emitted when state changes.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitTransition {
    /// Circuit opened - intervention should be triggered.
    Opened {
        session_id: SessionId,
        trigger_reason: String,
    },
    /// Circuit moved to half-open after cooldown.
    HalfOpened { session_id: SessionId },
    /// Circuit closed after successful recovery.
    Closed { session_id: SessionId },
}

/// Per-session state for the circuit breaker.
#[derive(Debug, Clone)]
struct SessionCircuitState {
    /// Current circuit state.
    state: CircuitState,
    /// Cumulative failure score (from signals).
    failure_score: f64,
    /// Number of interventions triggered in this session.
    intervention_count: u32,
    /// When the circuit last opened (for cooldown tracking).
    last_opened: Option<Instant>,
    /// When the state last changed.
    last_state_change: Instant,
}

impl SessionCircuitState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_score: 0.0,
            intervention_count: 0,
            last_opened: None,
            last_state_change: Instant::now(),
        }
    }
}

/// Circuit breaker for controlling intervention frequency.
///
/// Maintains per-session state and decides when to trigger interventions
/// based on signal accumulation, cooldowns, and intervention limits.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    /// Failure score threshold to open circuit.
    threshold: f64,
    /// Per-session state.
    sessions: std::collections::HashMap<SessionId, SessionCircuitState>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            threshold: 1.0, // Default threshold
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Create a circuit breaker with a custom threshold.
    #[must_use]
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Get or create session state.
    fn get_session_mut(&mut self, session_id: &SessionId) -> &mut SessionCircuitState {
        self.sessions
            .entry(session_id.clone())
            .or_insert_with(SessionCircuitState::new)
    }

    /// Get the current state for a session.
    pub fn state(&self, session_id: &SessionId) -> CircuitState {
        self.sessions
            .get(session_id)
            .map(|s| s.state)
            .unwrap_or(CircuitState::Closed)
    }

    /// Check if circuit breaker is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Record an event and potentially trigger state transitions.
    ///
    /// Returns `Some(CircuitTransition)` if a state change occurred.
    pub fn record_event(&mut self, event: &LightweightEvent) -> Option<CircuitTransition> {
        if !self.config.enabled {
            return None;
        }

        let session_id = event.context.session_id.clone();

        // Calculate values before getting mutable session reference (avoids borrow conflicts)
        let failure_delta = self.calculate_failure_delta(&event.signals);
        let success_detected = self.has_success_signals(&event.signals);
        let cooldown = Duration::from_secs(self.config.cooldown_seconds.into());
        let threshold = self.threshold;
        let max_interventions = self.config.max_interventions_per_session;

        let session = self.get_session_mut(&session_id);

        // Check for cooldown expiry (Open -> HalfOpen)
        if session.state == CircuitState::Open
            && let Some(last_opened) = session.last_opened
            && last_opened.elapsed() >= cooldown
        {
            session.state = CircuitState::HalfOpen;
            session.last_state_change = Instant::now();
            return Some(CircuitTransition::HalfOpened {
                session_id: session_id.clone(),
            });
        }

        match session.state {
            CircuitState::Closed => {
                // Accumulate failure score
                session.failure_score += failure_delta;

                // Check if threshold exceeded
                if session.failure_score >= threshold {
                    // Check intervention limit
                    if session.intervention_count >= max_interventions {
                        // At limit - don't open, just decay score
                        session.failure_score *= 0.5;
                        return None;
                    }

                    // Open the circuit
                    let final_score = session.failure_score;
                    session.state = CircuitState::Open;
                    session.intervention_count += 1;
                    session.last_opened = Some(Instant::now());
                    session.last_state_change = Instant::now();
                    session.failure_score = 0.0; // Reset score

                    return Some(CircuitTransition::Opened {
                        session_id,
                        trigger_reason: format!(
                            "Failure threshold exceeded (score: {:.2})",
                            final_score
                        ),
                    });
                }

                None
            }

            CircuitState::Open => {
                // In cooldown - just accumulate (already handled cooldown check above)
                None
            }

            CircuitState::HalfOpen => {
                if success_detected {
                    // Recovery detected - close circuit
                    session.state = CircuitState::Closed;
                    session.failure_score = 0.0;
                    session.last_state_change = Instant::now();
                    return Some(CircuitTransition::Closed { session_id });
                } else if failure_delta > 0.0 {
                    // Still failing - reopen circuit
                    session.state = CircuitState::Open;
                    session.last_opened = Some(Instant::now());
                    session.last_state_change = Instant::now();
                    return Some(CircuitTransition::Opened {
                        session_id,
                        trigger_reason: "Still failing after recovery test".to_string(),
                    });
                }

                None
            }
        }
    }

    /// Calculate failure contribution from signals.
    fn calculate_failure_delta(&self, signals: &[LightweightSignal]) -> f64 {
        signals
            .iter()
            .map(|s| match s {
                LightweightSignal::Negative { confidence, .. } => *confidence,
                LightweightSignal::ToolFailure { .. } => 0.5,
                _ => 0.0,
            })
            .sum()
    }

    /// Check if there are success signals.
    fn has_success_signals(&self, signals: &[LightweightSignal]) -> bool {
        signals.iter().any(|s| {
            matches!(
                s,
                LightweightSignal::Positive { .. }
                    | LightweightSignal::BuildStatus { passed: true }
            )
        })
    }

    /// Get the number of interventions for a session.
    pub fn intervention_count(&self, session_id: &SessionId) -> u32 {
        self.sessions
            .get(session_id)
            .map(|s| s.intervention_count)
            .unwrap_or(0)
    }

    /// Remove session state (e.g., when session ends).
    pub fn remove_session(&mut self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("enabled", &self.config.enabled)
            .field("threshold", &self.threshold)
            .field("cooldown_seconds", &self.config.cooldown_seconds)
            .field(
                "max_interventions",
                &self.config.max_interventions_per_session,
            )
            .field("session_count", &self.sessions.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::types::AssessmentContext;

    fn make_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            enabled: true,
            cooldown_seconds: 1, // 1 second for tests
            max_interventions_per_session: 3,
        }
    }

    fn make_event(session_id: &str, signals: Vec<LightweightSignal>) -> LightweightEvent {
        LightweightEvent {
            context: AssessmentContext::new(session_id),
            message_idx: 0,
            signals,
            frustration_ema: 0.0,
            success_ema: 0.0,
        }
    }

    fn make_negative_signal() -> LightweightSignal {
        LightweightSignal::Negative {
            pattern: "error".to_string(),
            confidence: 0.5,
        }
    }

    fn make_positive_signal() -> LightweightSignal {
        LightweightSignal::Positive {
            pattern: "success".to_string(),
            confidence: 0.8,
        }
    }

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(make_config());
        let session_id = SessionId::from("test-session");

        assert_eq!(cb.state(&session_id), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_on_threshold() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(1.0);
        let session_id = SessionId::from("test-session");

        // First event with failure signals (0.5 contribution)
        let event1 = make_event("test-session", vec![make_negative_signal()]);
        let result1 = cb.record_event(&event1);
        assert!(result1.is_none()); // Not enough to open
        assert_eq!(cb.state(&session_id), CircuitState::Closed);

        // Second event with more failure signals (0.5 + 0.5 = 1.0 >= threshold)
        let event2 = make_event("test-session", vec![make_negative_signal()]);
        let result2 = cb.record_event(&event2);

        // Should have opened
        assert!(matches!(result2, Some(CircuitTransition::Opened { .. })));
        assert_eq!(cb.state(&session_id), CircuitState::Open);
        assert_eq!(cb.intervention_count(&session_id), 1);
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(0.5);
        let session_id = SessionId::from("test-session");

        // Open the circuit
        let event = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&event);
        assert!(matches!(result, Some(CircuitTransition::Opened { .. })));
        assert_eq!(cb.state(&session_id), CircuitState::Open);

        // Wait for cooldown (1 second)
        std::thread::sleep(Duration::from_millis(1100));

        // Next event should trigger transition to HalfOpen
        let neutral_event = make_event("test-session", vec![]);
        let result = cb.record_event(&neutral_event);

        assert!(matches!(result, Some(CircuitTransition::HalfOpened { .. })));
        assert_eq!(cb.state(&session_id), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes_on_success() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(0.5);
        let session_id = SessionId::from("test-session");

        // Open the circuit
        let event = make_event("test-session", vec![make_negative_signal()]);
        cb.record_event(&event);
        assert_eq!(cb.state(&session_id), CircuitState::Open);

        // Wait for cooldown to go to HalfOpen
        std::thread::sleep(Duration::from_millis(1100));
        let neutral_event = make_event("test-session", vec![]);
        cb.record_event(&neutral_event);
        assert_eq!(cb.state(&session_id), CircuitState::HalfOpen);

        // Success signal should close
        let success_event = make_event("test-session", vec![make_positive_signal()]);
        let result = cb.record_event(&success_event);

        assert!(matches!(result, Some(CircuitTransition::Closed { .. })));
        assert_eq!(cb.state(&session_id), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_respects_max_interventions() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            enabled: true,
            cooldown_seconds: 0, // No cooldown for this test
            max_interventions_per_session: 2,
        })
        .with_threshold(0.5);

        let session_id = SessionId::from("test-session");

        // First intervention
        let event = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&event);
        assert!(matches!(result, Some(CircuitTransition::Opened { .. })));
        assert_eq!(cb.intervention_count(&session_id), 1);

        // Force back to closed for next test
        {
            let session = cb.get_session_mut(&session_id);
            session.state = CircuitState::Closed;
        }

        // Second intervention
        let event = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&event);
        assert!(matches!(result, Some(CircuitTransition::Opened { .. })));
        assert_eq!(cb.intervention_count(&session_id), 2);

        // Force back to closed
        {
            let session = cb.get_session_mut(&session_id);
            session.state = CircuitState::Closed;
        }

        // Third attempt - should NOT open (at limit)
        let event = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&event);
        assert!(result.is_none()); // No transition
        assert_eq!(cb.intervention_count(&session_id), 2); // Still 2
    }

    #[test]
    fn test_circuit_breaker_disabled() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            enabled: false,
            ..make_config()
        });

        // Should not trigger any transitions when disabled
        let event = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&event);
        assert!(result.is_none());
    }

    #[test]
    fn test_circuit_breaker_session_isolation() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(0.5);

        // Open circuit for session 1
        let event1 = make_event("session-1", vec![make_negative_signal()]);
        cb.record_event(&event1);
        assert_eq!(cb.state(&SessionId::from("session-1")), CircuitState::Open);

        // Session 2 should still be closed
        assert_eq!(
            cb.state(&SessionId::from("session-2")),
            CircuitState::Closed
        );
    }

    #[test]
    fn test_circuit_breaker_remove_session() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(0.5);
        let session_id = SessionId::from("test-session");

        // Create some state
        let event = make_event("test-session", vec![make_negative_signal()]);
        cb.record_event(&event);
        assert_eq!(cb.intervention_count(&session_id), 1);

        // Remove session
        cb.remove_session(&session_id);

        // Should be back to default state
        assert_eq!(cb.state(&session_id), CircuitState::Closed);
        assert_eq!(cb.intervention_count(&session_id), 0);
    }

    #[test]
    fn test_circuit_breaker_reopens_on_failure_in_half_open() {
        let mut cb = CircuitBreaker::new(make_config()).with_threshold(0.5);
        let session_id = SessionId::from("test-session");

        // Open the circuit
        let event = make_event("test-session", vec![make_negative_signal()]);
        cb.record_event(&event);

        // Wait for cooldown
        std::thread::sleep(Duration::from_millis(1100));

        // Go to HalfOpen
        let neutral = make_event("test-session", vec![]);
        cb.record_event(&neutral);
        assert_eq!(cb.state(&session_id), CircuitState::HalfOpen);

        // Another failure should reopen (but NOT increment intervention count again)
        let failure = make_event("test-session", vec![make_negative_signal()]);
        let result = cb.record_event(&failure);

        assert!(matches!(result, Some(CircuitTransition::Opened { .. })));
        assert_eq!(cb.state(&session_id), CircuitState::Open);
        // Intervention count should still be 1 (reopen doesn't count)
        assert_eq!(cb.intervention_count(&session_id), 1);
    }
}

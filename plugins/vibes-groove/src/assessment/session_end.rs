//! Session end detection via event or timeout.
//!
//! The `SessionEndDetector` determines when a session has ended, which triggers
//! final assessment processing. A session can end in two ways:
//!
//! 1. **Explicit**: A `SessionEnded` event is received from the harness
//! 2. **Inactivity timeout**: No activity for the configured timeout period
//!
//! ## Usage Flow
//!
//! ```text
//! VibesEvent → SessionEndDetector.process()
//!                       │
//!                       ├─ SessionEnded event → SessionEnd::Explicit
//!                       │
//!                       └─ Other events → Update last_activity timestamp
//!
//! Timer tick → SessionEndDetector.check_timeouts()
//!                       │
//!                       └─ For each session with elapsed timeout → SessionEnd::InactivityTimeout
//! ```
//!
//! ## Configuration
//!
//! - `timeout_minutes`: How long without activity before timeout (default: 15)
//! - `timeout_enabled`: Whether timeout detection is active
//! - `hook_enabled`: Whether to detect explicit session end events

use std::collections::HashMap;
use std::time::{Duration, Instant};

use vibes_core::VibesEvent;

use super::config::SessionEndConfig;
use super::types::SessionId;

/// Reason why a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionEndReason {
    /// Session ended via explicit SessionEnded event.
    Explicit,
    /// Session ended due to inactivity timeout.
    InactivityTimeout,
}

impl SessionEndReason {
    /// Convert to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::InactivityTimeout => "inactivity_timeout",
        }
    }
}

impl std::fmt::Display for SessionEndReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A detected session end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEnd {
    /// The session that ended.
    pub session_id: SessionId,
    /// Why the session ended.
    pub reason: SessionEndReason,
}

impl SessionEnd {
    /// Create a new session end.
    pub fn new(session_id: impl Into<SessionId>, reason: SessionEndReason) -> Self {
        Self {
            session_id: session_id.into(),
            reason,
        }
    }

    /// Create an explicit session end.
    pub fn explicit(session_id: impl Into<SessionId>) -> Self {
        Self::new(session_id, SessionEndReason::Explicit)
    }

    /// Create a timeout session end.
    pub fn timeout(session_id: impl Into<SessionId>) -> Self {
        Self::new(session_id, SessionEndReason::InactivityTimeout)
    }
}

/// Per-session tracking state.
#[derive(Debug)]
struct SessionActivity {
    /// Last activity timestamp.
    last_activity: Instant,
    /// Whether this session has already been marked as ended.
    ended: bool,
}

impl SessionActivity {
    fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            ended: false,
        }
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
    }
}

/// Detects session end via event or timeout.
///
/// Tracks activity per session and determines when sessions have ended,
/// either through explicit events or inactivity timeouts.
pub struct SessionEndDetector {
    config: SessionEndConfig,
    sessions: HashMap<SessionId, SessionActivity>,
}

impl SessionEndDetector {
    /// Create a new session end detector with the given configuration.
    pub fn new(config: SessionEndConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
        }
    }

    /// Get the configured timeout duration.
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.config.timeout_minutes as u64 * 60)
    }

    /// Process an event and check for session end.
    ///
    /// Returns `Some(SessionEnd)` if this event ends the session,
    /// `None` otherwise.
    pub fn process(&mut self, event: &VibesEvent) -> Option<SessionEnd> {
        let session_id_str = event.session_id()?;
        let session_id = SessionId::from(session_id_str);

        // Check for explicit session end via SessionRemoved event
        if self.config.hook_enabled
            && let VibesEvent::SessionRemoved { .. } = event
        {
            // Mark session as ended and remove tracking
            self.sessions.remove(&session_id);
            return Some(SessionEnd::explicit(session_id));
        }

        // Update activity timestamp for this session
        let session = self
            .sessions
            .entry(session_id)
            .or_insert_with(SessionActivity::new);
        session.touch();

        None
    }

    /// Check for sessions that have timed out.
    ///
    /// Returns a list of sessions that have exceeded the inactivity timeout.
    /// This should be called periodically (e.g., every minute).
    pub fn check_timeouts(&mut self) -> Vec<SessionEnd> {
        if !self.config.timeout_enabled {
            return Vec::new();
        }

        let timeout = self.timeout_duration();
        let now = Instant::now();
        let mut ended = Vec::new();

        // Find sessions that have timed out
        let timed_out: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, activity)| {
                !activity.ended && now.duration_since(activity.last_activity) >= timeout
            })
            .map(|(id, _)| id.clone())
            .collect();

        // Mark them as ended and collect results
        for session_id in timed_out {
            if let Some(activity) = self.sessions.get_mut(&session_id) {
                activity.ended = true;
            }
            ended.push(SessionEnd::timeout(session_id));
        }

        ended
    }

    /// Remove a session from tracking.
    ///
    /// Call this after processing a session end to clean up.
    pub fn remove_session(&mut self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }

    /// Get the number of active sessions being tracked.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if a session is being tracked.
    pub fn is_tracking(&self, session_id: &SessionId) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Get time since last activity for a session.
    ///
    /// Returns `None` if the session is not being tracked.
    pub fn time_since_activity(&self, session_id: &SessionId) -> Option<Duration> {
        self.sessions
            .get(session_id)
            .map(|s| s.last_activity.elapsed())
    }
}

impl Default for SessionEndDetector {
    fn default() -> Self {
        Self::new(SessionEndConfig::default())
    }
}

impl std::fmt::Debug for SessionEndDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionEndDetector")
            .field("hook_enabled", &self.config.hook_enabled)
            .field("timeout_enabled", &self.config.timeout_enabled)
            .field("timeout_minutes", &self.config.timeout_minutes)
            .field("session_count", &self.sessions.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user_input(session_id: &str) -> VibesEvent {
        VibesEvent::UserInput {
            session_id: session_id.to_string(),
            content: "test input".to_string(),
            source: vibes_core::InputSource::Unknown,
        }
    }

    fn make_session_removed(session_id: &str) -> VibesEvent {
        VibesEvent::SessionRemoved {
            session_id: session_id.to_string(),
            reason: "test".to_string(),
        }
    }

    fn make_session_created(session_id: &str) -> VibesEvent {
        VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        }
    }

    #[test]
    fn test_session_end_explicit() {
        let config = SessionEndConfig {
            hook_enabled: true,
            timeout_enabled: false,
            timeout_minutes: 15,
        };
        let mut detector = SessionEndDetector::new(config);

        // Process some events first to track the session
        let session_id = SessionId::from("test-session");
        detector.process(&make_session_created("test-session"));
        detector.process(&make_user_input("test-session"));
        assert!(detector.is_tracking(&session_id));

        // Now end the session explicitly
        let result = detector.process(&make_session_removed("test-session"));

        assert!(result.is_some());
        let end = result.unwrap();
        assert_eq!(end.session_id.as_str(), "test-session");
        assert_eq!(end.reason, SessionEndReason::Explicit);

        // Session should no longer be tracked
        assert!(!detector.is_tracking(&session_id));
    }

    #[test]
    fn test_session_end_timeout() {
        let config = SessionEndConfig {
            hook_enabled: false,
            timeout_enabled: true,
            timeout_minutes: 0, // Immediate timeout for testing
        };
        let mut detector = SessionEndDetector::new(config);

        // Process an event to start tracking
        detector.process(&make_user_input("test-session"));
        assert_eq!(detector.session_count(), 1);

        // Wait a tiny bit and check timeouts
        std::thread::sleep(std::time::Duration::from_millis(10));
        let timed_out = detector.check_timeouts();

        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0].session_id.as_str(), "test-session");
        assert_eq!(timed_out[0].reason, SessionEndReason::InactivityTimeout);
    }

    #[test]
    fn test_session_activity_updates() {
        let config = SessionEndConfig {
            hook_enabled: false,
            timeout_enabled: true,
            timeout_minutes: 1, // 1 minute timeout
        };
        let mut detector = SessionEndDetector::new(config);
        let session_id = SessionId::from("test-session");

        // Process first event
        detector.process(&make_user_input("test-session"));
        let time1 = detector.time_since_activity(&session_id);
        assert!(time1.is_some());

        // Wait long enough for reliable timing comparison
        std::thread::sleep(std::time::Duration::from_millis(100));
        let time1_after_wait = detector.time_since_activity(&session_id);
        assert!(time1_after_wait.unwrap() > time1.unwrap());

        // Process another event - resets the timer
        detector.process(&make_user_input("test-session"));
        let time2 = detector.time_since_activity(&session_id);

        // Time since activity should be less after the second event (timer reset)
        assert!(time2.unwrap() < time1_after_wait.unwrap());
    }

    #[test]
    fn test_hook_disabled_ignores_session_ended() {
        let config = SessionEndConfig {
            hook_enabled: false, // Disabled
            timeout_enabled: false,
            timeout_minutes: 15,
        };
        let mut detector = SessionEndDetector::new(config);

        // Track the session first
        detector.process(&make_user_input("test-session"));

        // SessionRemoved event should not trigger end when hook is disabled
        let result = detector.process(&make_session_removed("test-session"));
        assert!(result.is_none());

        // Session should still be tracked
        assert!(detector.is_tracking(&SessionId::from("test-session")));
    }

    #[test]
    fn test_timeout_disabled_returns_empty() {
        let config = SessionEndConfig {
            hook_enabled: false,
            timeout_enabled: false, // Disabled
            timeout_minutes: 0,
        };
        let mut detector = SessionEndDetector::new(config);

        // Track a session
        detector.process(&make_user_input("test-session"));

        // Even with 0 minute timeout, should return empty when disabled
        std::thread::sleep(std::time::Duration::from_millis(10));
        let timed_out = detector.check_timeouts();
        assert!(timed_out.is_empty());
    }

    #[test]
    fn test_multiple_sessions_timeout_independently() {
        let config = SessionEndConfig {
            hook_enabled: false,
            timeout_enabled: true,
            timeout_minutes: 0, // Immediate timeout
        };
        let mut detector = SessionEndDetector::new(config);

        // Track multiple sessions
        detector.process(&make_user_input("session-1"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        detector.process(&make_user_input("session-2"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        detector.process(&make_user_input("session-3"));

        assert_eq!(detector.session_count(), 3);

        // All should timeout
        std::thread::sleep(std::time::Duration::from_millis(10));
        let timed_out = detector.check_timeouts();
        assert_eq!(timed_out.len(), 3);
    }

    #[test]
    fn test_session_only_times_out_once() {
        let config = SessionEndConfig {
            hook_enabled: false,
            timeout_enabled: true,
            timeout_minutes: 0,
        };
        let mut detector = SessionEndDetector::new(config);

        detector.process(&make_user_input("test-session"));
        std::thread::sleep(std::time::Duration::from_millis(10));

        // First check should find timeout
        let first = detector.check_timeouts();
        assert_eq!(first.len(), 1);

        // Second check should not find it again
        let second = detector.check_timeouts();
        assert!(second.is_empty());
    }

    #[test]
    fn test_remove_session() {
        let mut detector = SessionEndDetector::default();
        let session_id = SessionId::from("test-session");

        detector.process(&make_user_input("test-session"));
        assert!(detector.is_tracking(&session_id));

        detector.remove_session(&session_id);
        assert!(!detector.is_tracking(&session_id));
        assert_eq!(detector.session_count(), 0);
    }

    #[test]
    fn test_session_end_reason_display() {
        assert_eq!(SessionEndReason::Explicit.to_string(), "explicit");
        assert_eq!(
            SessionEndReason::InactivityTimeout.to_string(),
            "inactivity_timeout"
        );
    }

    #[test]
    fn test_session_end_constructors() {
        let explicit = SessionEnd::explicit("session-1");
        assert_eq!(explicit.session_id.as_str(), "session-1");
        assert_eq!(explicit.reason, SessionEndReason::Explicit);

        let timeout = SessionEnd::timeout("session-2");
        assert_eq!(timeout.session_id.as_str(), "session-2");
        assert_eq!(timeout.reason, SessionEndReason::InactivityTimeout);
    }

    #[test]
    fn test_detector_debug() {
        let detector = SessionEndDetector::default();
        let debug = format!("{detector:?}");
        assert!(debug.contains("SessionEndDetector"));
        assert!(debug.contains("hook_enabled"));
        assert!(debug.contains("timeout_enabled"));
    }

    #[test]
    fn test_timeout_duration() {
        let config = SessionEndConfig {
            hook_enabled: true,
            timeout_enabled: true,
            timeout_minutes: 15,
        };
        let detector = SessionEndDetector::new(config);

        assert_eq!(detector.timeout_duration(), Duration::from_secs(15 * 60));
    }
}

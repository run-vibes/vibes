//! Session event buffer for batch processing.
//!
//! The `SessionBuffer` collects events per session for later batch processing.
//! It provides bounded storage with LRU eviction to prevent unbounded memory growth.
//!
//! ## Usage Pattern
//!
//! 1. Push events as they arrive: `buffer.push(session_id, event)`
//! 2. When checkpoint triggers, drain the session: `buffer.drain(&session_id)`
//! 3. Analyze the drained events for learnings
//!
//! ## Memory Management
//!
//! - `max_events_per_session`: Ring buffer per session (old events evicted)
//! - `max_sessions`: Total sessions tracked (LRU eviction of oldest sessions)

use std::collections::{HashMap, VecDeque};

use vibes_core::VibesEvent;

use super::types::SessionId;

/// Default maximum events per session buffer.
const DEFAULT_MAX_EVENTS_PER_SESSION: usize = 100;

/// Default maximum sessions to track.
const DEFAULT_MAX_SESSIONS: usize = 50;

/// Configuration for the session buffer.
#[derive(Debug, Clone)]
pub struct SessionBufferConfig {
    /// Maximum events to buffer per session.
    pub max_events_per_session: usize,
    /// Maximum sessions to track (LRU eviction applies).
    pub max_sessions: usize,
}

impl Default for SessionBufferConfig {
    fn default() -> Self {
        Self {
            max_events_per_session: DEFAULT_MAX_EVENTS_PER_SESSION,
            max_sessions: DEFAULT_MAX_SESSIONS,
        }
    }
}

/// Metadata for a buffered session.
#[derive(Debug)]
struct SessionEntry {
    /// The event buffer for this session.
    events: VecDeque<VibesEvent>,
    /// Last access time (for LRU ordering).
    last_access: std::time::Instant,
}

impl SessionEntry {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            last_access: std::time::Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.last_access = std::time::Instant::now();
    }
}

/// Buffer for collecting events per session.
///
/// Provides bounded storage with:
/// - Per-session ring buffer (old events evicted first)
/// - LRU eviction when max sessions exceeded
pub struct SessionBuffer {
    config: SessionBufferConfig,
    sessions: HashMap<SessionId, SessionEntry>,
}

impl SessionBuffer {
    /// Create a new session buffer with the given configuration.
    pub fn new(config: SessionBufferConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
        }
    }

    /// Push an event to the buffer for a session.
    ///
    /// If the session doesn't exist, it's created. If max sessions is exceeded,
    /// the least recently used session is evicted.
    pub fn push(&mut self, session_id: SessionId, event: VibesEvent) {
        // Check if we need to evict a session
        if !self.sessions.contains_key(&session_id)
            && self.sessions.len() >= self.config.max_sessions
        {
            self.evict_lru();
        }

        let entry = self
            .sessions
            .entry(session_id)
            .or_insert_with(SessionEntry::new);
        entry.touch();

        // Ring buffer: remove oldest if at capacity
        if entry.events.len() >= self.config.max_events_per_session {
            entry.events.pop_front();
        }

        entry.events.push_back(event);
    }

    /// Get a reference to the event buffer for a session.
    ///
    /// Returns `None` if the session doesn't exist.
    pub fn get(&self, session_id: &SessionId) -> Option<&VecDeque<VibesEvent>> {
        self.sessions.get(session_id).map(|e| &e.events)
    }

    /// Get a mutable reference to the event buffer for a session.
    ///
    /// Returns `None` if the session doesn't exist.
    pub fn get_mut(&mut self, session_id: &SessionId) -> Option<&mut VecDeque<VibesEvent>> {
        self.sessions.get_mut(session_id).map(|e| {
            e.touch();
            &mut e.events
        })
    }

    /// Drain all events from a session's buffer.
    ///
    /// Returns the events and clears the buffer. The session entry remains
    /// for future events. Returns an empty Vec if session doesn't exist.
    pub fn drain(&mut self, session_id: &SessionId) -> Vec<VibesEvent> {
        self.sessions
            .get_mut(session_id)
            .map(|e| {
                e.touch();
                e.events.drain(..).collect()
            })
            .unwrap_or_default()
    }

    /// Remove a session entirely from the buffer.
    ///
    /// Returns the buffered events, or None if session didn't exist.
    pub fn remove(&mut self, session_id: &SessionId) -> Option<Vec<VibesEvent>> {
        self.sessions
            .remove(session_id)
            .map(|e| e.events.into_iter().collect())
    }

    /// Get the number of events buffered for a session.
    pub fn len(&self, session_id: &SessionId) -> usize {
        self.sessions
            .get(session_id)
            .map(|e| e.events.len())
            .unwrap_or(0)
    }

    /// Check if a session's buffer is empty.
    pub fn is_empty(&self, session_id: &SessionId) -> bool {
        self.len(session_id) == 0
    }

    /// Get the number of sessions being tracked.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Evict the least recently used session.
    fn evict_lru(&mut self) {
        if let Some((lru_session, _)) = self
            .sessions
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, v)| (k.clone(), v.last_access))
        {
            self.sessions.remove(&lru_session);
        }
    }
}

impl Default for SessionBuffer {
    fn default() -> Self {
        Self::new(SessionBufferConfig::default())
    }
}

impl std::fmt::Debug for SessionBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionBuffer")
            .field(
                "max_events_per_session",
                &self.config.max_events_per_session,
            )
            .field("max_sessions", &self.config.max_sessions)
            .field("session_count", &self.sessions.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(session_id: &str) -> VibesEvent {
        VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        }
    }

    fn make_user_input(session_id: &str, content: &str) -> VibesEvent {
        VibesEvent::UserInput {
            session_id: session_id.to_string(),
            content: content.to_string(),
            source: vibes_core::InputSource::Unknown,
        }
    }

    #[test]
    fn test_session_buffer_collects_events() {
        let mut buffer = SessionBuffer::default();
        let session_id = SessionId::from("test-session");

        // Push some events
        buffer.push(session_id.clone(), make_user_input("test-session", "Hello"));
        buffer.push(session_id.clone(), make_user_input("test-session", "World"));

        // Verify events are collected
        assert_eq!(buffer.len(&session_id), 2);
        assert!(!buffer.is_empty(&session_id));

        let events = buffer.get(&session_id).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_session_buffer_drain() {
        let mut buffer = SessionBuffer::default();
        let session_id = SessionId::from("test-session");

        // Push events
        buffer.push(session_id.clone(), make_user_input("test-session", "One"));
        buffer.push(session_id.clone(), make_user_input("test-session", "Two"));

        // Drain
        let events = buffer.drain(&session_id);
        assert_eq!(events.len(), 2);

        // Buffer should be empty but session still exists
        assert!(buffer.is_empty(&session_id));
        assert_eq!(buffer.session_count(), 1);

        // Can push more events
        buffer.push(session_id.clone(), make_user_input("test-session", "Three"));
        assert_eq!(buffer.len(&session_id), 1);
    }

    #[test]
    fn test_session_buffer_per_session_limit() {
        let config = SessionBufferConfig {
            max_events_per_session: 3,
            max_sessions: 10,
        };
        let mut buffer = SessionBuffer::new(config);
        let session_id = SessionId::from("test-session");

        // Push more events than the limit
        for i in 0..5 {
            buffer.push(
                session_id.clone(),
                make_user_input("test-session", &format!("Event {}", i)),
            );
        }

        // Should only have the last 3 events
        assert_eq!(buffer.len(&session_id), 3);

        let events = buffer.drain(&session_id);
        // Verify they are the most recent events (ring buffer semantics)
        if let VibesEvent::UserInput { content, .. } = &events[0] {
            assert_eq!(content, "Event 2");
        } else {
            panic!("Expected UserInput event");
        }
        if let VibesEvent::UserInput { content, .. } = &events[2] {
            assert_eq!(content, "Event 4");
        } else {
            panic!("Expected UserInput event");
        }
    }

    #[test]
    fn test_session_buffer_lru_eviction() {
        let config = SessionBufferConfig {
            max_events_per_session: 10,
            max_sessions: 3,
        };
        let mut buffer = SessionBuffer::new(config);

        // Add 3 sessions
        buffer.push(SessionId::from("session-1"), make_event("session-1"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        buffer.push(SessionId::from("session-2"), make_event("session-2"));
        std::thread::sleep(std::time::Duration::from_millis(10));
        buffer.push(SessionId::from("session-3"), make_event("session-3"));

        assert_eq!(buffer.session_count(), 3);

        // Access session-1 to make it recently used
        std::thread::sleep(std::time::Duration::from_millis(10));
        buffer.push(SessionId::from("session-1"), make_event("session-1"));

        // Add 4th session - should evict session-2 (LRU)
        buffer.push(SessionId::from("session-4"), make_event("session-4"));

        assert_eq!(buffer.session_count(), 3);
        assert!(buffer.get(&SessionId::from("session-1")).is_some());
        assert!(buffer.get(&SessionId::from("session-2")).is_none()); // Evicted
        assert!(buffer.get(&SessionId::from("session-3")).is_some());
        assert!(buffer.get(&SessionId::from("session-4")).is_some());
    }

    #[test]
    fn test_session_buffer_remove() {
        let mut buffer = SessionBuffer::default();
        let session_id = SessionId::from("test-session");

        buffer.push(session_id.clone(), make_event("test-session"));
        buffer.push(session_id.clone(), make_event("test-session"));

        // Remove session
        let events = buffer.remove(&session_id);
        assert!(events.is_some());
        assert_eq!(events.unwrap().len(), 2);

        // Session no longer exists
        assert!(buffer.get(&session_id).is_none());
        assert_eq!(buffer.session_count(), 0);
    }

    #[test]
    fn test_session_buffer_drain_nonexistent() {
        let mut buffer = SessionBuffer::default();
        let session_id = SessionId::from("nonexistent");

        // Drain nonexistent session returns empty vec
        let events = buffer.drain(&session_id);
        assert!(events.is_empty());
    }

    #[test]
    fn test_session_buffer_multiple_sessions() {
        let mut buffer = SessionBuffer::default();

        // Push to multiple sessions
        buffer.push(
            SessionId::from("session-a"),
            make_user_input("session-a", "A1"),
        );
        buffer.push(
            SessionId::from("session-b"),
            make_user_input("session-b", "B1"),
        );
        buffer.push(
            SessionId::from("session-a"),
            make_user_input("session-a", "A2"),
        );

        assert_eq!(buffer.session_count(), 2);
        assert_eq!(buffer.len(&SessionId::from("session-a")), 2);
        assert_eq!(buffer.len(&SessionId::from("session-b")), 1);
    }

    #[test]
    fn test_session_buffer_config_defaults() {
        let config = SessionBufferConfig::default();

        assert_eq!(
            config.max_events_per_session,
            DEFAULT_MAX_EVENTS_PER_SESSION
        );
        assert_eq!(config.max_sessions, DEFAULT_MAX_SESSIONS);
    }
}

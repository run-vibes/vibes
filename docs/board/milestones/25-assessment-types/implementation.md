# Milestone 4.4.1: Assessment Framework Infrastructure - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Get Iggy running as a supervised subprocess with event log infrastructure ready for assessment logic.

**Architecture:** vibes daemon spawns and supervises an Iggy server process. Assessment events flow through a trait-based `AssessmentLog` abstraction with an Iggy implementation. A skeleton `AssessmentProcessor` subscribes to the EventBus and writes events via a fire-and-forget channel.

**Tech Stack:** Iggy SDK (`iggy` crate), tokio process management, serde for event serialization, existing vibes-core EventBus.

**Reference:** See `docs/plans/14-continual-learning/milestone-4.4-design.md` for full design context.

---

## Task 1: Add Iggy Dependencies

**Files:**
- Modify: `vibes-groove/Cargo.toml`
- Modify: `vibes-core/Cargo.toml`

**Step 1: Add iggy crate to vibes-groove**

Add to `vibes-groove/Cargo.toml` dependencies:

```toml
# Message streaming (assessment event log)
iggy = "0.6"
```

**Step 2: Add process management deps to vibes-core**

We'll need tokio process features. Check if already enabled in `vibes-core/Cargo.toml`:

```toml
tokio = { workspace = true, features = ["sync", "rt", "process", "time"] }
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-groove -p vibes-core`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add vibes-groove/Cargo.toml vibes-core/Cargo.toml
git commit -m "chore: add iggy dependency for assessment event log"
```

---

## Task 2: Assessment Event Types

**Files:**
- Create: `vibes-groove/src/assessment/mod.rs`
- Create: `vibes-groove/src/assessment/types.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write failing test for AssessmentContext**

Create `vibes-groove/src/assessment/types.rs`:

```rust
//! Assessment event types for the three-tier assessment model.
//!
//! These types carry full attribution context for lineage tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{LearningId, Scope};

/// Unique identifier for assessment events (UUIDv7 for time-ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    /// Create a new time-ordered event ID
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// How learnings were injected into the session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMethod {
    /// Injected via CLAUDE.md @import
    ClaudeMd,
    /// Injected via SessionStart hook
    Hook,
    /// Injected via both methods
    Both,
    /// No injection (baseline session)
    None,
}

/// Type of AI harness being used
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessType {
    ClaudeCode,
    Other(String),
}

impl Default for HarnessType {
    fn default() -> Self {
        Self::ClaudeCode
    }
}

/// User identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

/// Project identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

/// Context attached to every assessment event for attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentContext {
    /// Session this event belongs to
    pub session_id: SessionId,
    /// Unique event identifier (UUIDv7)
    pub event_id: EventId,
    /// When this event occurred
    pub timestamp: DateTime<Utc>,

    // Lineage
    /// Learnings that were active at session start
    pub active_learnings: Vec<LearningId>,
    /// How learnings were injected
    pub injection_method: InjectionMethod,
    /// Scope of injected learnings
    pub injection_scope: Option<Scope>,

    // Harness context
    /// Type of harness being used
    pub harness_type: HarnessType,
    /// Version of the harness (if known)
    pub harness_version: Option<String>,

    // Environment
    /// Project this session is in (if any)
    pub project_id: Option<ProjectId>,
    /// User running the session
    pub user_id: UserId,
}

impl AssessmentContext {
    /// Create a new assessment context for a session
    pub fn new(session_id: SessionId, user_id: UserId) -> Self {
        Self {
            session_id,
            event_id: EventId::new(),
            timestamp: Utc::now(),
            active_learnings: Vec::new(),
            injection_method: InjectionMethod::None,
            injection_scope: None,
            harness_type: HarnessType::default(),
            harness_version: None,
            project_id: None,
            user_id,
        }
    }

    /// Create context with a specific event ID (for testing)
    pub fn with_event_id(mut self, event_id: EventId) -> Self {
        self.event_id = event_id;
        self
    }

    /// Set active learnings
    pub fn with_learnings(mut self, learnings: Vec<LearningId>, method: InjectionMethod) -> Self {
        self.active_learnings = learnings;
        self.injection_method = method;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_is_time_ordered() {
        let id1 = EventId::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = EventId::new();
        // UUIDv7 timestamps should be monotonically increasing
        assert!(id2.0 > id1.0);
    }

    #[test]
    fn session_id_from_string() {
        let id: SessionId = "sess-123".into();
        assert_eq!(id.0, "sess-123");
    }

    #[test]
    fn assessment_context_new_has_defaults() {
        let ctx = AssessmentContext::new("sess-1".into(), UserId("user-1".into()));
        assert_eq!(ctx.session_id.0, "sess-1");
        assert_eq!(ctx.user_id.0, "user-1");
        assert!(ctx.active_learnings.is_empty());
        assert_eq!(ctx.injection_method, InjectionMethod::None);
        assert!(matches!(ctx.harness_type, HarnessType::ClaudeCode));
    }

    #[test]
    fn assessment_context_serialization_roundtrip() {
        let ctx = AssessmentContext::new("sess-1".into(), UserId("user-1".into()));
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: AssessmentContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id.0, ctx.session_id.0);
    }

    #[test]
    fn injection_method_serializes_to_snake_case() {
        let json = serde_json::to_string(&InjectionMethod::ClaudeMd).unwrap();
        assert_eq!(json, "\"claude_md\"");
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test -p vibes-groove assessment::types`
Expected: All tests pass

**Step 3: Create module file**

Create `vibes-groove/src/assessment/mod.rs`:

```rust
//! Assessment framework for measuring session outcomes.
//!
//! This module implements the three-tier assessment model:
//! - Lightweight: Per-message signal detection (<10ms)
//! - Medium: Checkpoint summarization (async)
//! - Heavy: Full session analysis (sampled)

pub mod types;

pub use types::*;
```

**Step 4: Export from lib.rs**

Add to `vibes-groove/src/lib.rs` after other module declarations:

```rust
pub mod assessment;
```

And add to re-exports section:

```rust
pub use assessment::{
    AssessmentContext, EventId, HarnessType, InjectionMethod, ProjectId, SessionId, UserId,
};
```

**Step 5: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add vibes-groove/src/assessment/
git add vibes-groove/src/lib.rs
git commit -m "feat(groove): add assessment context types with attribution lineage"
```

---

## Task 3: Assessment Event Types (Lightweight, Medium, Heavy)

**Files:**
- Modify: `vibes-groove/src/assessment/types.rs`

**Step 1: Add lightweight signal types**

Append to `vibes-groove/src/assessment/types.rs`:

```rust
/// A lightweight signal detected in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightweightSignal {
    /// Negative linguistic pattern detected
    Negative {
        pattern: String,
        confidence: f64,
    },
    /// Positive linguistic pattern detected
    Positive {
        pattern: String,
        confidence: f64,
    },
    /// Tool execution failed
    ToolFailure {
        tool_name: String,
    },
    /// User corrected Claude's output
    Correction,
    /// User retried the same request
    Retry,
    /// Build/test status changed
    BuildStatus {
        passed: bool,
    },
}

/// Lightweight assessment event (per-message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightEvent {
    /// Attribution context
    pub context: AssessmentContext,
    /// Message index in the session
    pub message_idx: u32,
    /// Signals detected in this message
    pub signals: Vec<LightweightSignal>,
    /// Running frustration EMA (0.0 - 1.0)
    pub frustration_ema: f64,
    /// Running success EMA (0.0 - 1.0)
    pub success_ema: f64,
}

/// What triggered a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointTrigger {
    /// N messages elapsed
    MessageCount(u32),
    /// Task boundary detected
    TaskBoundary,
    /// Git commit detected
    GitCommit,
    /// Build/test passed
    BuildPass,
    /// Session paused (inactivity)
    SessionPause,
}

/// Unique identifier for a checkpoint
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);

impl CheckpointId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for CheckpointId {
    fn default() -> Self {
        Self::new()
    }
}

/// Token usage metrics for a segment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Total tokens in segment
    pub total_tokens: u64,
    /// Ratio of correction tokens to total
    pub correction_ratio: f64,
    /// Number of retries in segment
    pub retry_count: u32,
}

/// Medium assessment event (per-checkpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumEvent {
    /// Attribution context
    pub context: AssessmentContext,
    /// Unique checkpoint identifier
    pub checkpoint_id: CheckpointId,
    /// Message range (start, end) inclusive
    pub message_range: (u32, u32),
    /// What triggered this checkpoint
    pub trigger: CheckpointTrigger,
    /// LLM-generated summary of the segment
    pub summary: String,
    /// Token metrics for the segment
    pub token_metrics: TokenMetrics,
    /// Computed segment score (0.0 - 1.0)
    pub segment_score: f64,
    /// Learnings that were referenced in this segment
    pub learnings_referenced: Vec<LearningId>,
}

/// Session outcome classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Partial,
    Failure,
    Abandoned,
}

/// Attribution score for a learning (-1.0 to 1.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AttributionScore(pub f64);

impl AttributionScore {
    pub fn new(score: f64) -> Self {
        Self(score.clamp(-1.0, 1.0))
    }
}

/// Candidate for learning extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionCandidate {
    /// Message range containing the pattern
    pub message_range: (u32, u32),
    /// Brief description of what could be learned
    pub description: String,
    /// Confidence that this is worth extracting
    pub confidence: f64,
}

/// Heavy assessment event (per-session, sampled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeavyEvent {
    /// Attribution context
    pub context: AssessmentContext,
    /// Overall session outcome
    pub outcome: Outcome,
    /// LLM-generated task summary
    pub task_summary: String,
    /// Frustration EMA trajectory over session
    pub frustration_trajectory: Vec<f64>,
    /// Attribution scores for active learnings
    pub learning_attributions: std::collections::HashMap<LearningId, AttributionScore>,
    /// Candidates for learning extraction (fed to 4.5)
    pub extraction_candidates: Vec<ExtractionCandidate>,
}

/// Union of all assessment event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tier", rename_all = "snake_case")]
pub enum AssessmentEvent {
    Lightweight(LightweightEvent),
    Medium(MediumEvent),
    Heavy(HeavyEvent),
}

impl AssessmentEvent {
    /// Get the session ID for this event
    pub fn session_id(&self) -> &SessionId {
        match self {
            Self::Lightweight(e) => &e.context.session_id,
            Self::Medium(e) => &e.context.session_id,
            Self::Heavy(e) => &e.context.session_id,
        }
    }

    /// Get the event ID for this event
    pub fn event_id(&self) -> &EventId {
        match self {
            Self::Lightweight(e) => &e.context.event_id,
            Self::Medium(e) => &e.context.event_id,
            Self::Heavy(e) => &e.context.event_id,
        }
    }
}
```

**Step 2: Add tests for event types**

Append to the tests module in `vibes-groove/src/assessment/types.rs`:

```rust
    #[test]
    fn lightweight_signal_serialization() {
        let signal = LightweightSignal::Negative {
            pattern: "why didn't you".into(),
            confidence: 0.9,
        };
        let json = serde_json::to_string(&signal).unwrap();
        assert!(json.contains("\"type\":\"negative\""));
        let parsed: LightweightSignal = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, LightweightSignal::Negative { .. }));
    }

    #[test]
    fn lightweight_event_serialization_roundtrip() {
        let event = LightweightEvent {
            context: AssessmentContext::new("sess-1".into(), UserId("user-1".into())),
            message_idx: 5,
            signals: vec![LightweightSignal::Correction],
            frustration_ema: 0.3,
            success_ema: 0.7,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: LightweightEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message_idx, 5);
        assert_eq!(parsed.signals.len(), 1);
    }

    #[test]
    fn medium_event_serialization_roundtrip() {
        let event = MediumEvent {
            context: AssessmentContext::new("sess-1".into(), UserId("user-1".into())),
            checkpoint_id: CheckpointId::new(),
            message_range: (0, 10),
            trigger: CheckpointTrigger::MessageCount(10),
            summary: "User implemented auth".into(),
            token_metrics: TokenMetrics::default(),
            segment_score: 0.8,
            learnings_referenced: vec![],
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: MediumEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message_range, (0, 10));
    }

    #[test]
    fn heavy_event_serialization_roundtrip() {
        let event = HeavyEvent {
            context: AssessmentContext::new("sess-1".into(), UserId("user-1".into())),
            outcome: Outcome::Success,
            task_summary: "Implemented feature X".into(),
            frustration_trajectory: vec![0.1, 0.2, 0.15, 0.1],
            learning_attributions: std::collections::HashMap::new(),
            extraction_candidates: vec![],
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HeavyEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.outcome, Outcome::Success);
    }

    #[test]
    fn assessment_event_session_id() {
        let light = AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new("sess-light".into(), UserId("u".into())),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
        });
        assert_eq!(light.session_id().0, "sess-light");
    }

    #[test]
    fn attribution_score_clamped() {
        let score = AttributionScore::new(1.5);
        assert_eq!(score.0, 1.0);
        let score = AttributionScore::new(-1.5);
        assert_eq!(score.0, -1.0);
    }
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove assessment::types`
Expected: All tests pass

**Step 4: Update exports in mod.rs**

Update `vibes-groove/src/assessment/mod.rs`:

```rust
//! Assessment framework for measuring session outcomes.
//!
//! This module implements the three-tier assessment model:
//! - Lightweight: Per-message signal detection (<10ms)
//! - Medium: Checkpoint summarization (async)
//! - Heavy: Full session analysis (sampled)

pub mod types;

pub use types::{
    AssessmentContext, AssessmentEvent, AttributionScore, CheckpointId, CheckpointTrigger,
    EventId, ExtractionCandidate, HarnessType, HeavyEvent, InjectionMethod, LightweightEvent,
    LightweightSignal, MediumEvent, Outcome, ProjectId, SessionId, TokenMetrics, UserId,
};
```

**Step 5: Update lib.rs exports**

Update the assessment re-exports in `vibes-groove/src/lib.rs`:

```rust
pub use assessment::{
    AssessmentContext, AssessmentEvent, AttributionScore, CheckpointId, CheckpointTrigger,
    EventId, ExtractionCandidate, HarnessType, HeavyEvent, InjectionMethod, LightweightEvent,
    LightweightSignal, MediumEvent, Outcome, ProjectId, SessionId, TokenMetrics, UserId,
};
```

**Step 6: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 7: Commit**

```bash
git add vibes-groove/src/assessment/
git add vibes-groove/src/lib.rs
git commit -m "feat(groove): add three-tier assessment event types"
```

---

## Task 4: AssessmentLog Trait

**Files:**
- Create: `vibes-groove/src/assessment/log.rs`
- Modify: `vibes-groove/src/assessment/mod.rs`

**Step 1: Write the trait definition**

Create `vibes-groove/src/assessment/log.rs`:

```rust
//! Assessment event log abstraction.
//!
//! Provides a trait-based interface for assessment event storage,
//! with Iggy as the default implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

use super::{AssessmentEvent, EventId, SessionId};
use crate::error::Result;

/// Trait for assessment event log storage.
///
/// Implementations must support:
/// - Append-only event storage (immutable log)
/// - Session-scoped queries
/// - Time-range queries
/// - Real-time subscription
#[async_trait]
pub trait AssessmentLog: Send + Sync {
    /// Append an event to the immutable log.
    ///
    /// Returns the event ID assigned to the event.
    async fn append(&self, event: AssessmentEvent) -> Result<EventId>;

    /// Read all events for a specific session.
    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>>;

    /// Read events in a time range.
    async fn read_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AssessmentEvent>>;

    /// Subscribe to real-time events.
    ///
    /// Returns a broadcast receiver for new events.
    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent>;
}

/// In-memory implementation for testing.
#[derive(Debug)]
pub struct InMemoryAssessmentLog {
    events: std::sync::RwLock<Vec<AssessmentEvent>>,
    tx: broadcast::Sender<AssessmentEvent>,
}

impl InMemoryAssessmentLog {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            events: std::sync::RwLock::new(Vec::new()),
            tx,
        }
    }

    /// Get count of stored events (for testing)
    pub fn len(&self) -> usize {
        self.events.read().unwrap().len()
    }

    /// Check if log is empty (for testing)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryAssessmentLog {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AssessmentLog for InMemoryAssessmentLog {
    async fn append(&self, event: AssessmentEvent) -> Result<EventId> {
        let event_id = *event.event_id();
        self.events.write().unwrap().push(event.clone());
        // Ignore send errors (no subscribers)
        let _ = self.tx.send(event);
        Ok(event_id)
    }

    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>> {
        let events = self.events.read().unwrap();
        Ok(events
            .iter()
            .filter(|e| e.session_id() == session_id)
            .cloned()
            .collect())
    }

    async fn read_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AssessmentEvent>> {
        let events = self.events.read().unwrap();
        Ok(events
            .iter()
            .filter(|e| {
                let ts = match e {
                    AssessmentEvent::Lightweight(e) => e.context.timestamp,
                    AssessmentEvent::Medium(e) => e.context.timestamp,
                    AssessmentEvent::Heavy(e) => e.context.timestamp,
                };
                ts >= start && ts <= end
            })
            .cloned()
            .collect())
    }

    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{AssessmentContext, LightweightEvent, UserId};

    fn make_lightweight_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new(session.into(), UserId("test".into())),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
        })
    }

    #[tokio::test]
    async fn in_memory_log_append_and_read() {
        let log = InMemoryAssessmentLog::new();
        let event = make_lightweight_event("sess-1");

        let event_id = log.append(event).await.unwrap();
        assert!(!log.is_empty());

        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(*events[0].event_id(), event_id);
    }

    #[tokio::test]
    async fn in_memory_log_filters_by_session() {
        let log = InMemoryAssessmentLog::new();
        log.append(make_lightweight_event("sess-1")).await.unwrap();
        log.append(make_lightweight_event("sess-2")).await.unwrap();
        log.append(make_lightweight_event("sess-1")).await.unwrap();

        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_log_subscribe() {
        let log = InMemoryAssessmentLog::new();
        let mut rx = log.subscribe();

        let event = make_lightweight_event("sess-1");
        log.append(event.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.session_id().0, "sess-1");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-groove assessment::log`
Expected: All tests pass

**Step 3: Export from mod.rs**

Update `vibes-groove/src/assessment/mod.rs` to add:

```rust
pub mod log;

pub use log::{AssessmentLog, InMemoryAssessmentLog};
```

**Step 4: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add vibes-groove/src/assessment/log.rs
git add vibes-groove/src/assessment/mod.rs
git commit -m "feat(groove): add AssessmentLog trait with in-memory implementation"
```

---

## Task 5: Iggy Manager (Subprocess Lifecycle)

**Files:**
- Create: `vibes-groove/src/assessment/iggy/mod.rs`
- Create: `vibes-groove/src/assessment/iggy/manager.rs`
- Modify: `vibes-groove/src/assessment/mod.rs`

**Step 1: Create iggy module structure**

Create `vibes-groove/src/assessment/iggy/mod.rs`:

```rust
//! Iggy integration for assessment event log.
//!
//! Manages Iggy server subprocess and provides event log implementation.

pub mod manager;

pub use manager::IggyManager;
```

**Step 2: Write IggyManager tests first**

Create `vibes-groove/src/assessment/iggy/manager.rs`:

```rust
//! Iggy server subprocess management.
//!
//! Handles spawning, supervising, and stopping the Iggy server.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::error::{GrooveError, Result};

/// Configuration for Iggy server
#[derive(Debug, Clone)]
pub struct IggyConfig {
    /// Path to Iggy server binary
    pub binary_path: PathBuf,
    /// Data directory for Iggy storage
    pub data_dir: PathBuf,
    /// Port for Iggy to listen on
    pub port: u16,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum restart attempts before giving up
    pub max_restart_attempts: u32,
}

impl Default for IggyConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vibes/iggy");

        Self {
            binary_path: PathBuf::from("iggy-server"),
            data_dir,
            port: 8090,
            health_check_interval: Duration::from_secs(30),
            max_restart_attempts: 3,
        }
    }
}

/// State of the Iggy server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IggyState {
    /// Server is not running
    Stopped,
    /// Server is starting up
    Starting,
    /// Server is running and healthy
    Running,
    /// Server crashed and is being restarted
    Restarting,
    /// Server failed and won't be restarted
    Failed,
}

/// Manages the Iggy server subprocess lifecycle.
pub struct IggyManager {
    config: IggyConfig,
    process: RwLock<Option<Child>>,
    state: RwLock<IggyState>,
    shutdown: Arc<AtomicBool>,
    restart_count: RwLock<u32>,
}

impl IggyManager {
    /// Create a new Iggy manager with the given configuration.
    pub fn new(config: IggyConfig) -> Self {
        Self {
            config,
            process: RwLock::new(None),
            state: RwLock::new(IggyState::Stopped),
            shutdown: Arc::new(AtomicBool::new(false)),
            restart_count: RwLock::new(0),
        }
    }

    /// Get current state of the Iggy server.
    pub async fn state(&self) -> IggyState {
        *self.state.read().await
    }

    /// Start the Iggy server.
    pub async fn start(&self) -> Result<()> {
        let current_state = *self.state.read().await;
        if current_state == IggyState::Running {
            return Ok(());
        }

        *self.state.write().await = IggyState::Starting;

        // Ensure data directory exists
        tokio::fs::create_dir_all(&self.config.data_dir)
            .await
            .map_err(|e| GrooveError::Storage(format!("Failed to create Iggy data dir: {}", e)))?;

        // Spawn the Iggy server process
        let child = Command::new(&self.config.binary_path)
            .arg("--data-dir")
            .arg(&self.config.data_dir)
            .arg("--tcp-port")
            .arg(self.config.port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GrooveError::Storage(format!("Failed to spawn Iggy server: {}", e)))?;

        *self.process.write().await = Some(child);
        *self.state.write().await = IggyState::Running;
        *self.restart_count.write().await = 0;

        tracing::info!(
            "Iggy server started on port {}, data_dir: {:?}",
            self.config.port,
            self.config.data_dir
        );

        Ok(())
    }

    /// Stop the Iggy server gracefully.
    pub async fn stop(&self) -> Result<()> {
        self.shutdown.store(true, Ordering::SeqCst);

        let mut process = self.process.write().await;
        if let Some(mut child) = process.take() {
            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                tracing::warn!("Failed to kill Iggy server: {}", e);
            }
            // Wait for process to exit
            let _ = child.wait().await;
        }

        *self.state.write().await = IggyState::Stopped;
        tracing::info!("Iggy server stopped");
        Ok(())
    }

    /// Check if the server process is still running.
    pub async fn is_running(&self) -> bool {
        let mut process = self.process.write().await;
        if let Some(child) = process.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited
                    false
                }
                Ok(None) => {
                    // Process still running
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Supervise the Iggy server, restarting if it crashes.
    ///
    /// This method runs until shutdown is signaled.
    pub async fn supervise(&self) -> Result<()> {
        while !self.shutdown.load(Ordering::SeqCst) {
            sleep(self.config.health_check_interval).await;

            if self.shutdown.load(Ordering::SeqCst) {
                break;
            }

            if !self.is_running().await {
                let restart_count = *self.restart_count.read().await;
                if restart_count >= self.config.max_restart_attempts {
                    *self.state.write().await = IggyState::Failed;
                    tracing::error!(
                        "Iggy server failed after {} restart attempts",
                        restart_count
                    );
                    return Err(GrooveError::Storage(
                        "Iggy server failed too many times".into(),
                    ));
                }

                tracing::warn!(
                    "Iggy server crashed, restarting (attempt {}/{})",
                    restart_count + 1,
                    self.config.max_restart_attempts
                );
                *self.state.write().await = IggyState::Restarting;
                *self.restart_count.write().await = restart_count + 1;

                // Exponential backoff
                let backoff = Duration::from_secs(2_u64.pow(restart_count));
                sleep(backoff).await;

                if let Err(e) = self.start().await {
                    tracing::error!("Failed to restart Iggy server: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get the connection address for Iggy client.
    pub fn connection_address(&self) -> String {
        format!("127.0.0.1:{}", self.config.port)
    }
}

impl Drop for IggyManager {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iggy_config_default_has_sensible_values() {
        let config = IggyConfig::default();
        assert_eq!(config.port, 8090);
        assert_eq!(config.max_restart_attempts, 3);
        assert!(config.data_dir.to_string_lossy().contains("vibes"));
    }

    #[tokio::test]
    async fn iggy_manager_initial_state_is_stopped() {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);
        assert_eq!(manager.state().await, IggyState::Stopped);
    }

    #[test]
    fn iggy_manager_connection_address() {
        let mut config = IggyConfig::default();
        config.port = 9999;
        let manager = IggyManager::new(config);
        assert_eq!(manager.connection_address(), "127.0.0.1:9999");
    }

    // Note: Integration tests for start/stop require actual Iggy binary
    // Those will be tested separately with the bundled binary
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove assessment::iggy::manager`
Expected: All tests pass

**Step 4: Export from assessment mod.rs**

Update `vibes-groove/src/assessment/mod.rs`:

```rust
pub mod iggy;

pub use iggy::{IggyConfig, IggyManager, IggyState};
```

**Step 5: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add vibes-groove/src/assessment/iggy/
git add vibes-groove/src/assessment/mod.rs
git commit -m "feat(groove): add IggyManager for subprocess lifecycle management"
```

---

## Task 6: Iggy Assessment Log Implementation

**Files:**
- Create: `vibes-groove/src/assessment/iggy/log.rs`
- Modify: `vibes-groove/src/assessment/iggy/mod.rs`

**Step 1: Write IggyAssessmentLog implementation**

Create `vibes-groove/src/assessment/iggy/log.rs`:

```rust
//! Iggy-backed implementation of AssessmentLog.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::IggyManager;
use crate::assessment::{AssessmentEvent, AssessmentLog, EventId, SessionId};
use crate::error::{GrooveError, Result};

/// Stream and topic names for Iggy
pub mod topics {
    pub const STREAM_NAME: &str = "groove.assessment";
    pub const LIGHTWEIGHT_TOPIC: &str = "groove.assessment.lightweight";
    pub const MEDIUM_TOPIC: &str = "groove.assessment.medium";
    pub const HEAVY_TOPIC: &str = "groove.assessment.heavy";
}

/// Iggy-backed assessment log.
///
/// Writes events to Iggy topics based on assessment tier.
pub struct IggyAssessmentLog {
    #[allow(dead_code)]
    manager: Arc<IggyManager>,
    /// Broadcast channel for real-time event subscription
    tx: broadcast::Sender<AssessmentEvent>,
    /// In-memory buffer for events (until Iggy client is connected)
    buffer: RwLock<Vec<AssessmentEvent>>,
    /// Whether we're connected to Iggy
    connected: RwLock<bool>,
}

impl IggyAssessmentLog {
    /// Create a new Iggy assessment log.
    ///
    /// The manager must be started separately.
    pub fn new(manager: Arc<IggyManager>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            manager,
            tx,
            buffer: RwLock::new(Vec::new()),
            connected: RwLock::new(false),
        }
    }

    /// Connect to the Iggy server and create streams/topics.
    ///
    /// This should be called after the IggyManager has started.
    pub async fn connect(&self) -> Result<()> {
        // TODO: Implement actual Iggy client connection
        // For now, we'll just mark as connected and use the buffer
        //
        // Full implementation will:
        // 1. Create IggyClient and connect
        // 2. Login with credentials
        // 3. Create stream if not exists
        // 4. Create topics if not exist
        // 5. Start background consumer for subscription

        *self.connected.write().await = true;
        tracing::info!("IggyAssessmentLog connected (stub implementation)");
        Ok(())
    }

    /// Get the appropriate topic for an event
    fn topic_for_event(event: &AssessmentEvent) -> &'static str {
        match event {
            AssessmentEvent::Lightweight(_) => topics::LIGHTWEIGHT_TOPIC,
            AssessmentEvent::Medium(_) => topics::MEDIUM_TOPIC,
            AssessmentEvent::Heavy(_) => topics::HEAVY_TOPIC,
        }
    }

    /// Flush buffered events to Iggy (call after connect)
    pub async fn flush_buffer(&self) -> Result<()> {
        let events = {
            let mut buffer = self.buffer.write().await;
            std::mem::take(&mut *buffer)
        };

        for event in events {
            // TODO: Actually write to Iggy
            let topic = Self::topic_for_event(&event);
            tracing::debug!("Flushing event to topic {}: {:?}", topic, event.event_id());
        }

        Ok(())
    }
}

#[async_trait]
impl AssessmentLog for IggyAssessmentLog {
    async fn append(&self, event: AssessmentEvent) -> Result<EventId> {
        let event_id = *event.event_id();
        let connected = *self.connected.read().await;

        if connected {
            // TODO: Write to Iggy
            // For now, just buffer
            let topic = Self::topic_for_event(&event);
            tracing::debug!("Would write to topic {}: {:?}", topic, event_id);
        }

        // Always buffer and broadcast
        self.buffer.write().await.push(event.clone());
        let _ = self.tx.send(event);

        Ok(event_id)
    }

    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>> {
        // TODO: Read from Iggy
        // For now, read from buffer
        let buffer = self.buffer.read().await;
        Ok(buffer
            .iter()
            .filter(|e| e.session_id() == session_id)
            .cloned()
            .collect())
    }

    async fn read_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AssessmentEvent>> {
        // TODO: Read from Iggy with time range filter
        // For now, read from buffer
        let buffer = self.buffer.read().await;
        Ok(buffer
            .iter()
            .filter(|e| {
                let ts = match e {
                    AssessmentEvent::Lightweight(e) => e.context.timestamp,
                    AssessmentEvent::Medium(e) => e.context.timestamp,
                    AssessmentEvent::Heavy(e) => e.context.timestamp,
                };
                ts >= start && ts <= end
            })
            .cloned()
            .collect())
    }

    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{AssessmentContext, IggyConfig, LightweightEvent, UserId};

    fn make_test_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new(session.into(), UserId("test".into())),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
        })
    }

    #[tokio::test]
    async fn iggy_log_append_and_read() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        let event = make_test_event("sess-1");
        let event_id = log.append(event).await.unwrap();

        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(*events[0].event_id(), event_id);
    }

    #[tokio::test]
    async fn iggy_log_subscribe() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);
        let mut rx = log.subscribe();

        let event = make_test_event("sess-1");
        log.append(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.session_id().0, "sess-1");
    }

    #[test]
    fn topic_for_event_returns_correct_topic() {
        let light = make_test_event("s");
        assert_eq!(
            IggyAssessmentLog::topic_for_event(&light),
            topics::LIGHTWEIGHT_TOPIC
        );
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-groove assessment::iggy::log`
Expected: All tests pass

**Step 3: Update iggy mod.rs**

Update `vibes-groove/src/assessment/iggy/mod.rs`:

```rust
//! Iggy integration for assessment event log.
//!
//! Manages Iggy server subprocess and provides event log implementation.

pub mod log;
pub mod manager;

pub use log::IggyAssessmentLog;
pub use manager::{IggyConfig, IggyManager, IggyState};
```

**Step 4: Update assessment mod.rs exports**

Add to `vibes-groove/src/assessment/mod.rs` exports:

```rust
pub use iggy::{IggyAssessmentLog, IggyConfig, IggyManager, IggyState};
```

**Step 5: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add vibes-groove/src/assessment/iggy/
git add vibes-groove/src/assessment/mod.rs
git commit -m "feat(groove): add IggyAssessmentLog implementation (stub)"
```

---

## Task 7: Assessment Configuration Schema

**Files:**
- Create: `vibes-groove/src/assessment/config.rs`
- Modify: `vibes-groove/src/assessment/mod.rs`

**Step 1: Define assessment config types**

Create `vibes-groove/src/assessment/config.rs`:

```rust
//! Configuration for the assessment framework.
//!
//! Lives under `[plugins.groove.assessment]` in config.toml.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level assessment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AssessmentConfig {
    /// Whether assessment is enabled
    pub enabled: bool,
    /// Whether circuit breaker interventions are enabled
    pub intervention_enabled: bool,
    /// Sampling configuration
    pub sampling: SamplingConfig,
    /// Session end detection configuration
    pub session_end: SessionEndConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// LLM backend configuration
    pub llm: LlmConfig,
    /// Custom linguistic patterns
    pub patterns: PatternConfig,
    /// Retention policies
    pub retention: RetentionConfig,
    /// Iggy server configuration
    pub iggy: IggyServerConfig,
}

impl Default for AssessmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intervention_enabled: true,
            sampling: SamplingConfig::default(),
            session_end: SessionEndConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            llm: LlmConfig::default(),
            patterns: PatternConfig::default(),
            retention: RetentionConfig::default(),
            iggy: IggyServerConfig::default(),
        }
    }
}

/// Sampling strategy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SamplingConfig {
    /// Base rate for heavy assessment (0.0 - 1.0)
    pub base_rate: f64,
    /// Number of burn-in sessions (100% sampling)
    pub burnin_sessions: u32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            base_rate: 0.2,
            burnin_sessions: 10,
        }
    }
}

/// Session end detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionEndConfig {
    /// Use Claude Stop hook for session end
    pub hook_enabled: bool,
    /// Use inactivity timeout for session end
    pub timeout_enabled: bool,
    /// Inactivity timeout in minutes
    pub timeout_minutes: u32,
}

impl Default for SessionEndConfig {
    fn default() -> Self {
        Self {
            hook_enabled: true,
            timeout_enabled: true,
            timeout_minutes: 15,
        }
    }
}

/// Circuit breaker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled
    pub enabled: bool,
    /// Cooldown between interventions in seconds
    pub cooldown_seconds: u32,
    /// Maximum interventions per session
    pub max_interventions_per_session: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_seconds: 120,
            max_interventions_per_session: 3,
        }
    }
}

/// LLM backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Backend to use: "harness", "anthropic", "openai", "ollama"
    pub backend: String,
    /// Model to use (ignored for harness backend)
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend: "harness".into(),
            model: "claude-3-haiku".into(),
        }
    }
}

/// Custom linguistic pattern configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternConfig {
    /// Additional negative patterns (merged with defaults)
    pub negative: Vec<String>,
    /// Additional positive patterns (merged with defaults)
    pub positive: Vec<String>,
}

/// Retention policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Days to retain lightweight events (-1 = forever)
    pub lightweight_days: i32,
    /// Days to retain medium events (-1 = forever)
    pub medium_days: i32,
    /// Days to retain heavy events (-1 = forever)
    pub heavy_days: i32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            lightweight_days: 7,
            medium_days: 30,
            heavy_days: -1,
        }
    }
}

/// Iggy server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IggyServerConfig {
    /// Data directory for Iggy storage
    pub data_dir: Option<PathBuf>,
    /// Port for Iggy to listen on
    pub port: u16,
}

impl Default for IggyServerConfig {
    fn default() -> Self {
        Self {
            data_dir: None, // Will use default based on scope
            port: 8090,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_config_default_values() {
        let config = AssessmentConfig::default();
        assert!(config.enabled);
        assert!(config.intervention_enabled);
        assert_eq!(config.sampling.base_rate, 0.2);
        assert_eq!(config.sampling.burnin_sessions, 10);
    }

    #[test]
    fn assessment_config_serialization_roundtrip() {
        let config = AssessmentConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: AssessmentConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.enabled, config.enabled);
        assert_eq!(parsed.sampling.base_rate, config.sampling.base_rate);
    }

    #[test]
    fn assessment_config_partial_deserialize() {
        // Should work with partial config, using defaults for missing fields
        let toml = r#"
            enabled = false
            [sampling]
            base_rate = 0.5
        "#;
        let parsed: AssessmentConfig = toml::from_str(toml).unwrap();
        assert!(!parsed.enabled);
        assert_eq!(parsed.sampling.base_rate, 0.5);
        assert_eq!(parsed.sampling.burnin_sessions, 10); // default
        assert!(parsed.intervention_enabled); // default
    }

    #[test]
    fn pattern_config_merge_semantics() {
        let toml = r#"
            negative = ["custom frustration"]
            positive = ["custom success"]
        "#;
        let parsed: PatternConfig = toml::from_str(toml).unwrap();
        assert_eq!(parsed.negative.len(), 1);
        assert_eq!(parsed.positive.len(), 1);
    }

    #[test]
    fn retention_config_forever_value() {
        let config = RetentionConfig::default();
        assert_eq!(config.heavy_days, -1); // -1 means forever
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-groove assessment::config`
Expected: All tests pass

**Step 3: Export from mod.rs**

Update `vibes-groove/src/assessment/mod.rs`:

```rust
pub mod config;

pub use config::{
    AssessmentConfig, CircuitBreakerConfig, IggyServerConfig, LlmConfig, PatternConfig,
    RetentionConfig, SamplingConfig, SessionEndConfig,
};
```

**Step 4: Verify compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add vibes-groove/src/assessment/config.rs
git add vibes-groove/src/assessment/mod.rs
git commit -m "feat(groove): add assessment configuration schema"
```

---

## Task 8: AssessmentProcessor Skeleton

**Files:**
- Create: `vibes-groove/src/assessment/processor.rs`
- Modify: `vibes-groove/src/assessment/mod.rs`

**Step 1: Write the processor skeleton**

Create `vibes-groove/src/assessment/processor.rs`:

```rust
//! Assessment processor - subscribes to EventBus and writes to AssessmentLog.
//!
//! This is the main entry point for assessment. It:
//! 1. Subscribes to VibesEvent from EventBus
//! 2. Runs lightweight detection on each message
//! 3. Fires events to AssessmentLog via fire-and-forget channel

use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::broadcast;

use crate::assessment::{AssessmentConfig, AssessmentEvent, AssessmentLog};
use crate::error::Result;

/// Message sent to the background writer task
enum WriterMessage {
    Event(AssessmentEvent),
    Shutdown,
}

/// Assessment processor that subscribes to events and writes assessments.
pub struct AssessmentProcessor {
    config: AssessmentConfig,
    log: Arc<dyn AssessmentLog>,
    writer_tx: mpsc::UnboundedSender<WriterMessage>,
}

impl AssessmentProcessor {
    /// Create a new assessment processor.
    ///
    /// Spawns a background task for writing events to the log.
    pub fn new(config: AssessmentConfig, log: Arc<dyn AssessmentLog>) -> Self {
        let (writer_tx, writer_rx) = mpsc::unbounded_channel();

        // Spawn background writer task
        let log_clone = Arc::clone(&log);
        tokio::spawn(async move {
            Self::writer_task(log_clone, writer_rx).await;
        });

        Self {
            config,
            log,
            writer_tx,
        }
    }

    /// Background task that writes events to the log.
    ///
    /// Uses fire-and-forget semantics - errors are logged but not propagated.
    async fn writer_task(
        log: Arc<dyn AssessmentLog>,
        mut rx: mpsc::UnboundedReceiver<WriterMessage>,
    ) {
        while let Some(msg) = rx.recv().await {
            match msg {
                WriterMessage::Event(event) => {
                    if let Err(e) = log.append(event).await {
                        tracing::warn!("Failed to write assessment event: {}", e);
                        // Fire-and-forget: don't retry, accept data loss
                    }
                }
                WriterMessage::Shutdown => {
                    tracing::debug!("Assessment writer task shutting down");
                    break;
                }
            }
        }
    }

    /// Check if assessment is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Submit an assessment event for writing.
    ///
    /// This is non-blocking - the event is sent to the background writer.
    pub fn submit(&self, event: AssessmentEvent) {
        if !self.config.enabled {
            return;
        }

        // Fire-and-forget: ignore send errors (channel closed)
        let _ = self.writer_tx.send(WriterMessage::Event(event));
    }

    /// Subscribe to assessment events from the log.
    pub fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent> {
        self.log.subscribe()
    }

    /// Shutdown the processor gracefully.
    pub fn shutdown(&self) {
        let _ = self.writer_tx.send(WriterMessage::Shutdown);
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &AssessmentConfig {
        &self.config
    }
}

impl Drop for AssessmentProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{
        AssessmentContext, InMemoryAssessmentLog, LightweightEvent, UserId,
    };

    fn make_test_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new(session.into(), UserId("test".into())),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
        })
    }

    #[tokio::test]
    async fn processor_submits_events() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, Arc::clone(&log) as Arc<dyn AssessmentLog>);

        processor.submit(make_test_event("sess-1"));

        // Give writer task time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert_eq!(log.len(), 1);
    }

    #[tokio::test]
    async fn processor_respects_disabled_config() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let mut config = AssessmentConfig::default();
        config.enabled = false;
        let processor = AssessmentProcessor::new(config, Arc::clone(&log) as Arc<dyn AssessmentLog>);

        processor.submit(make_test_event("sess-1"));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        assert!(log.is_empty());
    }

    #[tokio::test]
    async fn processor_subscribe_receives_events() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, Arc::clone(&log) as Arc<dyn AssessmentLog>);

        let mut rx = processor.subscribe();
        processor.submit(make_test_event("sess-1"));

        let received = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(received.session_id().0, "sess-1");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-groove assessment::processor`
Expected: All tests pass

**Step 3: Export from mod.rs**

Update `vibes-groove/src/assessment/mod.rs`:

```rust
pub mod processor;

pub use processor::AssessmentProcessor;
```

**Step 4: Update lib.rs with all exports**

Update `vibes-groove/src/lib.rs` to have comprehensive assessment exports:

```rust
// Assessment re-exports
pub use assessment::{
    // Types
    AssessmentContext, AssessmentEvent, AttributionScore, CheckpointId, CheckpointTrigger,
    EventId, ExtractionCandidate, HarnessType, HeavyEvent, InjectionMethod, LightweightEvent,
    LightweightSignal, MediumEvent, Outcome, ProjectId, SessionId, TokenMetrics, UserId,
    // Log
    AssessmentLog, InMemoryAssessmentLog,
    // Iggy
    IggyAssessmentLog, IggyConfig, IggyManager, IggyState,
    // Config
    AssessmentConfig, CircuitBreakerConfig, IggyServerConfig, LlmConfig, PatternConfig,
    RetentionConfig, SamplingConfig, SessionEndConfig,
    // Processor
    AssessmentProcessor,
};
```

**Step 5: Verify full compilation**

Run: `cargo check -p vibes-groove`
Expected: Compiles without errors

**Step 6: Run all assessment tests**

Run: `cargo test -p vibes-groove assessment`
Expected: All tests pass

**Step 7: Commit**

```bash
git add vibes-groove/src/assessment/processor.rs
git add vibes-groove/src/assessment/mod.rs
git add vibes-groove/src/lib.rs
git commit -m "feat(groove): add AssessmentProcessor with fire-and-forget writer"
```

---

## Task 9: Integration Test

**Files:**
- Create: `vibes-groove/tests/assessment_integration.rs`

**Step 1: Write integration test**

Create `vibes-groove/tests/assessment_integration.rs`:

```rust
//! Integration tests for the assessment framework.

use std::sync::Arc;

use vibes_groove::{
    AssessmentConfig, AssessmentContext, AssessmentEvent, AssessmentLog, AssessmentProcessor,
    InMemoryAssessmentLog, LightweightEvent, LightweightSignal, UserId,
};

fn make_lightweight_event(session: &str, msg_idx: u32) -> AssessmentEvent {
    AssessmentEvent::Lightweight(LightweightEvent {
        context: AssessmentContext::new(session.into(), UserId("test-user".into())),
        message_idx: msg_idx,
        signals: vec![LightweightSignal::Correction],
        frustration_ema: 0.3,
        success_ema: 0.7,
    })
}

#[tokio::test]
async fn assessment_pipeline_end_to_end() {
    // Setup
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig::default();
    let processor = AssessmentProcessor::new(config, Arc::clone(&log) as Arc<dyn AssessmentLog>);

    // Subscribe before submitting
    let mut rx = processor.subscribe();

    // Submit events for a session
    for i in 0..5 {
        processor.submit(make_lightweight_event("sess-integration", i));
    }

    // Wait for events to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify events were logged
    let events = log.read_session(&"sess-integration".into()).await.unwrap();
    assert_eq!(events.len(), 5);

    // Verify subscription received events
    let mut received_count = 0;
    while let Ok(Ok(_)) = tokio::time::timeout(
        tokio::time::Duration::from_millis(10),
        rx.recv(),
    ).await {
        received_count += 1;
    }
    assert_eq!(received_count, 5);

    // Cleanup
    processor.shutdown();
}

#[tokio::test]
async fn assessment_config_controls_behavior() {
    // Setup with disabled assessment
    let log = Arc::new(InMemoryAssessmentLog::new());
    let mut config = AssessmentConfig::default();
    config.enabled = false;
    let processor = AssessmentProcessor::new(config, Arc::clone(&log) as Arc<dyn AssessmentLog>);

    // Submit events
    processor.submit(make_lightweight_event("sess-disabled", 0));
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Should not be logged
    assert!(log.is_empty());
}

#[tokio::test]
async fn assessment_events_have_correct_lineage() {
    let event = make_lightweight_event("sess-lineage", 0);

    // Verify UUIDv7 event ID
    if let AssessmentEvent::Lightweight(e) = &event {
        let uuid = e.context.event_id.0;
        assert_eq!(uuid.get_version_num(), 7);
    }

    // Verify session ID propagation
    assert_eq!(event.session_id().0, "sess-lineage");
}
```

**Step 2: Run integration tests**

Run: `cargo test -p vibes-groove --test assessment_integration`
Expected: All tests pass

**Step 3: Commit**

```bash
git add vibes-groove/tests/assessment_integration.rs
git commit -m "test(groove): add assessment framework integration tests"
```

---

## Task 10: Update Progress and Create PR

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Update progress**

Add milestone 4.4.1 entry to `docs/PROGRESS.md`:

```markdown
### Milestone 4.4: Assessment Framework

- [x] **4.4.1: Infrastructure** - Iggy integration, event log, processor skeleton
- [ ] **4.4.2: Assessment Logic** - Signal detection, circuit breaker, CLI
```

**Step 2: Run pre-commit checks**

Run: `just pre-commit`
Expected: All checks pass

**Step 3: Commit progress update**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 4.4.1 complete"
```

**Step 4: Push and create PR**

```bash
git push -u origin assessment-framework
gh pr create --title "feat(groove): add assessment framework infrastructure (4.4.1)" --body "$(cat <<'EOF'
## Summary
- Add assessment event types with full attribution context
- Add AssessmentLog trait with in-memory and Iggy implementations
- Add IggyManager for subprocess lifecycle management
- Add assessment configuration schema
- Add AssessmentProcessor with fire-and-forget writer

## Test Plan
- [x] All unit tests passing (`cargo test -p vibes-groove assessment`)
- [x] Integration tests passing
- [x] Pre-commit checks pass (`just pre-commit`)
- [ ] Manual: Verify IggyManager can spawn server (requires Iggy binary)

Part of milestone 4.4: Assessment Framework
See design: docs/plans/14-continual-learning/milestone-4.4-design.md
EOF
)"
```

---

## Summary

**Milestone 4.4.1 delivers:**

1. **Assessment Event Types** (`types.rs`)
   - `AssessmentContext` with full attribution lineage
   - `LightweightEvent`, `MediumEvent`, `HeavyEvent`
   - All serializable via serde

2. **AssessmentLog Trait** (`log.rs`)
   - Append-only interface for event storage
   - `InMemoryAssessmentLog` for testing

3. **Iggy Integration** (`iggy/`)
   - `IggyManager` for subprocess lifecycle
   - `IggyAssessmentLog` implementation (stub, full in 4.4.2)

4. **Configuration** (`config.rs`)
   - `AssessmentConfig` with all sub-configs
   - Sensible defaults, partial deserialization

5. **AssessmentProcessor** (`processor.rs`)
   - Fire-and-forget event submission
   - Background writer task
   - Real-time subscription

**Exit Criteria:**
- [x] Types compile and serialize correctly
- [x] InMemoryAssessmentLog passes all tests
- [x] IggyManager has unit tests for config
- [x] AssessmentProcessor submits and subscribes
- [x] Integration test validates pipeline

**Next:** Milestone 4.4.2 implements the actual assessment logic (signal detection, circuit breaker, LLM integration, CLI commands).

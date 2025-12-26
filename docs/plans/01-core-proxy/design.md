# Milestone 1.1: Core Proxy - Design Document

> vibes-core crate with Session, EventBus, Claude Code subprocess management, stream-json parsing, and basic error handling.

## Overview

This milestone establishes the foundation of vibes: a Rust library that wraps Claude Code with an adapter-based architecture enabling backend swapping, event-driven communication, and late-joiner replay.

### Key Decisions

| Decision | Choice | ADR |
|----------|--------|-----|
| Async runtime | Tokio | - |
| Error handling | thiserror (library), anyhow (binaries) | - |
| Event bus | Adapter pattern, MemoryEventBus for MVP | ADR-007 |
| Claude interaction | Adapter pattern, PrintModeBackend for MVP | ADR-008 |
| Stream-JSON parsing | Strongly-typed enums with serde | - |
| Testing | Unit tests with MockBackend + gated integration tests | - |

---

## Crate Structure

```
vibes/
├── Cargo.toml                    # Workspace root
├── flake.nix                     # Nix dev environment
├── .envrc                        # Direnv auto-load
├── justfile                      # Task runner
├── vibes-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Public API exports
│       ├── session/
│       │   ├── mod.rs
│       │   ├── session.rs        # Session struct & state machine
│       │   └── manager.rs        # SessionManager (owns multiple sessions)
│       ├── backend/
│       │   ├── mod.rs
│       │   ├── traits.rs         # ClaudeBackend trait
│       │   ├── print_mode.rs     # PrintModeBackend implementation
│       │   └── mock.rs           # MockBackend for testing
│       ├── events/
│       │   ├── mod.rs
│       │   ├── types.rs          # ClaudeEvent, VibesEvent enums
│       │   ├── bus.rs            # EventBus trait
│       │   └── memory.rs         # MemoryEventBus implementation
│       ├── parser/
│       │   ├── mod.rs
│       │   └── stream_json.rs    # Stream-json parsing
│       ├── error.rs              # Error types (thiserror)
│       └── config.rs             # Configuration (future)
└── vibes-cli/                    # (Milestone 1.2)
```

---

## Development Environment

### Nix Flake

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.just
            pkgs.cargo-nextest
            pkgs.cargo-mutants
            pkgs.cargo-watch
            pkgs.direnv
          ];

          shellHook = ''
            echo "vibes dev shell loaded"
            echo "  just          - list commands"
            echo "  just test     - run tests"
            echo "  just dev      - watch mode"
          '';
        };
      });
}
```

### Direnv

```bash
# .envrc
use flake
```

### Task Runner (just)

```just
# justfile

# Default: show available commands
default:
    @just --list

# Development
dev:
    cargo watch -x check

# Testing
test:
    cargo nextest run

test-all:
    cargo nextest run --features integration

test-watch:
    cargo watch -x 'nextest run'

test-one NAME:
    cargo nextest run {{NAME}}

# Code quality
check:
    cargo check --all-targets

clippy:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

# Mutation testing
mutants:
    cargo mutants

# Build
build:
    cargo build

build-release:
    cargo build --release

# Run all checks (pre-commit)
pre-commit: fmt-check clippy test
```

---

## Core Types

### Event Types

```rust
// events/types.rs

/// Events emitted by Claude backends (normalized across backends)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaudeEvent {
    // Content streaming
    TextDelta { text: String },
    ThinkingDelta { text: String },

    // Tool use lifecycle
    ToolUseStart { id: String, name: String },
    ToolInputDelta { id: String, delta: String },
    ToolResult { id: String, output: String, is_error: bool },

    // Session lifecycle
    TurnStart,
    TurnComplete { usage: Usage },

    // Errors
    Error { message: String, recoverable: bool },
}

/// Events on the VibesEventBus (includes client events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VibesEvent {
    // From Claude (wrapped)
    Claude { session_id: String, event: ClaudeEvent },

    // From clients
    UserInput { session_id: String, content: String },
    PermissionResponse { session_id: String, request_id: String, approved: bool },

    // System events
    SessionCreated { session_id: String, name: Option<String> },
    SessionStateChanged { session_id: String, state: SessionState },
    ClientConnected { client_id: String },
    ClientDisconnected { client_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### Error Types

```rust
// error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VibesError {
    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition from {from:?}")]
    InvalidStateTransition { from: SessionState },

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
}

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Failed to spawn Claude process: {0}")]
    SpawnFailed(#[source] std::io::Error),

    #[error("Claude process exited unexpectedly: code {code:?}")]
    ProcessCrashed { code: Option<i32> },

    #[error("Claude binary not found. Is Claude Code installed?")]
    ClaudeNotFound,

    #[error("Parse error: {message}")]
    ParseError { message: String, recoverable: bool },

    #[error("Claude error: {0}")]
    ClaudeError(String),
}

#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Failed to publish event")]
    PublishFailed,
}
```

---

## Backend Abstraction (ADR-008)

### Trait Definition

```rust
// backend/traits.rs
use async_trait::async_trait;
use tokio::sync::broadcast;

#[async_trait]
pub trait ClaudeBackend: Send + Sync {
    /// Send user input to Claude
    async fn send(&mut self, input: &str) -> Result<(), BackendError>;

    /// Subscribe to events from this backend
    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent>;

    /// Respond to a permission request (interactive backends)
    async fn respond_permission(
        &mut self,
        request_id: &str,
        approved: bool
    ) -> Result<(), BackendError>;

    /// Claude's session ID for continuity
    fn claude_session_id(&self) -> &str;

    /// Current state
    fn state(&self) -> BackendState;

    /// Graceful shutdown
    async fn shutdown(&mut self) -> Result<(), BackendError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendState {
    Idle,
    Processing,
    WaitingPermission { request_id: String, tool: String },
    Finished,
    Failed { message: String, recoverable: bool },
}
```

### PrintModeBackend

```rust
// backend/print_mode.rs

pub struct PrintModeBackend {
    claude_session_id: String,
    state: BackendState,
    event_tx: broadcast::Sender<ClaudeEvent>,
    allowed_tools: Vec<String>,
    working_dir: PathBuf,
}

impl PrintModeBackend {
    pub fn new(claude_session_id: Option<String>) -> Self {
        let id = claude_session_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        // ...
    }

    /// Spawn claude -p and stream output
    async fn spawn_turn(&mut self, input: &str) -> Result<(), BackendError> {
        let mut cmd = Command::new("claude");
        cmd.args([
            "-p", input,
            "--output-format", "stream-json",
            "--session-id", &self.claude_session_id,
        ]);

        if !self.allowed_tools.is_empty() {
            cmd.args(["--allowedTools", &self.allowed_tools.join(",")]);
        }

        // Spawn, capture stdout, parse stream-json, emit events
        // ...
    }
}
```

### MockBackend (Testing)

```rust
// backend/mock.rs

pub struct MockBackend {
    claude_session_id: String,
    state: BackendState,
    tx: broadcast::Sender<ClaudeEvent>,
    script: VecDeque<Vec<ClaudeEvent>>,
}

impl MockBackend {
    pub fn new() -> Self { /* ... */ }

    /// Queue events to emit on next send()
    pub fn queue_response(&mut self, events: Vec<ClaudeEvent>) {
        self.script.push_back(events);
    }

    /// Queue an error response
    pub fn queue_error(&mut self, message: &str, recoverable: bool) {
        self.queue_response(vec![
            ClaudeEvent::Error {
                message: message.to_string(),
                recoverable
            }
        ]);
    }
}
```

---

## EventBus Abstraction (ADR-007)

### Trait Definition

```rust
// events/bus.rs
use async_trait::async_trait;
use tokio_stream::Stream;

pub type EventSeq = u64;

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event, returns its sequence number
    async fn publish(&self, event: VibesEvent) -> EventSeq;

    /// Subscribe to all events from now
    fn subscribe(&self) -> broadcast::Receiver<(EventSeq, VibesEvent)>;

    /// Subscribe starting from a sequence number (for replay)
    async fn subscribe_from(&self, seq: EventSeq) -> impl Stream<Item = (EventSeq, VibesEvent)>;

    /// Get all events for a session (for late joiners)
    async fn get_session_events(&self, session_id: &str) -> Vec<(EventSeq, VibesEvent)>;

    /// Current sequence number (high water mark)
    fn current_seq(&self) -> EventSeq;
}
```

### MemoryEventBus

```rust
// events/memory.rs

pub struct MemoryEventBus {
    events: RwLock<Vec<(EventSeq, VibesEvent)>>,
    next_seq: AtomicU64,
    tx: broadcast::Sender<(EventSeq, VibesEvent)>,
}

impl MemoryEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            events: RwLock::new(Vec::new()),
            next_seq: AtomicU64::new(0),
            tx,
        }
    }
}

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn publish(&self, event: VibesEvent) -> EventSeq {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);

        // Store for replay
        self.events.write().await.push((seq, event.clone()));

        // Broadcast to live subscribers
        let _ = self.tx.send((seq, event));

        seq
    }

    async fn subscribe_from(&self, seq: EventSeq) -> impl Stream<Item = (EventSeq, VibesEvent)> {
        let historical: Vec<_> = self.events.read().await
            .iter()
            .filter(|(s, _)| *s >= seq)
            .cloned()
            .collect();

        let live = BroadcastStream::new(self.tx.subscribe());

        stream::iter(historical).chain(live.filter_map(|r| r.ok()))
    }
}
```

---

## Session Management

### Session

```rust
// session/session.rs

pub struct Session {
    id: String,
    name: Option<String>,
    backend: Box<dyn ClaudeBackend>,
    event_bus: Arc<dyn EventBus>,
    state: SessionState,
    event_forwarder: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Processing,
    WaitingForInput,
    WaitingForPermission { request_id: String, tool: String },
    Completed,
    Failed { message: String },
}

impl Session {
    pub fn new(
        name: Option<String>,
        backend: Box<dyn ClaudeBackend>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self { /* ... */ }

    pub async fn send(&mut self, input: &str) -> Result<(), SessionError> {
        self.state = SessionState::Processing;
        self.publish_state_change().await;
        self.backend.send(input).await?;
        Ok(())
    }

    pub async fn retry(&mut self) -> Result<(), SessionError> {
        match &self.state {
            SessionState::Failed { .. } => {
                self.state = SessionState::Idle;
                self.publish_state_change().await;
                Ok(())
            }
            _ => Err(SessionError::InvalidStateTransition),
        }
    }
}
```

### SessionManager

```rust
// session/manager.rs

pub struct SessionManager {
    sessions: RwLock<HashMap<String, Session>>,
    event_bus: Arc<dyn EventBus>,
    backend_factory: Box<dyn BackendFactory>,
}

#[async_trait]
pub trait BackendFactory: Send + Sync {
    fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend>;
}

impl SessionManager {
    pub async fn create_session(&self, name: Option<String>) -> Result<String, SessionError>;
    pub async fn get_session(&self, id: &str) -> Option<&Session>;
    pub async fn list_sessions(&self) -> Vec<SessionInfo>;
}
```

---

## Stream-JSON Parser

```rust
// parser/stream_json.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    System { message: String },
    AssistantMessage { id: String, message: AssistantContent },
    ContentBlockStart { index: u32, content_block: ContentBlock },
    ContentBlockDelta { index: u32, delta: Delta },
    ContentBlockStop { index: u32 },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
    Result { is_error: bool, duration_ms: u64, usage: Usage },

    #[serde(other)]
    Unknown,
}

/// Parse a line of stream-json, returning None for unparseable lines
pub fn parse_line(line: &str) -> Option<StreamMessage> {
    if line.trim().is_empty() {
        return None;
    }

    match serde_json::from_str(line) {
        Ok(msg) => Some(msg),
        Err(e) => {
            tracing::warn!("Failed to parse stream-json line: {}", e);
            None  // Resilient: skip bad lines
        }
    }
}

/// Convert StreamMessage to ClaudeEvent
pub fn to_claude_event(msg: StreamMessage) -> Option<ClaudeEvent> {
    match msg {
        StreamMessage::ContentBlockDelta { delta: Delta::TextDelta { text }, .. } => {
            Some(ClaudeEvent::TextDelta { text })
        }
        StreamMessage::ToolUse { id, name, .. } => {
            Some(ClaudeEvent::ToolUseStart { id, name })
        }
        StreamMessage::Result { usage, .. } => {
            Some(ClaudeEvent::TurnComplete { usage })
        }
        StreamMessage::Unknown => None,
        // ... other conversions
    }
}
```

---

## Testing Strategy

### Test Organization

```
vibes-core/
└── tests/
    ├── unit/
    │   ├── mod.rs
    │   ├── session_test.rs
    │   ├── event_bus_test.rs
    │   └── parser_test.rs
    └── integration/
        ├── mod.rs
        └── print_mode_test.rs
```

### Feature Flags

```toml
# Cargo.toml
[features]
default = []
integration = []  # Enables tests that spawn real Claude
```

### Example Unit Test

```rust
#[tokio::test]
async fn test_session_state_transitions() {
    let mut backend = MockBackend::new();
    backend.queue_response(vec![
        ClaudeEvent::TextDelta { text: "Hello".into() },
        ClaudeEvent::TurnComplete { usage: Usage::default() },
    ]);

    let event_bus = Arc::new(MemoryEventBus::new(100));
    let mut session = Session::new(None, Box::new(backend), event_bus.clone());

    assert_eq!(session.state(), SessionState::Idle);

    session.send("Hi").await.unwrap();

    let events = event_bus.get_session_events(&session.id()).await;
    assert!(events.iter().any(|(_, e)| matches!(e,
        VibesEvent::SessionStateChanged { state: SessionState::Processing, .. }
    )));
}
```

---

## Public API Surface

```rust
// lib.rs
pub mod session;
pub mod backend;
pub mod events;
pub mod parser;
pub mod error;

pub use session::{Session, SessionManager, SessionState};
pub use backend::{ClaudeBackend, BackendState, BackendFactory};
pub use events::{EventBus, VibesEvent, ClaudeEvent};
pub use error::{VibesError, SessionError, BackendError};
```

---

## Dependencies

```toml
# vibes-core/Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
async-trait = "0.1"
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
tokio-test = "0.4"
```

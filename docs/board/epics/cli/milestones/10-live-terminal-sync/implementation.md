# CLI ↔ Web Mirroring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable real-time bidirectional sync between CLI and Web UI with input attribution and late-joiner catch-up.

**Architecture:** Add `InputSource` enum to track input origin (`cli`, `web_ui`). Extend `Subscribe` with `catch_up` flag for paginated history replay. CLI displays remote input with `[Web UI]:` prefix; Web UI shows source badges. Add CLI input history with up/down navigation.

**Tech Stack:** Rust (vibes-core, vibes-server, vibes-cli), TypeScript/React (web-ui), WebSocket protocol

---

## Phase 1: Input Source Type

### Task 1.1: Create InputSource enum

**Files:**
- Modify: `vibes-core/src/events/types.rs`

**Step 1: Write the failing test**

Add to `vibes-core/src/events/types.rs` tests module:

```rust
// ==================== InputSource Tests ====================

#[test]
fn input_source_as_str_returns_correct_values() {
    assert_eq!(InputSource::Cli.as_str(), "cli");
    assert_eq!(InputSource::WebUi.as_str(), "web_ui");
    assert_eq!(InputSource::Unknown.as_str(), "unknown");
}

#[test]
fn input_source_parse_returns_correct_variants() {
    assert_eq!(InputSource::parse("cli"), Some(InputSource::Cli));
    assert_eq!(InputSource::parse("web_ui"), Some(InputSource::WebUi));
    assert_eq!(InputSource::parse("unknown"), Some(InputSource::Unknown));
    assert_eq!(InputSource::parse("invalid"), None);
}

#[test]
fn input_source_serialization_roundtrip() {
    for source in [InputSource::Cli, InputSource::WebUi, InputSource::Unknown] {
        let json = serde_json::to_string(&source).unwrap();
        let parsed: InputSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, parsed);
    }
}

#[test]
fn input_source_serializes_to_snake_case() {
    let json = serde_json::to_string(&InputSource::WebUi).unwrap();
    assert_eq!(json, "\"web_ui\"");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core types::tests::input_source --no-default-features`
Expected: FAIL with "cannot find type `InputSource`"

**Step 3: Write minimal implementation**

Add to `vibes-core/src/events/types.rs` (before VibesEvent):

```rust
/// Source of user input for attribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// Input from CLI client
    Cli,
    /// Input from Web UI client
    WebUi,
    /// Source unknown (e.g., historical data before migration)
    Unknown,
}

impl InputSource {
    /// Convert to database/JSON string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::WebUi => "web_ui",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from database/JSON string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cli" => Some(Self::Cli),
            "web_ui" => Some(Self::WebUi),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

impl Default for InputSource {
    fn default() -> Self {
        Self::Unknown
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core types::tests::input_source --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/events/types.rs
git commit -m "feat(events): add InputSource enum for input attribution"
```

---

### Task 1.2: Update UserInput event with source field

**Files:**
- Modify: `vibes-core/src/events/types.rs`

**Step 1: Write the failing test**

Add to tests:

```rust
#[test]
fn vibes_event_user_input_with_source_serialization_roundtrip() {
    let event = VibesEvent::UserInput {
        session_id: "sess-456".to_string(),
        content: "Help me code".to_string(),
        source: InputSource::Cli,
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
    assert!(
        matches!(parsed, VibesEvent::UserInput { session_id, content, source }
        if session_id == "sess-456" && content == "Help me code" && source == InputSource::Cli)
    );
}

#[test]
fn vibes_event_user_input_deserializes_without_source() {
    // Backward compatibility: old messages without source field
    let json = r#"{"type":"user_input","session_id":"sess-1","content":"hello"}"#;
    let parsed: VibesEvent = serde_json::from_str(json).unwrap();
    assert!(
        matches!(parsed, VibesEvent::UserInput { session_id, content, source }
        if session_id == "sess-1" && content == "hello" && source == InputSource::Unknown)
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core types::tests::vibes_event_user_input --no-default-features`
Expected: FAIL (missing source field or wrong structure)

**Step 3: Write minimal implementation**

Update the `VibesEvent::UserInput` variant:

```rust
/// User input from any client
UserInput {
    session_id: String,
    content: String,
    #[serde(default)]
    source: InputSource,
},
```

**Step 4: Update existing test**

Update the existing `vibes_event_user_input_serialization_roundtrip` test:

```rust
#[test]
fn vibes_event_user_input_serialization_roundtrip() {
    let event = VibesEvent::UserInput {
        session_id: "sess-456".to_string(),
        content: "Help me code".to_string(),
        source: InputSource::Unknown,
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
    assert!(
        matches!(parsed, VibesEvent::UserInput { session_id, content, source }
        if session_id == "sess-456" && content == "Help me code" && source == InputSource::Unknown)
    );
}
```

Also update the `vibes_event_session_id_works_for_all_session_event_types` test to include source:

```rust
VibesEvent::UserInput {
    session_id: "s2".to_string(),
    content: "test".to_string(),
    source: InputSource::Unknown,
},
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p vibes-core types::tests --no-default-features`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-core/src/events/types.rs
git commit -m "feat(events): add source field to UserInput event"
```

---

### Task 1.3: Export InputSource from vibes-core

**Files:**
- Modify: `vibes-core/src/events/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Update events/mod.rs**

Add to exports:

```rust
pub use types::InputSource;
```

**Step 2: Update lib.rs**

Add to exports:

```rust
pub use events::InputSource;
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-core/src/events/mod.rs vibes-core/src/lib.rs
git commit -m "feat(core): export InputSource from vibes-core"
```

---

### Task 1.4: Fix compilation errors in dependent crates

**Files:**
- Modify: Any files with `VibesEvent::UserInput` construction

**Step 1: Find all usages**

Run: `cargo check --all` and fix any compilation errors where `UserInput` is constructed without source.

Common pattern - update from:
```rust
VibesEvent::UserInput {
    session_id: session_id.clone(),
    content: content.clone(),
}
```

To:
```rust
VibesEvent::UserInput {
    session_id: session_id.clone(),
    content: content.clone(),
    source: InputSource::Unknown, // Or appropriate source
}
```

**Step 2: Verify compilation**

Run: `cargo check --all`
Expected: PASS

**Step 3: Run all tests**

Run: `cargo test --all`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "fix: update UserInput construction with source field"
```

---

## Phase 2: History Schema Update

### Task 2.1: Add source field to HistoricalMessage

**Files:**
- Modify: `vibes-core/src/history/types.rs`

**Step 1: Write the failing test**

Add to tests:

```rust
#[test]
fn test_historical_message_user_with_source() {
    let msg = HistoricalMessage::user_with_source(
        "sess-1".into(),
        "Hello".into(),
        InputSource::Cli,
        1234567890,
    );
    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.source, InputSource::Cli);
}

#[test]
fn test_historical_message_user_defaults_to_unknown_source() {
    let msg = HistoricalMessage::user("sess-1".into(), "Hello".into(), 1234567890);
    assert_eq!(msg.source, InputSource::Unknown);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core history::types::tests --no-default-features`
Expected: FAIL with missing source field or method

**Step 3: Write minimal implementation**

Add to `HistoricalMessage` struct:

```rust
use crate::events::InputSource;

/// Aggregated message stored in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    // ... existing fields ...

    /// Source of user input (for user messages)
    #[serde(default)]
    pub source: InputSource,
}
```

Add constructor:

```rust
/// Create a new user message with source attribution
pub fn user_with_source(
    session_id: String,
    content: String,
    source: InputSource,
    created_at: i64,
) -> Self {
    Self {
        id: 0,
        session_id,
        role: MessageRole::User,
        content,
        tool_name: None,
        tool_id: None,
        created_at,
        input_tokens: None,
        output_tokens: None,
        source,
    }
}
```

Update existing `user()` to set `source: InputSource::Unknown`.

Also update `assistant()`, `tool_use()`, and `tool_result()` to include `source: InputSource::Unknown`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p vibes-core history::types::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/history/types.rs
git commit -m "feat(history): add source field to HistoricalMessage"
```

---

### Task 2.2: Create source column migration

**Files:**
- Create: `vibes-core/src/history/migrations/003_add_message_source.sql`
- Modify: `vibes-core/src/history/migrations/mod.rs`

**Step 1: Create migration file**

Create `vibes-core/src/history/migrations/003_add_message_source.sql`:

```sql
-- Add source column to track input origin (cli, web_ui, unknown)
ALTER TABLE messages ADD COLUMN source TEXT NOT NULL DEFAULT 'unknown';

-- Index for filtering by source (useful for analytics)
CREATE INDEX idx_messages_source ON messages(source);
```

**Step 2: Register migration**

Update `vibes-core/src/history/migrations/mod.rs` to include the new migration in the migrations list.

**Step 3: Write test**

Add test to verify migration works:

```rust
#[test]
fn test_migration_003_adds_source_column() {
    let store = SqliteHistoryStore::open_in_memory().unwrap();

    // Insert a message and verify source defaults correctly
    let session = HistoricalSession::new("sess-1".into(), None);
    store.save_session(&session).unwrap();

    let msg = HistoricalMessage::user_with_source(
        "sess-1".into(),
        "test".into(),
        InputSource::Cli,
        1234567890,
    );
    store.save_message(&msg).unwrap();

    let messages = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
    assert_eq!(messages.messages[0].source, InputSource::Cli);
}
```

**Step 4: Verify migration runs**

Run: `cargo test -p vibes-core history --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/history/migrations/
git commit -m "feat(history): add migration for source column"
```

---

### Task 2.3: Update HistoryStore to persist source

**Files:**
- Modify: `vibes-core/src/history/store.rs`

**Step 1: Update save_message**

Update the INSERT query to include source:

```rust
"INSERT INTO messages (session_id, role, content, tool_name, tool_id, source, created_at, input_tokens, output_tokens)
 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
```

Add `.bind(message.source.as_str())` in the appropriate position.

**Step 2: Update message retrieval**

Update the SELECT query and row parsing to include source:

```rust
// In row parsing:
source: InputSource::parse(row.get::<_, String>("source")?.as_str())
    .unwrap_or(InputSource::Unknown),
```

**Step 3: Write test**

```rust
#[test]
fn test_source_persisted_and_retrieved() {
    let store = SqliteHistoryStore::open_in_memory().unwrap();
    let session = HistoricalSession::new("sess-1".into(), None);
    store.save_session(&session).unwrap();

    // Save messages with different sources
    let msg_cli = HistoricalMessage::user_with_source(
        "sess-1".into(),
        "from cli".into(),
        InputSource::Cli,
        1000,
    );
    let msg_web = HistoricalMessage::user_with_source(
        "sess-1".into(),
        "from web".into(),
        InputSource::WebUi,
        2000,
    );

    store.save_message(&msg_cli).unwrap();
    store.save_message(&msg_web).unwrap();

    let messages = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
    assert_eq!(messages.messages.len(), 2);
    assert_eq!(messages.messages[0].source, InputSource::Cli);
    assert_eq!(messages.messages[1].source, InputSource::WebUi);
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core history::store --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/history/store.rs
git commit -m "feat(history): persist and retrieve source field"
```

---

### Task 2.4: Update HistoryService to use source from events

**Files:**
- Modify: `vibes-core/src/history/service.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_process_user_input_preserves_source() {
    let service = create_test_service();
    service.create_session("sess-1".into(), None).await.unwrap();

    service
        .process_event(&VibesEvent::UserInput {
            session_id: "sess-1".into(),
            content: "Hello from CLI".into(),
            source: InputSource::Cli,
        })
        .await
        .unwrap();

    let messages = service
        .get_messages("sess-1", &MessageQuery::new())
        .unwrap();
    assert_eq!(messages.messages[0].source, InputSource::Cli);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core service::tests::test_process_user_input_preserves_source --no-default-features`
Expected: FAIL (source not being used)

**Step 3: Update implementation**

Modify the `process_event` method to pass source:

```rust
VibesEvent::UserInput {
    session_id,
    content,
    source,
} => {
    let mut builders = self.builders.write().await;
    if let Some(builder) = builders.get_mut(session_id) {
        builder.add_user_input_with_source(content.clone(), *source);
        self.persist_pending(session_id, builder)?;
    }
}
```

Also update `MessageBuilder` to handle source (in builder.rs).

**Step 4: Run tests**

Run: `cargo test -p vibes-core service::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/history/service.rs vibes-core/src/history/builder.rs
git commit -m "feat(history): preserve source when processing UserInput events"
```

---

## Phase 3: WebSocket Protocol - Catch-Up Messages

### Task 3.1: Add HistoryEvent struct

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_history_event_serialization() {
    let event = HistoryEvent {
        seq: 42,
        event: VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta { text: "hi".to_string() },
        },
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&event).unwrap();
    let parsed: HistoryEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.seq, 42);
    assert_eq!(parsed.timestamp, 1234567890);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_history_event --no-default-features`
Expected: FAIL with "cannot find type `HistoryEvent`"

**Step 3: Write minimal implementation**

Add to `protocol.rs`:

```rust
/// A historical event with sequence number for catch-up
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEvent {
    /// Sequence number for ordering
    pub seq: u64,
    /// The actual event
    pub event: VibesEvent,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-server protocol::tests::test_history_event --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add HistoryEvent struct for catch-up"
```

---

### Task 3.2: Add catch_up flag to Subscribe

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_client_message_subscribe_with_catch_up() {
    let msg = ClientMessage::Subscribe {
        session_ids: vec!["sess-1".to_string()],
        catch_up: true,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains("catch_up"));
}

#[test]
fn test_client_message_subscribe_defaults_catch_up_false() {
    let json = r#"{"type":"subscribe","session_ids":["sess-1"]}"#;
    let parsed: ClientMessage = serde_json::from_str(json).unwrap();
    assert!(matches!(
        parsed,
        ClientMessage::Subscribe { session_ids, catch_up }
        if !catch_up && session_ids.len() == 1
    ));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_client_message_subscribe --no-default-features`
Expected: FAIL (missing catch_up field)

**Step 3: Update implementation**

Modify `ClientMessage::Subscribe`:

```rust
/// Subscribe to session events
Subscribe {
    /// Session IDs to subscribe to
    session_ids: Vec<String>,
    /// Request catch-up with historical events
    #[serde(default)]
    catch_up: bool,
},
```

Update existing subscribe tests to include `catch_up: false`.

**Step 4: Run tests**

Run: `cargo test -p vibes-server protocol::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add catch_up flag to Subscribe message"
```

---

### Task 3.3: Add SubscribeAck message

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_server_message_subscribe_ack_roundtrip() {
    let msg = ServerMessage::SubscribeAck {
        session_id: "sess-1".to_string(),
        current_seq: 150,
        history: vec![
            HistoryEvent {
                seq: 148,
                event: VibesEvent::UserInput {
                    session_id: "sess-1".to_string(),
                    content: "hello".to_string(),
                    source: InputSource::Cli,
                },
                timestamp: 1234567890,
            },
        ],
        has_more: true,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"subscribe_ack""#));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_server_message_subscribe_ack --no-default-features`
Expected: FAIL with variant not found

**Step 3: Write minimal implementation**

Add to `ServerMessage`:

```rust
/// Subscribe acknowledgment with history catch-up
SubscribeAck {
    /// Session ID
    session_id: String,
    /// Current sequence number (live events continue from current_seq + 1)
    current_seq: u64,
    /// Historical events (most recent page)
    history: Vec<HistoryEvent>,
    /// Whether more history pages are available
    has_more: bool,
},
```

**Step 4: Run test**

Run: `cargo test -p vibes-server protocol::tests::test_server_message_subscribe_ack --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add SubscribeAck message for catch-up response"
```

---

### Task 3.4: Add RequestHistory message

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_client_message_request_history_roundtrip() {
    let msg = ClientMessage::RequestHistory {
        session_id: "sess-1".to_string(),
        before_seq: 100,
        limit: 50,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"request_history""#));
}

#[test]
fn test_client_message_request_history_default_limit() {
    let json = r#"{"type":"request_history","session_id":"s1","before_seq":50}"#;
    let parsed: ClientMessage = serde_json::from_str(json).unwrap();
    assert!(matches!(
        parsed,
        ClientMessage::RequestHistory { limit, .. } if limit == 50
    ));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_client_message_request_history --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add helper function and variant:

```rust
fn default_history_limit() -> u32 {
    50
}

// In ClientMessage enum:
/// Request additional history page
RequestHistory {
    /// Session ID
    session_id: String,
    /// Return events with seq < before_seq
    before_seq: u64,
    /// Max events to return
    #[serde(default = "default_history_limit")]
    limit: u32,
},
```

**Step 4: Run test**

Run: `cargo test -p vibes-server protocol::tests::test_client_message_request_history --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add RequestHistory message for pagination"
```

---

### Task 3.5: Add HistoryPage message

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_server_message_history_page_roundtrip() {
    let msg = ServerMessage::HistoryPage {
        session_id: "sess-1".to_string(),
        events: vec![],
        has_more: false,
        oldest_seq: 1,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"history_page""#));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_server_message_history_page --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `ServerMessage`:

```rust
/// Additional history page response
HistoryPage {
    /// Session ID
    session_id: String,
    /// Historical events for this page
    events: Vec<HistoryEvent>,
    /// Whether more pages exist before oldest_seq
    has_more: bool,
    /// Oldest sequence number in this page
    oldest_seq: u64,
},
```

**Step 4: Run test**

Run: `cargo test -p vibes-server protocol::tests::test_server_message_history_page --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add HistoryPage message for pagination"
```

---

### Task 3.6: Add UserInput server message for broadcasting

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_server_message_user_input_roundtrip() {
    let msg = ServerMessage::UserInput {
        session_id: "sess-1".to_string(),
        content: "hello".to_string(),
        source: InputSource::Cli,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"user_input""#));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-server protocol::tests::test_server_message_user_input --no-default-features`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `ServerMessage`:

```rust
/// User input broadcast to other subscribers
UserInput {
    /// Session ID
    session_id: String,
    /// Input content
    content: String,
    /// Source of the input
    source: InputSource,
},
```

Also update `vibes_event_to_server_message` to handle UserInput:

```rust
VibesEvent::UserInput { session_id, content, source } => {
    Some(ServerMessage::UserInput {
        session_id: session_id.clone(),
        content: content.clone(),
        source: *source,
    })
}
```

**Step 4: Run test**

Run: `cargo test -p vibes-server protocol::tests --no-default-features`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add UserInput broadcast message with source"
```

---

## Phase 4: Connection State Updates

### Task 4.1: Add client_type to ConnectionState

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Update ConnectionState struct**

```rust
use vibes_core::InputSource;

struct ConnectionState {
    /// Unique ID for this connection
    client_id: String,
    /// Type of client (CLI, Web UI)
    client_type: InputSource,
    /// Session IDs this connection is subscribed to
    subscribed_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new(client_type: InputSource) -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            client_type,
            subscribed_sessions: HashSet::new(),
        }
    }

    fn client_type(&self) -> InputSource {
        self.client_type
    }

    // ... existing methods
}
```

**Step 2: Update constructor calls**

Update places where `ConnectionState::new()` is called to pass the client type.

**Step 3: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): add client_type to WebSocket ConnectionState"
```

---

### Task 4.2: Detect client type from request headers

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Create detection function**

```rust
use axum::http::Request;

fn detect_client_type<B>(req: &Request<B>) -> InputSource {
    if let Some(header) = req.headers().get("X-Vibes-Client-Type") {
        if let Ok(value) = header.to_str() {
            if value == "cli" {
                return InputSource::Cli;
            }
        }
    }
    // Default to Web UI for browser connections
    InputSource::WebUi
}
```

**Step 2: Use in WebSocket handler**

Update the WebSocket upgrade handler to detect client type:

```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let client_type = detect_client_type(&req);
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_type))
}
```

**Step 3: Update handle_socket**

```rust
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    client_type: InputSource,
) {
    let conn_state = ConnectionState::new(client_type);
    // ...
}
```

**Step 4: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): detect client type from X-Vibes-Client-Type header"
```

---

### Task 4.3: Attach source to input events

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Update Input handling**

When handling `ClientMessage::Input`, use the connection's client_type:

```rust
ClientMessage::Input { session_id, content } => {
    // Publish event with source
    let event = VibesEvent::UserInput {
        session_id: session_id.clone(),
        content: content.clone(),
        source: conn_state.client_type(),
    };
    state.event_bus.publish(event).await;

    // Forward to session
    if let Err(e) = state.session_manager.send_to_session(&session_id, &content).await {
        // ... error handling
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): attach client source to input events"
```

---

## Phase 5: Catch-Up Implementation

### Task 5.1: Handle Subscribe with catch_up

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Update Subscribe handler**

```rust
ClientMessage::Subscribe { session_ids, catch_up } => {
    for session_id in &session_ids {
        conn_state.subscribe(session_id);

        if catch_up {
            // Get history and send SubscribeAck
            let (history, current_seq, has_more) =
                get_session_history(state.as_ref(), session_id, 50).await;

            let ack = ServerMessage::SubscribeAck {
                session_id: session_id.clone(),
                current_seq,
                history,
                has_more,
            };
            let json = serde_json::to_string(&ack)?;
            sender.send(Message::Text(json)).await?;
        }
    }
}
```

**Step 2: Create helper function**

```rust
async fn get_session_history(
    state: &AppState,
    session_id: &str,
    limit: u32,
) -> (Vec<HistoryEvent>, u64, bool) {
    // Get messages from history service
    let query = MessageQuery::new().with_limit(limit + 1);
    let result = state.history_service
        .get_messages(session_id, &query)
        .unwrap_or_default();

    let has_more = result.messages.len() > limit as usize;
    let messages: Vec<_> = result.messages.into_iter()
        .take(limit as usize)
        .collect();

    let current_seq = messages.last()
        .map(|m| m.id as u64)
        .unwrap_or(0);

    let history: Vec<HistoryEvent> = messages.into_iter()
        .map(|m| HistoryEvent {
            seq: m.id as u64,
            event: message_to_vibes_event(&m),
            timestamp: m.created_at * 1000,
        })
        .collect();

    (history, current_seq, has_more)
}

fn message_to_vibes_event(msg: &HistoricalMessage) -> VibesEvent {
    match msg.role {
        MessageRole::User => VibesEvent::UserInput {
            session_id: msg.session_id.clone(),
            content: msg.content.clone(),
            source: msg.source,
        },
        MessageRole::Assistant => VibesEvent::Claude {
            session_id: msg.session_id.clone(),
            event: ClaudeEvent::TextDelta { text: msg.content.clone() },
        },
        // ... handle other roles
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): implement catch-up on subscribe"
```

---

### Task 5.2: Handle RequestHistory message

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Add handler**

```rust
ClientMessage::RequestHistory { session_id, before_seq, limit } => {
    if !conn_state.is_subscribed_to(&session_id) {
        let error = ServerMessage::Error {
            session_id: Some(session_id),
            message: "Not subscribed to session".to_string(),
            code: "NOT_SUBSCRIBED".to_string(),
        };
        let json = serde_json::to_string(&error)?;
        sender.send(Message::Text(json)).await?;
        return Ok(());
    }

    let (events, oldest_seq, has_more) =
        get_history_page(state.as_ref(), &session_id, before_seq, limit).await;

    let page = ServerMessage::HistoryPage {
        session_id,
        events,
        has_more,
        oldest_seq,
    };
    let json = serde_json::to_string(&page)?;
    sender.send(Message::Text(json)).await?;
}
```

**Step 2: Create helper function**

```rust
async fn get_history_page(
    state: &AppState,
    session_id: &str,
    before_seq: u64,
    limit: u32,
) -> (Vec<HistoryEvent>, u64, bool) {
    let query = MessageQuery::new()
        .with_limit(limit + 1)
        .with_before_id(before_seq as i64);

    let result = state.history_service
        .get_messages(session_id, &query)
        .unwrap_or_default();

    let has_more = result.messages.len() > limit as usize;
    let messages: Vec<_> = result.messages.into_iter()
        .take(limit as usize)
        .collect();

    let oldest_seq = messages.first()
        .map(|m| m.id as u64)
        .unwrap_or(0);

    let events: Vec<HistoryEvent> = messages.into_iter()
        .map(|m| HistoryEvent {
            seq: m.id as u64,
            event: message_to_vibes_event(&m),
            timestamp: m.created_at * 1000,
        })
        .collect();

    (events, oldest_seq, has_more)
}
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(server): implement paginated history requests"
```

---

## Phase 6: CLI Input History

### Task 6.1: Create InputHistory struct

**Files:**
- Create: `vibes-cli/src/input/mod.rs`
- Create: `vibes-cli/src/input/history.rs`

**Step 1: Create module**

Create `vibes-cli/src/input/mod.rs`:

```rust
//! Input handling for CLI

mod history;

pub use history::InputHistory;
```

**Step 2: Write the failing tests**

Create `vibes-cli/src/input/history.rs`:

```rust
//! In-memory input history for up/down navigation

/// Manages input history for CLI sessions
#[derive(Debug, Default)]
pub struct InputHistory {
    /// Previous inputs
    entries: Vec<String>,
    /// Current navigation position (None = not navigating)
    position: Option<usize>,
    /// Draft input when starting navigation
    draft: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_history_is_empty() {
        let history = InputHistory::new();
        assert!(history.entries.is_empty());
        assert!(history.position.is_none());
    }

    #[test]
    fn push_adds_entry() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn push_deduplicates_consecutive() {
        let mut history = InputHistory::new();
        history.push("same".to_string());
        history.push("same".to_string());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn navigate_up_returns_previous() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());

        assert_eq!(history.navigate_up("current"), Some("second"));
        assert_eq!(history.navigate_up("current"), Some("first"));
        assert_eq!(history.navigate_up("current"), None); // No more
    }

    #[test]
    fn navigate_down_returns_next() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());

        history.navigate_up("draft");
        history.navigate_up("draft");

        assert_eq!(history.navigate_down(), Some("second"));
        assert_eq!(history.navigate_down(), Some("draft")); // Returns to draft
        assert_eq!(history.navigate_down(), None);
    }

    #[test]
    fn push_resets_navigation() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.navigate_up("draft");

        history.push("new".to_string());

        assert!(history.position.is_none());
    }
}
```

**Step 3: Run tests to verify they fail**

Run: `cargo test -p vibes-cli input::history::tests --no-default-features`
Expected: FAIL with methods not found

**Step 4: Write minimal implementation**

```rust
impl InputHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add input to history
    pub fn push(&mut self, input: String) {
        // Don't add duplicates consecutively
        if self.entries.last() != Some(&input) {
            self.entries.push(input);
        }
        self.position = None;
    }

    /// Navigate up (older entries)
    pub fn navigate_up(&mut self, current: &str) -> Option<&str> {
        match self.position {
            None if !self.entries.is_empty() => {
                self.draft = current.to_string();
                self.position = Some(self.entries.len() - 1);
                self.entries.last().map(|s| s.as_str())
            }
            Some(0) => None,
            Some(pos) => {
                self.position = Some(pos - 1);
                Some(&self.entries[pos - 1])
            }
            _ => None,
        }
    }

    /// Navigate down (newer entries, back to draft)
    pub fn navigate_down(&mut self) -> Option<&str> {
        match self.position {
            None => None,
            Some(pos) if pos >= self.entries.len() - 1 => {
                self.position = None;
                Some(&self.draft)
            }
            Some(pos) => {
                self.position = Some(pos + 1);
                Some(&self.entries[pos + 1])
            }
        }
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p vibes-cli input::history::tests`
Expected: PASS

**Step 6: Export module**

Update `vibes-cli/src/lib.rs` or `main.rs` to include:

```rust
mod input;
pub use input::InputHistory;
```

**Step 7: Commit**

```bash
git add vibes-cli/src/input/
git commit -m "feat(cli): add InputHistory for up/down navigation"
```

---

## Phase 7: CLI Remote Input Display

### Task 7.1: Add remote input display function

**Files:**
- Modify: `vibes-cli/src/commands/sessions.rs` (or claude.rs)

**Step 1: Create display function**

```rust
use colored::Colorize;
use vibes_core::InputSource;

fn display_remote_input(source: InputSource, content: &str) {
    let prefix = match source {
        InputSource::WebUi => "[Web UI]:".cyan(),
        InputSource::Cli => "[CLI]:".cyan(),
        InputSource::Unknown => "[Remote]:".dimmed(),
    };

    println!();
    println!("{} {}", prefix, content);
}
```

**Step 2: Handle UserInput in event loop**

Update the session attach event loop to handle `ServerMessage::UserInput`:

```rust
ServerMessage::UserInput { session_id: sid, content, source }
    if sid == session_id && source != InputSource::Cli => {
    display_remote_input(source, &content);
}
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-cli/src/commands/
git commit -m "feat(cli): display remote input with source prefix"
```

---

### Task 7.2: Add X-Vibes-Client-Type header

**Files:**
- Modify: `vibes-cli/src/client.rs` (or wherever WsClient is defined)

**Step 1: Update WebSocket connection**

When connecting, add the header:

```rust
use tokio_tungstenite::tungstenite::http::Request;

impl WsClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let request = Request::builder()
            .uri(url)
            .header("X-Vibes-Client-Type", "cli")
            .body(())?;

        let (socket, _response) = tokio_tungstenite::connect_async(request).await?;
        // ...
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/client.rs
git commit -m "feat(cli): send X-Vibes-Client-Type header on connect"
```

---

### Task 7.3: Handle catch-up on attach

**Files:**
- Modify: `vibes-cli/src/commands/sessions.rs`

**Step 1: Update attach to request catch-up**

```rust
async fn attach_session(session_id: &str) -> anyhow::Result<()> {
    daemon::ensure_running().await?;
    let mut client = WsClient::connect("ws://127.0.0.1:7432/ws").await?;

    // Subscribe with catch-up
    client.send(&ClientMessage::Subscribe {
        session_ids: vec![session_id.to_string()],
        catch_up: true,
    }).await?;

    println!("Attaching to session {}...", session_id.cyan());

    // Wait for SubscribeAck and display history
    loop {
        match client.recv().await? {
            ServerMessage::SubscribeAck { session_id: sid, history, has_more, .. }
                if sid == session_id => {
                display_catch_up_history(&history);

                // Request more pages if needed
                if has_more {
                    fetch_remaining_history(&mut client, session_id, &history).await?;
                }

                println!("Connected. You are now a subscriber.\n");
                break;
            }
            _ => continue,
        }
    }

    run_interactive_session(&mut client, session_id).await
}

fn display_catch_up_history(history: &[HistoryEvent]) {
    for event in history {
        match &event.event {
            VibesEvent::UserInput { content, source, .. } => {
                let prefix = match source {
                    InputSource::Cli => "You:".green(),
                    InputSource::WebUi => "[Web UI]:".cyan(),
                    InputSource::Unknown => "User:".dimmed(),
                };
                println!("{} {}", prefix, content);
            }
            VibesEvent::Claude { event: ClaudeEvent::TextDelta { text }, .. } => {
                print!("{}", text);
            }
            VibesEvent::Claude { event: ClaudeEvent::TurnComplete { .. }, .. } => {
                println!();
            }
            _ => {}
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p vibes-cli`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-cli/src/commands/sessions.rs
git commit -m "feat(cli): handle catch-up on session attach"
```

---

## Phase 8: Web UI Updates

### Task 8.1: Update Message type with source

**Files:**
- Modify: `web-ui/src/lib/types.ts`

**Step 1: Add source to Message type**

```typescript
export type InputSource = 'cli' | 'web_ui' | 'unknown';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'tool_use' | 'tool_result';
  content: string;
  timestamp: number;
  source?: InputSource;
}
```

**Step 2: Commit**

```bash
git add web-ui/src/lib/types.ts
git commit -m "feat(web-ui): add source field to Message type"
```

---

### Task 8.2: Update MessageBubble with source display

**Files:**
- Modify: `web-ui/src/components/MessageBubble.tsx`

**Step 1: Update component**

```tsx
import { InputSource } from '../lib/types';

interface MessageBubbleProps {
  role: 'user' | 'assistant';
  content: string;
  source?: InputSource;
  isOwnInput?: boolean;
}

function getSourceLabel(source: InputSource): string {
  switch (source) {
    case 'cli': return '📟 CLI';
    case 'web_ui': return '🌐 Web';
    default: return '';
  }
}

export function MessageBubble({ role, content, source, isOwnInput = true }: MessageBubbleProps) {
  const isRemote = role === 'user' && !isOwnInput;

  if (role === 'user') {
    return (
      <div className={cn(
        "flex flex-col",
        isRemote ? "items-start" : "items-end"
      )}>
        {isRemote && source && (
          <span className="text-xs text-muted-foreground mb-1">
            {getSourceLabel(source)}
          </span>
        )}
        <div className={cn(
          "rounded-lg px-4 py-2 max-w-[80%]",
          isRemote
            ? "bg-secondary text-secondary-foreground"
            : "bg-primary text-primary-foreground"
        )}>
          {content}
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-start">
      <div className="rounded-lg px-4 py-2 max-w-[80%] bg-muted">
        {content}
      </div>
    </div>
  );
}
```

**Step 2: Commit**

```bash
git add web-ui/src/components/MessageBubble.tsx
git commit -m "feat(web-ui): show source attribution on remote messages"
```

---

### Task 8.3: Create useCatchUp hook

**Files:**
- Create: `web-ui/src/hooks/useCatchUp.ts`

**Step 1: Create hook**

```typescript
import { useState, useCallback } from 'react';
import { useWebSocket } from './useWebSocket';
import type { Message, InputSource } from '../lib/types';

interface HistoryEvent {
  seq: number;
  event: {
    type: string;
    session_id: string;
    content?: string;
    source?: InputSource;
    event?: { type: string; text?: string };
  };
  timestamp: number;
}

interface SubscribeAck {
  type: 'subscribe_ack';
  session_id: string;
  current_seq: number;
  history: HistoryEvent[];
  has_more: boolean;
}

interface HistoryPage {
  type: 'history_page';
  session_id: string;
  events: HistoryEvent[];
  has_more: boolean;
  oldest_seq: number;
}

function parseHistoryEvents(events: HistoryEvent[]): Message[] {
  const messages: Message[] = [];

  for (const { seq, event, timestamp } of events) {
    if (event.type === 'user_input') {
      messages.push({
        id: `msg-${seq}`,
        role: 'user',
        content: event.content || '',
        timestamp,
        source: event.source,
      });
    } else if (event.type === 'claude' && event.event?.type === 'text_delta') {
      // Aggregate text deltas into assistant messages
      const lastMsg = messages[messages.length - 1];
      if (lastMsg?.role === 'assistant') {
        lastMsg.content += event.event.text || '';
      } else {
        messages.push({
          id: `msg-${seq}`,
          role: 'assistant',
          content: event.event.text || '',
          timestamp,
        });
      }
    }
  }

  return messages;
}

export function useCatchUp(sessionId: string) {
  const { sendMessage, waitForMessage } = useWebSocket();
  const [loading, setLoading] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);

  const fetchHistory = useCallback(async () => {
    setLoading(true);

    // Subscribe with catch-up
    sendMessage({
      type: 'subscribe',
      session_ids: [sessionId],
      catch_up: true,
    });

    // Wait for SubscribeAck
    const ack = await waitForMessage<SubscribeAck>(
      msg => msg.type === 'subscribe_ack' && msg.session_id === sessionId
    );

    let allMessages = parseHistoryEvents(ack.history);
    let hasMore = ack.has_more;
    let oldestSeq = ack.history[0]?.seq ?? 0;

    // Fetch remaining pages
    while (hasMore) {
      sendMessage({
        type: 'request_history',
        session_id: sessionId,
        before_seq: oldestSeq,
        limit: 50,
      });

      const page = await waitForMessage<HistoryPage>(
        msg => msg.type === 'history_page' && msg.session_id === sessionId
      );

      allMessages = [...parseHistoryEvents(page.events), ...allMessages];
      hasMore = page.has_more;
      oldestSeq = page.oldest_seq;
    }

    setMessages(allMessages);
    setLoading(false);
  }, [sessionId, sendMessage, waitForMessage]);

  return { messages, loading, fetchHistory };
}
```

**Step 2: Commit**

```bash
git add web-ui/src/hooks/useCatchUp.ts
git commit -m "feat(web-ui): add useCatchUp hook for late-joiner history"
```

---

### Task 8.4: Handle UserInput events in session view

**Files:**
- Modify: `web-ui/src/hooks/useSessionEvents.ts`

**Step 1: Update event handling**

Add handling for `user_input` server messages:

```typescript
case 'user_input': {
  const { content, source } = message;
  // Only show if from remote source (not our own input)
  if (source !== 'web_ui') {
    addMessage({
      id: `remote-${Date.now()}`,
      role: 'user',
      content,
      timestamp: Date.now(),
      source,
    });
  }
  break;
}
```

**Step 2: Commit**

```bash
git add web-ui/src/hooks/useSessionEvents.ts
git commit -m "feat(web-ui): handle remote UserInput events"
```

---

### Task 8.5: Integrate catch-up in session page

**Files:**
- Modify: `web-ui/src/pages/ClaudeSession.tsx`

**Step 1: Use catch-up hook**

```typescript
import { useCatchUp } from '../hooks/useCatchUp';

export function ClaudeSession({ sessionId }: { sessionId: string }) {
  const { messages: historyMessages, loading, fetchHistory } = useCatchUp(sessionId);
  const { messages: liveMessages } = useSessionEvents(sessionId);

  // Fetch history on mount
  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  const allMessages = [...historyMessages, ...liveMessages];

  // ... rest of component
}
```

**Step 2: Commit**

```bash
git add web-ui/src/pages/ClaudeSession.tsx
git commit -m "feat(web-ui): integrate catch-up in session view"
```

---

## Phase 9: Integration Testing & Documentation

### Task 9.1: Run all tests

**Step 1: Run full test suite**

```bash
just test
```

Expected: All tests pass

**Step 2: Run pre-commit checks**

```bash
just pre-commit
```

Expected: All checks pass

**Step 3: Fix any issues**

```bash
git add -A
git commit -m "fix: address test failures and lint issues"
```

---

### Task 9.2: Manual testing checklist

Test the following scenarios:

- [ ] Send input from CLI, verify Web UI shows `📟 CLI` label
- [ ] Send input from Web UI, verify CLI shows `[Web UI]:` prefix
- [ ] Open Web UI mid-session, verify full history loads with source
- [ ] Test up/down arrow history navigation in CLI
- [ ] Verify source column populated in SQLite
- [ ] Test catch-up pagination with session >50 messages
- [ ] Verify live events continue after catch-up
- [ ] Test both clients on same session, verify bidirectional sync

---

### Task 9.3: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Update milestone status**

Mark CLI ↔ Web Mirroring items as complete:

```markdown
### Milestone 3.3: CLI ↔ Web Mirroring
- [x] Design complete (input attribution, catch-up protocol)
- [x] Add `InputSource` enum and update `UserInput` event
- [x] Add `source` column to messages table
- [x] Implement `SubscribeAck` with paginated catch-up
- [x] CLI displays remote input with `[Web UI]:` prefix
- [x] CLI input history with up/down navigation
- [x] Web UI shows source attribution on messages
- [x] Web UI catch-up on session join
```

**Step 2: Add changelog entry**

```markdown
| 2025-12-27 | Milestone 3.3 (CLI ↔ Web Mirroring) complete - InputSource enum, paginated catch-up, bidirectional input display |
```

**Step 3: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 3.3 as complete"
```

---

### Task 9.4: Final commit and PR

**Step 1: Verify everything passes**

```bash
just pre-commit
```

**Step 2: Push and create PR**

```bash
git push -u origin cli-web-mirror
gh pr create --title "feat: add CLI ↔ Web mirroring (milestone 3.3)" --body "$(cat <<'EOF'
## Summary
- Add `InputSource` enum for tracking input origin (cli, web_ui)
- Add `source` column to messages table with migration
- Implement paginated catch-up protocol (SubscribeAck, RequestHistory, HistoryPage)
- CLI displays remote input with `[Web UI]:` prefix
- CLI input history with up/down arrow navigation
- Web UI shows source attribution badges on messages
- Web UI late-joiner catch-up with automatic pagination

## Test Plan
- [x] All unit tests passing (`just test`)
- [x] Pre-commit checks pass (`just pre-commit`)
- [ ] Manual test: input from CLI shows in Web UI with source
- [ ] Manual test: input from Web UI shows in CLI with prefix
- [ ] Manual test: late-joiner receives full history
- [ ] Manual test: CLI up/down navigation works
EOF
)"
```

---

## Summary

This implementation plan covers:

1. **Phase 1:** InputSource enum and UserInput event update (vibes-core)
2. **Phase 2:** History schema migration for source column (vibes-core)
3. **Phase 3:** WebSocket protocol messages for catch-up (vibes-server)
4. **Phase 4:** Connection state with client type detection (vibes-server)
5. **Phase 5:** Catch-up implementation (vibes-server)
6. **Phase 6:** CLI input history with up/down navigation (vibes-cli)
7. **Phase 7:** CLI remote input display and headers (vibes-cli)
8. **Phase 8:** Web UI source attribution and catch-up (web-ui)
9. **Phase 9:** Integration testing and documentation

Each task follows TDD: write failing test → verify failure → implement → verify pass → commit.

Estimated tasks: ~40 bite-sized steps across 9 phases.

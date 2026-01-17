# Milestone 3.1: Chat History - Design Document

> Persistent session history with search, filtering, and session resumption.

## Overview

Chat History enables users to browse, search, and resume past Claude Code sessions from any client (CLI or Web UI). Sessions are persisted to SQLite with full-text search capabilities, allowing users to find conversations by content, tool usage, or metadata.

Currently, all session data is lost when the vibes server restarts. This milestone adds durable storage for session metadata and aggregated messages, transforming vibes from a transient proxy into a persistent conversation manager.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Storage backend | SQLite | Fast queries, FTS5 for search, single file, mature ecosystem |
| What to persist | Aggregated messages | Clean history without streaming delta noise |
| Persistence trigger | On turn complete | Natural checkpoints, durable, SQLite handles frequent writes |
| Access pattern | HTTP REST | Request/response fits history queries; WebSocket stays live-focused |
| Search capabilities | Advanced (FTS5 + filters) | Full-text search + tool/token/state filters |
| Session resumption | Attempt with graceful error | Store Claude session ID, fail gracefully if expired |
| Schema evolution | Versioned migrations | Safe upgrades as features are added |

---

## ADR: SQLite for Chat History Storage

### Status

Accepted

### Context

We need to persist chat history across server restarts. The codebase currently uses file-based JSON for push notification subscriptions (`SubscriptionStore`). For chat history, we need:

- Fast search across message content
- Filtering by session metadata (date, state, tool usage)
- Pagination for large history
- Reasonable storage efficiency

### Options Considered

**Option A: JSONL Files (1 file per session)**
- Pros: Simple, human-readable, follows existing patterns, no new dependencies
- Cons: Slow search (must scan all files), no indexing, complex pagination

**Option B: SQLite**
- Pros: Fast queries, built-in FTS5 for full-text search, single file database, ACID compliance, mature `rusqlite` crate
- Cons: New dependency, slightly more complex than file I/O

**Option C: Hybrid (SQLite index + JSONL events)**
- Pros: Fast search in SQLite, raw events preserved in files
- Cons: Two storage mechanisms to maintain, sync complexity

### Decision

**SQLite (Option B)** for all chat history storage.

### Rationale

1. **Search requirements**: Advanced search with full-text and filters requires indexing. SQLite's FTS5 provides this out of the box.

2. **Pagination**: SQLite's `LIMIT/OFFSET` is trivial. File-based pagination requires loading and sorting all records.

3. **Single file simplicity**: SQLite is still a single file (`~/.config/vibes/history.db`), easy to backup or delete.

4. **Proven in Rust**: `rusqlite` is mature, well-tested, and commonly used.

5. **Migration path**: SQLite's schema can evolve with versioned migrations. JSON schema changes are harder to manage.

### Consequences

- Add `rusqlite` dependency to vibes-core
- Implement migration runner for schema evolution
- Database file location: `{config_dir}/history.db`

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        vibes-core                           │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐ │
│  │   Session   │───▶│  EventBus   │───▶│  HistoryStore   │ │
│  │   Manager   │    │  (Memory)   │    │   (SQLite)      │ │
│  └─────────────┘    └─────────────┘    └─────────────────┘ │
│         │                                      │            │
│         │                                      │            │
│         ▼                                      ▼            │
│  ┌─────────────┐                      ┌─────────────────┐  │
│  │   Session   │                      │  MessageBuilder │  │
│  │  (Claude)   │                      │  (aggregates    │  │
│  │             │                      │   streaming     │  │
│  └─────────────┘                      │   events)       │  │
│         │                             └─────────────────┘  │
│         │                                      │            │
│         ▼                                      ▼            │
│  ┌─────────────┐                      ┌─────────────────┐  │
│  │   Claude    │                      │ HistoryService  │  │
│  │   Backend   │                      │ (search, resume │  │
│  └─────────────┘                      │  pagination)    │  │
│                                       └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       vibes-server                          │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   REST API                           │   │
│  │                                                      │   │
│  │  GET  /api/history/sessions      (list + search)    │   │
│  │  GET  /api/history/sessions/:id  (session detail)   │   │
│  │  GET  /api/history/sessions/:id/messages            │   │
│  │  POST /api/history/sessions/:id/resume              │   │
│  │  DELETE /api/history/sessions/:id                   │   │
│  │                                                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                              │                              │
└──────────────────────────────│──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                        web-ui                               │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐ │
│  │  History    │    │  Session    │    │    Resume       │ │
│  │  List View  │───▶│  Detail     │───▶│    Button       │ │
│  │  + Search   │    │  + Messages │    │                 │ │
│  └─────────────┘    └─────────────┘    └─────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| `HistoryStore` | vibes-core/src/history/store.rs | SQLite connection, CRUD, search queries |
| `MessageBuilder` | vibes-core/src/history/builder.rs | Aggregates streaming events into complete messages |
| `HistoryService` | vibes-core/src/history/service.rs | Business logic: search, pagination, resume |
| `Migrator` | vibes-core/src/history/migrations/ | Schema versioning and upgrades |
| History routes | vibes-server/src/routes/history.rs | REST endpoints |

### Event Flow for Persistence

```
1. User sends input
   └─▶ VibesEvent::UserInput { session_id, content }
       └─▶ MessageBuilder.add_user_input(content)
           └─▶ HistoryStore.save_message(user_message)

2. Claude streams response
   └─▶ ClaudeEvent::TextDelta { text }
       └─▶ MessageBuilder.append_text(text)  // accumulates
   └─▶ ClaudeEvent::ToolUseStart { id, name }
       └─▶ MessageBuilder.start_tool(id, name)
   └─▶ ClaudeEvent::ToolResult { id, output }
       └─▶ MessageBuilder.complete_tool(id, output)
       └─▶ HistoryStore.save_message(tool_result)

3. Turn completes
   └─▶ ClaudeEvent::TurnComplete { usage }
       └─▶ MessageBuilder.finalize()
           └─▶ HistoryStore.save_message(assistant_message)
           └─▶ HistoryStore.update_session_stats(usage)
```

---

## Database Schema

### Migration v001: Initial Schema

```sql
-- Store schema version
PRAGMA user_version = 1;

-- Session metadata
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT,
    claude_session_id TEXT,
    state TEXT NOT NULL DEFAULT 'Idle',
    created_at INTEGER NOT NULL,
    last_accessed_at INTEGER NOT NULL,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    message_count INTEGER DEFAULT 0,
    error_message TEXT
);

CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_state ON sessions(state);
CREATE INDEX idx_sessions_name ON sessions(name);

-- Aggregated messages
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    tool_name TEXT,
    tool_id TEXT,
    created_at INTEGER NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER
);

CREATE INDEX idx_messages_session_id ON messages(session_id);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_tool_name ON messages(tool_name);
```

### Migration v002: Full-Text Search

```sql
PRAGMA user_version = 2;

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=id
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.id, old.content);
END;

CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.id, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;
```

---

## Types and Interfaces

### Core Types

```rust
// vibes-core/src/history/types.rs

use serde::{Deserialize, Serialize};
use crate::session::SessionState;

/// Persisted session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSession {
    pub id: String,
    pub name: Option<String>,
    pub claude_session_id: Option<String>,
    pub state: SessionState,
    pub created_at: i64,          // Unix timestamp (seconds)
    pub last_accessed_at: i64,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub message_count: u32,
    pub error_message: Option<String>,
}

/// Aggregated message (not raw streaming events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    pub id: i64,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_id: Option<String>,
    pub created_at: i64,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

/// Message role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    ToolUse,
    ToolResult,
}

/// Session summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: Option<String>,
    pub state: SessionState,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub message_count: u32,
    pub total_tokens: u32,
    pub preview: String,          // First ~100 chars of first message
}
```

### Query Types

```rust
// vibes-core/src/history/query.rs

/// Query parameters for session list
#[derive(Debug, Default)]
pub struct SessionQuery {
    pub search: Option<String>,       // Full-text search
    pub name: Option<String>,         // LIKE pattern
    pub state: Option<SessionState>,
    pub tool: Option<String>,         // Sessions using this tool
    pub min_tokens: Option<u32>,
    pub after: Option<i64>,           // Created after timestamp
    pub before: Option<i64>,          // Created before timestamp
    pub limit: u32,                   // Default 20, max 100
    pub offset: u32,
    pub sort: SortField,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortField {
    #[default]
    CreatedAt,
    LastAccessedAt,
    MessageCount,
    TotalTokens,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Paginated session list response
#[derive(Debug, Serialize)]
pub struct SessionListResult {
    pub sessions: Vec<SessionSummary>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

/// Query parameters for message list
#[derive(Debug, Default)]
pub struct MessageQuery {
    pub limit: u32,                   // Default 50
    pub offset: u32,
    pub role: Option<MessageRole>,
}

/// Paginated message list response
#[derive(Debug, Serialize)]
pub struct MessageListResult {
    pub messages: Vec<HistoricalMessage>,
    pub total: u32,
}
```

### Store Trait

```rust
// vibes-core/src/history/store.rs

use async_trait::async_trait;

#[async_trait]
pub trait HistoryStore: Send + Sync {
    /// Initialize store (run migrations)
    async fn init(&self) -> Result<()>;

    // Session operations
    async fn save_session(&self, session: &HistoricalSession) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>>;
    async fn update_session(&self, session: &HistoricalSession) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    async fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult>;

    // Message operations
    async fn save_message(&self, message: &HistoricalMessage) -> Result<i64>;
    async fn get_messages(&self, session_id: &str, query: &MessageQuery) -> Result<MessageListResult>;

    // Stats
    async fn update_session_stats(&self, session_id: &str, input_tokens: u32, output_tokens: u32) -> Result<()>;
}
```

---

## API Design

### REST Endpoints

```
GET  /api/history/sessions
     Query Parameters:
     - q: string              # Full-text search across messages
     - name: string           # Filter by session name (LIKE pattern)
     - state: string          # Filter: Finished, Failed, Idle
     - tool: string           # Sessions that used this tool
     - min_tokens: integer    # Minimum total tokens
     - after: integer         # Created after (Unix timestamp)
     - before: integer        # Created before (Unix timestamp)
     - limit: integer         # Default 20, max 100
     - offset: integer        # For pagination
     - sort: string           # created_at, last_accessed_at, message_count, total_tokens
     - order: string          # asc, desc (default: desc)

     Response: SessionListResult

GET  /api/history/sessions/:id
     Response: HistoricalSession

GET  /api/history/sessions/:id/messages
     Query Parameters:
     - limit: integer         # Default 50
     - offset: integer        # For pagination
     - role: string           # Filter by role

     Response: MessageListResult

POST /api/history/sessions/:id/resume
     Response: { "session_id": "new-active-session-id" }
     Error: { "error": "Session expired or not resumable" }

DELETE /api/history/sessions/:id
     Response: 204 No Content
```

### Error Responses

```json
{
  "error": "Session not found",
  "code": "NOT_FOUND"
}
```

---

## Crate Structure

### New/Modified Files

```
vibes/
├── vibes-core/
│   ├── Cargo.toml                    # Add rusqlite dependency
│   └── src/
│       ├── lib.rs                    # Export history module
│       └── history/                  # NEW MODULE
│           ├── mod.rs                # Module exports
│           ├── types.rs              # HistoricalSession, HistoricalMessage, etc.
│           ├── query.rs              # SessionQuery, MessageQuery, results
│           ├── store.rs              # HistoryStore trait + SqliteHistoryStore
│           ├── builder.rs            # MessageBuilder for event aggregation
│           ├── service.rs            # HistoryService business logic
│           ├── error.rs              # HistoryError enum
│           └── migrations/
│               ├── mod.rs            # Migrator implementation
│               ├── v001_initial.sql
│               └── v002_fts.sql
├── vibes-server/
│   └── src/
│       ├── routes/
│       │   ├── mod.rs                # Add history routes
│       │   └── history.rs            # NEW: REST endpoints
│       └── state.rs                  # Add HistoryStore to AppState
└── web-ui/
    └── src/
        ├── api/
        │   └── history.ts            # NEW: API client functions
        ├── components/
        │   ├── HistoryList.tsx       # NEW: Session list with search
        │   ├── HistoryDetail.tsx     # NEW: Session messages view
        │   └── HistorySearch.tsx     # NEW: Search/filter controls
        └── routes/
            └── history.tsx           # NEW: History page route
```

---

## Dependencies

### vibes-core/Cargo.toml

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }  # SQLite with FTS5
```

**Why `bundled` feature**: Includes SQLite source, ensuring FTS5 support regardless of system SQLite version.

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| `SqliteHistoryStore` | CRUD operations, search queries, FTS, pagination |
| `MessageBuilder` | Event aggregation, tool handling, edge cases |
| `Migrator` | Version detection, migration ordering, idempotency |
| Query parsing | All filter combinations, edge cases |

### Integration Tests

| Test | Description |
|------|-------------|
| Full persistence flow | Create session → send messages → query history |
| Search accuracy | FTS5 returns correct results, ranking |
| Resume flow | Attempt resume, handle expired session |
| Pagination | Large result sets paginate correctly |

### Test Utilities

```rust
// In-memory SQLite for fast tests
#[tokio::test]
async fn test_session_crud() {
    let store = SqliteHistoryStore::open(":memory:").await.unwrap();
    store.init().await.unwrap();

    let session = HistoricalSession {
        id: "test-123".into(),
        name: Some("Test".into()),
        state: SessionState::Finished,
        created_at: now(),
        // ...
    };

    store.save_session(&session).await.unwrap();
    let loaded = store.get_session("test-123").await.unwrap();
    assert_eq!(loaded.unwrap().name, Some("Test".into()));
}
```

---

## Deliverables

### Milestone 3.1 Checklist

**Backend (vibes-core):**
- [ ] Add `rusqlite` dependency with bundled feature
- [ ] Create `history` module structure
- [ ] Implement `Migrator` with v001 and v002 migrations
- [ ] Implement `SqliteHistoryStore` with all CRUD operations
- [ ] Implement `MessageBuilder` for event aggregation
- [ ] Implement `HistoryService` with search/pagination/resume
- [ ] Integrate with EventBus (persist on TurnComplete)
- [ ] Unit tests for all components

**Server (vibes-server):**
- [ ] Add history routes to router
- [ ] Implement all REST endpoints
- [ ] Query parameter parsing with validation
- [ ] Add `HistoryStore` to `AppState`
- [ ] Integration tests for endpoints

**Web UI:**
- [ ] History list view with search input
- [ ] Filter controls (state, date range, tool)
- [ ] Session detail view with message list
- [ ] Pagination controls
- [ ] Resume session button
- [ ] Delete session with confirmation

**Documentation:**
- [ ] Design document (this file)
- [ ] Update PROGRESS.md

---

## Future Considerations

These are explicitly **out of scope** for milestone 3.1 but inform the design:

1. **Export to Markdown/JSON** (Phase 4): Schema supports this; add endpoint later
2. **Session archiving**: Add `archived_at` column, filter from default queries
3. **Storage limits**: Add config for max sessions/messages, implement pruning
4. **Sync across devices**: Current design is single-server; would need sync protocol

# Milestone 3.1: Chat History - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Persist session history to SQLite with full-text search, REST API, and Web UI.

**Architecture:** SQLite database stores aggregated messages (collapsed deltas). HistoryStore handles persistence, MessageBuilder aggregates streaming events, REST endpoints expose history to clients. Web UI displays searchable session list with detail views.

**Tech Stack:** rusqlite (bundled FTS5), axum routes, React Query, TypeScript

---

## Task 1: Add rusqlite dependency and create module structure

**Files:**
- Modify: `vibes-core/Cargo.toml`
- Create: `vibes-core/src/history/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Add rusqlite dependency**

Add to `vibes-core/Cargo.toml` in `[dependencies]`:

```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

**Step 2: Create history module skeleton**

Create `vibes-core/src/history/mod.rs`:

```rust
//! Chat history persistence with SQLite storage

mod error;
mod types;
mod query;
mod migrations;
mod store;
mod builder;
mod service;

pub use error::HistoryError;
pub use types::{HistoricalSession, HistoricalMessage, MessageRole, SessionSummary};
pub use query::{SessionQuery, MessageQuery, SessionListResult, MessageListResult, SortField, SortOrder};
pub use store::{HistoryStore, SqliteHistoryStore};
pub use builder::MessageBuilder;
pub use service::HistoryService;
```

**Step 3: Export history module from lib.rs**

Add to `vibes-core/src/lib.rs`:

```rust
pub mod history;
```

**Step 4: Create placeholder submodules**

Create each submodule with minimal content:

`vibes-core/src/history/error.rs`:
```rust
//! History error types
```

`vibes-core/src/history/types.rs`:
```rust
//! Core history types
```

`vibes-core/src/history/query.rs`:
```rust
//! Query parameter types
```

`vibes-core/src/history/migrations/mod.rs`:
```rust
//! Database migrations
```

`vibes-core/src/history/store.rs`:
```rust
//! History storage trait and SQLite implementation
```

`vibes-core/src/history/builder.rs`:
```rust
//! Message aggregation from streaming events
```

`vibes-core/src/history/service.rs`:
```rust
//! History business logic
```

**Step 5: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: Compiles with warnings about unused modules

**Step 6: Commit**

```bash
git add vibes-core/Cargo.toml vibes-core/src/history/ vibes-core/src/lib.rs
git commit -m "feat(history): add rusqlite dependency and module skeleton"
```

---

## Task 2: Implement HistoryError

**Files:**
- Modify: `vibes-core/src/history/error.rs`
- Modify: `vibes-core/src/error.rs`

**Step 1: Write the error type tests**

Add to `vibes-core/src/history/error.rs`:

```rust
//! History error types

use thiserror::Error;

/// Errors for chat history operations
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HistoryError::SessionNotFound("test-123".into());
        assert_eq!(err.to_string(), "Session not found: test-123");
    }

    #[test]
    fn test_database_error_conversion() {
        // rusqlite::Error is not easily constructed, so just verify the enum compiles
        let err = HistoryError::Migration("version 2 failed".into());
        assert!(err.to_string().contains("version 2 failed"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::error`
Expected: PASS (2 tests)

**Step 3: Add HistoryError to main error type**

Add to `vibes-core/src/error.rs`:

```rust
use crate::history::HistoryError;

// In the VibesError enum, add:
#[error("History error: {0}")]
History(#[from] HistoryError),
```

**Step 4: Verify compilation**

Run: `cargo check -p vibes-core`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add vibes-core/src/history/error.rs vibes-core/src/error.rs
git commit -m "feat(history): add HistoryError type with database and session variants"
```

---

## Task 3: Implement core types

**Files:**
- Modify: `vibes-core/src/history/types.rs`

**Step 1: Write MessageRole enum with tests**

Add to `vibes-core/src/history/types.rs`:

```rust
//! Core history types

use serde::{Deserialize, Serialize};

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    ToolUse,
    ToolResult,
}

impl MessageRole {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::ToolUse => "tool_use",
            Self::ToolResult => "tool_result",
        }
    }

    /// Parse from database string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "tool_use" => Some(Self::ToolUse),
            "tool_result" => Some(Self::ToolResult),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_roundtrip() {
        for role in [MessageRole::User, MessageRole::Assistant, MessageRole::ToolUse, MessageRole::ToolResult] {
            let s = role.as_str();
            let parsed = MessageRole::from_str(s);
            assert_eq!(parsed, Some(role));
        }
    }

    #[test]
    fn test_message_role_serde() {
        let role = MessageRole::ToolUse;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"tool_use\"");

        let parsed: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::types`
Expected: PASS (2 tests)

**Step 3: Add HistoricalMessage struct**

Add below MessageRole in `types.rs`:

```rust
/// Aggregated message stored in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    /// Auto-incremented database ID
    pub id: i64,
    /// Parent session ID
    pub session_id: String,
    /// Message role
    pub role: MessageRole,
    /// Message content (aggregated from deltas)
    pub content: String,
    /// Tool name for tool_use/tool_result messages
    pub tool_name: Option<String>,
    /// Tool invocation ID linking tool_use to tool_result
    pub tool_id: Option<String>,
    /// Unix timestamp (seconds)
    pub created_at: i64,
    /// Input tokens for this message
    pub input_tokens: Option<u32>,
    /// Output tokens for this message
    pub output_tokens: Option<u32>,
}

impl HistoricalMessage {
    /// Create a new user message
    pub fn user(session_id: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0, // Set by database
            session_id,
            role: MessageRole::User,
            content,
            tool_name: None,
            tool_id: None,
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(session_id: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::Assistant,
            content,
            tool_name: None,
            tool_id: None,
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a tool use message
    pub fn tool_use(session_id: String, tool_id: String, tool_name: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::ToolUse,
            content,
            tool_name: Some(tool_name),
            tool_id: Some(tool_id),
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a tool result message
    pub fn tool_result(session_id: String, tool_id: String, tool_name: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::ToolResult,
            content,
            tool_name: Some(tool_name),
            tool_id: Some(tool_id),
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }
}
```

**Step 4: Add tests for HistoricalMessage**

Add to the tests module:

```rust
    #[test]
    fn test_historical_message_user() {
        let msg = HistoricalMessage::user("sess-1".into(), "Hello".into(), 1234567890);
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.session_id, "sess-1");
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_name.is_none());
    }

    #[test]
    fn test_historical_message_tool_use() {
        let msg = HistoricalMessage::tool_use(
            "sess-1".into(),
            "tool-123".into(),
            "Read".into(),
            "{\"path\": \"/tmp\"}".into(),
            1234567890,
        );
        assert_eq!(msg.role, MessageRole::ToolUse);
        assert_eq!(msg.tool_name, Some("Read".into()));
        assert_eq!(msg.tool_id, Some("tool-123".into()));
    }
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core history::types`
Expected: PASS (4 tests)

**Step 6: Add HistoricalSession and SessionSummary**

Add below HistoricalMessage:

```rust
use crate::session::SessionState;

/// Persisted session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSession {
    /// Session UUID
    pub id: String,
    /// Human-readable name
    pub name: Option<String>,
    /// Claude's session ID for resume
    pub claude_session_id: Option<String>,
    /// Final session state
    pub state: SessionState,
    /// Unix timestamp (seconds)
    pub created_at: i64,
    /// Last activity timestamp
    pub last_accessed_at: i64,
    /// Total input tokens used
    pub total_input_tokens: u32,
    /// Total output tokens used
    pub total_output_tokens: u32,
    /// Number of messages
    pub message_count: u32,
    /// Error message if state is Failed
    pub error_message: Option<String>,
}

impl HistoricalSession {
    /// Create a new session with timestamps set to now
    pub fn new(id: String, name: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id,
            name,
            claude_session_id: None,
            state: SessionState::Idle,
            created_at: now,
            last_accessed_at: now,
            total_input_tokens: 0,
            total_output_tokens: 0,
            message_count: 0,
            error_message: None,
        }
    }
}

/// Session summary for list views (lighter than full HistoricalSession)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: Option<String>,
    pub state: SessionState,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub message_count: u32,
    pub total_tokens: u32,
    /// First ~100 chars of first user message
    pub preview: String,
}
```

**Step 7: Add tests for HistoricalSession**

Add to tests:

```rust
    #[test]
    fn test_historical_session_new() {
        let session = HistoricalSession::new("sess-123".into(), Some("Test".into()));
        assert_eq!(session.id, "sess-123");
        assert_eq!(session.name, Some("Test".into()));
        assert_eq!(session.state, SessionState::Idle);
        assert!(session.created_at > 0);
        assert_eq!(session.total_input_tokens, 0);
    }
```

**Step 8: Run tests**

Run: `cargo test -p vibes-core history::types`
Expected: PASS (5 tests)

**Step 9: Commit**

```bash
git add vibes-core/src/history/types.rs
git commit -m "feat(history): add core types - MessageRole, HistoricalMessage, HistoricalSession"
```

---

## Task 4: Implement query types

**Files:**
- Modify: `vibes-core/src/history/query.rs`

**Step 1: Write query types with defaults**

Replace `vibes-core/src/history/query.rs`:

```rust
//! Query parameter types for history search

use serde::{Deserialize, Serialize};
use crate::session::SessionState;
use super::types::{SessionSummary, HistoricalMessage};

/// Sort field for session queries
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    #[default]
    CreatedAt,
    LastAccessedAt,
    MessageCount,
    TotalTokens,
}

impl SortField {
    pub fn as_column(&self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::LastAccessedAt => "last_accessed_at",
            Self::MessageCount => "message_count",
            Self::TotalTokens => "(total_input_tokens + total_output_tokens)",
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Query parameters for listing sessions
#[derive(Debug, Clone, Default)]
pub struct SessionQuery {
    /// Full-text search across message content
    pub search: Option<String>,
    /// Filter by session name (LIKE pattern)
    pub name: Option<String>,
    /// Filter by session state
    pub state: Option<SessionState>,
    /// Filter sessions that used this tool
    pub tool: Option<String>,
    /// Minimum total tokens
    pub min_tokens: Option<u32>,
    /// Created after this timestamp
    pub after: Option<i64>,
    /// Created before this timestamp
    pub before: Option<i64>,
    /// Max results (default 20, max 100)
    pub limit: u32,
    /// Offset for pagination
    pub offset: u32,
    /// Sort field
    pub sort: SortField,
    /// Sort order
    pub order: SortOrder,
}

impl SessionQuery {
    pub fn new() -> Self {
        Self {
            limit: 20,
            ..Default::default()
        }
    }

    /// Clamp limit to valid range
    pub fn effective_limit(&self) -> u32 {
        self.limit.clamp(1, 100)
    }
}

/// Query parameters for listing messages
#[derive(Debug, Clone, Default)]
pub struct MessageQuery {
    /// Max results (default 50)
    pub limit: u32,
    /// Offset for pagination
    pub offset: u32,
    /// Filter by role
    pub role: Option<super::types::MessageRole>,
}

impl MessageQuery {
    pub fn new() -> Self {
        Self {
            limit: 50,
            ..Default::default()
        }
    }

    pub fn effective_limit(&self) -> u32 {
        self.limit.clamp(1, 500)
    }
}

/// Paginated session list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResult {
    pub sessions: Vec<SessionSummary>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

/// Paginated message list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageListResult {
    pub messages: Vec<HistoricalMessage>,
    pub total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_query_defaults() {
        let query = SessionQuery::new();
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(query.sort, SortField::CreatedAt);
        assert_eq!(query.order, SortOrder::Desc);
    }

    #[test]
    fn test_effective_limit_clamping() {
        let mut query = SessionQuery::new();
        query.limit = 0;
        assert_eq!(query.effective_limit(), 1);

        query.limit = 500;
        assert_eq!(query.effective_limit(), 100);
    }

    #[test]
    fn test_sort_field_column() {
        assert_eq!(SortField::CreatedAt.as_column(), "created_at");
        assert_eq!(SortField::TotalTokens.as_column(), "(total_input_tokens + total_output_tokens)");
    }

    #[test]
    fn test_sort_order_sql() {
        assert_eq!(SortOrder::Asc.as_sql(), "ASC");
        assert_eq!(SortOrder::Desc.as_sql(), "DESC");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::query`
Expected: PASS (4 tests)

**Step 3: Commit**

```bash
git add vibes-core/src/history/query.rs
git commit -m "feat(history): add query types with pagination and sorting"
```

---

## Task 5: Implement Migrator with v001 migration

**Files:**
- Create: `vibes-core/src/history/migrations/v001_initial.sql`
- Modify: `vibes-core/src/history/migrations/mod.rs`

**Step 1: Create v001 SQL migration file**

Create `vibes-core/src/history/migrations/v001_initial.sql`:

```sql
-- Migration v001: Initial schema for chat history

-- Session metadata
CREATE TABLE IF NOT EXISTS sessions (
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

CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_state ON sessions(state);
CREATE INDEX IF NOT EXISTS idx_sessions_name ON sessions(name);

-- Aggregated messages
CREATE TABLE IF NOT EXISTS messages (
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

CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
CREATE INDEX IF NOT EXISTS idx_messages_role ON messages(role);
CREATE INDEX IF NOT EXISTS idx_messages_tool_name ON messages(tool_name);
```

**Step 2: Implement Migrator**

Replace `vibes-core/src/history/migrations/mod.rs`:

```rust
//! Database migrations for chat history

use rusqlite::Connection;
use crate::history::HistoryError;

/// SQL for each migration version
const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
];

/// Runs database migrations
pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get current schema version
    pub fn current_version(&self) -> Result<i32, HistoryError> {
        let version: i32 = self.conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(version)
    }

    /// Set schema version
    fn set_version(&self, version: i32) -> Result<(), HistoryError> {
        self.conn.pragma_update(None, "user_version", version)?;
        Ok(())
    }

    /// Run all pending migrations
    pub fn migrate(&self) -> Result<(), HistoryError> {
        let current = self.current_version()?;
        let target = MIGRATIONS.len() as i32;

        if current >= target {
            return Ok(());
        }

        for (idx, (name, sql)) in MIGRATIONS.iter().enumerate() {
            let version = (idx + 1) as i32;
            if version > current {
                tracing::info!("Running migration {}: {}", version, name);
                self.conn.execute_batch(sql).map_err(|e| {
                    HistoryError::Migration(format!("{}: {}", name, e))
                })?;
                self.set_version(version)?;
            }
        }

        Ok(())
    }

    /// Get target version (latest migration)
    pub fn target_version(&self) -> i32 {
        MIGRATIONS.len() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_fresh_database() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);

        assert_eq!(migrator.current_version().unwrap(), 0);
        migrator.migrate().unwrap();
        assert_eq!(migrator.current_version().unwrap(), 1);
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);

        migrator.migrate().unwrap();
        let v1 = migrator.current_version().unwrap();

        migrator.migrate().unwrap();
        let v2 = migrator.current_version().unwrap();

        assert_eq!(v1, v2);
    }

    #[test]
    fn test_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        // Verify sessions table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify messages table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-core history::migrations`
Expected: PASS (3 tests)

**Step 4: Commit**

```bash
git add vibes-core/src/history/migrations/
git commit -m "feat(history): add Migrator with v001 initial schema"
```

---

## Task 6: Add v002 FTS migration

**Files:**
- Create: `vibes-core/src/history/migrations/v002_fts.sql`
- Modify: `vibes-core/src/history/migrations/mod.rs`

**Step 1: Create v002 SQL migration**

Create `vibes-core/src/history/migrations/v002_fts.sql`:

```sql
-- Migration v002: Full-text search with FTS5

-- FTS5 virtual table for message content search
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=id
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.id, old.content);
END;

CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.id, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;
```

**Step 2: Add v002 to MIGRATIONS array**

Update `vibes-core/src/history/migrations/mod.rs`:

```rust
const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_fts", include_str!("v002_fts.sql")),
];
```

**Step 3: Add FTS test**

Add to the tests module:

```rust
    #[test]
    fn test_fts_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        assert_eq!(migrator.current_version().unwrap(), 2);

        // Verify FTS table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_triggers_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        // Count triggers
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'messages_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3); // ai, ad, au
    }
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core history::migrations`
Expected: PASS (5 tests)

**Step 5: Commit**

```bash
git add vibes-core/src/history/migrations/
git commit -m "feat(history): add v002 FTS5 migration with sync triggers"
```

---

## Task 7: Implement SqliteHistoryStore - session CRUD

**Files:**
- Modify: `vibes-core/src/history/store.rs`

**Step 1: Write failing test for session save/get**

Replace `vibes-core/src/history/store.rs`:

```rust
//! History storage trait and SQLite implementation

use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;

use super::error::HistoryError;
use super::types::{HistoricalSession, HistoricalMessage, SessionSummary, MessageRole};
use super::query::{SessionQuery, MessageQuery, SessionListResult, MessageListResult};
use super::migrations::Migrator;
use crate::session::SessionState;

/// History storage trait
pub trait HistoryStore: Send + Sync {
    fn save_session(&self, session: &HistoricalSession) -> Result<(), HistoryError>;
    fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError>;
    fn update_session(&self, session: &HistoricalSession) -> Result<(), HistoryError>;
    fn delete_session(&self, id: &str) -> Result<(), HistoryError>;
    fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError>;

    fn save_message(&self, message: &HistoricalMessage) -> Result<i64, HistoryError>;
    fn get_messages(&self, session_id: &str, query: &MessageQuery) -> Result<MessageListResult, HistoryError>;

    fn update_session_stats(&self, session_id: &str, input_tokens: u32, output_tokens: u32) -> Result<(), HistoryError>;
}

/// SQLite-backed history store
pub struct SqliteHistoryStore {
    conn: Mutex<Connection>,
}

impl SqliteHistoryStore {
    /// Open or create database at path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HistoryError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self { conn: Mutex::new(conn) };
        store.init()?;
        Ok(store)
    }

    /// Open in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self, HistoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self { conn: Mutex::new(conn) };
        store.init()?;
        Ok(store)
    }

    /// Run migrations
    fn init(&self) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate()
    }

    fn state_to_str(state: &SessionState) -> &'static str {
        match state {
            SessionState::Idle => "Idle",
            SessionState::Processing => "Processing",
            SessionState::WaitingPermission => "WaitingPermission",
            SessionState::Failed => "Failed",
            SessionState::Finished => "Finished",
        }
    }

    fn str_to_state(s: &str) -> SessionState {
        match s {
            "Processing" => SessionState::Processing,
            "WaitingPermission" => SessionState::WaitingPermission,
            "Failed" => SessionState::Failed,
            "Finished" => SessionState::Finished,
            _ => SessionState::Idle,
        }
    }
}

impl HistoryStore for SqliteHistoryStore {
    fn save_session(&self, session: &HistoricalSession) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, name, claude_session_id, state, created_at, last_accessed_at, total_input_tokens, total_output_tokens, message_count, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                session.id,
                session.name,
                session.claude_session_id,
                Self::state_to_str(&session.state),
                session.created_at,
                session.last_accessed_at,
                session.total_input_tokens,
                session.total_output_tokens,
                session.message_count,
                session.error_message,
            ],
        )?;
        Ok(())
    }

    fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, claude_session_id, state, created_at, last_accessed_at,
                    total_input_tokens, total_output_tokens, message_count, error_message
             FROM sessions WHERE id = ?1"
        )?;

        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => {
                let state_str: String = row.get(3)?;
                Ok(Some(HistoricalSession {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    claude_session_id: row.get(2)?,
                    state: Self::str_to_state(&state_str),
                    created_at: row.get(4)?,
                    last_accessed_at: row.get(5)?,
                    total_input_tokens: row.get(6)?,
                    total_output_tokens: row.get(7)?,
                    message_count: row.get(8)?,
                    error_message: row.get(9)?,
                }))
            }
            None => Ok(None),
        }
    }

    fn update_session(&self, session: &HistoricalSession) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET
                name = ?2, claude_session_id = ?3, state = ?4,
                last_accessed_at = ?5, total_input_tokens = ?6,
                total_output_tokens = ?7, message_count = ?8, error_message = ?9
             WHERE id = ?1",
            rusqlite::params![
                session.id,
                session.name,
                session.claude_session_id,
                Self::state_to_str(&session.state),
                session.last_accessed_at,
                session.total_input_tokens,
                session.total_output_tokens,
                session.message_count,
                session.error_message,
            ],
        )?;
        Ok(())
    }

    fn delete_session(&self, id: &str) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
        Ok(())
    }

    fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError> {
        // Placeholder - will implement in Task 9
        Ok(SessionListResult {
            sessions: vec![],
            total: 0,
            limit: query.effective_limit(),
            offset: query.offset,
        })
    }

    fn save_message(&self, message: &HistoricalMessage) -> Result<i64, HistoryError> {
        // Placeholder - will implement in Task 8
        Ok(0)
    }

    fn get_messages(&self, _session_id: &str, query: &MessageQuery) -> Result<MessageListResult, HistoryError> {
        // Placeholder - will implement in Task 8
        Ok(MessageListResult {
            messages: vec![],
            total: 0,
        })
    }

    fn update_session_stats(&self, session_id: &str, input_tokens: u32, output_tokens: u32) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET
                total_input_tokens = total_input_tokens + ?2,
                total_output_tokens = total_output_tokens + ?3,
                last_accessed_at = strftime('%s', 'now')
             WHERE id = ?1",
            rusqlite::params![session_id, input_tokens, output_tokens],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_save_and_get() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), Some("Test Session".into()));
        store.save_session(&session).unwrap();

        let loaded = store.get_session("test-123").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-123");
        assert_eq!(loaded.name, Some("Test Session".into()));
        assert_eq!(loaded.state, SessionState::Idle);
    }

    #[test]
    fn test_session_not_found() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();
        let loaded = store.get_session("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_session_update() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let mut session = HistoricalSession::new("test-123".into(), Some("Original".into()));
        store.save_session(&session).unwrap();

        session.name = Some("Updated".into());
        session.state = SessionState::Finished;
        store.update_session(&session).unwrap();

        let loaded = store.get_session("test-123").unwrap().unwrap();
        assert_eq!(loaded.name, Some("Updated".into()));
        assert_eq!(loaded.state, SessionState::Finished);
    }

    #[test]
    fn test_session_delete() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), None);
        store.save_session(&session).unwrap();

        store.delete_session("test-123").unwrap();

        let loaded = store.get_session("test-123").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_update_session_stats() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), None);
        store.save_session(&session).unwrap();

        store.update_session_stats("test-123", 100, 200).unwrap();
        store.update_session_stats("test-123", 50, 100).unwrap();

        let loaded = store.get_session("test-123").unwrap().unwrap();
        assert_eq!(loaded.total_input_tokens, 150);
        assert_eq!(loaded.total_output_tokens, 300);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::store`
Expected: PASS (5 tests)

**Step 3: Commit**

```bash
git add vibes-core/src/history/store.rs
git commit -m "feat(history): implement SqliteHistoryStore session CRUD"
```

---

## Task 8: Implement SqliteHistoryStore - message CRUD

**Files:**
- Modify: `vibes-core/src/history/store.rs`

**Step 1: Implement save_message**

Replace the placeholder `save_message` in `store.rs`:

```rust
    fn save_message(&self, message: &HistoricalMessage) -> Result<i64, HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                message.session_id,
                message.role.as_str(),
                message.content,
                message.tool_name,
                message.tool_id,
                message.created_at,
                message.input_tokens,
                message.output_tokens,
            ],
        )?;

        // Update session message count
        conn.execute(
            "UPDATE sessions SET message_count = message_count + 1, last_accessed_at = ?2 WHERE id = ?1",
            rusqlite::params![message.session_id, message.created_at],
        )?;

        Ok(conn.last_insert_rowid())
    }
```

**Step 2: Implement get_messages**

Replace the placeholder `get_messages`:

```rust
    fn get_messages(&self, session_id: &str, query: &MessageQuery) -> Result<MessageListResult, HistoryError> {
        let conn = self.conn.lock().unwrap();

        // Get total count
        let total: u32 = if let Some(role) = &query.role {
            conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1 AND role = ?2",
                rusqlite::params![session_id, role.as_str()],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
                [session_id],
                |row| row.get(0),
            )?
        };

        // Build query
        let sql = if query.role.is_some() {
            "SELECT id, session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens
             FROM messages WHERE session_id = ?1 AND role = ?2
             ORDER BY created_at ASC LIMIT ?3 OFFSET ?4"
        } else {
            "SELECT id, session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens
             FROM messages WHERE session_id = ?1
             ORDER BY created_at ASC LIMIT ?2 OFFSET ?3"
        };

        let messages = if let Some(role) = &query.role {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(
                rusqlite::params![session_id, role.as_str(), query.effective_limit(), query.offset],
                |row| self.row_to_message(row),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(
                rusqlite::params![session_id, query.effective_limit(), query.offset],
                |row| self.row_to_message(row),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(MessageListResult { messages, total })
    }
```

**Step 3: Add helper method for row mapping**

Add to `impl SqliteHistoryStore`:

```rust
    fn row_to_message(&self, row: &rusqlite::Row) -> Result<HistoricalMessage, rusqlite::Error> {
        let role_str: String = row.get(2)?;
        Ok(HistoricalMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: MessageRole::from_str(&role_str).unwrap_or(MessageRole::User),
            content: row.get(3)?,
            tool_name: row.get(4)?,
            tool_id: row.get(5)?,
            created_at: row.get(6)?,
            input_tokens: row.get(7)?,
            output_tokens: row.get(8)?,
        })
    }
```

**Step 4: Add message tests**

Add to tests module:

```rust
    #[test]
    fn test_message_save_and_get() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        let msg = HistoricalMessage::user("sess-1".into(), "Hello".into(), 1000);
        let id = store.save_message(&msg).unwrap();
        assert!(id > 0);

        let result = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].content, "Hello");
    }

    #[test]
    fn test_message_updates_session_count() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        store.save_message(&HistoricalMessage::user("sess-1".into(), "One".into(), 1000)).unwrap();
        store.save_message(&HistoricalMessage::assistant("sess-1".into(), "Two".into(), 1001)).unwrap();

        let loaded = store.get_session("sess-1").unwrap().unwrap();
        assert_eq!(loaded.message_count, 2);
    }

    #[test]
    fn test_message_filter_by_role() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        store.save_message(&HistoricalMessage::user("sess-1".into(), "User msg".into(), 1000)).unwrap();
        store.save_message(&HistoricalMessage::assistant("sess-1".into(), "Assistant msg".into(), 1001)).unwrap();

        let mut query = MessageQuery::new();
        query.role = Some(MessageRole::User);
        let result = store.get_messages("sess-1", &query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.messages[0].role, MessageRole::User);
    }

    #[test]
    fn test_message_cascade_delete() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();
        store.save_message(&HistoricalMessage::user("sess-1".into(), "Hello".into(), 1000)).unwrap();

        store.delete_session("sess-1").unwrap();

        let result = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(result.total, 0);
    }
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core history::store`
Expected: PASS (9 tests)

**Step 6: Commit**

```bash
git add vibes-core/src/history/store.rs
git commit -m "feat(history): implement message CRUD with cascade delete"
```

---

## Task 9: Implement SqliteHistoryStore - search with FTS

**Files:**
- Modify: `vibes-core/src/history/store.rs`

**Step 1: Implement list_sessions with all filters**

Replace the placeholder `list_sessions`:

```rust
    fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError> {
        let conn = self.conn.lock().unwrap();

        // Build WHERE clauses
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = query.name {
            conditions.push(format!("s.name LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", name)));
        }

        if let Some(ref state) = query.state {
            conditions.push(format!("s.state = ?{}", params.len() + 1));
            params.push(Box::new(Self::state_to_str(state).to_string()));
        }

        if let Some(min_tokens) = query.min_tokens {
            conditions.push(format!("(s.total_input_tokens + s.total_output_tokens) >= ?{}", params.len() + 1));
            params.push(Box::new(min_tokens as i64));
        }

        if let Some(after) = query.after {
            conditions.push(format!("s.created_at >= ?{}", params.len() + 1));
            params.push(Box::new(after));
        }

        if let Some(before) = query.before {
            conditions.push(format!("s.created_at <= ?{}", params.len() + 1));
            params.push(Box::new(before));
        }

        // Tool filter requires join
        let tool_join = if let Some(ref tool) = query.tool {
            conditions.push(format!("m.tool_name = ?{}", params.len() + 1));
            params.push(Box::new(tool.clone()));
            "INNER JOIN messages m ON s.id = m.session_id"
        } else {
            ""
        };

        // FTS search requires subquery
        let fts_join = if let Some(ref search) = query.search {
            conditions.push(format!(
                "s.id IN (SELECT DISTINCT m2.session_id FROM messages m2
                 INNER JOIN messages_fts ON m2.id = messages_fts.rowid
                 WHERE messages_fts MATCH ?{})",
                params.len() + 1
            ));
            params.push(Box::new(search.clone()));
            ""
        } else {
            ""
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count query
        let count_sql = format!(
            "SELECT COUNT(DISTINCT s.id) FROM sessions s {} {} {}",
            tool_join, fts_join, where_clause
        );

        let total: u32 = {
            let mut stmt = conn.prepare(&count_sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            stmt.query_row(params_refs.as_slice(), |row| row.get(0))?
        };

        // Main query with pagination
        let order = format!("{} {}", query.sort.as_column(), query.order.as_sql());
        let select_sql = format!(
            "SELECT DISTINCT s.id, s.name, s.state, s.created_at, s.last_accessed_at,
                    s.message_count, s.total_input_tokens + s.total_output_tokens as total_tokens,
                    COALESCE((SELECT SUBSTR(content, 1, 100) FROM messages WHERE session_id = s.id ORDER BY created_at LIMIT 1), '') as preview
             FROM sessions s {} {} {}
             ORDER BY {} LIMIT ?{} OFFSET ?{}",
            tool_join, fts_join, where_clause, order, params.len() + 1, params.len() + 2
        );

        params.push(Box::new(query.effective_limit() as i64));
        params.push(Box::new(query.offset as i64));

        let sessions = {
            let mut stmt = conn.prepare(&select_sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let state_str: String = row.get(2)?;
                Ok(SessionSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state: Self::str_to_state(&state_str),
                    created_at: row.get(3)?,
                    last_accessed_at: row.get(4)?,
                    message_count: row.get(5)?,
                    total_tokens: row.get(6)?,
                    preview: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(SessionListResult {
            sessions,
            total,
            limit: query.effective_limit(),
            offset: query.offset,
        })
    }
```

**Step 2: Add search tests**

Add to tests module:

```rust
    #[test]
    fn test_list_sessions_basic() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store.save_session(&HistoricalSession::new("sess-1".into(), Some("First".into()))).unwrap();
        store.save_session(&HistoricalSession::new("sess-2".into(), Some("Second".into()))).unwrap();

        let result = store.list_sessions(&SessionQuery::new()).unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.sessions.len(), 2);
    }

    #[test]
    fn test_list_sessions_filter_by_name() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store.save_session(&HistoricalSession::new("sess-1".into(), Some("Alpha Test".into()))).unwrap();
        store.save_session(&HistoricalSession::new("sess-2".into(), Some("Beta Test".into()))).unwrap();
        store.save_session(&HistoricalSession::new("sess-3".into(), Some("Gamma".into()))).unwrap();

        let mut query = SessionQuery::new();
        query.name = Some("Test".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_list_sessions_filter_by_state() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let mut s1 = HistoricalSession::new("sess-1".into(), None);
        s1.state = SessionState::Finished;
        store.save_session(&s1).unwrap();

        let s2 = HistoricalSession::new("sess-2".into(), None);
        store.save_session(&s2).unwrap();

        let mut query = SessionQuery::new();
        query.state = Some(SessionState::Finished);
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }

    #[test]
    fn test_list_sessions_fts_search() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store.save_session(&HistoricalSession::new("sess-1".into(), None)).unwrap();
        store.save_session(&HistoricalSession::new("sess-2".into(), None)).unwrap();

        store.save_message(&HistoricalMessage::user("sess-1".into(), "How do I use Rust?".into(), 1000)).unwrap();
        store.save_message(&HistoricalMessage::user("sess-2".into(), "Hello world".into(), 1000)).unwrap();

        let mut query = SessionQuery::new();
        query.search = Some("Rust".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }

    #[test]
    fn test_list_sessions_pagination() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        for i in 0..5 {
            store.save_session(&HistoricalSession::new(format!("sess-{}", i), None)).unwrap();
        }

        let mut query = SessionQuery::new();
        query.limit = 2;
        query.offset = 0;
        let page1 = store.list_sessions(&query).unwrap();

        query.offset = 2;
        let page2 = store.list_sessions(&query).unwrap();

        assert_eq!(page1.total, 5);
        assert_eq!(page1.sessions.len(), 2);
        assert_eq!(page2.sessions.len(), 2);
        assert_ne!(page1.sessions[0].id, page2.sessions[0].id);
    }

    #[test]
    fn test_list_sessions_filter_by_tool() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store.save_session(&HistoricalSession::new("sess-1".into(), None)).unwrap();
        store.save_session(&HistoricalSession::new("sess-2".into(), None)).unwrap();

        store.save_message(&HistoricalMessage::tool_use(
            "sess-1".into(), "t1".into(), "Read".into(), "{}".into(), 1000
        )).unwrap();
        store.save_message(&HistoricalMessage::tool_use(
            "sess-2".into(), "t2".into(), "Write".into(), "{}".into(), 1000
        )).unwrap();

        let mut query = SessionQuery::new();
        query.tool = Some("Read".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }
```

**Step 3: Run tests**

Run: `cargo test -p vibes-core history::store`
Expected: PASS (15 tests)

**Step 4: Commit**

```bash
git add vibes-core/src/history/store.rs
git commit -m "feat(history): implement session search with FTS5 and filters"
```

---

## Task 10: Implement MessageBuilder

**Files:**
- Modify: `vibes-core/src/history/builder.rs`

**Step 1: Implement MessageBuilder**

Replace `vibes-core/src/history/builder.rs`:

```rust
//! Message aggregation from streaming events

use std::collections::HashMap;
use crate::events::ClaudeEvent;
use super::types::{HistoricalMessage, MessageRole};

/// Aggregates streaming events into complete messages
pub struct MessageBuilder {
    session_id: String,
    /// Accumulated text for current assistant turn
    current_text: String,
    /// Active tool calls being built
    active_tools: HashMap<String, ToolBuilder>,
    /// Completed messages ready to persist
    pending_messages: Vec<HistoricalMessage>,
    /// Current timestamp (updated on each event)
    current_time: i64,
}

struct ToolBuilder {
    name: String,
    input: String,
    started_at: i64,
}

impl MessageBuilder {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_text: String::new(),
            active_tools: HashMap::new(),
            pending_messages: Vec::new(),
            current_time: now(),
        }
    }

    /// Process a Claude event
    pub fn process_event(&mut self, event: &ClaudeEvent) {
        self.current_time = now();

        match event {
            ClaudeEvent::TextDelta { text } => {
                self.current_text.push_str(text);
            }

            ClaudeEvent::ToolUseStart { id, name } => {
                self.active_tools.insert(
                    id.clone(),
                    ToolBuilder {
                        name: name.clone(),
                        input: String::new(),
                        started_at: self.current_time,
                    },
                );
            }

            ClaudeEvent::ToolInputDelta { id, delta } => {
                if let Some(tool) = self.active_tools.get_mut(id) {
                    tool.input.push_str(delta);
                }
            }

            ClaudeEvent::ToolResult { id, output, is_error: _ } => {
                // Finalize tool_use message
                if let Some(tool) = self.active_tools.remove(id) {
                    self.pending_messages.push(HistoricalMessage::tool_use(
                        self.session_id.clone(),
                        id.clone(),
                        tool.name.clone(),
                        tool.input,
                        tool.started_at,
                    ));

                    // Add tool_result message
                    self.pending_messages.push(HistoricalMessage::tool_result(
                        self.session_id.clone(),
                        id.clone(),
                        tool.name,
                        output.clone(),
                        self.current_time,
                    ));
                }
            }

            ClaudeEvent::TurnComplete { usage: _ } => {
                // Finalize assistant message if there's accumulated text
                if !self.current_text.is_empty() {
                    self.pending_messages.push(HistoricalMessage::assistant(
                        self.session_id.clone(),
                        std::mem::take(&mut self.current_text),
                        self.current_time,
                    ));
                }
            }

            _ => {}
        }
    }

    /// Add a user input message
    pub fn add_user_input(&mut self, content: String) {
        self.pending_messages.push(HistoricalMessage::user(
            self.session_id.clone(),
            content,
            now(),
        ));
    }

    /// Drain all pending messages
    pub fn take_pending(&mut self) -> Vec<HistoricalMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Check if there are pending messages
    pub fn has_pending(&self) -> bool {
        !self.pending_messages.is_empty()
    }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::TokenUsage;

    #[test]
    fn test_user_input() {
        let mut builder = MessageBuilder::new("sess-1".into());
        builder.add_user_input("Hello".into());

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_text_aggregation() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::TextDelta { text: "Hello ".into() });
        builder.process_event(&ClaudeEvent::TextDelta { text: "world!".into() });
        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: TokenUsage { input_tokens: 10, output_tokens: 5 }
        });

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::Assistant);
        assert_eq!(messages[0].content, "Hello world!");
    }

    #[test]
    fn test_tool_use_flow() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::ToolUseStart {
            id: "tool-1".into(),
            name: "Read".into()
        });
        builder.process_event(&ClaudeEvent::ToolInputDelta {
            id: "tool-1".into(),
            delta: "{\"path\":".into()
        });
        builder.process_event(&ClaudeEvent::ToolInputDelta {
            id: "tool-1".into(),
            delta: "\"/tmp\"}".into()
        });
        builder.process_event(&ClaudeEvent::ToolResult {
            id: "tool-1".into(),
            output: "file contents".into(),
            is_error: false,
        });

        let messages = builder.take_pending();
        assert_eq!(messages.len(), 2);

        assert_eq!(messages[0].role, MessageRole::ToolUse);
        assert_eq!(messages[0].tool_name, Some("Read".into()));
        assert_eq!(messages[0].content, "{\"path\":\"/tmp\"}");

        assert_eq!(messages[1].role, MessageRole::ToolResult);
        assert_eq!(messages[1].content, "file contents");
    }

    #[test]
    fn test_mixed_text_and_tools() {
        let mut builder = MessageBuilder::new("sess-1".into());

        // Text before tool
        builder.process_event(&ClaudeEvent::TextDelta { text: "Let me check ".into() });

        // Tool use
        builder.process_event(&ClaudeEvent::ToolUseStart {
            id: "t1".into(),
            name: "Read".into()
        });
        builder.process_event(&ClaudeEvent::ToolResult {
            id: "t1".into(),
            output: "data".into(),
            is_error: false,
        });

        // More text
        builder.process_event(&ClaudeEvent::TextDelta { text: "that file.".into() });
        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: TokenUsage { input_tokens: 10, output_tokens: 5 }
        });

        let messages = builder.take_pending();
        // tool_use, tool_result, assistant text
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[2].content, "Let me check that file.");
    }

    #[test]
    fn test_empty_turn_no_message() {
        let mut builder = MessageBuilder::new("sess-1".into());

        builder.process_event(&ClaudeEvent::TurnComplete {
            usage: TokenUsage { input_tokens: 0, output_tokens: 0 }
        });

        let messages = builder.take_pending();
        assert!(messages.is_empty());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::builder`
Expected: PASS (5 tests)

**Step 3: Commit**

```bash
git add vibes-core/src/history/builder.rs
git commit -m "feat(history): implement MessageBuilder for event aggregation"
```

---

## Task 11: Implement HistoryService

**Files:**
- Modify: `vibes-core/src/history/service.rs`

**Step 1: Implement HistoryService**

Replace `vibes-core/src/history/service.rs`:

```rust
//! History business logic

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use super::store::HistoryStore;
use super::builder::MessageBuilder;
use super::types::{HistoricalSession, HistoricalMessage};
use super::query::{SessionQuery, MessageQuery, SessionListResult, MessageListResult};
use super::error::HistoryError;
use crate::events::{VibesEvent, ClaudeEvent};

/// Service for managing chat history
pub struct HistoryService<S: HistoryStore> {
    store: Arc<S>,
    /// Active message builders per session
    builders: RwLock<HashMap<String, MessageBuilder>>,
}

impl<S: HistoryStore> HistoryService<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self {
            store,
            builders: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new session in history
    pub async fn create_session(&self, id: String, name: Option<String>) -> Result<(), HistoryError> {
        let session = HistoricalSession::new(id.clone(), name);
        self.store.save_session(&session)?;

        // Initialize message builder
        let mut builders = self.builders.write().await;
        builders.insert(id.clone(), MessageBuilder::new(id));

        Ok(())
    }

    /// Process an event for history persistence
    pub async fn process_event(&self, event: &VibesEvent) -> Result<(), HistoryError> {
        match event {
            VibesEvent::SessionCreated { session_id, name } => {
                self.create_session(session_id.clone(), name.clone()).await?;
            }

            VibesEvent::UserInput { session_id, content } => {
                let mut builders = self.builders.write().await;
                if let Some(builder) = builders.get_mut(session_id) {
                    builder.add_user_input(content.clone());
                    self.persist_pending(session_id, builder)?;
                }
            }

            VibesEvent::Claude { session_id, event } => {
                let mut builders = self.builders.write().await;
                if let Some(builder) = builders.get_mut(session_id) {
                    builder.process_event(event);

                    // Persist on turn complete
                    if matches!(event, ClaudeEvent::TurnComplete { .. }) {
                        self.persist_pending(session_id, builder)?;

                        // Update token stats
                        if let ClaudeEvent::TurnComplete { usage } = event {
                            self.store.update_session_stats(
                                session_id,
                                usage.input_tokens,
                                usage.output_tokens,
                            )?;
                        }
                    }
                }
            }

            VibesEvent::SessionStateChanged { session_id, state } => {
                if let Some(mut session) = self.store.get_session(session_id)? {
                    session.state = state.clone();
                    self.store.update_session(&session)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn persist_pending(&self, session_id: &str, builder: &mut MessageBuilder) -> Result<(), HistoryError> {
        for message in builder.take_pending() {
            self.store.save_message(&message)?;
        }
        Ok(())
    }

    /// List sessions with filtering
    pub fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError> {
        self.store.list_sessions(query)
    }

    /// Get a specific session
    pub fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError> {
        self.store.get_session(id)
    }

    /// Get messages for a session
    pub fn get_messages(&self, session_id: &str, query: &MessageQuery) -> Result<MessageListResult, HistoryError> {
        self.store.get_messages(session_id, query)
    }

    /// Delete a session
    pub fn delete_session(&self, id: &str) -> Result<(), HistoryError> {
        self.store.delete_session(id)
    }

    /// Get Claude session ID for resume
    pub fn get_claude_session_id(&self, id: &str) -> Result<Option<String>, HistoryError> {
        Ok(self.store.get_session(id)?.and_then(|s| s.claude_session_id))
    }

    /// Update Claude session ID
    pub fn set_claude_session_id(&self, id: &str, claude_id: String) -> Result<(), HistoryError> {
        if let Some(mut session) = self.store.get_session(id)? {
            session.claude_session_id = Some(claude_id);
            self.store.update_session(&session)?;
        }
        Ok(())
    }

    /// Clean up builder for ended session
    pub async fn end_session(&self, session_id: &str) {
        let mut builders = self.builders.write().await;
        builders.remove(session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::TokenUsage;
    use crate::session::SessionState;

    fn create_test_service() -> HistoryService<super::super::store::SqliteHistoryStore> {
        let store = super::super::store::SqliteHistoryStore::open_in_memory().unwrap();
        HistoryService::new(Arc::new(store))
    }

    #[tokio::test]
    async fn test_create_session() {
        let service = create_test_service();

        service.create_session("sess-1".into(), Some("Test".into())).await.unwrap();

        let session = service.get_session("sess-1").unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().name, Some("Test".into()));
    }

    #[tokio::test]
    async fn test_process_user_input() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service.process_event(&VibesEvent::UserInput {
            session_id: "sess-1".into(),
            content: "Hello".into(),
        }).await.unwrap();

        let messages = service.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(messages.total, 1);
        assert_eq!(messages.messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_process_claude_turn() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service.process_event(&VibesEvent::Claude {
            session_id: "sess-1".into(),
            event: ClaudeEvent::TextDelta { text: "Hello!".into() },
        }).await.unwrap();

        service.process_event(&VibesEvent::Claude {
            session_id: "sess-1".into(),
            event: ClaudeEvent::TurnComplete {
                usage: TokenUsage { input_tokens: 10, output_tokens: 5 }
            },
        }).await.unwrap();

        let messages = service.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(messages.total, 1);
        assert_eq!(messages.messages[0].content, "Hello!");

        let session = service.get_session("sess-1").unwrap().unwrap();
        assert_eq!(session.total_input_tokens, 10);
        assert_eq!(session.total_output_tokens, 5);
    }

    #[tokio::test]
    async fn test_session_state_update() {
        let service = create_test_service();
        service.create_session("sess-1".into(), None).await.unwrap();

        service.process_event(&VibesEvent::SessionStateChanged {
            session_id: "sess-1".into(),
            state: SessionState::Finished,
        }).await.unwrap();

        let session = service.get_session("sess-1").unwrap().unwrap();
        assert_eq!(session.state, SessionState::Finished);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p vibes-core history::service`
Expected: PASS (4 tests)

**Step 3: Commit**

```bash
git add vibes-core/src/history/service.rs
git commit -m "feat(history): implement HistoryService with event processing"
```

---

## Task 12: Add HistoryStore to AppState

**Files:**
- Modify: `vibes-server/Cargo.toml`
- Modify: `vibes-server/src/state.rs`

**Step 1: Verify vibes-core re-exports history module**

The `vibes-core` should already export the history module. Check that `vibes-server/Cargo.toml` has vibes-core as a dependency.

**Step 2: Add history to AppState**

Modify `vibes-server/src/state.rs` to add:

```rust
use vibes_core::history::{HistoryService, SqliteHistoryStore};

// Add to AppState struct:
pub history: Option<Arc<HistoryService<SqliteHistoryStore>>>,

// Add builder method:
pub fn with_history(mut self, service: Arc<HistoryService<SqliteHistoryStore>>) -> Self {
    self.history = Some(service);
    self
}
```

Initialize history field to `None` in `AppState::new()`.

**Step 3: Initialize history in server startup**

Modify `vibes-server/src/lib.rs` or wherever the server is initialized:

```rust
use vibes_core::history::{HistoryService, SqliteHistoryStore};

// In server initialization:
let history_path = config_dir.join("history.db");
let history_store = SqliteHistoryStore::open(&history_path)?;
let history_service = Arc::new(HistoryService::new(Arc::new(history_store)));

let state = AppState::new()
    .with_history(history_service.clone());
```

**Step 4: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add vibes-server/src/state.rs vibes-server/src/lib.rs
git commit -m "feat(history): add HistoryService to AppState"
```

---

## Task 13: Implement REST endpoints

**Files:**
- Create: `vibes-server/src/http/history.rs`
- Modify: `vibes-server/src/http/mod.rs`

**Step 1: Create history routes file**

Create `vibes-server/src/http/history.rs`:

```rust
//! History REST API endpoints

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use vibes_core::history::{
    SessionQuery, MessageQuery, SortField, SortOrder,
    SessionListResult, MessageListResult, HistoricalSession,
};
use vibes_core::session::SessionState;

use crate::state::AppState;

/// Query params for session list
#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub q: Option<String>,
    pub name: Option<String>,
    pub state: Option<String>,
    pub tool: Option<String>,
    pub min_tokens: Option<u32>,
    pub after: Option<i64>,
    pub before: Option<i64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl From<ListSessionsQuery> for SessionQuery {
    fn from(q: ListSessionsQuery) -> Self {
        Self {
            search: q.q,
            name: q.name,
            state: q.state.and_then(|s| match s.as_str() {
                "Idle" => Some(SessionState::Idle),
                "Processing" => Some(SessionState::Processing),
                "WaitingPermission" => Some(SessionState::WaitingPermission),
                "Failed" => Some(SessionState::Failed),
                "Finished" => Some(SessionState::Finished),
                _ => None,
            }),
            tool: q.tool,
            min_tokens: q.min_tokens,
            after: q.after,
            before: q.before,
            limit: q.limit.unwrap_or(20),
            offset: q.offset.unwrap_or(0),
            sort: q.sort.map(|s| match s.as_str() {
                "last_accessed_at" => SortField::LastAccessedAt,
                "message_count" => SortField::MessageCount,
                "total_tokens" => SortField::TotalTokens,
                _ => SortField::CreatedAt,
            }).unwrap_or_default(),
            order: q.order.map(|o| match o.as_str() {
                "asc" => SortOrder::Asc,
                _ => SortOrder::Desc,
            }).unwrap_or_default(),
        }
    }
}

/// Query params for message list
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub role: Option<String>,
}

impl From<ListMessagesQuery> for MessageQuery {
    fn from(q: ListMessagesQuery) -> Self {
        use vibes_core::history::MessageRole;
        Self {
            limit: q.limit.unwrap_or(50),
            offset: q.offset.unwrap_or(0),
            role: q.role.and_then(|r| MessageRole::from_str(&r)),
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// GET /api/history/sessions
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        ).into_response();
    };

    match history.list_sessions(&query.into()) {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        ).into_response(),
    }
}

/// GET /api/history/sessions/:id
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        ).into_response();
    };

    match history.get_session(&id) {
        Ok(Some(session)) => Json(session).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session not found: {}", id),
                code: "NOT_FOUND".into(),
            }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        ).into_response(),
    }
}

/// GET /api/history/sessions/:id/messages
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        ).into_response();
    };

    match history.get_messages(&id, &query.into()) {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        ).into_response(),
    }
}

#[derive(Serialize)]
pub struct ResumeResponse {
    pub session_id: String,
    pub claude_session_id: Option<String>,
}

/// POST /api/history/sessions/:id/resume
pub async fn resume_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        ).into_response();
    };

    // Get the Claude session ID from history
    match history.get_claude_session_id(&id) {
        Ok(Some(claude_id)) => {
            // Return the Claude session ID for the client to use with --resume
            Json(ResumeResponse {
                session_id: id,
                claude_session_id: Some(claude_id),
            }).into_response()
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Session has no Claude session ID for resume".into(),
                code: "NOT_RESUMABLE".into(),
            }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        ).into_response(),
    }
}

/// DELETE /api/history/sessions/:id
pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    match history.delete_session(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        ).into_response(),
    }
}
```

**Step 2: Register routes**

Add to `vibes-server/src/http/mod.rs`:

```rust
mod history;

// In create_router function, add:
.route("/api/history/sessions", get(history::list_sessions))
.route("/api/history/sessions/:id", get(history::get_session))
.route("/api/history/sessions/:id/messages", get(history::get_messages))
.route("/api/history/sessions/:id/resume", post(history::resume_session))
.route("/api/history/sessions/:id", delete(history::delete_session))
```

**Step 3: Verify compilation**

Run: `cargo check -p vibes-server`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add vibes-server/src/http/history.rs vibes-server/src/http/mod.rs
git commit -m "feat(history): add REST API endpoints for history"
```

---

## Task 14: Web UI - API client functions

**Files:**
- Create: `web-ui/src/api/history.ts`

**Step 1: Create TypeScript types and API client**

Create `web-ui/src/api/history.ts`:

```typescript
// History API types and client functions

export interface SessionSummary {
  id: string;
  name: string | null;
  state: 'Idle' | 'Processing' | 'WaitingPermission' | 'Failed' | 'Finished';
  created_at: number;
  last_accessed_at: number;
  message_count: number;
  total_tokens: number;
  preview: string;
}

export interface SessionListResult {
  sessions: SessionSummary[];
  total: number;
  limit: number;
  offset: number;
}

export interface HistoricalSession {
  id: string;
  name: string | null;
  claude_session_id: string | null;
  state: string;
  created_at: number;
  last_accessed_at: number;
  total_input_tokens: number;
  total_output_tokens: number;
  message_count: number;
  error_message: string | null;
}

export interface HistoricalMessage {
  id: number;
  session_id: string;
  role: 'user' | 'assistant' | 'tool_use' | 'tool_result';
  content: string;
  tool_name: string | null;
  tool_id: string | null;
  created_at: number;
  input_tokens: number | null;
  output_tokens: number | null;
}

export interface MessageListResult {
  messages: HistoricalMessage[];
  total: number;
}

export interface SessionQueryParams {
  q?: string;
  name?: string;
  state?: string;
  tool?: string;
  min_tokens?: number;
  after?: number;
  before?: number;
  limit?: number;
  offset?: number;
  sort?: 'created_at' | 'last_accessed_at' | 'message_count' | 'total_tokens';
  order?: 'asc' | 'desc';
}

export interface MessageQueryParams {
  limit?: number;
  offset?: number;
  role?: 'user' | 'assistant' | 'tool_use' | 'tool_result';
}

export interface ResumeResponse {
  session_id: string;
  claude_session_id: string | null;
}

function buildQueryString(params: Record<string, unknown>): string {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      searchParams.append(key, String(value));
    }
  }
  const qs = searchParams.toString();
  return qs ? `?${qs}` : '';
}

export async function listSessions(params: SessionQueryParams = {}): Promise<SessionListResult> {
  const response = await fetch(`/api/history/sessions${buildQueryString(params)}`);
  if (!response.ok) {
    throw new Error(`Failed to list sessions: ${response.statusText}`);
  }
  return response.json();
}

export async function getSession(id: string): Promise<HistoricalSession> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}`);
  if (!response.ok) {
    throw new Error(`Failed to get session: ${response.statusText}`);
  }
  return response.json();
}

export async function getSessionMessages(
  sessionId: string,
  params: MessageQueryParams = {}
): Promise<MessageListResult> {
  const response = await fetch(
    `/api/history/sessions/${encodeURIComponent(sessionId)}/messages${buildQueryString(params)}`
  );
  if (!response.ok) {
    throw new Error(`Failed to get messages: ${response.statusText}`);
  }
  return response.json();
}

export async function resumeSession(id: string): Promise<ResumeResponse> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}/resume`, {
    method: 'POST',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || 'Failed to resume session');
  }
  return response.json();
}

export async function deleteSession(id: string): Promise<void> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error(`Failed to delete session: ${response.statusText}`);
  }
}
```

**Step 2: Verify TypeScript compilation**

Run: `cd web-ui && npm run typecheck` (or equivalent)
Expected: No type errors

**Step 3: Commit**

```bash
git add web-ui/src/api/history.ts
git commit -m "feat(web-ui): add history API client functions"
```

---

## Task 15: Web UI - HistoryList component

**Files:**
- Create: `web-ui/src/components/HistoryList.tsx`
- Create: `web-ui/src/components/HistorySearch.tsx`

**Step 1: Create HistorySearch component**

Create `web-ui/src/components/HistorySearch.tsx`:

```tsx
import { useState } from 'react';

export interface SearchFilters {
  q: string;
  state: string;
  sort: string;
  order: string;
}

interface HistorySearchProps {
  onSearch: (filters: SearchFilters) => void;
  isLoading?: boolean;
}

export function HistorySearch({ onSearch, isLoading }: HistorySearchProps) {
  const [q, setQ] = useState('');
  const [state, setState] = useState('');
  const [sort, setSort] = useState('created_at');
  const [order, setOrder] = useState('desc');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSearch({ q, state, sort, order });
  };

  return (
    <form onSubmit={handleSubmit} className="history-search">
      <input
        type="text"
        placeholder="Search messages..."
        value={q}
        onChange={(e) => setQ(e.target.value)}
        className="search-input"
      />

      <select value={state} onChange={(e) => setState(e.target.value)}>
        <option value="">All states</option>
        <option value="Finished">Finished</option>
        <option value="Failed">Failed</option>
        <option value="Idle">Idle</option>
      </select>

      <select value={sort} onChange={(e) => setSort(e.target.value)}>
        <option value="created_at">Created</option>
        <option value="last_accessed_at">Last Active</option>
        <option value="message_count">Messages</option>
        <option value="total_tokens">Tokens</option>
      </select>

      <select value={order} onChange={(e) => setOrder(e.target.value)}>
        <option value="desc">Newest</option>
        <option value="asc">Oldest</option>
      </select>

      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Searching...' : 'Search'}
      </button>
    </form>
  );
}
```

**Step 2: Create HistoryList component**

Create `web-ui/src/components/HistoryList.tsx`:

```tsx
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { listSessions, deleteSession, SessionSummary } from '../api/history';
import { HistorySearch, SearchFilters } from './HistorySearch';

interface HistoryListProps {
  onSelectSession: (sessionId: string) => void;
}

export function HistoryList({ onSelectSession }: HistoryListProps) {
  const [filters, setFilters] = useState<SearchFilters>({
    q: '',
    state: '',
    sort: 'created_at',
    order: 'desc',
  });
  const [page, setPage] = useState(0);
  const limit = 20;

  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['history-sessions', filters, page],
    queryFn: () => listSessions({
      q: filters.q || undefined,
      state: filters.state || undefined,
      sort: filters.sort as any,
      order: filters.order as any,
      limit,
      offset: page * limit,
    }),
  });

  const handleSearch = (newFilters: SearchFilters) => {
    setFilters(newFilters);
    setPage(0);
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm('Delete this session?')) {
      await deleteSession(id);
      refetch();
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const formatTokens = (tokens: number) => {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
  };

  if (error) {
    return <div className="error">Error loading history: {error.message}</div>;
  }

  const totalPages = data ? Math.ceil(data.total / limit) : 0;

  return (
    <div className="history-list">
      <HistorySearch onSearch={handleSearch} isLoading={isLoading} />

      {isLoading ? (
        <div className="loading">Loading...</div>
      ) : data?.sessions.length === 0 ? (
        <div className="empty">No sessions found</div>
      ) : (
        <>
          <ul className="session-list">
            {data?.sessions.map((session: SessionSummary) => (
              <li
                key={session.id}
                className={`session-item state-${session.state.toLowerCase()}`}
                onClick={() => onSelectSession(session.id)}
              >
                <div className="session-header">
                  <span className="session-name">
                    {session.name || 'Unnamed Session'}
                  </span>
                  <span className={`session-state ${session.state.toLowerCase()}`}>
                    {session.state}
                  </span>
                </div>
                <div className="session-preview">{session.preview}</div>
                <div className="session-meta">
                  <span>{formatDate(session.created_at)}</span>
                  <span>{session.message_count} messages</span>
                  <span>{formatTokens(session.total_tokens)} tokens</span>
                  <button
                    className="delete-btn"
                    onClick={(e) => handleDelete(session.id, e)}
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>

          {totalPages > 1 && (
            <div className="pagination">
              <button
                disabled={page === 0}
                onClick={() => setPage(p => p - 1)}
              >
                Previous
              </button>
              <span>Page {page + 1} of {totalPages}</span>
              <button
                disabled={page >= totalPages - 1}
                onClick={() => setPage(p => p + 1)}
              >
                Next
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
```

**Step 3: Verify compilation**

Run: `cd web-ui && npm run typecheck`
Expected: No type errors

**Step 4: Commit**

```bash
git add web-ui/src/components/HistoryList.tsx web-ui/src/components/HistorySearch.tsx
git commit -m "feat(web-ui): add HistoryList and HistorySearch components"
```

---

## Task 16: Web UI - HistoryDetail component

**Files:**
- Create: `web-ui/src/components/HistoryDetail.tsx`

**Step 1: Create HistoryDetail component**

Create `web-ui/src/components/HistoryDetail.tsx`:

```tsx
import { useQuery } from '@tanstack/react-query';
import { getSession, getSessionMessages, resumeSession, HistoricalMessage } from '../api/history';
import { useState } from 'react';

interface HistoryDetailProps {
  sessionId: string;
  onBack: () => void;
  onResume?: (claudeSessionId: string) => void;
}

export function HistoryDetail({ sessionId, onBack, onResume }: HistoryDetailProps) {
  const [resumeError, setResumeError] = useState<string | null>(null);
  const [isResuming, setIsResuming] = useState(false);

  const { data: session, isLoading: sessionLoading } = useQuery({
    queryKey: ['history-session', sessionId],
    queryFn: () => getSession(sessionId),
  });

  const { data: messages, isLoading: messagesLoading } = useQuery({
    queryKey: ['history-messages', sessionId],
    queryFn: () => getSessionMessages(sessionId, { limit: 500 }),
  });

  const handleResume = async () => {
    setResumeError(null);
    setIsResuming(true);
    try {
      const result = await resumeSession(sessionId);
      if (result.claude_session_id && onResume) {
        onResume(result.claude_session_id);
      }
    } catch (e) {
      setResumeError(e instanceof Error ? e.message : 'Failed to resume');
    } finally {
      setIsResuming(false);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const renderMessage = (msg: HistoricalMessage) => {
    const roleClass = msg.role.replace('_', '-');
    return (
      <div key={msg.id} className={`message message-${roleClass}`}>
        <div className="message-header">
          <span className="message-role">{msg.role}</span>
          {msg.tool_name && (
            <span className="message-tool">{msg.tool_name}</span>
          )}
          <span className="message-time">{formatDate(msg.created_at)}</span>
        </div>
        <div className="message-content">
          {msg.role === 'tool_use' || msg.role === 'tool_result' ? (
            <pre>{msg.content}</pre>
          ) : (
            <p>{msg.content}</p>
          )}
        </div>
      </div>
    );
  };

  if (sessionLoading) {
    return <div className="loading">Loading session...</div>;
  }

  if (!session) {
    return <div className="error">Session not found</div>;
  }

  return (
    <div className="history-detail">
      <div className="detail-header">
        <button onClick={onBack} className="back-btn">
          &larr; Back to History
        </button>

        <h2>{session.name || 'Unnamed Session'}</h2>

        <div className="session-info">
          <span className={`state ${session.state.toLowerCase()}`}>
            {session.state}
          </span>
          <span>Created: {formatDate(session.created_at)}</span>
          <span>Messages: {session.message_count}</span>
          <span>
            Tokens: {session.total_input_tokens} in / {session.total_output_tokens} out
          </span>
        </div>

        {session.claude_session_id && onResume && (
          <div className="resume-section">
            <button
              onClick={handleResume}
              disabled={isResuming}
              className="resume-btn"
            >
              {isResuming ? 'Resuming...' : 'Resume Session'}
            </button>
            {resumeError && (
              <div className="resume-error">{resumeError}</div>
            )}
          </div>
        )}

        {session.error_message && (
          <div className="error-message">
            Error: {session.error_message}
          </div>
        )}
      </div>

      <div className="messages-container">
        {messagesLoading ? (
          <div className="loading">Loading messages...</div>
        ) : messages?.messages.length === 0 ? (
          <div className="empty">No messages</div>
        ) : (
          <div className="messages-list">
            {messages?.messages.map(renderMessage)}
          </div>
        )}
      </div>
    </div>
  );
}
```

**Step 2: Verify compilation**

Run: `cd web-ui && npm run typecheck`
Expected: No type errors

**Step 3: Commit**

```bash
git add web-ui/src/components/HistoryDetail.tsx
git commit -m "feat(web-ui): add HistoryDetail component with resume support"
```

---

## Task 17: Wire up history page route

**Files:**
- Create or modify: `web-ui/src/routes/history.tsx` (or integrate into existing routing)

**Step 1: Create history page**

Create `web-ui/src/routes/history.tsx`:

```tsx
import { useState } from 'react';
import { HistoryList } from '../components/HistoryList';
import { HistoryDetail } from '../components/HistoryDetail';

export function HistoryPage() {
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

  const handleResume = (claudeSessionId: string) => {
    // Navigate to active session or trigger resume flow
    console.log('Resume with Claude session:', claudeSessionId);
    // TODO: Integrate with session management
  };

  return (
    <div className="history-page">
      {selectedSessionId ? (
        <HistoryDetail
          sessionId={selectedSessionId}
          onBack={() => setSelectedSessionId(null)}
          onResume={handleResume}
        />
      ) : (
        <HistoryList onSelectSession={setSelectedSessionId} />
      )}
    </div>
  );
}
```

**Step 2: Add route to router**

Add to the app router configuration:

```tsx
// In routes configuration
{
  path: '/history',
  element: <HistoryPage />,
}
```

**Step 3: Add navigation link**

Add a link to history in the navigation:

```tsx
<Link to="/history">History</Link>
```

**Step 4: Verify compilation and navigation**

Run: `cd web-ui && npm run dev`
Expected: Can navigate to /history and see the history list

**Step 5: Commit**

```bash
git add web-ui/src/routes/history.tsx web-ui/src/App.tsx
git commit -m "feat(web-ui): add history page with routing"
```

---

## Task 18: Integration - Connect HistoryService to EventBus

**Files:**
- Modify: `vibes-server/src/lib.rs` or main server initialization

**Step 1: Subscribe HistoryService to events**

In server startup, after creating the history service:

```rust
// Subscribe history service to event bus
let history_service = history_service.clone();
let mut event_rx = state.event_bus.subscribe();

tokio::spawn(async move {
    loop {
        match event_rx.recv().await {
            Ok((_seq, event)) => {
                if let Err(e) = history_service.process_event(&event).await {
                    tracing::error!("History persistence error: {}", e);
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("History service lagged by {} events", n);
            }
        }
    }
});
```

**Step 2: Verify the integration**

Run: `cargo test -p vibes-server`
Expected: All tests pass

**Step 3: Commit**

```bash
git add vibes-server/src/lib.rs
git commit -m "feat(history): integrate HistoryService with EventBus"
```

---

## Task 19: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Mark milestone 3.1 as complete**

Update `docs/PROGRESS.md`:

```markdown
### Milestone 3.1: Chat History
- [x] Persistent session history storage (SQLite)
- [x] Session search and filtering (FTS5)
- [x] Replay previous sessions from any client
- [x] History pagination for large session counts
```

Update the Quick Links table:

```markdown
| 3.1 Chat History | Complete | [design](plans/08-chat-history/design.md) | [implementation](plans/08-chat-history/implementation.md) |
```

Add changelog entry:

```markdown
| 2025-XX-XX | Milestone 3.1 (Chat History) complete - SQLite storage, FTS5 search, REST API, Web UI |
```

**Step 2: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: mark milestone 3.1 (Chat History) as complete"
```

---

## Summary

This implementation plan covers 19 tasks:

| Task | Component | Description |
|------|-----------|-------------|
| 1 | Setup | Add rusqlite, create module structure |
| 2 | Error | HistoryError type |
| 3 | Types | MessageRole, HistoricalMessage, HistoricalSession |
| 4 | Query | SessionQuery, MessageQuery, results |
| 5 | Migrations | Migrator with v001 |
| 6 | Migrations | v002 FTS5 |
| 7 | Store | Session CRUD |
| 8 | Store | Message CRUD |
| 9 | Store | Search with FTS |
| 10 | Builder | MessageBuilder event aggregation |
| 11 | Service | HistoryService |
| 12 | Server | Add to AppState |
| 13 | Server | REST endpoints |
| 14 | Web UI | API client |
| 15 | Web UI | HistoryList + Search |
| 16 | Web UI | HistoryDetail |
| 17 | Web UI | History page route |
| 18 | Integration | EventBus connection |
| 19 | Docs | Update PROGRESS.md |

---

Plan complete and saved to `docs/plans/08-chat-history/implementation.md`.

**Two execution options:**

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session in worktree with executing-plans, batch execution with checkpoints

**Which approach?**

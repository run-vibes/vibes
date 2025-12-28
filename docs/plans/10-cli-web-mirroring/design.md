# Milestone 3.3: CLI ↔ Web Mirroring - Design Document

> Real-time bidirectional sync between CLI and Web UI with input attribution and late-joiner catch-up.

## Overview

CLI ↔ Web Mirroring enables true multi-client collaboration on Claude sessions. Any connected client (CLI or Web UI) can send input, all clients see updates in real-time, and late-joiners catch up with full session history.

This milestone builds on the multi-session infrastructure from milestone 3.2, adding source attribution to events and a paginated catch-up protocol.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Input attribution | Client type only (`cli`, `web_ui`) | Simple, no identity requirements |
| Catch-up mechanism | Enhanced Subscribe with `catch_up` flag | Single round-trip, no race conditions |
| CLI remote display | Inline with `[Web UI]:` prefix | Flows naturally in conversation |
| History persistence | Add `source` column to messages | Enables analytics, clean schema |
| Permission attribution | Skip for MVP | Keep scope focused |
| Catch-up pagination | 50 events per page with `has_more` | Keeps responses fast |
| CLI input history | In-memory only | Simple, resets on exit |

---

## ADR: Late-Joiner Catch-Up Protocol

### Status

Accepted

### Context

When a client subscribes to an existing session, they need full conversation history without gaps or duplicates during the transition from "replaying history" to "receiving live events."

### Options Considered

| Option | Description | Trade-off |
|--------|-------------|-----------|
| WebSocket replay ✓ | Subscribe returns history + live events continue | Single connection, atomic handoff |
| REST + WebSocket | Fetch history via REST, then subscribe | Race conditions, two code paths |
| Automatic on subscribe | Always send full history | No control, large payloads |

### Decision

Enhanced Subscribe with atomic catch-up:

1. Client sends `Subscribe { session_ids, catch_up: true }`
2. Server responds with `SubscribeAck { current_seq, history, has_more }`
3. If `has_more`, client can request additional pages
4. Live events continue from `current_seq + 1`

### Consequences

- Single round-trip for subscribe + first history page
- Sequence numbers guarantee no gaps
- Backward compatible (`catch_up` defaults to false)
- Need new message types: `SubscribeAck`, `RequestHistory`, `HistoryPage`

---

## Architecture

### Event Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Input Attribution Flow                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────┐         ┌──────────────┐         ┌──────────────────┐ │
│  │   CLI    │         │    Server    │         │     Web UI       │ │
│  └────┬─────┘         └──────┬───────┘         └────────┬─────────┘ │
│       │                      │                          │           │
│       │ Input{content,       │                          │           │
│       │   source:cli}        │                          │           │
│       │─────────────────────>│                          │           │
│       │                      │                          │           │
│       │                      │ UserInput{               │           │
│       │                      │   content,               │           │
│       │                      │   source:cli}            │           │
│       │                      │────────────────────────->│           │
│       │                      │                          │           │
│       │                      │ Claude{response}         │           │
│       │<─────────────────────│────────────────────────->│           │
│       │                      │                          │           │
│       │                      │                          │ Input{    │
│       │                      │                          │   content,│
│       │                      │                          │   source: │
│       │                      │<─────────────────────────│   web_ui} │
│       │                      │                          │           │
│       │ UserInput{           │                          │           │
│       │   content,           │                          │           │
│       │   source:web_ui}     │                          │           │
│       │<─────────────────────│                          │           │
│       │                      │                          │           │
└───────┴──────────────────────┴──────────────────────────┴───────────┘
```

### Late-Joiner Catch-Up Flow

```
Client                              Server
   |                                   |
   |-- Subscribe { catch_up: true } -->|
   |                                   | (fetch session events)
   |<-- SubscribeAck {                 |
   |      current_seq: 150,            |
   |      history: [events 101-150],   |
   |      has_more: true               |
   |    } -----------------------------|
   |                                   |
   |-- RequestHistory {                |
   |      before_seq: 101,             |
   |      limit: 50                    |
   |    } ----------------------------->|
   |                                   |
   |<-- HistoryPage {                  |
   |      events: [51-100],            |
   |      has_more: true               |
   |    } -----------------------------|
   |                                   |
   |-- RequestHistory {                |
   |      before_seq: 51,              |
   |      limit: 50                    |
   |    } ----------------------------->|
   |                                   |
   |<-- HistoryPage {                  |
   |      events: [1-50],              |
   |      has_more: false              |
   |    } -----------------------------|
   |                                   |
   |<-- Claude { seq: 151, ... } ------|  (live events resume)
```

---

## Types and Interfaces

### Input Source Type

```rust
// vibes-core/src/events/types.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    Cli,
    WebUi,
    /// For events replayed from history where source is unknown
    Unknown,
}

impl InputSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::WebUi => "web_ui",
            Self::Unknown => "unknown",
        }
    }
}
```

### Updated UserInput Event

```rust
// vibes-core/src/events/types.rs

pub enum VibesEvent {
    // ... existing variants ...

    UserInput {
        session_id: String,
        content: String,
        source: InputSource,  // NEW
    },
}
```

### WebSocket Protocol Updates

```rust
// vibes-server/src/ws/protocol.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    // Enhanced Subscribe
    Subscribe {
        session_ids: Vec<String>,
        #[serde(default)]
        catch_up: bool,
    },

    // NEW: Request additional history
    RequestHistory {
        session_id: String,
        before_seq: u64,
        #[serde(default = "default_history_limit")]
        limit: u32,
    },

    // ... existing variants unchanged ...
}

fn default_history_limit() -> u32 { 50 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // NEW: Subscribe acknowledgment with history
    SubscribeAck {
        session_id: String,
        current_seq: u64,
        history: Vec<HistoryEvent>,
        has_more: bool,
    },

    // NEW: Additional history page
    HistoryPage {
        session_id: String,
        events: Vec<HistoryEvent>,
        has_more: bool,
        oldest_seq: u64,
    },

    // ... existing variants unchanged ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub seq: u64,
    pub event: VibesEvent,
    pub timestamp: i64,
}
```

### Connection State

```rust
// vibes-server/src/ws/connection.rs

struct ConnectionState {
    client_id: String,
    client_type: InputSource,
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
}

fn detect_client_type(req: &Request) -> InputSource {
    if req.headers().get("X-Vibes-Client-Type")
        .and_then(|v| v.to_str().ok())
        == Some("cli")
    {
        return InputSource::Cli;
    }
    InputSource::WebUi
}
```

---

## CLI Integration

### Remote Input Display

```rust
// vibes-cli/src/commands/claude.rs

fn display_remote_input(source: &InputSource, content: &str) {
    let prefix = match source {
        InputSource::WebUi => "[Web UI]:",
        InputSource::Cli => "[CLI]:",
        InputSource::Unknown => "[Remote]:",
    };

    println!("{} {}", prefix.cyan(), content);
}

async fn handle_event(event: &VibesEvent, our_source: InputSource) {
    match event {
        VibesEvent::UserInput { content, source, .. } if *source != our_source => {
            display_remote_input(source, content);
        }
        VibesEvent::Claude { event, .. } => {
            display_claude_event(event);
        }
        _ => {}
    }
}
```

### Input History (Up/Down Keys)

```rust
// vibes-cli/src/input/history.rs

pub struct InputHistory {
    entries: Vec<String>,
    position: Option<usize>,
    draft: String,
}

impl InputHistory {
    pub fn new() -> Self {
        Self { entries: Vec::new(), position: None, draft: String::new() }
    }

    pub fn push(&mut self, input: String) {
        if self.entries.last() != Some(&input) {
            self.entries.push(input);
        }
        self.position = None;
    }

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

### Example CLI Output

```
You: What files handle routing?

Claude: The routing is handled in src/router/...

[Web UI]: Can you show the middleware too?

Claude: Sure, the middleware chain is defined in...

You: Thanks, one more question about auth

Claude: For authentication, we use...
```

---

## Web UI Integration

### Message Component

```tsx
// web-ui/src/components/MessageBubble.tsx

interface MessageProps {
  role: 'user' | 'assistant';
  content: string;
  source?: 'cli' | 'web_ui' | 'unknown';
  isRemote: boolean;
}

function MessageBubble({ role, content, source, isRemote }: MessageProps) {
  if (role === 'user') {
    return (
      <div className={cn(
        "flex flex-col items-end",
        isRemote && "opacity-90"
      )}>
        {isRemote && (
          <span className="text-xs text-muted-foreground mb-1">
            {source === 'cli' ? '📟 CLI' : '🌐 Web UI'}
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

### Catch-Up Hook

```tsx
// web-ui/src/hooks/useCatchUp.ts

async function catchUpSession(ws: WebSocket, sessionId: string): Promise<Message[]> {
  const messages: Message[] = [];

  ws.send(JSON.stringify({
    type: 'subscribe',
    session_ids: [sessionId],
    catch_up: true,
  }));

  const ack = await waitForMessage<SubscribeAck>(ws, 'subscribe_ack');
  messages.push(...parseHistoryEvents(ack.history));

  let hasMore = ack.has_more;
  while (hasMore) {
    const oldestSeq = messages[0]?.seq ?? 0;
    ws.send(JSON.stringify({
      type: 'request_history',
      session_id: sessionId,
      before_seq: oldestSeq,
      limit: 50,
    }));

    const page = await waitForMessage<HistoryPage>(ws, 'history_page');
    messages.unshift(...parseHistoryEvents(page.events));
    hasMore = page.has_more;
  }

  return messages;
}
```

---

## Database Schema

### Migration

```sql
-- vibes-core/src/history/migrations/003_add_message_source.sql

ALTER TABLE messages ADD COLUMN source TEXT NOT NULL DEFAULT 'unknown';
CREATE INDEX idx_messages_source ON messages(source);
```

### Updated Insert

```rust
// vibes-core/src/history/service.rs

async fn save_user_message(&self, session_id: &str, content: &str, source: InputSource) {
    sqlx::query(
        "INSERT INTO messages (session_id, role, content, source, created_at)
         VALUES (?, 'user', ?, ?, ?)"
    )
    .bind(session_id)
    .bind(content)
    .bind(source.as_str())
    .bind(chrono::Utc::now())
    .execute(&self.pool)
    .await?;
}
```

---

## File Structure

### Modified Files

```
vibes/
├── vibes-core/
│   └── src/
│       ├── events/
│       │   └── types.rs           # Add InputSource, update UserInput
│       ├── session/
│       │   ├── state.rs           # send() → send_with_source()
│       │   └── manager.rs         # send_to_session_with_source()
│       └── history/
│           ├── service.rs         # Process source in UserInput
│           └── migrations/        # Add source column migration
├── vibes-server/
│   └── src/
│       └── ws/
│           ├── protocol.rs        # SubscribeAck, HistoryPage, RequestHistory
│           └── connection.rs      # Client type detection, catch-up handler
├── vibes-cli/
│   └── src/
│       ├── commands/
│       │   └── claude.rs          # Remote input display, input history
│       └── input/
│           └── history.rs         # NEW: InputHistory struct
└── web-ui/
    └── src/
        ├── components/
        │   └── MessageBubble.tsx  # Source attribution display
        ├── hooks/
        │   ├── useSessionEvents.ts # Handle UserInput with source
        │   └── useCatchUp.ts       # NEW: Catch-up on subscribe
        └── lib/
            └── types.ts            # Add source to Message type
```

---

## Testing Strategy

### Unit Tests

| Component | Test Cases |
|-----------|------------|
| `InputSource` | Serialization roundtrip, `as_str()` |
| `InputHistory` | Navigate up/down, push, edge cases |
| Protocol messages | Serialize/deserialize new message types |
| Catch-up logic | Pagination, empty history, single page |

### Integration Tests

| Test | Description |
|------|-------------|
| Two-client input | CLI sends input, Web UI sees it with attribution |
| Late joiner | New client subscribes, receives full history |
| Catch-up pagination | Session with 150 events, verify all pages received |
| Source persistence | Input saved to history with correct source |

### Manual Testing

- [ ] Send input from CLI, verify Web UI shows `📟 CLI` label
- [ ] Send input from Web UI, verify CLI shows `[Web UI]:` prefix
- [ ] Open Web UI mid-session, verify full history loads
- [ ] Test up/down arrow history navigation in CLI
- [ ] Verify source column populated in SQLite

---

## Out of Scope

| Feature | Deferred To | Reason |
|---------|-------------|--------|
| Permission response attribution | Future | MVP focus on input |
| Persistent CLI input history | Future | Keep simple |
| Identity-aware attribution | Future | Requires auth integration |
| Typing indicators | Future | Nice to have, not core |

---

## Deliverables Checklist

### Backend (vibes-core)

- [ ] Add `InputSource` enum to `events/types.rs`
- [ ] Update `UserInput` event with `source` field
- [ ] Add `source` column migration
- [ ] Update `HistoryService` to persist source
- [ ] Update `SessionManager.send_to_session_with_source()`
- [ ] Unit tests for new types

### Server (vibes-server)

- [ ] Add `SubscribeAck`, `RequestHistory`, `HistoryPage` messages
- [ ] Detect client type from request headers
- [ ] Implement catch-up on subscribe
- [ ] Implement paginated history requests
- [ ] Attach source to forwarded input
- [ ] Integration tests for catch-up flow

### CLI (vibes-cli)

- [ ] Create `InputHistory` struct with up/down navigation
- [ ] Display remote input with source prefix
- [ ] Send `X-Vibes-Client-Type: cli` header
- [ ] Handle catch-up on `sessions attach`
- [ ] Unit tests for input history

### Web UI

- [ ] Update `MessageBubble` with source display
- [ ] Create `useCatchUp` hook
- [ ] Handle `UserInput` events with source
- [ ] Request catch-up on session join
- [ ] Visual distinction for remote messages

### Documentation

- [ ] Design document (this file)
- [ ] Update PROGRESS.md

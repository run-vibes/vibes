# Milestone 3.2: Multi-Session Support - Design Document

> Multiple concurrent Claude sessions with ownership tracking, lifecycle management, and responsive UI.

## Overview

Multi-Session Support enables users to run multiple concurrent Claude Code sessions on the same vibes server. Each session has an owner (the creating client), can be subscribed to by multiple clients, and supports seamless ownership handoff when the owner disconnects.

This milestone extends existing infrastructure rather than replacing it. The `SessionManager` already supports multiple sessions; we're adding ownership tracking, lifecycle management, and improved UX across CLI and Web UI.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Session lifecycle | Client-owned | Sessions removed when owner disconnects |
| Ownership handoff | Transfer to subscribers | Enables seamless device handoff |
| Web UI view | Split (desktop) / Single (mobile) | Responsive to device capabilities |
| Status indicators | Name + State badge + Permission alert | Essential info at a glance |
| CLI behavior | Always new session | Predictable, explicit |
| Cleanup signaling | Persist to history only | Leverages milestone 3.1 SQLite |
| Ownership tracking | Explicit `owner_id` field | Enables future ownership features |

---

## ADR: Session Ownership and Lifecycle Management

### Status

Accepted

### Context

With multiple concurrent sessions, we need to define:
1. When sessions should be cleaned up
2. How ownership transfers between clients
3. How to prevent session accumulation

### Options Considered

**Session Cleanup Strategy:**

| Option | Behavior | Trade-off |
|--------|----------|-----------|
| Client-owned ✓ | Session removed when creating client disconnects | Simple, matches "session = task" mental model |
| Manual cleanup only | Sessions persist until explicitly deleted | Maximum flexibility, risk of accumulation |
| Idle timeout | Auto-removed after N minutes of inactivity | Automatic, but might lose wanted sessions |
| Hybrid | Owner disconnect queues removal, timeout triggers cleanup | Graceful handoff window, more complex |

**Ownership Handoff:**

| Option | Behavior | Trade-off |
|--------|----------|-----------|
| Transfer ownership ✓ | If another client is subscribed, ownership transfers | Sessions survive handoffs, collaborative |
| Immediate cleanup | Session terminates regardless of subscribers | Simple, but disruptive |
| Grace period | Session stays alive for N seconds for claiming | Middle ground, adds complexity |

### Decision

1. **Client-owned sessions** with **ownership transfer** to subscribers
2. When owner disconnects, ownership transfers to the next subscriber
3. When no subscribers remain, session is removed from active `SessionManager`
4. Session history already persisted via milestone 3.1 SQLite - cleanup just removes from memory

### Consequences

- Add `SessionOwnership` struct with `owner_id` and `subscriber_ids`
- Create `SessionLifecycleManager` to handle disconnect/cleanup logic
- Each WebSocket connection needs a unique `client_id`
- Ownership transfer events broadcast to subscribers

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        vibes-core                                │
│                                                                  │
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │  SessionManager  │───▶│  Session         │                   │
│  │  (HashMap)       │    │  + ownership     │◀──── NEW FIELD    │
│  └──────────────────┘    └──────────────────┘                   │
│          │                        │                              │
│          │                        ▼                              │
│          │               ┌──────────────────┐                   │
│          │               │ SessionOwnership │                   │
│          │               │  - owner_id      │                   │
│          │               │  - subscriber_ids│                   │
│          │               └──────────────────┘                   │
│          │                                                       │
│          ▼                                                       │
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │ SessionLifecycle │    │   HistoryStore   │                   │
│  │    Manager (NEW) │    │   (SQLite)       │                   │
│  │ - ownership xfer │    │   - persisted    │                   │
│  │ - cleanup        │    └──────────────────┘                   │
│  └──────────────────┘                                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       vibes-server                               │
│                                                                  │
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │  ConnectionState │    │   WS Protocol    │                   │
│  │  + client_id     │    │  + ListSessions  │                   │
│  │  + subscriptions │    │  + SessionList   │                   │
│  └──────────────────┘    │  + KillSession   │                   │
│                          │  + SessionRemoved│                   │
│                          └──────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        vibes-cli                                 │
│                                                                  │
│  vibes sessions         - List active sessions                  │
│  vibes sessions attach  - Attach to existing session            │
│  vibes sessions kill    - Terminate a session                   │
└─────────────────────────────────────────────────────────────────┘
```

### Event Flow for Cleanup

```
Client disconnects
       │
       ▼
┌──────────────────────┐
│ Remove from          │
│ subscriber_ids       │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐     Yes    ┌─────────────────┐
│ Is client the owner? │───────────▶│ Transfer to     │
└──────────┬───────────┘            │ next subscriber │
           │ No                     └────────┬────────┘
           ▼                                 │
┌──────────────────────┐                     │
│ Any subscribers left?│◀────────────────────┘
└──────────┬───────────┘
           │ No
           ▼
┌──────────────────────┐
│ Remove from active   │
│ SessionManager       │
│ (history preserved)  │
└──────────────────────┘
```

---

## Types and Interfaces

### Session Ownership Types

```rust
// vibes-core/src/session/ownership.rs

use std::collections::HashSet;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Unique identifier for a connected client
pub type ClientId = String;

/// Session ownership and subscription tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOwnership {
    /// The client that created/owns this session
    pub owner_id: ClientId,
    /// All clients currently subscribed to this session
    pub subscriber_ids: HashSet<ClientId>,
    /// When ownership was last transferred
    pub owned_since: SystemTime,
}

impl SessionOwnership {
    pub fn new(owner_id: ClientId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            subscriber_ids: HashSet::from([owner_id]),
            owned_since: SystemTime::now(),
        }
    }

    /// Add a subscriber
    pub fn add_subscriber(&mut self, client_id: ClientId) {
        self.subscriber_ids.insert(client_id);
    }

    /// Remove a subscriber, returns true if was the owner
    pub fn remove_subscriber(&mut self, client_id: &ClientId) -> bool {
        self.subscriber_ids.remove(client_id);
        &self.owner_id == client_id
    }

    /// Transfer ownership to another subscriber
    pub fn transfer_to(&mut self, new_owner: &ClientId) -> bool {
        if self.subscriber_ids.contains(new_owner) {
            self.owner_id = new_owner.clone();
            self.owned_since = SystemTime::now();
            true
        } else {
            false
        }
    }

    /// Pick next owner from subscribers (if any)
    pub fn pick_next_owner(&self) -> Option<&ClientId> {
        self.subscriber_ids.iter().next()
    }

    /// Returns true if session should be cleaned up
    pub fn should_cleanup(&self) -> bool {
        self.subscriber_ids.is_empty()
    }

    /// Check if client is the owner
    pub fn is_owner(&self, client_id: &ClientId) -> bool {
        &self.owner_id == client_id
    }
}
```

### Updated Session Struct

```rust
// vibes-core/src/session/state.rs

pub struct Session {
    id: String,
    name: Option<String>,
    backend: Box<dyn ClaudeBackend>,
    event_bus: Arc<dyn EventBus>,
    state: SessionState,
    created_at: SystemTime,
    last_activity_at: SystemTime,     // NEW
    ownership: SessionOwnership,       // NEW
}

impl Session {
    /// Create a new session with owner
    pub fn new_with_owner(
        id: impl Into<String>,
        name: Option<String>,
        owner_id: ClientId,
        backend: Box<dyn ClaudeBackend>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id: id.into(),
            name,
            backend,
            event_bus,
            state: SessionState::Idle,
            created_at: now,
            last_activity_at: now,
            ownership: SessionOwnership::new(owner_id),
        }
    }

    pub fn ownership(&self) -> &SessionOwnership {
        &self.ownership
    }

    pub fn ownership_mut(&mut self) -> &mut SessionOwnership {
        &mut self.ownership
    }

    pub fn last_activity_at(&self) -> SystemTime {
        self.last_activity_at
    }

    pub fn touch(&mut self) {
        self.last_activity_at = SystemTime::now();
    }
}
```

### Session Lifecycle Manager

```rust
// vibes-core/src/session/lifecycle.rs

use std::sync::Arc;
use crate::history::{HistoryService, SqliteHistoryStore};
use super::manager::SessionManager;
use super::ownership::ClientId;

pub struct SessionLifecycleManager {
    session_manager: Arc<SessionManager>,
}

impl SessionLifecycleManager {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Called when a client disconnects
    /// Returns list of (session_id, new_owner_id) for ownership transfers
    /// and list of session_ids that were cleaned up
    pub async fn handle_client_disconnect(
        &self,
        client_id: &ClientId,
    ) -> (Vec<(String, ClientId)>, Vec<String>) {
        let mut transfers = Vec::new();
        let mut cleanups = Vec::new();

        // Get all sessions this client was subscribed to
        let session_ids = self.session_manager.list_sessions().await;

        for session_id in session_ids {
            let result = self.session_manager.with_session(&session_id, |session| {
                let was_owner = session.ownership_mut().remove_subscriber(client_id);

                if was_owner {
                    // Try to transfer ownership
                    if let Some(new_owner) = session.ownership().pick_next_owner() {
                        let new_owner = new_owner.clone();
                        session.ownership_mut().transfer_to(&new_owner);
                        return Some((true, Some(new_owner)));
                    }
                }

                // Check if cleanup needed
                if session.ownership().should_cleanup() {
                    return Some((false, None));
                }

                None
            }).await;

            if let Ok(Some((transferred, new_owner))) = result {
                if let Some(new_owner_id) = new_owner {
                    transfers.push((session_id.clone(), new_owner_id));
                } else if !transferred {
                    // No subscribers left, cleanup
                    self.session_manager.remove_session(&session_id).await.ok();
                    cleanups.push(session_id);
                }
            }
        }

        (transfers, cleanups)
    }

    /// Subscribe a client to a session
    pub async fn subscribe_client(
        &self,
        session_id: &str,
        client_id: &ClientId,
    ) -> Result<(), crate::error::SessionError> {
        self.session_manager.with_session(session_id, |session| {
            session.ownership_mut().add_subscriber(client_id.clone());
        }).await
    }

    /// Unsubscribe a client from a session (without triggering cleanup)
    pub async fn unsubscribe_client(
        &self,
        session_id: &str,
        client_id: &ClientId,
    ) -> Result<bool, crate::error::SessionError> {
        self.session_manager.with_session(session_id, |session| {
            session.ownership_mut().remove_subscriber(client_id)
        }).await
    }
}
```

---

## WebSocket Protocol Updates

### New Client Messages

```rust
// vibes-server/src/ws/protocol.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    // Existing
    Subscribe { session_ids: Vec<String> },
    Unsubscribe { session_ids: Vec<String> },
    CreateSession { name: Option<String>, request_id: String },
    Input { session_id: String, content: String },
    PermissionResponse { session_id: String, request_id: String, approved: bool },

    // NEW
    /// Request list of all active sessions
    ListSessions { request_id: String },

    /// Terminate a session (must be owner or subscriber)
    KillSession { session_id: String },
}
```

### New Server Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // Existing
    SessionCreated { request_id: String, session_id: String, name: Option<String> },
    SessionNotification { session_id: String, name: Option<String> },
    Claude { session_id: String, event: ClaudeEvent },
    SessionState { session_id: String, state: String },
    Error { session_id: Option<String>, message: String, code: String },
    TunnelState { state: String, url: Option<String> },
    AuthContext(AuthContext),

    // NEW
    /// Full session list response
    SessionList {
        request_id: String,
        sessions: Vec<SessionInfo>,
    },

    /// Session was removed (cleanup or killed)
    SessionRemoved {
        session_id: String,
        reason: RemovalReason,
    },

    /// Ownership transferred
    OwnershipTransferred {
        session_id: String,
        new_owner_id: String,
        /// True if you are the new owner
        you_are_owner: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: String,
    pub owner_id: String,
    pub is_owner: bool,
    pub subscriber_count: u32,
    pub created_at: i64,
    pub last_activity_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RemovalReason {
    OwnerDisconnected,
    Killed,
    SessionFinished,
}
```

### Updated Connection State

```rust
// vibes-server/src/ws/connection.rs

struct ConnectionState {
    /// Unique ID for this connection
    client_id: String,
    /// Session IDs this connection is subscribed to
    subscribed_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            subscribed_sessions: HashSet::new(),
        }
    }

    fn client_id(&self) -> &str {
        &self.client_id
    }
}
```

---

## Web UI Design

### Responsive Layout

**Desktop (≥1024px) - Split View:**

```
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────┬──────────────────────────────────────┐│
│  │   SESSION LIST       │         SESSION DETAIL               ││
│  │   (sidebar, ~300px)  │         (main area)                  ││
│  │                      │                                      ││
│  │  ┌────────────────┐  │  ┌────────────────────────────────┐ ││
│  │  │ 🟢 refactor    │◀─┼──│  Session: refactor              │ ││
│  │  │    Processing  │  │  │  State: Processing              │ ││
│  │  └────────────────┘  │  │                                  │ ││
│  │  ┌────────────────┐  │  │  [Claude output streaming...]   │ ││
│  │  │ 🟡 bugfix      │  │  │                                  │ ││
│  │  │    Waiting     │  │  │                                  │ ││
│  │  └────────────────┘  │  │  ┌────────────────────────────┐ │ ││
│  │  ┌────────────────┐  │  │  │ Your message...      [Send]│ │ ││
│  │  │ 🔵 tests       │  │  │  └────────────────────────────┘ │ ││
│  │  │    Idle        │  │  └────────────────────────────────┘ ││
│  │  └────────────────┘  │                                      ││
│  └──────────────────────┴──────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

**Mobile (<1024px) - Single View with Navigation:**

```
SESSION LIST VIEW              SESSION DETAIL VIEW
┌───────────────────────┐      ┌───────────────────────────┐
│ ← Sessions            │      │ ← Back    refactor        │
├───────────────────────┤      ├───────────────────────────┤
│ 🟢 refactor           │─────▶│                           │
│    Processing         │      │ [Claude output...]        │
├───────────────────────┤      │                           │
│ 🟡 bugfix             │      │                           │
│    ⚠️ Needs approval  │      ├───────────────────────────┤
├───────────────────────┤      │ [Input...]        [Send]  │
│ 🔵 tests              │      └───────────────────────────┘
│    Idle               │
└───────────────────────┘
```

### Status Indicator Colors

| State | Color | Icon | Badge Text |
|-------|-------|------|------------|
| Idle | Blue | 🔵 | "Idle" |
| Processing | Green (animated pulse) | 🟢 | "Processing" |
| WaitingPermission | Yellow | 🟡 + ⚠️ | "Needs approval" |
| Failed | Red | 🔴 | "Failed" |
| Finished | Gray | ⚪ | "Finished" |

### Session Card Information

**Always visible:**
- Session name (or truncated ID if no name)
- State badge with color
- Permission alert icon if WaitingPermission

**On hover (desktop) / on expand (mobile):**
- Time since last activity ("2m ago")
- Current tool if Processing
- Subscriber count
- "You own this" indicator if owner

---

## CLI Session Commands

### Command Structure

```
vibes sessions                    # List active sessions
vibes sessions list               # Same as above
vibes sessions attach <id>        # Attach to existing session
vibes sessions kill <id>          # Terminate a session
```

### Output Format

```bash
$ vibes sessions
ACTIVE SESSIONS (3)

  ID          NAME        STATE              AGE      OWNER
  a1b2c3d4    refactor    🟢 Processing      5m       you (CLI)
  e5f6g7h8    bugfix      🟡 Needs approval  12m      Web UI
  i9j0k1l2    -           🔵 Idle            1h       you (CLI)

Use 'vibes sessions attach <id>' to connect to a session.
Use 'vibes sessions kill <id>' to terminate a session.
```

### Attach Behavior

```bash
$ vibes sessions attach e5f6g7h8

Attaching to session 'bugfix' (e5f6g7h8)...
Connected. You are now a subscriber.

⚠️  Permission Request
    Tool: Bash
    Command: rm -rf node_modules && npm install

    [A]pprove  [D]eny  [V]iew details
```

When attaching:
1. CLI sends `Subscribe` message for that session
2. Server adds CLI as subscriber
3. CLI receives current session state
4. CLI can send input and respond to permissions
5. If owner disconnects while CLI is attached, CLI becomes new owner

---

## Crate Structure

### New/Modified Files

```
vibes/
├── vibes-core/
│   └── src/
│       ├── session/
│       │   ├── mod.rs              # Export ownership, lifecycle
│       │   ├── state.rs            # Add ownership field to Session
│       │   ├── manager.rs          # Update create_session signature
│       │   ├── ownership.rs        # NEW: SessionOwnership, ClientId
│       │   └── lifecycle.rs        # NEW: SessionLifecycleManager
│       └── lib.rs                  # Export new types
├── vibes-server/
│   └── src/
│       ├── ws/
│       │   ├── protocol.rs         # Add new message types
│       │   └── connection.rs       # Add client_id, handle lifecycle
│       └── state.rs                # Add SessionLifecycleManager
├── vibes-cli/
│   └── src/
│       └── commands/
│           ├── mod.rs              # Add sessions module
│           └── sessions.rs         # NEW: list, attach, kill commands
└── web-ui/
    └── src/
        ├── pages/
        │   └── ClaudeSessions.tsx  # Responsive split/single layout
        ├── components/
        │   ├── SessionCard.tsx     # Enhanced status indicators
        │   ├── SessionList.tsx     # NEW: Sidebar session list
        │   └── SessionDetail.tsx   # NEW: Main session view area
        ├── hooks/
        │   └── useSessionList.ts   # NEW: WebSocket session list hook
        └── lib/
            └── types.ts            # Add SessionInfo type
```

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| `SessionOwnership` | add/remove subscriber, transfer, cleanup detection |
| `SessionLifecycleManager` | disconnect handling, ownership transfer, cleanup |
| Protocol messages | Serialization roundtrip for new message types |

### Integration Tests

| Test | Description |
|------|-------------|
| Multi-client ownership | Two clients, owner disconnects, verify transfer |
| Cleanup on empty | All clients disconnect, verify session removed |
| Session list sync | Create/remove sessions, verify list updates |
| CLI attach | Attach to existing session, verify interaction works |

### Manual Testing

- [ ] Create session from CLI, view in Web UI
- [ ] Create session from Web UI, attach from CLI
- [ ] Disconnect owner, verify ownership transfers
- [ ] Disconnect all clients, verify session removed
- [ ] Test split view on desktop, single view on mobile
- [ ] Test session status indicators update in real-time

---

## Out of Scope

Explicitly excluded from this milestone:

| Feature | Deferred To | Reason |
|---------|-------------|--------|
| Late-joiner full replay | Milestone 3.3 | Part of CLI ↔ Web Mirroring |
| Input source attribution | Milestone 3.3 | Part of CLI ↔ Web Mirroring |
| Session naming from Web UI | Future | Minor enhancement |
| Per-session notification settings | Phase 4 | Advanced permissions feature |

---

## Deliverables

### Milestone 3.2 Checklist

**Backend (vibes-core):**
- [ ] Create `SessionOwnership` struct with subscriber tracking
- [ ] Add `ownership` field to `Session`
- [ ] Add `last_activity_at` field to `Session`
- [ ] Create `SessionLifecycleManager` for disconnect handling
- [ ] Update `SessionManager.create_session` to accept `owner_id`
- [ ] Unit tests for ownership and lifecycle

**Server (vibes-server):**
- [ ] Add `client_id` to `ConnectionState`
- [ ] Add `ListSessions`, `KillSession` client messages
- [ ] Add `SessionList`, `SessionRemoved`, `OwnershipTransferred` server messages
- [ ] Handle client disconnect in WebSocket handler
- [ ] Broadcast ownership transfers and session removals
- [ ] Integration tests for multi-client scenarios

**CLI (vibes-cli):**
- [ ] Add `vibes sessions` command group
- [ ] Implement `list` subcommand with table output
- [ ] Implement `attach` subcommand for joining sessions
- [ ] Implement `kill` subcommand for terminating sessions
- [ ] Update `vibes claude` to pass client_id on session creation

**Web UI:**
- [ ] Implement responsive split/single layout
- [ ] Create `SessionList` sidebar component
- [ ] Enhance `SessionCard` with status indicators
- [ ] Add real-time session list updates via WebSocket
- [ ] Handle `SessionRemoved` and `OwnershipTransferred` messages
- [ ] Mobile-friendly navigation between list and detail views

**Documentation:**
- [ ] Design document (this file)
- [ ] Update PROGRESS.md

# Scrollback Buffer & History Simplification - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace SQLite chat history with a 1MB in-memory scrollback buffer for PTY output replay on attach.

**Architecture:** Add `ScrollbackBuffer` to each PTY session, capture output before broadcasting, send replay on client attach.

**Tech Stack:** Rust (VecDeque ring buffer), TypeScript (xterm.js), WebSocket protocol extension.

---

## Task 1: ScrollbackBuffer Core Implementation

**Files:**
- Create: `vibes-core/src/pty/scrollback.rs`
- Modify: `vibes-core/src/pty/mod.rs`
- Test: `vibes-core/src/pty/scrollback.rs` (inline tests)

### Step 1: Write failing tests for ScrollbackBuffer

Create `vibes-core/src/pty/scrollback.rs`:

```rust
//! Scrollback buffer for PTY output

use std::collections::VecDeque;

/// Default buffer capacity: 1MB
pub const DEFAULT_CAPACITY: usize = 1_048_576;

/// Ring buffer for PTY output with fixed byte capacity.
pub struct ScrollbackBuffer {
    buffer: VecDeque<u8>,
    capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_buffer() {
        let buf = ScrollbackBuffer::new(1024);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn append_stores_data() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get_all(), b"hello");
    }

    #[test]
    fn append_multiple_times() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"hello ");
        buf.append(b"world");
        assert_eq!(buf.get_all(), b"hello world");
    }

    #[test]
    fn overflow_drops_oldest_bytes() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.append(b"hello"); // 5 bytes
        buf.append(b"world!"); // 6 bytes, total 11, drops 1
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.get_all(), b"elloworld!");
    }

    #[test]
    fn large_append_keeps_only_capacity() {
        let mut buf = ScrollbackBuffer::new(5);
        buf.append(b"hello world"); // 11 bytes
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get_all(), b"world");
    }

    #[test]
    fn default_uses_1mb_capacity() {
        let buf = ScrollbackBuffer::default();
        assert_eq!(buf.capacity, DEFAULT_CAPACITY);
    }
}
```

### Step 2: Run tests to verify they fail

Run: `cd vibes-core && cargo test scrollback --lib`
Expected: FAIL - methods not implemented

### Step 3: Implement ScrollbackBuffer

Add implementation above the tests in `scrollback.rs`:

```rust
impl ScrollbackBuffer {
    /// Create buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity.min(capacity)),
            capacity,
        }
    }

    /// Append data, dropping oldest bytes if over capacity
    pub fn append(&mut self, data: &[u8]) {
        for &byte in data {
            if self.buffer.len() >= self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(byte);
        }
    }

    /// Get all buffered data for replay
    pub fn get_all(&self) -> Vec<u8> {
        self.buffer.iter().copied().collect()
    }

    /// Current buffer size in bytes
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}
```

### Step 4: Run tests to verify they pass

Run: `cd vibes-core && cargo test scrollback --lib`
Expected: PASS (6 tests)

### Step 5: Export from pty module

Modify `vibes-core/src/pty/mod.rs` - add after line 8:

```rust
mod scrollback;
pub use scrollback::{ScrollbackBuffer, DEFAULT_CAPACITY};
```

### Step 6: Verify module compiles

Run: `cd vibes-core && cargo check`
Expected: Success

### Step 7: Commit

```bash
git add vibes-core/src/pty/scrollback.rs vibes-core/src/pty/mod.rs
git commit -m "feat(pty): add ScrollbackBuffer for terminal output history"
```

---

## Task 2: Integrate ScrollbackBuffer into PtySessionHandle

**Files:**
- Modify: `vibes-core/src/pty/session.rs`
- Test: `vibes-core/src/pty/session.rs` (add test)

### Step 1: Write failing test for scrollback access

Add to `vibes-core/src/pty/session.rs` in the `tests` module:

```rust
#[tokio::test]
async fn handle_provides_scrollback_access() {
    let config = PtyConfig {
        claude_path: "cat".into(),
        ..Default::default()
    };

    let session = PtySession::spawn("test-id".to_string(), None, &config).unwrap();

    // Write data and read it back (cat echoes)
    session.handle.write(b"test\n").await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Read the output
    let data = session.handle.read().await.unwrap();
    assert!(!data.is_empty());

    // Append to scrollback
    session.handle.append_scrollback(&data);

    // Verify scrollback contains data
    let scrollback = session.handle.get_scrollback();
    assert!(!scrollback.is_empty());
}
```

### Step 2: Run test to verify it fails

Run: `cd vibes-core && cargo test handle_provides_scrollback_access --lib`
Expected: FAIL - `append_scrollback` and `get_scrollback` not found

### Step 3: Add scrollback to PtySessionHandle

Modify `vibes-core/src/pty/session.rs`:

Add import at top:
```rust
use super::scrollback::ScrollbackBuffer;
```

Add field to `PtySessionHandle` struct (after line 23):
```rust
/// Scrollback buffer for replay on reconnect
scrollback: Arc<std::sync::Mutex<ScrollbackBuffer>>,
```

Add methods to `impl PtySessionHandle` (after `resize` method, ~line 93):
```rust
/// Append data to the scrollback buffer
pub fn append_scrollback(&self, data: &[u8]) {
    if let Ok(mut scrollback) = self.scrollback.lock() {
        scrollback.append(data);
    }
}

/// Get all scrollback data for replay
pub fn get_scrollback(&self) -> Vec<u8> {
    self.scrollback
        .lock()
        .map(|s| s.get_all())
        .unwrap_or_default()
}
```

Update `PtySession::spawn` to initialize scrollback (in the handle creation, ~line 147-151):
```rust
let handle = PtySessionHandle {
    inner: Arc::new(Mutex::new(inner)),
    reader: Arc::new(std::sync::Mutex::new(reader)),
    writer: Arc::new(std::sync::Mutex::new(writer)),
    scrollback: Arc::new(std::sync::Mutex::new(ScrollbackBuffer::default())),
};
```

### Step 4: Run test to verify it passes

Run: `cd vibes-core && cargo test handle_provides_scrollback_access --lib`
Expected: PASS

### Step 5: Run all pty tests

Run: `cd vibes-core && cargo test pty --lib`
Expected: All tests pass

### Step 6: Commit

```bash
git add vibes-core/src/pty/session.rs
git commit -m "feat(pty): integrate ScrollbackBuffer into PtySessionHandle"
```

---

## Task 3: Add PtyReplay to Protocol

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`
- Test: inline in protocol.rs

### Step 1: Write failing test for PtyReplay serialization

Add to `vibes-server/src/ws/protocol.rs` in tests module (after line ~1006):

```rust
#[test]
fn test_server_message_pty_replay_roundtrip() {
    let msg = ServerMessage::PtyReplay {
        session_id: "sess-1".to_string(),
        data: "aGVsbG8gd29ybGQ=".to_string(), // base64 for "hello world"
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, parsed);
    assert!(json.contains(r#""type":"pty_replay""#));
}
```

### Step 2: Run test to verify it fails

Run: `cd vibes-server && cargo test pty_replay --lib`
Expected: FAIL - `PtyReplay` variant not found

### Step 3: Add PtyReplay variant to ServerMessage

Modify `vibes-server/src/ws/protocol.rs` - add after `AttachAck` variant (~line 305):

```rust
/// Replay scrollback buffer on attach
PtyReplay {
    /// Session ID
    session_id: String,
    /// Scrollback data (base64 encoded)
    data: String,
},
```

### Step 4: Run test to verify it passes

Run: `cd vibes-server && cargo test pty_replay --lib`
Expected: PASS

### Step 5: Commit

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(protocol): add PtyReplay message for scrollback replay"
```

---

## Task 4: Capture Output in Scrollback Buffer

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

### Step 1: Locate pty_output_reader function

The function is at line 795. We need to add scrollback capture after reading data.

### Step 2: Add scrollback capture

Modify `pty_output_reader` in `vibes-server/src/ws/connection.rs`.

Find this block (~line 807-815):
```rust
Ok(data) if !data.is_empty() => {
    // Encode as base64 and broadcast
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    let event = PtyEvent::Output {
        session_id: session_id.clone(),
        data: encoded,
    };
    state.broadcast_pty_event(event);
}
```

Replace with:
```rust
Ok(data) if !data.is_empty() => {
    // Append to scrollback buffer
    handle.append_scrollback(&data);

    // Encode as base64 and broadcast
    let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
    let event = PtyEvent::Output {
        session_id: session_id.clone(),
        data: encoded,
    };
    state.broadcast_pty_event(event);
}
```

### Step 3: Verify compilation

Run: `cd vibes-server && cargo check`
Expected: Success

### Step 4: Commit

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(ws): capture PTY output in scrollback buffer"
```

---

## Task 5: Send Replay on Attach

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

### Step 1: Locate attach handler

The handler starts at line 573 (`ClientMessage::Attach`). We need to send replay before AttachAck.

### Step 2: Add replay before AttachAck

Find the AttachAck sending block (~line 617-623):
```rust
// Send AttachAck
let ack = ServerMessage::AttachAck {
    session_id,
    cols,
    rows,
};
let json = serde_json::to_string(&ack)?;
sender.send(Message::Text(json)).await?;
```

Insert BEFORE it (after `conn_state.attach_pty(&session_id);`):
```rust
// Send scrollback replay if available
if let Some(handle) = pty_manager.get_handle(&session_id) {
    let scrollback = handle.get_scrollback();
    if !scrollback.is_empty() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&scrollback);
        let replay = ServerMessage::PtyReplay {
            session_id: session_id.clone(),
            data: encoded,
        };
        let json = serde_json::to_string(&replay)?;
        sender.send(Message::Text(json)).await?;
    }
}
// Drop the lock before sending AttachAck
drop(pty_manager);
```

Note: We need to move the `drop(pty_manager)` that should already be there, or ensure we don't hold the lock across await points.

### Step 3: Fix borrow issue

The current code holds `pty_manager` write lock. We need to restructure to:
1. Get session info while holding lock
2. Drop lock
3. Send messages

Find the full attach handler and restructure:

```rust
ClientMessage::Attach { session_id } => {
    debug!("PTY attach requested for session: {}", session_id);

    let (cols, rows, scrollback_data) = {
        let mut pty_manager = state.pty_manager.write().await;

        // Check if session exists; if not, create it
        if pty_manager.get_session(&session_id).is_some() {
            // Session exists, get scrollback
            let scrollback = pty_manager
                .get_handle(&session_id)
                .map(|h| h.get_scrollback())
                .unwrap_or_default();
            (120, 40, scrollback)
        } else {
            // Create new PTY session with the client's session ID
            match pty_manager.create_session_with_id(session_id.clone(), None) {
                Ok(created_id) => {
                    debug!("Created new PTY session: {}", created_id);

                    // Get handle for output reading
                    if let Some(handle) = pty_manager.get_handle(&created_id) {
                        // Spawn background task to read PTY output
                        let state_clone = state.clone();
                        let session_id_clone = created_id.clone();
                        let handle_clone = handle.clone();
                        tokio::spawn(async move {
                            pty_output_reader(state_clone, session_id_clone, handle_clone).await;
                        });
                    }

                    (120, 40, vec![]) // New session, no scrollback
                }
                Err(e) => {
                    let error = ServerMessage::Error {
                        session_id: Some(session_id),
                        message: format!("Failed to create PTY session: {}", e),
                        code: "PTY_CREATE_FAILED".to_string(),
                    };
                    let json = serde_json::to_string(&error)?;
                    sender.send(Message::Text(json)).await?;
                    return Ok(());
                }
            }
        }
    }; // pty_manager lock dropped here

    // Mark this connection as attached
    conn_state.attach_pty(&session_id);

    // Send scrollback replay if available
    if !scrollback_data.is_empty() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&scrollback_data);
        let replay = ServerMessage::PtyReplay {
            session_id: session_id.clone(),
            data: encoded,
        };
        let json = serde_json::to_string(&replay)?;
        sender.send(Message::Text(json)).await?;
    }

    // Send AttachAck
    let ack = ServerMessage::AttachAck {
        session_id,
        cols,
        rows,
    };
    let json = serde_json::to_string(&ack)?;
    sender.send(Message::Text(json)).await?;
}
```

### Step 4: Verify compilation

Run: `cd vibes-server && cargo check`
Expected: Success

### Step 5: Run server tests

Run: `cd vibes-server && cargo test --lib`
Expected: All tests pass

### Step 6: Commit

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "feat(ws): send scrollback replay on PTY attach"
```

---

## Task 6: Handle PtyReplay in Frontend

**Files:**
- Modify: `web-ui/src/lib/types.ts`
- Modify: `web-ui/src/pages/ClaudeSession.tsx`

### Step 1: Add PtyReplay type

Modify `web-ui/src/lib/types.ts` - add to `ServerMessage` type after `attach_ack` (~line 50):

```typescript
| { type: 'pty_replay'; session_id: string; data: string }  // base64 encoded scrollback
```

Add type guard after `isAttachAckMessage` (~line 203):

```typescript
export function isPtyReplayMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'pty_replay' }> {
  return msg.type === 'pty_replay';
}
```

### Step 2: Handle PtyReplay in ClaudeSession

Modify `web-ui/src/pages/ClaudeSession.tsx` - add case in `handleMessage` switch (after `attach_ack` case, ~line 41):

```typescript
case 'pty_replay':
  if (msg.session_id === sessionId && terminalRef.current) {
    terminalRef.current.write(msg.data);
  }
  break;
```

### Step 3: Build and verify

Run: `cd web-ui && npm run build`
Expected: Success

### Step 4: Commit

```bash
git add web-ui/src/lib/types.ts web-ui/src/pages/ClaudeSession.tsx
git commit -m "feat(web-ui): handle PtyReplay message for scrollback"
```

---

## Task 7: Remove History Module from vibes-core

**Files:**
- Delete: `vibes-core/src/history/` (entire directory)
- Modify: `vibes-core/src/lib.rs`
- Modify: `vibes-core/Cargo.toml`

### Step 1: Remove history module export

Modify `vibes-core/src/lib.rs`:

Remove line 50:
```rust
pub mod history;
```

Remove line 63:
```rust
pub use history::HistoryError;
```

### Step 2: Check if rusqlite is used elsewhere

Run: `cd vibes-core && grep -r "rusqlite" --include="*.rs" | grep -v history`
Expected: No results (rusqlite only used by history)

### Step 3: Remove rusqlite dependency

Modify `vibes-core/Cargo.toml` - remove line 33:
```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

### Step 4: Delete history directory

```bash
rm -rf vibes-core/src/history
```

### Step 5: Verify compilation

Run: `cd vibes-core && cargo check`
Expected: Success (or identify other files that import history)

### Step 6: Fix any remaining imports

If cargo check fails, fix imports in other files that reference history types.

### Step 7: Run tests

Run: `cd vibes-core && cargo test --lib`
Expected: All tests pass

### Step 8: Commit

```bash
git add -A vibes-core/
git commit -m "refactor(core): remove SQLite history module"
```

---

## Task 8: Remove History from Protocol

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

### Step 1: Remove HistoryEvent struct

Remove lines 8-17:
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

fn default_history_limit() -> u32 {
    50
}
```

### Step 2: Remove history-related ClientMessage variants

Remove from `ClientMessage` enum:
- `Subscribe` variant (lines 49-56)
- `Unsubscribe` variant (lines 58-62)
- `RequestHistory` variant (lines 112-121)

### Step 3: Remove history-related ServerMessage variants

Remove from `ServerMessage` enum:
- `SubscribeAck` variant (lines 243-253)
- `HistoryPage` variant (lines 255-265)

### Step 4: Remove history-related tests

Remove all tests that reference:
- `HistoryEvent`
- `Subscribe`
- `RequestHistory`
- `SubscribeAck`
- `HistoryPage`

(Tests at lines ~800-864)

### Step 5: Verify compilation

Run: `cd vibes-server && cargo check`
Expected: Success (or identify connection.rs that uses these types)

### Step 6: Commit

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "refactor(protocol): remove history-related message types"
```

---

## Task 9: Remove History Handling from Connection

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

### Step 1: Find and remove Subscribe handler

Search for `ClientMessage::Subscribe` and remove the entire match arm.

### Step 2: Find and remove Unsubscribe handler

Search for `ClientMessage::Unsubscribe` and remove the entire match arm.

### Step 3: Find and remove RequestHistory handler

Search for `ClientMessage::RequestHistory` and remove the entire match arm.

### Step 4: Remove any history service imports/initialization

Search for `history` imports and remove them.

### Step 5: Remove subscription tracking from ConnectionState if unused

Check if `subscribed_sessions` is still needed. If only used for history, remove it.

### Step 6: Verify compilation

Run: `cd vibes-server && cargo check`
Expected: Success

### Step 7: Run tests

Run: `cd vibes-server && cargo test --lib`
Expected: All tests pass

### Step 8: Commit

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "refactor(ws): remove history handling from connection"
```

---

## Task 10: Clean Up Frontend Types

**Files:**
- Modify: `web-ui/src/lib/types.ts`

### Step 1: Remove history-related types

Remove from `ClientMessage`:
- `subscribe` variant
- `unsubscribe` variant
- `request_history` variant

Remove from `ServerMessage`:
- `subscribe_ack` variant
- `history_page` variant

Remove:
- `HistoryEvent` interface
- `VibesEvent` type (if only used for history)
- `isSubscribeAckMessage` function
- `isHistoryPageMessage` function

### Step 2: Build and verify

Run: `cd web-ui && npm run build`
Expected: Success

### Step 3: Commit

```bash
git add web-ui/src/lib/types.ts
git commit -m "refactor(web-ui): remove history-related types"
```

---

## Task 11: Final Verification

### Step 1: Run all tests

```bash
cd vibes && just test
```
Expected: All tests pass

### Step 2: Build everything

```bash
cd vibes && just build
```
Expected: Success

### Step 3: Manual test

1. Start the daemon
2. Open web UI
3. Attach to a session
4. Generate some output
5. Refresh the page
6. Verify scrollback is replayed

### Step 4: Final commit

```bash
git add -A
git commit -m "chore: clean up any remaining history references"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | ScrollbackBuffer core | `pty/scrollback.rs`, `pty/mod.rs` |
| 2 | Integrate into PtySessionHandle | `pty/session.rs` |
| 3 | Add PtyReplay to protocol | `ws/protocol.rs` |
| 4 | Capture output in scrollback | `ws/connection.rs` |
| 5 | Send replay on attach | `ws/connection.rs` |
| 6 | Handle replay in frontend | `types.ts`, `ClaudeSession.tsx` |
| 7 | Remove history module | `vibes-core/src/history/`, `lib.rs`, `Cargo.toml` |
| 8 | Remove history from protocol | `ws/protocol.rs` |
| 9 | Remove history from connection | `ws/connection.rs` |
| 10 | Clean up frontend types | `types.ts` |
| 11 | Final verification | All |

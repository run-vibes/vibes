# 13: Scrollback Buffer & History Simplification

## Overview

Replace SQLite chat history with a simple in-memory scrollback buffer for PTY output. This simplifies the architecture now that PTY is the primary interface.

## Goals

1. Preserve terminal output across page refreshes
2. Remove unnecessary SQLite complexity
3. Keep memory usage predictable and efficient

## Non-Goals

- Persist history across daemon restarts (revisit later)
- Structured data persistence for analytics/learning (revisit later)
- Full terminal state serialization (cursor position, colors)

## Design

### ScrollbackBuffer

New module: `vibes-core/src/pty/scrollback.rs`

```rust
use std::collections::VecDeque;

/// Ring buffer for PTY output with fixed byte capacity.
/// When capacity is exceeded, oldest bytes are dropped.
pub struct ScrollbackBuffer {
    buffer: VecDeque<u8>,
    capacity: usize,
}

impl ScrollbackBuffer {
    /// Create buffer with specified capacity (default: 1MB)
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
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
        Self::new(1_048_576) // 1MB
    }
}
```

### Integration with PtySession

Add scrollback buffer to `PtySession`:

```rust
pub struct PtySession {
    // ... existing fields ...
    scrollback: Arc<Mutex<ScrollbackBuffer>>,
}
```

In `pty_output_reader()`, after reading data:

```rust
// Append to scrollback
if let Ok(mut scrollback) = handle.scrollback.lock() {
    scrollback.append(&data);
}

// Broadcast to clients (existing)
let encoded = base64::encode(&data);
state.broadcast_pty_event(PtyEvent::Output { session_id, data: encoded });
```

### Protocol Addition

New message in `ws/protocol.rs`:

```rust
pub enum ServerMessage {
    // ... existing variants ...

    /// Replay scrollback buffer on attach
    PtyReplay {
        session_id: String,
        data: String,  // base64-encoded
    },
}
```

### Attach Flow

When client sends `Attach { session_id }`:

1. Validate session exists
2. Add client to attached set
3. Get scrollback buffer contents
4. Send `PtyReplay { session_id, data }` (if buffer non-empty)
5. Send `AttachAck { session_id, cols, rows }`
6. Client now receives live output

### Frontend Handling

In `ClaudeSession.tsx`, add handler:

```typescript
case 'pty_replay':
  if (msg.session_id === sessionId && terminalRef.current) {
    terminalRef.current.write(msg.data);
  }
  break;
```

## Removal Scope

### Files to Delete

```
vibes-core/src/history/           # entire directory
├── mod.rs
├── store.rs
├── builder.rs
├── service.rs
└── migrations/
```

### Code to Remove

1. `vibes-core/src/lib.rs` - Remove `pub mod history;`
2. `vibes-server/src/ws/protocol.rs` - Remove:
   - `Subscribe`, `RequestHistory` (client messages)
   - `SubscribeAck`, `HistoryChunk` (server messages)
3. `vibes-server/src/ws/connection.rs` - Remove:
   - History catch-up logic
   - HistoryService initialization
   - Subscription tracking
4. `vibes-core/Cargo.toml` - Remove `rusqlite` if unused
5. Frontend - Remove history message handlers

## Memory Impact

- 1MB per active PTY session
- 10 concurrent sessions = 10MB total
- Negligible compared to typical daemon memory usage

## Implementation Order

1. Add `ScrollbackBuffer` with unit tests
2. Integrate into `PtySession`
3. Add `PtyReplay` to protocol
4. Update attach handler to send replay
5. Update frontend to handle replay
6. Remove SQLite history layer
7. Clean up dependencies

## Testing

- Unit: ScrollbackBuffer append, overflow, retrieval
- Unit: Empty buffer returns empty vec
- Integration: attach → replay → live output sequence
- Manual: refresh page, verify history preserved

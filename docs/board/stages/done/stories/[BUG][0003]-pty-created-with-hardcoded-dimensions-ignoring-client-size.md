---
id: BUG0003
title: PTY created with hardcoded dimensions ignoring client size
type: bug
status: done
priority: high
scope: core
depends: []
estimate: M
created: 2026-01-08
---

# PTY created with hardcoded dimensions ignoring client size

## Problem

PTY sessions are created with hardcoded 120x40 dimensions before the client can provide its
actual terminal size. This causes resize issues:

- **Mobile**: 120 columns is too wide, content doesn't wrap properly for small screens
- **Large monitors**: New sessions start at 120 width even when the display has much more space
- **Shells may not reflow**: Some shells capture the initial size and don't reflow content
  when resized later, leading to misaligned output

## Root Cause

In `vibes-server/src/ws/connection.rs`, the `PtyAttach` handler creates or attaches to PTY
sessions with hardcoded 120x40 dimensions:

```rust
// Line 379 - attaching to existing session:
(120, 40, scrollback_len)

// Line 407 - creating new session:
(120, 40, 0)
```

The expectation is that clients send `PtyResize` immediately after attach. However:
1. The PTY is already created with wrong dimensions
2. Shell initialization runs with wrong size
3. Content output before resize arrives is formatted for 120 columns

## Current Flow

```
Client: PtyAttach { session_id, name?, cwd? }
Server: Creates PTY at 120x40 ← WRONG SIZE
Server: PtyAttached { session_id, cols: 120, rows: 40 }
Client: PtyResize { cols: actual, rows: actual }
Server: Resizes PTY (too late, shell already initialized)
```

## Proposed Fix

Add optional `cols` and `rows` to `PtyAttach` message so clients can specify dimensions upfront:

```
Client: PtyAttach { session_id, name?, cwd?, cols: 80, rows: 24 }
Server: Creates PTY at 80x24 ← CORRECT SIZE
Server: PtyAttached { session_id, cols: 80, rows: 24 }
```

## Acceptance Criteria

- [ ] `PtyAttach` message accepts optional `cols` and `rows` fields
- [ ] New PTY sessions use client-provided dimensions (or sensible defaults if not provided)
- [ ] Attaching to existing sessions still works (existing session dimensions are preserved)
- [ ] Web UI sends terminal dimensions with initial attach
- [ ] CLI sends terminal dimensions with initial attach
- [ ] Mobile screens get appropriately sized PTY from start
- [ ] Large screens get appropriately sized PTY from start

## Implementation Notes

1. Update `ClientMessage::PtyAttach` in `vibes-server/src/ws/protocol.rs`:
   ```rust
   PtyAttach {
       session_id: String,
       name: Option<String>,
       cwd: Option<String>,
       cols: Option<u16>,  // NEW
       rows: Option<u16>,  // NEW
   }
   ```

2. Pass dimensions to PTY creation in `connection.rs`

3. Update web-ui `Session.tsx` to send dimensions with attach

4. Update CLI `claude.rs` to send dimensions with attach

5. Keep `PtyResize` working for subsequent resize events

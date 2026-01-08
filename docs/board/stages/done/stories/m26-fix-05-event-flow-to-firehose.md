---
id: B004
title: "Fix: Events Don't Flow from Claude Hooks to Firehose"
type: fix
status: done
priority: medium
epics: [core, cli, plugin-system]
depends: []
estimate:
created: 2026-01-03
updated: 2026-01-07
milestone: 26-assessment-framework
---

# Fix: Events Don't Flow from Claude Hooks to Firehose

> **For Claude:** Use superpowers:systematic-debugging to trace the event path.

## Problem

When running `vibes claude`, no session or assessment events appear in the firehose websocket. The websocket connection itself works (connection events appear), but Claude session events (MessageReceived, SessionStarted, etc.) and assessment events (LightweightSignal, etc.) are missing.

## Root Causes Found

### 1. Off-by-one error in `SeekPosition::End` for empty topics

In `vibes-iggy/src/iggy_log.rs`, when seeking to `End` on an empty topic:
1. `partition.current_offset` is 0 (default value for empty topics)
2. We computed `self.offset = 0 + 1 = 1`
3. First message via HTTP gets offset **0**
4. Consumer polling from offset **1** misses the event at offset 0

This explains why the WebSocket consumer (which uses `SeekPosition::End` for live mode) missed all hook events - when the topic was empty at server start, it was waiting for offset 1 while events went to offset 0.

### 2. Race condition: Iggy HTTP not ready when hooks fire

When starting `vibes claude` for the first time:
1. Daemon spawns, starts Iggy subprocess
2. Server waited 500ms fixed grace period
3. Server connected via TCP (ready before HTTP)
4. Health endpoint returned OK
5. CLI started Claude PTY, hooks fired
6. Hooks sent events via HTTP - but Iggy HTTP wasn't ready yet!

Iggy has **two separate listeners**: TCP (port 8090) and HTTP (port 3001). The TCP listener became ready before HTTP, so the fixed grace period wasn't enough.

### 3. CLI not wrapping events in StoredEvent

The `vibes event send` command was serializing raw `VibesEvent` but consumers expect `StoredEvent` format (with `event_id`). This caused deserialization failures that were silently dropped.

### 4. Injection hooks not sending to Iggy

Injection hooks (`session_start`, `user_prompt_submit`) used `vibes-hook-inject.sh` which tried to connect to a Unix socket (`/tmp/vibes-hooks.sock`) that was removed. Connection failed silently, returning `{}` and exiting 0, so events were never sent to Iggy.

Only `stop` worked because it used `vibes-hook-send.sh` → `vibes event send` → Iggy.

### 5. Hooks can't find vibes binary during development

During development, `vibes` is not on PATH - it's in `./target/release/vibes` or `./target/debug/vibes`. The hook scripts checked `command -v vibes` which returned false, so they silently skipped sending events.

## Fixes

### Fix 1: Check `messages_count` for empty topics

```rust
SeekPosition::End => {
    if topic_details.messages_count == 0 {
        // Topic is empty - start from 0 to catch the first message
        self.offset = 0;
    } else {
        // Topic has messages - start from one past the last
        self.offset = partition.current_offset.saturating_add(1);
    }
}
```

### Fix 2: Wait for HTTP + TCP readiness

Replaced fixed 500ms grace period with proper protocol polling in `IggyManager::wait_for_ready()`:

```rust
// Wait for both HTTP and TCP concurrently
let http_ready = self.wait_for_http_ready(start);
let tcp_ready = self.wait_for_tcp_ready(start);
let (http_result, tcp_result) = tokio::join!(http_ready, tcp_ready);
```

- Polls HTTP endpoint until any response
- Polls TCP by attempting connection
- 100ms retry interval, 30s timeout
- Checks if server crashed during wait

### Fix 3: Wrap events in StoredEvent in CLI

```rust
// In vibes-cli/src/commands/event.rs
let stored = StoredEvent::new(event);
let serialized = serde_json::to_vec(&stored)?;
```

### Fix 4: Update vibes-hook-inject.sh to use CLI

Changed injection hooks to send events via `vibes event send` instead of trying to connect to the removed Unix socket:

```bash
# Send event to Iggy via vibes CLI (fire-and-forget for event logging)
if command -v vibes &>/dev/null; then
    vibes event send --type hook --data "$EVENT_JSON" \
        ${VIBES_SESSION_ID:+--session "$VIBES_SESSION_ID"} 2>/dev/null || true
fi
```

### Fix 5: Set VIBES_BIN environment variable in PTY spawn

When spawning Claude, the PTY backend now sets `VIBES_BIN` to the current executable path:

```rust
// In vibes-core/src/pty/backend.rs
if let Ok(current_exe) = std::env::current_exe() {
    cmd.env("VIBES_BIN", current_exe);
}
```

Hook scripts now check for `VIBES_BIN` first:

```bash
# Use VIBES_BIN if set (for development), otherwise fall back to PATH
VIBES_CMD="${VIBES_BIN:-vibes}"
if [ -x "$VIBES_CMD" ] || command -v "$VIBES_CMD" &>/dev/null; then
    "$VIBES_CMD" event send --type hook --data "$EVENT_JSON" ...
fi
```

## Tasks

- [x] Trace hook installation in `vibes claude` (hooks work, HTTP API receives events)
- [x] Verify hook → server communication (HTTP → Iggy works)
- [x] Check EventLog writes to Iggy (TCP consumer reads events correctly)
- [x] Verify firehose consumer subscription (SeekPosition::End was the issue)
- [x] Fix off-by-one in SeekPosition::End for empty topics
- [x] Fix race condition: wait for HTTP + TCP readiness
- [x] Fix CLI: wrap events in StoredEvent
- [x] Fix injection hooks: use CLI instead of removed Unix socket
- [x] Fix development: set VIBES_BIN in PTY spawn for hook scripts to find binary
- [x] Add E2E tests: `test_http_events_received_by_tcp_consumer`, `test_http_events_received_by_live_consumer`, `test_vibes_event_send_cli_to_consumer`, `test_injection_hook_events_flow_to_firehose`, `test_real_backend_sets_vibes_bin_env`

## Acceptance Criteria

- [x] Running `vibes claude` and sending a message shows events in the firehose
- [x] Both session events and assessment events appear
- [x] Events appear in real-time (< 1 second latency)

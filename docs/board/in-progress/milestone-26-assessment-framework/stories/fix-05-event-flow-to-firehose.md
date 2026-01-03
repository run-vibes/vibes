---
created: 2026-01-03
status: done
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

## Tasks

- [x] Trace hook installation in `vibes claude` (hooks work, HTTP API receives events)
- [x] Verify hook → server communication (HTTP → Iggy works)
- [x] Check EventLog writes to Iggy (TCP consumer reads events correctly)
- [x] Verify firehose consumer subscription (SeekPosition::End was the issue)
- [x] Fix the broken link in the chain (fixed off-by-one in iggy_log.rs)
- [x] Add E2E tests: `test_http_events_received_by_tcp_consumer` and `test_http_events_received_by_live_consumer`

## Acceptance Criteria

- [x] Running `vibes claude` and sending a message shows events in the firehose
- [x] Both session events and assessment events appear
- [x] Events appear in real-time (< 1 second latency)

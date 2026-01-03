---
created: 2026-01-03
status: pending
---

# Fix: Events Don't Flow from Claude Hooks to Firehose

> **For Claude:** Use superpowers:systematic-debugging to trace the event path.

## Problem

When running `vibes claude`, no session or assessment events appear in the firehose websocket. The websocket connection itself works (connection events appear), but Claude session events (MessageReceived, SessionStarted, etc.) and assessment events (LightweightSignal, etc.) are missing.

## Investigation Path

Trace the event flow through each component:

```
Claude hooks → vibes-server HTTP/WS → EventLog (Iggy) → Firehose consumer → WebSocket broadcast
```

### 1. Are hooks installed and firing?

- Check `~/.claude/hooks/` for vibes hooks
- Add logging to confirm hooks execute on Claude events
- Verify hooks can reach the vibes server

### 2. Is the server receiving hook calls?

- Check server logs for incoming hook requests
- Verify the hook endpoint exists and is reachable

### 3. Are events written to Iggy?

- Check Iggy topics for events
- Verify the EventLog writer is connected
- Look for errors in event persistence

### 4. Is the firehose consumer reading?

- Verify consumer is subscribed to correct topics
- Check consumer logs for activity
- Confirm events are being broadcast to websocket

## Tasks

- [ ] Trace hook installation in `vibes claude`
- [ ] Verify hook → server communication
- [ ] Check EventLog writes to Iggy
- [ ] Verify firehose consumer subscription
- [ ] Fix the broken link in the chain
- [ ] Add E2E test: Claude event → firehose websocket

## Acceptance Criteria

- Running `vibes claude` and sending a message shows events in the firehose
- Both session events and assessment events appear
- Events appear in real-time (< 1 second latency)

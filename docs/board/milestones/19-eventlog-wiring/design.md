# Milestone: EventLog Wiring - Design Document

> Wire event producers to append events to EventLog instead of direct broadcast. This completes the consumer-based architecture designed in 4.4.2a.

## Overview

### The Problem

Currently, events are published via two separate paths:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CURRENT (BROKEN) FLOW                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  WebSocket Handler (ws/connection.rs)                               │
│       │                                                             │
│       ├──► broadcast_event() ──► tokio::broadcast ──► Firehose      │
│       │         ✓ Working but WRONG PATH                            │
│       │                                                             │
│       └──► event_log.append() ──► Iggy ──► Consumers                │
│                 ✗ NEVER CALLED                                      │
│                                                                     │
│  EventLog is empty. Consumers are starving.                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### The Solution

Route all events through EventLog. The WebSocket consumer (already implemented) will handle fan-out to the broadcast channel:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CORRECT FLOW                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Event Producers (WebSocket, PTY, HTTP handlers)                    │
│       │                                                             │
│       ▼                                                             │
│  event_log.append(event)  ◄── Single source of truth                │
│       │                                                             │
│       ├─────────────────────┬────────────────────┬─────────────────┐│
│       ▼                     ▼                    ▼                 ││
│  ┌─────────────┐     ┌─────────────┐     ┌──────────────┐          ││
│  │ WebSocket   │     │ Notification│     │ Assessment   │          ││
│  │ Consumer    │     │ Consumer    │     │ Consumer     │          ││
│  │ (live)      │     │ (live)      │     │ (replay)     │          ││
│  └──────┬──────┘     └──────┬──────┘     └──────────────┘          ││
│         │                   │                                       ││
│         ▼                   ▼                                       ││
│  broadcast channel     Push Notifications                           ││
│         │                                                           ││
│         ▼                                                           ││
│  Firehose WebSocket clients                                         ││
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Insight

The `broadcast_event()` method and `broadcast::Sender` are still needed - they provide efficient fan-out to multiple WebSocket clients. But they should be an **output** of the WebSocket consumer, not an **input** from event producers.

---

## Scope

### In Scope

1. Change event producers to call `event_log.append()` instead of `broadcast_event()`
2. Ensure WebSocket consumer broadcasts to the tokio channel
3. Update/fix architecture diagrams in documentation
4. Keep `broadcast_event()` as internal API for WebSocket consumer

### Out of Scope

1. IggyEventLog full SDK implementation (stub is acceptable for now)
2. New consumers or event types
3. Changes to the consumer infrastructure itself

---

## Implementation Plan

### Phase 1: Wire Event Producers to EventLog

#### Task 1.1: Make broadcast_event internal to WebSocket consumer

**Files:**
- Modify: `vibes-server/src/state.rs`

Change `broadcast_event` to be non-public or document it as internal:

```rust
/// Internal: Publish an event to all subscribed WebSocket clients.
///
/// NOTE: Event producers should NOT call this directly. Use `event_log.append()`
/// instead. This method is called by the WebSocket consumer after reading from
/// the EventLog.
pub(crate) fn broadcast_event(&self, event: VibesEvent) -> usize {
    self.event_broadcaster.send(event).unwrap_or(0)
}
```

**Tests:**
- [ ] Existing tests still pass

---

#### Task 1.2: Create helper method for async event appending

**Files:**
- Modify: `vibes-server/src/state.rs`

Add a helper that wraps the async append call:

```rust
/// Append an event to the EventLog.
///
/// This is the primary way to publish events. The event will be:
/// 1. Persisted to the EventLog (Iggy when available)
/// 2. Picked up by consumers (WebSocket, Notification, Assessment)
/// 3. Broadcast to connected clients via the WebSocket consumer
///
/// This method spawns a task to avoid blocking the caller.
pub fn append_event(&self, event: VibesEvent) {
    let event_log = Arc::clone(&self.event_log);
    tokio::spawn(async move {
        if let Err(e) = event_log.append(event).await {
            tracing::warn!("Failed to append event to EventLog: {}", e);
        }
    });
}
```

**Tests:**
- [ ] `test_append_event_writes_to_log`
- [ ] `test_append_event_does_not_block`

---

#### Task 1.3: Migrate WebSocket connection handler

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

Replace all `state.broadcast_event()` calls with `state.append_event()`:

| Line | Current | New |
|------|---------|-----|
| 134-136 | `state.broadcast_event(VibesEvent::ClientConnected {...})` | `state.append_event(VibesEvent::ClientConnected {...})` |
| 210-212 | `state.broadcast_event(VibesEvent::ClientDisconnected {...})` | `state.append_event(VibesEvent::ClientDisconnected {...})` |
| 319-322 | `state.broadcast_event(VibesEvent::SessionRemoved {...})` | `state.append_event(VibesEvent::SessionRemoved {...})` |
| 387-390 | `state.broadcast_event(VibesEvent::SessionCreated {...})` | `state.append_event(VibesEvent::SessionCreated {...})` |

**Tests:**
- [ ] Existing WebSocket tests still pass
- [ ] Events appear in EventLog after WebSocket actions

---

#### Task 1.4: Verify WebSocket consumer broadcasts correctly

**Files:**
- Review: `vibes-server/src/consumers/websocket.rs`
- Review: `vibes-server/src/lib.rs` (consumer startup)

The WebSocket consumer should already call `broadcast_event()` after reading from EventLog. Verify this is working.

**Tests:**
- [ ] `test_websocket_consumer_broadcasts_events` (existing)
- [ ] Integration test: event flows from append → consumer → firehose client

---

### Phase 2: Update Documentation

#### Task 2.1: Update consumers/mod.rs diagram

**Files:**
- Modify: `vibes-server/src/consumers/mod.rs`

The existing diagram at lines 9-22 is correct! Just verify it matches implementation.

---

#### Task 2.2: Update milestone-4.4-design.md EventFlow diagram

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.4-design.md`

The diagram at lines 35-72 references "EventBus". Update to clarify the distinction:

```
┌─────────────────────────────────────────────────────────────────────┐
│                           EventLog                                   │
│    (Iggy-backed persistent log - single source of truth)            │
└──────────────┬─────────────────────────────────────┬────────────────┘
               │                                     │
               ▼                                     ▼
    ┌──────────────────┐              ┌─────────────────────┐
    │ SessionCollector │              │ AssessmentProcessor │
    │   (from 4.3)     │              │      (new)          │
    └──────────────────┘              └──────────┬──────────┘
                                                 │
                    ┌────────────────────────────┼────────────────────────────┐
                    │                            │                            │
                    ▼                            ▼                            ▼
         ┌──────────────────┐        ┌───────────────────┐        ┌──────────────────┐
         │ LightweightLayer │        │  CheckpointMgr    │        │  CircuitBreaker  │
         │  (every message) │        │ (medium triggers) │        │  (intervention)  │
         └────────┬─────────┘        └─────────┬─────────┘        └────────┬─────────┘
                  │                            │                           │
                  └────────────────────────────┴───────────────────────────┘
```

---

#### Task 2.3: Update milestone-4.4.2a-design.md

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.4.2a-design.md`

Update the "Current vs Target" diagram to mark current state as completed:

```
BEFORE (Milestone 4.4.2a - COMPLETED):
                    ┌─────────────────┐
                    │ InMemoryEventLog│
                    │   (EventLog)    │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              ▼                             ▼
    ┌──────────────────┐          ┌──────────────────┐
    │ WebSocket        │          │ Notification     │
    │ Consumer         │          │ Consumer         │
    └──────────────────┘          └──────────────────┘

AFTER (This milestone - wire producers):
    Producers → event_log.append() → Consumers → Effects
```

---

#### Task 2.4: Update milestone-4.3-design.md capture pipeline diagram

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.3-design.md`

Update diagram at lines 49-53 to show EventLog instead of EventBus:

```
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │ HookReceiver │───▶│   EventLog   │───▶│     groove plugin        │  │
│  │ (extended)   │    │ (Iggy/Mem)   │    │   (consumer-based)       │  │
│  └──────────────┘    └──────────────┘    │  ┌────────────────────┐  │  │
```

---

#### Task 2.5: Add migration note to PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

Add entry documenting this architectural fix.

---

### Phase 3: Cleanup and Verification

#### Task 3.1: Search for remaining broadcast_event calls

**Command:**
```bash
grep -r "broadcast_event" vibes-server/src --include="*.rs" | grep -v "test" | grep -v "//"
```

Ensure no production code directly calls `broadcast_event` except the WebSocket consumer.

---

#### Task 3.2: Run full test suite

**Command:**
```bash
just test
just pre-commit
```

All tests must pass.

---

#### Task 3.3: Manual verification

1. Start vibes server with Iggy
2. Connect a firehose client
3. Create a session
4. Verify events flow: EventLog → Consumer → Firehose

---

## Checklist

### Phase 1: Wire Event Producers
- [ ] Task 1.1: Make broadcast_event internal/documented
- [ ] Task 1.2: Create append_event helper method
- [ ] Task 1.3: Migrate ws/connection.rs to append_event
- [ ] Task 1.4: Verify WebSocket consumer broadcasts correctly

### Phase 2: Update Documentation
- [ ] Task 2.1: Verify consumers/mod.rs diagram
- [ ] Task 2.2: Update milestone-4.4-design.md diagram
- [ ] Task 2.3: Update milestone-4.4.2a-design.md diagram
- [ ] Task 2.4: Update milestone-4.3-design.md diagram
- [ ] Task 2.5: Add migration note to PROGRESS.md

### Phase 3: Cleanup and Verification
- [ ] Task 3.1: Search for remaining broadcast_event calls
- [ ] Task 3.2: Run full test suite
- [ ] Task 3.3: Manual verification

---

## Exit Criteria

- [ ] All events flow through EventLog before reaching consumers
- [ ] WebSocket clients receive events via consumer → broadcast chain
- [ ] No direct `broadcast_event()` calls from event producers
- [ ] All architecture diagrams updated to reflect EventLog model
- [ ] `just pre-commit` passes
- [ ] Manual verification successful

---

## Decision Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Keep broadcast channel | Yes | Efficient fan-out to WebSocket clients |
| Spawn task for append | Yes | Non-blocking, fire-and-forget semantics |
| Update diagrams | Yes | Documentation must match implementation |
| Internal vs public | `pub(crate)` | Clear signal that producers shouldn't use directly |

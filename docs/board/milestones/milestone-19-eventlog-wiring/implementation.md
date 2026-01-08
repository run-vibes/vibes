# Milestone: EventLog Wiring - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire event producers to append events to EventLog instead of calling `broadcast_event()` directly. This completes the consumer-based architecture.

**Architecture:** Event producers call `event_log.append()`. The WebSocket consumer reads from EventLog and calls `broadcast_event()` for fan-out to firehose clients. This is the final step of the pub/sub → producer/consumer migration.

**Reference:** See `docs/plans/18-eventlog-wiring/design.md` for full design context.

---

## Phase 1: Wire Event Producers to EventLog

### Task 1.1: Add append_event helper to AppState

**Files:**
- Modify: `vibes-server/src/state.rs`

**Step 1: Write failing test**

Add to the test module in `state.rs`:

```rust
#[tokio::test]
async fn test_append_event_writes_to_log() {
    let state = AppState::new();

    let event = VibesEvent::SessionCreated {
        session_id: "test-session".to_string(),
        name: Some("Test".to_string()),
    };

    state.append_event(event.clone());

    // Give the spawned task time to complete
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Create a consumer and verify the event is in the log
    let mut consumer = state.event_log.consumer("test-reader").await.unwrap();
    consumer.seek(vibes_iggy::SeekPosition::Beginning).await.unwrap();

    let events = consumer.poll(10, std::time::Duration::from_millis(100)).await.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0].1 {
        VibesEvent::SessionCreated { session_id, .. } => {
            assert_eq!(session_id, "test-session");
        }
        _ => panic!("Expected SessionCreated event"),
    }
}
```

**Step 2: Implement append_event**

Add to `impl AppState`:

```rust
/// Append an event to the EventLog.
///
/// This is the primary way to publish events. The event will be:
/// 1. Persisted to the EventLog (Iggy when available, in-memory otherwise)
/// 2. Picked up by consumers (WebSocket, Notification, Assessment)
/// 3. Broadcast to connected clients via the WebSocket consumer
///
/// This method spawns a task to avoid blocking the caller.
/// If persistence fails, the error is logged but not propagated.
pub fn append_event(&self, event: VibesEvent) {
    let event_log = Arc::clone(&self.event_log);
    tokio::spawn(async move {
        if let Err(e) = event_log.append(event).await {
            tracing::warn!("Failed to append event to EventLog: {}", e);
        }
    });
}
```

**Step 3: Run test**

```bash
cargo test -p vibes-server test_append_event_writes_to_log
```

**Step 4: Commit**

```bash
git add vibes-server/src/state.rs
git commit -m "feat(server): add append_event helper for EventLog publishing"
```

---

### Task 1.2: Document broadcast_event as internal

**Files:**
- Modify: `vibes-server/src/state.rs`

**Step 1: Update broadcast_event documentation**

Change the doc comment on `broadcast_event`:

```rust
/// Broadcast an event to all subscribed WebSocket clients.
///
/// **Internal API:** Event producers should NOT call this directly.
/// Use [`append_event`] instead, which writes to the EventLog.
/// The WebSocket consumer will then call this method after reading
/// from the log.
///
/// This method is public for use by the WebSocket consumer in
/// `consumers::websocket`.
///
/// Returns the number of receivers that received the event.
/// Returns 0 if there are no active subscribers.
pub fn broadcast_event(&self, event: VibesEvent) -> usize {
    self.event_broadcaster.send(event).unwrap_or(0)
}
```

**Step 2: Commit**

```bash
git add vibes-server/src/state.rs
git commit -m "docs(server): document broadcast_event as internal API"
```

---

### Task 1.3: Migrate ClientConnected event

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Find and update the ClientConnected event**

Around line 134-136, change:

```rust
// OLD:
state.broadcast_event(VibesEvent::ClientConnected {
    client_id: conn_state.client_id.clone(),
});

// NEW:
state.append_event(VibesEvent::ClientConnected {
    client_id: conn_state.client_id.clone(),
});
```

**Step 2: Run tests**

```bash
cargo test -p vibes-server
```

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "refactor(server): route ClientConnected through EventLog"
```

---

### Task 1.4: Migrate ClientDisconnected event

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Find and update the ClientDisconnected event**

Around line 210-212, change:

```rust
// OLD:
state.broadcast_event(VibesEvent::ClientDisconnected {
    client_id: conn_state.client_id.clone(),
});

// NEW:
state.append_event(VibesEvent::ClientDisconnected {
    client_id: conn_state.client_id.clone(),
});
```

**Step 2: Run tests**

```bash
cargo test -p vibes-server
```

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "refactor(server): route ClientDisconnected through EventLog"
```

---

### Task 1.5: Migrate SessionRemoved event

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Find and update the SessionRemoved event**

Around line 319-322, change:

```rust
// OLD:
state.broadcast_event(VibesEvent::SessionRemoved {
    session_id: session_id.clone(),
    reason: "killed".to_string(),
});

// NEW:
state.append_event(VibesEvent::SessionRemoved {
    session_id: session_id.clone(),
    reason: "killed".to_string(),
});
```

**Step 2: Run tests**

```bash
cargo test -p vibes-server
```

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "refactor(server): route SessionRemoved through EventLog"
```

---

### Task 1.6: Migrate SessionCreated event

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`

**Step 1: Find and update the SessionCreated event**

Around line 387-390, change:

```rust
// OLD:
state.broadcast_event(VibesEvent::SessionCreated {
    session_id: created_id.clone(),
    name: session_name,
});

// NEW:
state.append_event(VibesEvent::SessionCreated {
    session_id: created_id.clone(),
    name: session_name,
});
```

**Step 2: Run tests**

```bash
cargo test -p vibes-server
```

**Step 3: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git commit -m "refactor(server): route SessionCreated through EventLog"
```

---

### Task 1.7: Search for remaining broadcast_event calls

**Step 1: Search codebase**

```bash
grep -rn "broadcast_event" vibes-server/src --include="*.rs" | grep -v test | grep -v "/// " | grep -v "//!"
```

**Expected results:** Only the definition in `state.rs` and usage in `consumers/websocket.rs` (or `consumers/mod.rs`).

**If other calls found:** Migrate them to `append_event()`.

**Step 2: Verify no remaining producer calls**

Ensure the only places calling `broadcast_event` are:
1. Its definition in `state.rs`
2. The WebSocket consumer broadcasting after reading from EventLog
3. Test code

**Step 3: Commit if changes made**

```bash
git add -A
git commit -m "refactor(server): migrate remaining broadcast_event calls to append_event"
```

---

### Task 1.8: Run full test suite

**Step 1: Run all tests**

```bash
just test
```

**Step 2: Run pre-commit checks**

```bash
just pre-commit
```

**Step 3: Fix any failures**

If tests fail, investigate and fix. Common issues:
- Tests that directly call `broadcast_event` may need updating
- Consumer timing issues (events take time to flow through)

---

## Phase 2: Update Documentation

### Task 2.1: Update milestone-4.4-design.md

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.4-design.md`

**Step 1: Find the EventFlow diagram (around line 35-72)**

Update the diagram header to clarify:

```
### Event Flow

Events flow through the EventLog (Iggy-backed when available). Each consumer
reads independently with its own offset.

```
┌─────────────────────────────────────────────────────────────────────┐
│                      EventLog (Iggy/InMemory)                        │
│         (Single source of truth - append only)                       │
└──────────────┬─────────────────────────────────────┬────────────────┘
               │                                     │
               ▼                                     ▼
    ┌──────────────────┐              ┌─────────────────────┐
    │ SessionCollector │              │ AssessmentProcessor │
    │   (consumer)     │              │    (consumer)       │
    └──────────────────┘              └──────────┬──────────┘
```

**Step 2: Update Key Boundaries section (around line 74-78)**

```markdown
### Key Boundaries

1. **Append boundary**: Event producers → `event_log.append()` (async, non-blocking)
2. **Consumer boundary**: EventLog → Consumers (poll-based with offset tracking)
3. **Broadcast boundary**: WebSocket Consumer → `broadcast_event()` → Firehose clients
4. **Intervention boundary**: CircuitBreaker → InterventionMgr → Claude hooks
```

**Step 3: Commit**

```bash
git add docs/plans/14-continual-learning/milestone-4.4-design.md
git commit -m "docs: update milestone-4.4 diagrams for EventLog architecture"
```

---

### Task 2.2: Update milestone-4.3-design.md

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.3-design.md`

**Step 1: Find the capture pipeline diagram (around line 49-53)**

Update to reference EventLog:

```
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │ HookReceiver │───▶│   EventLog   │───▶│     groove plugin        │  │
│  │ (extended)   │    │ (Iggy/Mem)   │    │   (consumer-based)       │  │
│  └──────────────┘    └──────────────┘    │  ┌────────────────────┐  │  │
```

**Step 2: Update section headers mentioning EventBus**

Search for "EventBus" and update references:
- "EventBus Extension" → "EventLog Integration"
- "VibesEvent::Hook" description → note it flows through EventLog

**Step 3: Commit**

```bash
git add docs/plans/14-continual-learning/milestone-4.3-design.md
git commit -m "docs: update milestone-4.3 diagrams for EventLog architecture"
```

---

### Task 2.3: Update milestone-4.4.2a-design.md

**Files:**
- Modify: `docs/plans/14-continual-learning/milestone-4.4.2a-design.md`

**Step 1: Add completion note at top**

After the overview, add:

```markdown
> **Status:** Infrastructure complete. Event producers wired in milestone 18.
```

**Step 2: Update the "BEFORE/AFTER" diagram**

Mark the migration as complete:

```markdown
### Migration Status

**COMPLETED (4.4.2a + 18):**

```
Event Producers (WebSocket handler, PTY, etc.)
       │
       ▼
event_log.append()
       │
       ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    EventLog (Iggy or InMemory)                       │
└──────────────┬─────────────────────────────────────┬────────────────┘
               │                                     │
               ▼                                     ▼
    ┌──────────────────┐              ┌─────────────────────┐
    │ WebSocket        │              │ Notification        │
    │ Consumer         │              │ Consumer            │
    │     │            │              └─────────────────────┘
    │     ▼            │
    │ broadcast_event()│
    │     │            │
    │     ▼            │
    │ Firehose clients │
    └──────────────────┘
```
```

**Step 3: Commit**

```bash
git add docs/plans/14-continual-learning/milestone-4.4.2a-design.md
git commit -m "docs: mark EventLog producer wiring as complete in 4.4.2a"
```

---

### Task 2.4: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Find the Phase 4 section**

Add an entry for this milestone:

```markdown
### Milestone 18: EventLog Wiring ✓

Wire event producers to append to EventLog instead of direct broadcast.
Completes the consumer-based architecture from 4.4.2a.

- [x] Add `append_event()` helper to AppState
- [x] Migrate all `broadcast_event()` calls to `append_event()`
- [x] Update architecture diagrams in docs
```

**Step 2: Add changelog entry**

Add to the changelog table:

```markdown
| 2024-XX-XX | EventLog wiring complete - events flow through EventLog to consumers |
```

**Step 3: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: add EventLog wiring milestone to PROGRESS.md"
```

---

## Phase 3: Verification

### Task 3.1: Integration test

**Files:**
- Review/add: `vibes-server/tests/eventlog_flow.rs` (or add to existing integration tests)

**Step 1: Write integration test**

```rust
#[tokio::test]
async fn test_events_flow_through_eventlog_to_firehose() {
    // 1. Create AppState
    let state = Arc::new(AppState::new());

    // 2. Subscribe to the broadcast channel (simulating firehose)
    let mut rx = state.subscribe_events();

    // 3. Start the WebSocket consumer (normally done in lib.rs)
    let event_log = Arc::clone(&state.event_log);
    let broadcaster = state.event_broadcaster();
    let mut manager = ConsumerManager::new(event_log);
    start_websocket_consumer(&mut manager, broadcaster).await.unwrap();

    // 4. Append an event via append_event
    state.append_event(VibesEvent::SessionCreated {
        session_id: "integration-test".to_string(),
        name: Some("Test Session".to_string()),
    });

    // 5. Wait for event to flow through: append → consumer → broadcast
    let received = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        rx.recv()
    ).await.expect("timeout waiting for event").expect("channel closed");

    // 6. Verify
    match received {
        VibesEvent::SessionCreated { session_id, .. } => {
            assert_eq!(session_id, "integration-test");
        }
        _ => panic!("Expected SessionCreated"),
    }

    manager.shutdown();
    manager.wait_for_shutdown().await;
}
```

**Step 2: Run test**

```bash
cargo test -p vibes-server test_events_flow_through_eventlog_to_firehose
```

**Step 3: Commit**

```bash
git add vibes-server/tests/
git commit -m "test(server): add integration test for EventLog → Consumer → Broadcast flow"
```

---

### Task 3.2: Manual verification

**Step 1: Build and start server**

```bash
just build
./target/debug/vibes serve
```

**Step 2: Connect firehose client**

In another terminal:
```bash
websocat ws://localhost:7432/api/events/firehose
```

**Step 3: Trigger events**

In another terminal, connect a CLI session:
```bash
./target/debug/vibes claude
```

**Step 4: Verify firehose receives events**

The firehose client should see:
- `ClientConnected` event
- `SessionCreated` event
- Other events as the session progresses

**Step 5: Check Iggy metrics**

```bash
curl http://localhost:3001/stats
```

Verify `messages` count increases as events are appended.

---

### Task 3.3: Final commit and PR

**Step 1: Ensure all tests pass**

```bash
just pre-commit
```

**Step 2: Create summary commit if needed**

```bash
git add -A
git commit -m "feat(server): complete EventLog wiring for consumer-based architecture"
```

**Step 3: Push and create PR**

```bash
git push -u origin eventlog-wiring
gh pr create --title "feat(server): wire event producers to EventLog" --body "..."
```

---

## Summary

| Phase | Tasks | Key Changes |
|-------|-------|-------------|
| 1. Wire Producers | 1.1-1.8 | `append_event()` helper, migrate 4 event sites |
| 2. Update Docs | 2.1-2.4 | Fix diagrams in 4.4, 4.3, 4.4.2a, PROGRESS.md |
| 3. Verification | 3.1-3.3 | Integration test, manual verification |

**Total: ~15 tasks**

---

## Exit Criteria

- [ ] All `broadcast_event()` calls in production code migrated to `append_event()`
- [ ] WebSocket consumer is the only caller of `broadcast_event()`
- [ ] Events flow: Producer → EventLog → Consumer → Broadcast → Clients
- [ ] All architecture diagrams updated
- [ ] Integration test passes
- [ ] Manual verification successful
- [ ] `just pre-commit` passes

---
id: F006
title: feat: UUIDv7 Global Event Sequencing
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-02
updated: 2026-01-07
milestone: milestone-36-firehose-infinite-scroll
---

# feat: UUIDv7 Global Event Sequencing

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Add globally unique, time-ordered event IDs using UUIDv7 to replace partition-scoped offsets as the primary event identifier.

## Context

Iggy partitions events by session_id, meaning each partition has its own offset counter starting from 0. Multiple events across different partitions can have the same offset, breaking unique identification in the firehose UI (causing multi-select bugs).

UUIDv7 provides:
- **Globally unique:** No collisions across partitions
- **Time-ordered:** Monotonically increasing, sortable
- **No coordination:** Each producer generates IDs independently

## Tasks

### Task 1: Create StoredEvent wrapper type

**Files:**
- Modify: `vibes-core/src/events/types.rs`

**Steps:**
1. Create `StoredEvent` struct with `event_id: Uuid` and `event: VibesEvent`
2. Generate UUIDv7 in `StoredEvent::new(event)` constructor
3. Implement `Partitionable` for `StoredEvent` (delegate to inner event)
4. Write tests for event_id generation and serialization
5. Run tests: `cargo test -p vibes-core`
6. Commit: `feat(core): add StoredEvent wrapper with UUIDv7 event_id`

### Task 2: Update EventLog to use StoredEvent

**Files:**
- Modify: `vibes-server/src/state.rs`
- Modify: `vibes-server/src/consumers/*.rs`

**Steps:**
1. Change `EventLog<VibesEvent>` to `EventLog<StoredEvent>`
2. Wrap events in `StoredEvent::new()` when appending
3. Update consumers to extract event from StoredEvent
4. Update broadcast channel to include event_id
5. Run tests: `cargo test -p vibes-server`
6. Commit: `feat(server): store events with UUIDv7 event_id`

### Task 3: Update firehose to use event_id

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Update `EventWithOffset` to include `event_id: Uuid`
2. Update `FirehoseEventMessage` to include `event_id: Uuid`
3. Extract event_id from StoredEvent when building messages
4. Add tests verifying event_id appears in JSON output
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): include event_id in WebSocket messages`

### Task 4: Unify design system types and update frontend

**Files:**
- Create: `design-system/src/types/events.ts`
- Modify: `design-system/src/compositions/StreamView/StreamView.tsx`
- Modify: `design-system/src/compositions/EventInspector/EventInspector.tsx`
- Modify: `design-system/src/index.ts`
- Modify: `web-ui/src/hooks/useFirehose.ts`
- Modify: `web-ui/src/pages/Firehose.tsx`

**Design system refactor:**
1. Create unified `DisplayEvent` type (merges StreamEvent + InspectorEvent)
2. Rename `ContextEvent.offset` → `ContextEvent.relativePosition` for clarity
3. Update StreamView to use `DisplayEvent`
4. Update EventInspector to use `DisplayEvent`
5. Export new types, remove old StreamEvent/InspectorEvent exports

**Hook changes:**
6. Rename hook's `StreamEvent` → `FirehoseEvent` (raw server data)
7. Add `event_id: string` field to `FirehoseEvent`

**Firehose page changes:**
8. Create `toDisplayEvent()` mapping function
9. Use `event_id` as unique identifier (remove array index workaround)
10. Update selectedEvent/contextEvents lookups to use event_id

**Verification:**
11. Run design-system tests: `npm test --workspace=@vibes/design-system -- --run`
12. Run web-ui tests: `npm test --workspace=web-ui -- --run`
13. Commit: `feat(frontend): unify event types and use event_id for identification`

## Acceptance Criteria

- [x] All VibesEvent instances have a unique UUIDv7 event_id
- [x] event_id appears in WebSocket message JSON
- [x] Design system has unified `DisplayEvent` type (single source of truth)
- [x] Frontend uses event_id for unique identification
- [x] Row selection in firehose works correctly (no multi-select bugs)
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Commit, push, and create PR

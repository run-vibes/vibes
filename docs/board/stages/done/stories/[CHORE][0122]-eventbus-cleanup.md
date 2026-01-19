---
id: CHORE0122
title: "Chore: Remove Dead EventBus Code"
type: chore
status: done
priority: medium
scope: web-ui/26-infinite-event-stream
depends: []
estimate:
created: 2026-01-01
---

# Chore: Remove Dead EventBus Code

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Delete the deprecated EventBus code from vibes-core now that EventLog is the active event system.

## Context

The codebase migrated from pub/sub EventBus to producer/consumer EventLog pattern. The old EventBus code remains in vibes-core but is no longer used anywhere. This cleanup removes dead code and clarifies the architecture.

See [design.md](../design.md) for the EventLog architecture.

## Tasks

### Task 1: Verify EventBus is Unused

**Steps:**
1. Search for any remaining usages of `EventBus`, `MemoryEventBus`, or `EventBusError`
2. Confirm no production code depends on these types
3. Document any test-only usages that need migration

**Verification:**
```bash
grep -r "EventBus\|MemoryEventBus" --include="*.rs" vibes-server/ vibes-cli/ plugins/
```

Expected: No matches (only vibes-core should have these)

### Task 2: Delete EventBus Files

**Files:**
- Delete: `vibes-core/src/events/bus.rs`
- Delete: `vibes-core/src/events/memory.rs`

**Steps:**
1. Remove the files
2. Update `vibes-core/src/events/mod.rs` to remove exports
3. Run `cargo check` to verify no compilation errors

**Commit:** `chore(core): remove deprecated EventBus code`

### Task 3: Update Module Exports

**File:** `vibes-core/src/events/mod.rs`

**Steps:**
1. Remove `pub mod bus;` and `pub mod memory;`
2. Remove re-exports: `EventBus`, `EventSeq`, `MemoryEventBus`
3. Keep EventLog re-exports from vibes-iggy

**Verification:**
```bash
cargo check -p vibes-core
cargo test -p vibes-core
```

**Commit:** `chore(core): update events module exports`

## Acceptance Criteria

- [x] No EventBus code remains in vibes-core
- [x] `cargo check` passes
- [x] `cargo test` passes
- [x] No other crates break

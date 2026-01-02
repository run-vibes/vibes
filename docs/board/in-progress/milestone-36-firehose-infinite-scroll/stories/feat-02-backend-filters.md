---
created: 2026-01-02
status: done
---

# feat: Backend Filter Support

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Add WebSocket protocol support for server-side event filtering by type and session.

## Context

See [design.md](../design.md) for filter protocol specification. Filters should be applied server-side for efficiency—no point sending events the client will discard.

## Tasks

### Task 1: Add set_filters message handler

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Add `SetFilters` message type with `types: Vec<String>` and `session: Option<String>`
2. Store active filters in connection state
3. Apply filters when streaming live events
4. Write tests for filter state management
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): add set_filters message handler`

### Task 2: Apply filters to fetch_older

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Pass active filters to EventLog query
2. Ensure `fetch_older` respects current filter state
3. Write tests for filtered pagination
4. Run tests: `cargo test -p vibes-server firehose`
5. Commit: `feat(firehose): apply filters to fetch_older queries`

### Task 3: Reset to latest on filter change

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. When filters change, fetch fresh batch of latest matching events
2. Send new `EventsBatch` to replace client state
3. Clear any pending pagination state
4. Write tests for filter change behavior
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): reset to latest on filter change`

## Acceptance Criteria

- [x] `set_filters` message updates connection filter state
- [x] Live events respect active filters
- [x] `fetch_older` respects active filters
- [x] Filter change triggers fresh latest batch
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Commit, push, and create PR

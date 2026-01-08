---
id: F001
title: feat: Backend Pagination Support
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

# feat: Backend Pagination Support

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Add WebSocket protocol support for fetching older events with offset-based pagination.

## Context

See [design.md](../design.md) for WebSocket protocol specification. The firehose currently only streams live events. We need to add `fetch_older` message handling to enable infinite scroll.

## Tasks

### Task 1: Add offset field to event messages

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Add `offset: u64` field to outgoing event messages
2. Track current offset from EventLog subscription
3. Include offset in both live events and batch responses
4. Write tests for offset tracking
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): add offset field to event messages`

### Task 2: Implement fetch_older handler

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Add `FetchOlder` message type with `before_offset` and `limit` fields
2. Implement handler that queries EventLog for events before offset
3. Return `EventsBatch` response with events, oldest_offset, has_more
4. Write tests for pagination logic
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): implement fetch_older pagination`

### Task 3: Send initial batch on connection

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. On WebSocket connect, fetch latest 100 events
2. Send `EventsBatch` with events and `has_more` flag
3. Then begin streaming live events
4. Write integration test for connection flow
5. Run tests: `cargo test -p vibes-server firehose`
6. Commit: `feat(firehose): send initial events batch on connect`

## Acceptance Criteria

- [x] All event messages include offset field
- [x] `fetch_older` message returns older events with correct offsets
- [x] `has_more` flag correctly indicates more history available
- [x] Initial connection sends batch of latest events
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Commit, push, and create PR

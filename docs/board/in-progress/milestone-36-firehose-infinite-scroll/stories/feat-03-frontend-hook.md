---
created: 2026-01-02
status: pending
---

# feat: Rewrite useFirehose Hook

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Rewrite the useFirehose hook to support offset tracking, pagination requests, and proper state management for infinite scroll.

## Context

See [design.md](../design.md) for state model. The current hook buffers events in memory with no pagination support. The new hook must track offsets and manage the isFollowing state.

## Tasks

### Task 1: Define new state model

**Files:**
- Modify: `web-ui/src/hooks/useFirehose.ts`

**Steps:**
1. Define `FirehoseState` interface per design doc
2. Add offset tracking: `oldestOffset`, `newestOffset`
3. Add pagination state: `isLoadingOlder`, `hasMore`
4. Add following state: `isFollowing`
5. Write tests for state initialization
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `feat(web-ui): define useFirehose state model`

### Task 2: Handle events_batch messages

**Files:**
- Modify: `web-ui/src/hooks/useFirehose.ts`

**Steps:**
1. Parse `events_batch` messages with offset metadata
2. For initial batch: set events, update offsets, set hasMore
3. For pagination: prepend events, update oldestOffset
4. Maintain sort order by offset
5. Write tests for batch handling
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `feat(web-ui): handle events_batch in useFirehose`

### Task 3: Implement fetchOlder function

**Files:**
- Modify: `web-ui/src/hooks/useFirehose.ts`

**Steps:**
1. Add `fetchOlder()` function that sends `fetch_older` message
2. Set `isLoadingOlder` while waiting for response
3. Prevent duplicate requests (debounce/guard)
4. Write tests for fetch behavior
5. Run tests: `npm test --workspace=web-ui`
6. Commit: `feat(web-ui): implement fetchOlder in useFirehose`

### Task 4: Implement filter management

**Files:**
- Modify: `web-ui/src/hooks/useFirehose.ts`

**Steps:**
1. Add `setFilters()` function that sends `set_filters` message
2. Store filter state locally
3. Reset following state on filter change
4. Write tests for filter behavior
5. Run tests: `npm test --workspace=web-ui`
6. Commit: `feat(web-ui): implement filter management in useFirehose`

## Acceptance Criteria

- [ ] Hook tracks oldest and newest offsets
- [ ] Initial batch populates state correctly
- [ ] `fetchOlder()` requests more history
- [ ] `setFilters()` sends filter update to server
- [ ] `isFollowing` state managed correctly
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Commit, push, and create PR

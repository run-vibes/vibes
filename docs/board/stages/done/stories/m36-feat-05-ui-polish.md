---
id: F005
title: feat: UI Polish and Cleanup
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-02
updated: 2026-01-07
milestone: 36-firehose-infinite-scroll
---

# feat: UI Polish and Cleanup

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Add filter controls, millisecond timestamps, "Jump to latest" button, and remove obsolete Pause/Clear buttons.

## Context

See [design.md](../design.md) for UX requirements. This story polishes the firehose into a proper log viewer with working controls.

## Tasks

### Task 1: Add filter controls

**Files:**
- Modify: `web-ui/src/pages/Firehose.tsx`

**Steps:**
1. Add type filter chips (toggle buttons for event types)
2. Add session filter text input
3. Add search input (client-side text filter)
4. Wire controls to useFirehose setFilters/local state
5. Write tests for filter UI interactions
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `feat(web-ui): add filter controls to Firehose`

### Task 2: Update timestamp display

**Files:**
- Modify: `web-ui/src/components/` (event rendering)

**Steps:**
1. Change timestamp format to `HH:mm:ss.SSS`
2. Ensure consistent formatting across all event types
3. Write tests for timestamp formatting
4. Run tests: `npm test --workspace=web-ui`
5. Commit: `feat(web-ui): show millisecond precision timestamps`

### Task 3: Add "Jump to latest" button

**Files:**
- Modify: `web-ui/src/pages/Firehose.tsx`

**Steps:**
1. Add floating button that appears when not following
2. On click: scroll to bottom, resume following
3. Style as subtle floating action button
4. Write tests for button visibility logic
5. Run tests: `npm test --workspace=web-ui`
6. Commit: `feat(web-ui): add Jump to latest button`

### Task 4: Remove Pause and Clear buttons

**Files:**
- Modify: `web-ui/src/pages/Firehose.tsx`

**Steps:**
1. Remove Pause button (scrolling up is natural pause)
2. Remove Clear button (events are persistent, clearing doesn't make sense)
3. Clean up any related state/handlers
4. Update tests to reflect removed functionality
5. Run tests: `npm test --workspace=web-ui`
6. Commit: `refactor(web-ui): remove Pause and Clear buttons from Firehose`

### Task 5: Integration testing

**Files:**
- Create: `web-ui/src/pages/Firehose.test.tsx`

**Steps:**
1. Write integration tests for full firehose flow
2. Test filter interactions
3. Test scroll behavior with mocked data
4. Run tests: `npm test --workspace=web-ui`
5. Commit: `test(web-ui): add Firehose integration tests`

## Acceptance Criteria

- [ ] Type filter chips toggle event visibility
- [ ] Session filter narrows to specific session
- [ ] Search filters event content
- [ ] Timestamps show milliseconds (HH:mm:ss.SSS)
- [ ] "Jump to latest" appears when scrolled up
- [ ] Pause and Clear buttons removed
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Move milestone to done: `just board done milestone-36-firehose-infinite-scroll`
4. Commit, push, and create PR

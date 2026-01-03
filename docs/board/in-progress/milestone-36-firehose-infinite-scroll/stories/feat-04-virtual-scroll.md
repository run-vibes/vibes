---
created: 2026-01-02
status: done
---

# feat: Virtualized Scroll View

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Implement virtualized scrolling with @tanstack/react-virtual to handle 100K+ events efficiently with scroll-to-top pagination triggers.

## Context

See [design.md](../design.md) for UX requirements. Virtual scrolling is essential for performance—we cannot render 100K DOM nodes. The library handles viewport culling.

## Tasks

### Task 1: Add @tanstack/react-virtual dependency

**Files:**
- Modify: `web-ui/package.json`

**Steps:**
1. Add `@tanstack/react-virtual` version 3.x
2. Run `npm install`
3. Verify build still works
4. Commit: `chore(web-ui): add @tanstack/react-virtual dependency`

### Task 2: Create StreamView component

**Files:**
- Create: `web-ui/src/components/StreamView.tsx`
- Create: `web-ui/src/components/StreamView.test.tsx`

**Steps:**
1. Create virtualized list component using useVirtualizer
2. Accept events array and render callback
3. Handle dynamic row heights
4. Write basic rendering tests
5. Run tests: `npm test --workspace=web-ui`
6. Commit: `feat(web-ui): create StreamView virtualized component`

### Task 3: Implement scroll-to-top pagination

**Files:**
- Modify: `web-ui/src/components/StreamView.tsx`

**Steps:**
1. Detect when user scrolls near top (within threshold)
2. Call `onLoadMore` callback when threshold reached
3. Show loading indicator at top while fetching
4. Preserve scroll position when prepending events
5. Write tests for scroll detection
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `feat(web-ui): implement scroll-to-top pagination in StreamView`

### Task 4: Implement auto-follow behavior

**Files:**
- Modify: `web-ui/src/components/StreamView.tsx`

**Steps:**
1. Track if user is "at bottom" (within threshold of end)
2. Export `isFollowing` state based on scroll position
3. Auto-scroll to bottom when new events arrive if following
4. Break following when user scrolls up
5. Write tests for auto-follow logic
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `feat(web-ui): implement auto-follow in StreamView`

## Acceptance Criteria

- [x] Virtual scrolling handles 10K+ events without performance issues
- [x] Scrolling to top triggers pagination request
- [x] Scroll position preserved when older events load
- [x] Auto-follow works correctly (scroll up breaks, scroll to bottom resumes)
- [x] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Regenerate board: `just board`
3. Commit, push, and create PR

---
id: FEAT0046
title: Learning actions (enable/disable/delete)
type: feat
status: in-progress
priority: medium
epics: [plugin-system]
depends: [FEAT0045]
estimate: 2h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Learning actions (enable/disable/delete)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add learning management actions with confirmation dialogs for enable, disable, and delete operations.

## Context

Users need to manage learnings from the dashboard. Actions include disabling (prevents injection), enabling (re-enables), and deleting (permanent removal). All destructive actions require confirmation. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create confirmation dialog

**Files:**
- Create: `web-ui/src/components/dashboard/ConfirmDialog.tsx`
- Create: `web-ui/src/components/dashboard/ConfirmDialog.css`

**Steps:**
1. Create reusable `ConfirmDialog` component:
   - Title and message props
   - Confirm and cancel buttons
   - Destructive mode (red confirm button)
   - Modal overlay
2. Style with CRT theme
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add ConfirmDialog component`

### Task 2: Create action components

**Files:**
- Create: `web-ui/src/components/dashboard/learnings/LearningActions.tsx`
- Modify: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Create `LearningActions` component:
   - Enable button (shown when disabled/deprecated)
   - Disable button (shown when active)
   - Delete button (always shown)
   - Loading states during action
2. Add action methods to useDashboard:
   ```typescript
   disableLearning(id: string): Promise<void>
   enableLearning(id: string): Promise<void>
   deleteLearning(id: string): Promise<void>
   ```
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add LearningActions component`

### Task 3: Add backend handlers

**Files:**
- Modify: `plugins/vibes-groove/src/dashboard/handler.rs`

**Steps:**
1. Add action handlers in `DashboardHandler`:
   ```rust
   async fn handle_disable_learning(&self, id: LearningId) -> Result<()>
   async fn handle_enable_learning(&self, id: LearningId) -> Result<()>
   async fn handle_delete_learning(&self, id: LearningId) -> Result<()>
   ```
2. Update learning store on action
3. Broadcast updates to subscribers
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add learning action handlers`

### Task 4: Implement optimistic updates

**Files:**
- Modify: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Implement optimistic update pattern:
   - Update local state immediately
   - Revert on error
   - Show error toast on failure
2. Handle concurrent updates gracefully
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add optimistic updates for learning actions`

### Task 5: Integrate and test

**Files:**
- Modify: `web-ui/src/components/dashboard/learnings/LearningDetail.tsx`
- Create: `web-ui/src/components/dashboard/learnings/__tests__/LearningActions.test.tsx`

**Steps:**
1. Add LearningActions to LearningDetail component
2. Wire up confirmation dialogs:
   - Disable: "This learning won't be injected. Continue?"
   - Delete: "This will permanently remove the learning. Continue?"
3. Write tests:
   - Test action button visibility
   - Test confirmation dialog flow
   - Test optimistic updates
   - Test error handling
4. Run: `npm test --workspace=web-ui -- --run`
5. Run: `cargo test -p vibes-groove dashboard::actions`
6. Commit: `test(groove): add learning action tests`

## Acceptance Criteria

- [ ] Enable button shown for disabled/deprecated learnings
- [ ] Disable button shown for active learnings
- [ ] Delete button always available
- [ ] Confirmation dialogs for destructive actions
- [ ] Optimistic updates with rollback on error
- [ ] List updates after action
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0046`
3. Commit, push, and create PR

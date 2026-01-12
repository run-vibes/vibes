---
id: FEAT0083
title: Solutions tab with actions
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0080]
estimate: 2h
created: 2026-01-11
milestone: 36-openworld-dashboard
---

# Solutions tab with actions

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement solutions viewer with apply/dismiss actions.

## Context

The Solutions tab displays suggested solutions grouped by status, with actions to apply or dismiss them. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Create solution components

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/SolutionsList.tsx`
- Create: `web-ui/src/components/dashboard/openworld/SolutionItem.tsx`
- Create: `web-ui/src/components/dashboard/openworld/SolutionActions.tsx`
- Create: `web-ui/src/components/dashboard/openworld/SolutionConfidenceBadge.tsx`

**Steps:**
1. Create `SolutionsList` - solutions grouped by status:
   - Pending Review
   - Applied
   - Dismissed
2. Create `SolutionItem` - row showing gap, source, confidence
3. Create `SolutionActions` - Apply/Dismiss buttons
4. Create `SolutionConfidenceBadge` - confidence indicator
5. Style with CRT design system
6. Run: `npm run typecheck --workspace=web-ui`
7. Commit: `feat(web-ui): add solution components`

### Task 2: Add solution actions to backend

**Files:**
- Modify: `plugins/vibes-groove/src/dashboard/handler.rs`

**Steps:**
1. Add `handle_apply_solution()` handler
2. Add `handle_dismiss_solution()` handler
3. Update OpenWorldStore to mark solutions
4. Run: `cargo test -p vibes-groove dashboard`
5. Commit: `feat(groove): add solution action handlers`

### Task 3: Wire into DashboardOpenWorld

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`
- Modify: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Subscribe to `OpenWorldSolutions` topic
2. Add `applySolution()` and `dismissSolution()` to hook
3. Add confirmation dialog for actions
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): wire solutions tab to backend`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/SolutionsList.test.tsx`

**Steps:**
1. Test solutions grouping by status
2. Test action buttons
3. Test confirmation dialogs
4. Run: `npm test --workspace=web-ui -- --run`
5. Commit: `test(web-ui): add solutions tab tests`

## Acceptance Criteria

- [ ] Solutions grouped by status (Pending, Applied, Dismissed)
- [ ] Apply/Dismiss actions work
- [ ] Confirmation dialog before actions
- [ ] Backend handlers update solution status
- [ ] Follows CRT design system
- [ ] Tests pass

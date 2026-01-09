---
id: FEAT0045
title: Learnings page (split view)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0043]
estimate: 4h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Learnings page (split view)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement split view with filterable list and detail panel for browsing and managing learnings.

## Context

The learnings page is the primary interface for exploring extracted learnings. It uses a split-panel design with filters on the left and detail view on the right. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create filter components

**Files:**
- Create: `web-ui/src/components/dashboard/learnings/LearningsFilters.tsx`
- Create: `web-ui/src/components/dashboard/learnings/LearningsFilters.css`

**Steps:**
1. Create filter dropdowns:
   - Scope: Project, User, Global
   - Category: Correction, ErrorRecovery, Pattern, Preference
   - Status: Active, Disabled, UnderReview, Deprecated
   - Sort: Value, Confidence, Usage, Recency
2. Implement filter state management
3. Emit filter changes to parent
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add LearningsFilters component`

### Task 2: Create list component

**Files:**
- Create: `web-ui/src/components/dashboard/learnings/LearningsList.tsx`
- Create: `web-ui/src/components/dashboard/learnings/LearningStatusBadge.tsx`
- Create: `web-ui/src/components/dashboard/learnings/ValueBar.tsx`

**Steps:**
1. Create `LearningsList`:
   - Virtualized list for performance
   - Learning item with title, category, status
   - Value indicator
   - Selection highlighting
2. Create `LearningStatusBadge`:
   - Color-coded status indicators
   - Active (green), Disabled (gray), UnderReview (yellow), Deprecated (red)
3. Create `ValueBar`:
   - Visual value indicator (-1 to +1 range)
   - Color gradient from red to green
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add learnings list components`

### Task 3: Create detail component

**Files:**
- Create: `web-ui/src/components/dashboard/learnings/LearningDetail.tsx`
- Create: `web-ui/src/components/dashboard/learnings/LearningDetail.css`

**Steps:**
1. Create `LearningDetail`:
   - Full learning content display
   - Metrics: value, confidence, session count
   - Source information
   - Attribution history summary
   - Action buttons placeholder (FEAT0046)
2. Style with CRT theme
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add LearningDetail component`

### Task 4: Implement page layout

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardLearnings.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardLearnings.css`

**Steps:**
1. Implement split panel layout:
   - Left: Filters + List (40%)
   - Right: Detail (60%)
2. Subscribe to Learnings topic with filters
3. Subscribe to LearningDetail on selection
4. Implement responsive behavior:
   - Stack panels on mobile
   - Show/hide detail with back button
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): implement DashboardLearnings page`

### Task 5: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/learnings/__tests__/LearningsList.test.tsx`
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardLearnings.test.tsx`

**Steps:**
1. Write component tests
2. Write page integration tests:
   - Test filter application
   - Test list rendering
   - Test selection and detail display
   - Test responsive behavior
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add learnings page tests`

## Acceptance Criteria

- [ ] Split panel layout with filters, list, and detail
- [ ] Filters update list in real-time
- [ ] Sorting works correctly
- [ ] Selection shows detail panel
- [ ] Status badges show correct colors
- [ ] Value bar shows visual indicator
- [ ] Responsive stacking on mobile
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0045`
3. Commit, push, and create PR

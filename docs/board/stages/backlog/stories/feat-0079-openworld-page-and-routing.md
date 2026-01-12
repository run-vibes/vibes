---
id: FEAT0079
title: OpenWorld page and routing
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-11
milestone: 44-openworld-dashboard
---

# OpenWorld page and routing

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add OpenWorld tab to the groove dashboard with internal tabs for Novelty, Gaps, Solutions, and Activity.

## Context

This is the first story for milestone 44. It creates the page structure and routing for the openworld dashboard. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Add OpenWorld to dashboard tabs

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardLayout.tsx`

**Steps:**
1. Add "OpenWorld" to the tabs array
2. Add route for `/dashboard/openworld`
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add openworld tab to dashboard`

### Task 2: Create DashboardOpenWorld page

**Files:**
- Create: `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardOpenWorld.css`

**Steps:**
1. Create page component with internal tabs:
   - Novelty (default)
   - Gaps
   - Solutions
   - Activity
2. Add placeholder content for each tab
3. Style following CRT design system
4. Export from `web-ui/src/pages/dashboard/index.ts`
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): create DashboardOpenWorld page`

### Task 3: Add tests

**Files:**
- Create: `web-ui/src/pages/dashboard/DashboardOpenWorld.test.tsx`

**Steps:**
1. Test tab rendering
2. Test tab switching
3. Test routing integration
4. Run: `npm test --workspace=web-ui -- --run`
5. Commit: `test(web-ui): add DashboardOpenWorld tests`

## Acceptance Criteria

- [ ] OpenWorld tab appears in dashboard navigation
- [ ] `/dashboard/openworld` route works
- [ ] Internal tabs (Novelty, Gaps, Solutions, Activity) switch correctly
- [ ] Follows CRT design system styling
- [ ] Tests pass

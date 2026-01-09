---
id: FEAT0042
title: Dashboard layout and routing
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Dashboard layout and routing

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Create dashboard navigation structure extending groove subnav with internal tabs for each page.

## Context

The groove dashboard surfaces data from milestones 30-32. The layout extends the existing groove subnav pattern and provides internal navigation between Overview, Learnings, Attribution, Strategy, and Health pages. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Update App routing

**Files:**
- Modify: `web-ui/src/App.tsx`

**Steps:**
1. Add Dashboard to `grooveSubnavItems`
2. Create dashboard layout route with child routes
3. Add routes for each dashboard page:
   - `/groove/dashboard/overview`
   - `/groove/dashboard/learnings`
   - `/groove/dashboard/attribution`
   - `/groove/dashboard/strategy`
   - `/groove/dashboard/health`
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add dashboard routes`

### Task 2: Create DashboardLayout

**Files:**
- Create: `web-ui/src/pages/dashboard/DashboardLayout.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardLayout.css`

**Steps:**
1. Create `DashboardLayout.tsx`:
   - Internal tabs: Overview, Learnings, Attribution, Strategy, Health
   - Follow AssessmentLayout pattern
   - Use NavLink for active tab styling
2. Create `DashboardLayout.css`:
   - Tab bar styling
   - Content area layout
   - CRT theme integration
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add DashboardLayout component`

### Task 3: Create placeholder pages

**Files:**
- Create: `web-ui/src/pages/dashboard/DashboardOverview.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardLearnings.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardAttribution.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardStrategy.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardHealth.tsx`
- Create: `web-ui/src/pages/dashboard/index.ts`

**Steps:**
1. Create placeholder component for each page
2. Each should show page title and "Coming soon" message
3. Create index.ts with exports
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add dashboard placeholder pages`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardLayout.test.tsx`

**Steps:**
1. Write tests:
   - Test tab navigation renders all tabs
   - Test active tab highlighting
   - Test navigation between pages
2. Run: `npm test --workspace=web-ui -- --run`
3. Commit: `test(web-ui): add dashboard layout tests`

## Acceptance Criteria

- [ ] Dashboard appears in groove subnav
- [ ] Internal tabs for Overview, Learnings, Attribution, Strategy, Health
- [ ] Active tab highlighting works
- [ ] Navigation between pages works
- [ ] All placeholder pages render
- [ ] Tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0042`
3. Commit, push, and create PR

---
id: FEAT0044
title: Overview page with cards
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0043]
estimate: 3h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Overview page with cards

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement grid of summary cards with drill-down links for the dashboard overview page.

## Context

The overview page provides at-a-glance system status with cards for learnings, attribution, strategy, and health. Each card shows key metrics and links to the detailed page. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create card components

**Files:**
- Create: `web-ui/src/components/dashboard/TrendCard.tsx`
- Create: `web-ui/src/components/dashboard/TrendCard.css`

**Steps:**
1. Create `TrendCard` component:
   - Sparkline placeholder (replaced in FEAT0050)
   - Primary metric with label
   - Trend indicator (↑/↓/→)
   - Optional secondary metrics
2. Style with CRT theme
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add TrendCard component`

### Task 2: Create domain-specific cards

**Files:**
- Create: `web-ui/src/components/dashboard/LearningsCard.tsx`
- Create: `web-ui/src/components/dashboard/AttributionCard.tsx`
- Create: `web-ui/src/components/dashboard/StrategyCard.tsx`
- Create: `web-ui/src/components/dashboard/HealthCard.tsx`

**Steps:**
1. Create `LearningsCard`:
   - Total/active/deprecated counts
   - Recent learnings list (last 5)
   - Link to learnings page
2. Create `AttributionCard`:
   - Top contributors (top 3)
   - Negative impact warnings
   - Link to attribution page
3. Create `StrategyCard`:
   - Distribution summary
   - Active experiments count
   - Link to strategy page
4. Create `HealthCard`:
   - System status (green/yellow/red)
   - Subsystem status bars
   - Link to health page
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): add dashboard cards`

### Task 3: Update overview page

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOverview.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardOverview.css`

**Steps:**
1. Subscribe to Overview topic via useDashboard
2. Implement responsive grid layout:
   - 2x2 on desktop
   - 1 column on mobile
3. Wire up card click navigation
4. Add loading and error states
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): implement DashboardOverview`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/__tests__/TrendCard.test.tsx`
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardOverview.test.tsx`

**Steps:**
1. Write card component tests
2. Write overview page tests:
   - Test loading state
   - Test error state
   - Test card rendering
   - Test navigation
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add overview page tests`

## Acceptance Criteria

- [ ] Overview page shows 4 summary cards
- [ ] Cards display correct metrics from WebSocket data
- [ ] Trend indicators show direction
- [ ] Clicking cards navigates to detail pages
- [ ] Responsive grid works on mobile
- [ ] Loading and error states display correctly
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0044`
3. Commit, push, and create PR

---
id: FEAT0049
title: Health page
type: feat
status: done
priority: medium
scope: plugin-system
depends: [FEAT0043]
estimate: 2h
created: 2026-01-09
---

# Health page

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement system health monitoring page showing subsystem status, adaptive parameters, and recent activity.

## Context

The health page provides operational visibility into the groove system. It shows status for each subsystem (extraction, attribution, strategy), adaptive parameter trends, and recent activity. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create status components

**Files:**
- Create: `web-ui/src/components/dashboard/health/SystemStatusBanner.tsx`
- Create: `web-ui/src/components/dashboard/health/SubsystemCard.tsx`

**Steps:**
1. Create `SystemStatusBanner`:
   - Overall status indicator (Operational, Degraded, Error)
   - Color-coded: Green/Yellow/Red
   - Last check timestamp
2. Create `SubsystemCard`:
   - Subsystem name
   - Status indicator (●/◐/○)
   - Key metrics (events processed, errors, latency)
   - Optional warning/error message
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add health status components`

### Task 2: Create parameter and activity components

**Files:**
- Create: `web-ui/src/components/dashboard/health/AdaptiveParamsTable.tsx`
- Create: `web-ui/src/components/dashboard/health/RecentActivity.tsx`

**Steps:**
1. Create `AdaptiveParamsTable`:
   - Table of adaptive parameters
   - Columns: Name, Current, Mean, Trend
   - Trend arrow indicator
   - Sparkline placeholder (updated in FEAT0050)
2. Create `RecentActivity`:
   - Activity feed showing recent events
   - Event type icons
   - Timestamp and description
   - Auto-scrolling with pause on hover
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add health parameter and activity components`

### Task 3: Implement page layout

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardHealth.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardHealth.css`

**Steps:**
1. Implement `DashboardHealth`:
   - Subscribe to Health topic
   - System status banner at top
   - Grid of subsystem cards
   - Adaptive parameters table
   - Recent activity feed
2. Layout:
   - Banner: full width
   - Subsystem cards: 3-column grid
   - Parameters and Activity: 2-column split
3. Style with CRT theme
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): implement DashboardHealth page`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/health/__tests__/SubsystemCard.test.tsx`
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardHealth.test.tsx`

**Steps:**
1. Write component tests:
   - Test status banner states
   - Test subsystem status indicators
   - Test parameter table rendering
   - Test activity feed
2. Write page integration tests:
   - Test data subscription
   - Test status updates
   - Test layout
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add health page tests`

## Acceptance Criteria

- [ ] System status banner shows overall health
- [ ] Subsystem cards show extraction, attribution, strategy status
- [ ] Status indicators use correct colors (green/yellow/red)
- [ ] Adaptive parameters table shows trends
- [ ] Recent activity feed updates in real-time
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0049`
3. Commit, push, and create PR

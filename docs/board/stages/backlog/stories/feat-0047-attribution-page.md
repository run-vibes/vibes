---
id: FEAT0047
title: Attribution page (leaderboard + timeline)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0043]
estimate: 4h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Attribution page (leaderboard + timeline)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement dual-view attribution page with leaderboard and session timeline views.

## Context

The attribution page shows which learnings help or hurt sessions. The leaderboard view ranks learnings by value contribution. The timeline view shows session-by-session attribution history. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create leaderboard components

**Files:**
- Create: `web-ui/src/components/dashboard/attribution/Leaderboard.tsx`
- Create: `web-ui/src/components/dashboard/attribution/ContributorCard.tsx`
- Create: `web-ui/src/components/dashboard/attribution/NegativeImpact.tsx`
- Create: `web-ui/src/components/dashboard/attribution/AblationCoverage.tsx`

**Steps:**
1. Create `Leaderboard`:
   - Ranked list of top contributors
   - Period selector (7/30/90 days)
   - Total value summary
2. Create `ContributorCard`:
   - Learning title and category
   - Value contribution with bar
   - Confidence indicator
   - Session count
3. Create `NegativeImpact`:
   - Warning section for negative learnings
   - Quick action to disable
4. Create `AblationCoverage`:
   - Progress bar showing experiment coverage
   - Stats: covered, pending, excluded
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): add attribution leaderboard components`

### Task 2: Create timeline components

**Files:**
- Create: `web-ui/src/components/dashboard/attribution/SessionTimeline.tsx`
- Create: `web-ui/src/components/dashboard/attribution/SessionTimelineItem.tsx`

**Steps:**
1. Create `SessionTimeline`:
   - Grouped by day (Today, Yesterday, etc.)
   - Infinite scroll or pagination
   - Filter by outcome (positive/negative)
2. Create `SessionTimelineItem`:
   - Session info (time, duration)
   - Activated learnings list
   - Outcome summary
   - Click to expand details
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add attribution timeline components`

### Task 3: Implement page layout

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardAttribution.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardAttribution.css`
- Create: `web-ui/src/components/dashboard/attribution/AttributionTabs.tsx`

**Steps:**
1. Create `AttributionTabs`:
   - Leaderboard / Timeline toggle
   - Tab state management
2. Implement `DashboardAttribution`:
   - Tab navigation
   - Subscribe to Attribution or SessionTimeline topic based on tab
   - Period selector updates subscription
3. Style with CRT theme
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): implement DashboardAttribution page`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/attribution/__tests__/Leaderboard.test.tsx`
- Create: `web-ui/src/components/dashboard/attribution/__tests__/SessionTimeline.test.tsx`
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardAttribution.test.tsx`

**Steps:**
1. Write component tests
2. Write page integration tests:
   - Test tab switching
   - Test period selector
   - Test leaderboard rendering
   - Test timeline grouping
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add attribution page tests`

## Acceptance Criteria

- [ ] Tab toggle between Leaderboard and Timeline views
- [ ] Leaderboard shows ranked contributors
- [ ] Period selector updates data (7/30/90 days)
- [ ] Negative impact section with warnings
- [ ] Ablation coverage progress bar
- [ ] Timeline groups sessions by day
- [ ] Timeline items expand to show details
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0047`
3. Commit, push, and create PR

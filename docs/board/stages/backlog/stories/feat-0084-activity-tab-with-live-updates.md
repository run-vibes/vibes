---
id: FEAT0084
title: Activity tab with live updates
type: feat
status: pending
priority: medium
epics: [plugin-system]
depends: [FEAT0080]
estimate: 2h
created: 2026-01-11
milestone: 44-openworld-dashboard
---

# Activity tab with live updates

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement real-time response activity feed with stats and live event stream.

## Context

The Activity tab shows graduated response activity in real-time, subscribing to the OPENWORLD_STREAM for live updates. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Create activity components

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/ActivityStats.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ActivityFeed.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ActivityItem.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ResponseActionBadge.tsx`

**Steps:**
1. Create `ActivityStats` with 3 cards:
   - Outcomes Today (processed count)
   - Negative Rate (percentage)
   - Exploration Adjustment (current bonus)
2. Create `ActivityFeed` - scrolling list of events
3. Create `ActivityItem` - single event with timestamp, type, details
4. Create `ResponseActionBadge` - colored action type indicator
5. Add live indicator (pulsing dot)
6. Style with CRT design system
7. Run: `npm run typecheck --workspace=web-ui`
8. Commit: `feat(web-ui): add activity tab components`

### Task 2: Wire into DashboardOpenWorld

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`

**Steps:**
1. Subscribe to `OpenWorldActivity` topic
2. Implement live update handling
3. Auto-scroll feed on new events
4. Show live indicator when connected
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): wire activity tab to backend`

### Task 3: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/ActivityFeed.test.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ActivityStats.test.tsx`

**Steps:**
1. Test stats rendering
2. Test feed with events
3. Test live indicator
4. Test auto-scroll behavior
5. Run: `npm test --workspace=web-ui -- --run`
6. Commit: `test(web-ui): add activity tab tests`

## Acceptance Criteria

- [ ] Stats cards display outcomes, negative rate, exploration
- [ ] Activity feed shows recent events
- [ ] Live updates from OPENWORLD_STREAM
- [ ] Live indicator shows connection status
- [ ] Follows CRT design system
- [ ] Tests pass

---
id: FEAT0080
title: OpenWorld backend data providers
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0079]
estimate: 3h
created: 2026-01-11
milestone: 44-openworld-dashboard
---

# OpenWorld backend data providers

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add WebSocket topics and handlers for openworld data in the dashboard backend.

## Context

This story adds the data layer that connects the openworld components (from M34) to the dashboard WebSocket system. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Add openworld types to dashboard

**Files:**
- Modify: `plugins/vibes-groove/src/dashboard/types.rs`

**Steps:**
1. Add new topics to `DashboardTopic`:
   - `OpenWorldOverview`
   - `OpenWorldGaps`
   - `OpenWorldGapDetail(GapId)`
   - `OpenWorldSolutions`
   - `OpenWorldActivity`
2. Add data structures:
   - `OpenWorldOverviewData`
   - `GapCounts`
   - `GapListData`
   - `GapBrief`
   - `GapDetailData`
   - `OpenWorldActivityData`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add openworld dashboard types`

### Task 2: Add openworld handlers

**Files:**
- Modify: `plugins/vibes-groove/src/dashboard/handler.rs`

**Steps:**
1. Add `handle_openworld_overview()` - query hook stats, gap counts
2. Add `handle_openworld_gaps()` - query store with filters
3. Add `handle_openworld_gap_detail()` - query store for single gap
4. Add `handle_openworld_solutions()` - query store for solutions
5. Add `handle_openworld_activity()` - return recent events
6. Wire handlers in message routing
7. Run: `cargo test -p vibes-groove dashboard`
8. Commit: `feat(groove): add openworld dashboard handlers`

### Task 3: Update frontend hooks

**Files:**
- Modify: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Add TypeScript types matching backend
2. Add subscription helpers for openworld topics
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `feat(web-ui): add openworld dashboard types`

## Acceptance Criteria

- [x] All openworld topics defined in DashboardTopic enum
- [x] Handlers query OpenWorldStore and OpenWorldHook
- [x] Frontend types match backend structures
- [x] useDashboard hook supports openworld subscriptions
- [x] Tests pass

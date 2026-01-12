---
id: FEAT0081
title: Novelty tab components
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0080]
estimate: 2h
created: 2026-01-11
milestone: 44-openworld-dashboard
---

# Novelty tab components

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement novelty detection visualization with stats cards and cluster list.

## Context

The Novelty tab shows the current state of novelty detection: adaptive threshold, pending outliers, and recent clusters. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Create novelty components

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/NoveltyStats.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ClusterList.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ClusterItem.tsx`

**Steps:**
1. Create `NoveltyStats` with 3 cards:
   - Threshold (current adaptive value)
   - Pending Outliers (count / max)
   - Clusters (total formed)
2. Create `ClusterList` - table of recent clusters
3. Create `ClusterItem` - single row with category, members, age
4. Style with CRT design system
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): add novelty tab components`

### Task 2: Wire into DashboardOpenWorld

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`

**Steps:**
1. Subscribe to `OpenWorldOverview` topic
2. Render NoveltyStats and ClusterList in Novelty tab
3. Add loading state
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): wire novelty tab to backend`

### Task 3: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/NoveltyStats.test.tsx`
- Create: `web-ui/src/components/dashboard/openworld/ClusterList.test.tsx`

**Steps:**
1. Test stats rendering
2. Test cluster list with data
3. Test empty state
4. Run: `npm test --workspace=web-ui -- --run`
5. Commit: `test(web-ui): add novelty tab tests`

## Acceptance Criteria

- [x] Stats cards display threshold, pending, clusters
- [x] Cluster list shows recent clusters with details
- [x] Subscribes to OpenWorldOverview topic
- [x] Follows CRT design system
- [x] Tests pass

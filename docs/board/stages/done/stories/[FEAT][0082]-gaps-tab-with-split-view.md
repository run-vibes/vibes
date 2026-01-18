---
id: FEAT0082
title: Gaps tab with split view
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0080]
estimate: 3h
created: 2026-01-11
milestone: 36-openworld-dashboard
---

# Gaps tab with split view

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement capability gaps browser with filterable list and detail panel.

## Context

The Gaps tab provides a split view for browsing capability gaps, similar to the Learnings page pattern. See [design.md](../../../milestones/44-openworld-dashboard/design.md).

## Tasks

### Task 1: Create gap list components

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/GapsList.tsx`
- Create: `web-ui/src/components/dashboard/openworld/GapsFilters.tsx`
- Create: `web-ui/src/components/dashboard/openworld/GapItem.tsx`
- Create: `web-ui/src/components/dashboard/openworld/GapSeverityBadge.tsx`

**Steps:**
1. Create `GapsList` - filterable list of gaps
2. Create `GapsFilters` - dropdowns for:
   - Severity: Critical, Medium, Low
   - Status: Open, Investigating, Resolved, WontFix
   - Category: CodePattern, Error, Performance, Security
3. Create `GapItem` - row showing severity, description, counts
4. Create `GapSeverityBadge` - colored severity indicator
5. Style with CRT design system
6. Run: `npm run typecheck --workspace=web-ui`
7. Commit: `feat(web-ui): add gap list components`

### Task 2: Create gap detail components

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/GapDetail.tsx`
- Create: `web-ui/src/components/dashboard/openworld/FailureRecordList.tsx`

**Steps:**
1. Create `GapDetail` - full gap info panel showing:
   - Severity, status, category, context
   - Failure records
   - Solutions preview
2. Create `FailureRecordList` - failure entries
3. Style with CRT design system
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add gap detail components`

### Task 3: Wire into DashboardOpenWorld

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`

**Steps:**
1. Subscribe to `OpenWorldGaps` topic with filters
2. Subscribe to `OpenWorldGapDetail` on selection
3. Implement split panel layout
4. Handle gap selection state
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): wire gaps tab to backend`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/openworld/GapsList.test.tsx`
- Create: `web-ui/src/components/dashboard/openworld/GapDetail.test.tsx`

**Steps:**
1. Test gap list rendering
2. Test filtering
3. Test detail panel
4. Run: `npm test --workspace=web-ui -- --run`
5. Commit: `test(web-ui): add gaps tab tests`

## Acceptance Criteria

- [ ] Gap list shows all gaps with severity badges
- [ ] Filters work for severity, status, category
- [ ] Selecting gap shows detail panel
- [ ] Detail shows failure records
- [ ] Follows CRT design system
- [ ] Tests pass

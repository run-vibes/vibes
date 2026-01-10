---
id: FEAT0048
title: Strategy page (distributions + overrides)
type: feat
status: in-progress
priority: high
epics: [plugin-system]
depends: [FEAT0043]
estimate: 3h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Strategy page (distributions + overrides)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement strategy visualization page with category distributions and per-learning overrides.

## Context

The strategy page shows how the adaptive strategies system selects injection methods. It displays category-level distributions and per-learning specializations. See [design.md](../../../milestones/33-groove-dashboard/design.md) and [M32 design](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create distribution components

**Files:**
- Create: `web-ui/src/components/dashboard/strategy/DistributionCard.tsx`
- Create: `web-ui/src/components/dashboard/strategy/StrategyBar.tsx`

**Steps:**
1. Create `DistributionCard`:
   - Category and context type header
   - Strategy variant weights as bars
   - Session count
   - Last updated timestamp
2. Create `StrategyBar`:
   - Horizontal bar for weight visualization
   - Color-coded by strategy variant
   - Percentage label
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add strategy distribution components`

### Task 2: Create override components

**Files:**
- Create: `web-ui/src/components/dashboard/strategy/OverridesList.tsx`
- Create: `web-ui/src/components/dashboard/strategy/OverrideItem.tsx`

**Steps:**
1. Create `OverridesList`:
   - List of learning overrides
   - Filter: All / Specialized only
   - Sortable by session count
2. Create `OverrideItem`:
   - Learning ID with link to learnings page
   - Status: Inheriting / Specialized
   - Session count and threshold progress
   - Expand to show weights if specialized
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add strategy override components`

### Task 3: Implement page layout

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardStrategy.tsx`
- Create: `web-ui/src/pages/dashboard/DashboardStrategy.css`
- Create: `web-ui/src/components/dashboard/strategy/StrategyTabs.tsx`

**Steps:**
1. Create `StrategyTabs`:
   - Distributions / Overrides toggle
   - Tab state management
2. Implement `DashboardStrategy`:
   - Tab navigation
   - Subscribe to StrategyDistributions or StrategyOverrides topic
   - Grid layout for distribution cards
   - List layout for overrides
3. Style with CRT theme
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): implement DashboardStrategy page`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/components/dashboard/strategy/__tests__/DistributionCard.test.tsx`
- Create: `web-ui/src/pages/dashboard/__tests__/DashboardStrategy.test.tsx`

**Steps:**
1. Write component tests:
   - Test distribution card rendering
   - Test strategy bar percentages
   - Test override status display
2. Write page integration tests:
   - Test tab switching
   - Test distribution grid
   - Test override filtering
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add strategy page tests`

## Acceptance Criteria

- [ ] Tab toggle between Distributions and Overrides views
- [ ] Distribution cards show category weights
- [ ] Strategy bars visualize weights correctly
- [ ] Override list shows inheritance status
- [ ] Specialized overrides can expand to show weights
- [ ] Filter for specialized-only works
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0048`
3. Commit, push, and create PR

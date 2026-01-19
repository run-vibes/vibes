---
id: FEAT0050
title: Sparkline and chart components
type: feat
status: done
priority: medium
scope: plugin-system
depends: [FEAT0044]
estimate: 3h
created: 2026-01-09
---

# Sparkline and chart components

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Create reusable chart components using visx for sparklines, trend charts, and value indicators.

## Context

The dashboard uses sparklines and charts to visualize trends. These components are shared across overview cards, health page, and detail views. Uses visx (already in project) with CRT styling. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Create sparkline component

**Files:**
- Create: `web-ui/src/components/charts/Sparkline.tsx`
- Create: `web-ui/src/components/charts/Sparkline.css`

**Steps:**
1. Create `Sparkline` component:
   - Props: data points, width, height, color
   - Use @visx/shape for line
   - Use @visx/scale for axes
   - Use @visx/responsive for sizing
2. Apply CRT styling:
   - Phosphor green default color
   - Subtle glow effect
   - Monospace font for labels
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add Sparkline component`

### Task 2: Create trend chart component

**Files:**
- Create: `web-ui/src/components/charts/TrendChart.tsx`
- Create: `web-ui/src/components/charts/TrendChart.css`

**Steps:**
1. Create `TrendChart` component:
   - Full-size chart for detail views
   - X-axis: time
   - Y-axis: value
   - Multiple series support
   - Tooltip on hover
2. Features:
   - Zoom/pan optional
   - Axis labels
   - Legend for multiple series
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): add TrendChart component`

### Task 3: Create indicator components

**Files:**
- Create: `web-ui/src/components/charts/ProgressBar.tsx`
- Create: `web-ui/src/components/charts/ValueBar.tsx`
- Create: `web-ui/src/components/charts/index.ts`

**Steps:**
1. Create `ProgressBar`:
   - Horizontal progress indicator
   - Percentage and label
   - Color customizable
2. Create `ValueBar`:
   - Value indicator for -1 to +1 range
   - Color gradient: red → yellow → green
   - Zero marker
3. Create index.ts with exports
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add progress and value bar components`

### Task 4: Integrate into dashboard

**Files:**
- Modify: `web-ui/src/components/dashboard/TrendCard.tsx`
- Modify: `web-ui/src/components/dashboard/health/AdaptiveParamsTable.tsx`

**Steps:**
1. Update `TrendCard` to use real `Sparkline`
2. Update `AdaptiveParamsTable` to use `Sparkline` for trends
3. Verify visual consistency
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): integrate chart components`

### Task 5: Add tests

**Files:**
- Create: `web-ui/src/components/charts/__tests__/Sparkline.test.tsx`
- Create: `web-ui/src/components/charts/__tests__/ValueBar.test.tsx`

**Steps:**
1. Write tests:
   - Test Sparkline renders with data
   - Test TrendChart renders axes
   - Test ProgressBar percentage
   - Test ValueBar color gradient
2. Run: `npm test --workspace=web-ui -- --run`
3. Commit: `test(web-ui): add chart component tests`

## Acceptance Criteria

- [ ] Sparkline renders inline charts
- [ ] TrendChart renders full charts with axes
- [ ] ProgressBar shows percentage completion
- [ ] ValueBar shows -1 to +1 range with colors
- [ ] CRT styling applied to all charts
- [ ] Overview cards use real sparklines
- [ ] Health page uses sparklines for params
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0050`
3. Commit, push, and create PR

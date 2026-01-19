---
id: REFACTOR0069
title: Dashboard visual consistency with CRT theme
type: refactor
status: done
priority: medium
scope: web-ui
depends: []
estimate: 3h
created: 2026-01-09
---

# Dashboard visual consistency with CRT theme

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Align the groove dashboard pages with the established CRT visual style used in `/firehose`, `/sessions`, and `/groove/assessment/status`.

## Context

The new dashboard section (Overview, Learnings, Attribution, Strategy, Health) was built with a simpler visual style that doesn't match the CRT aesthetic of other pages. This creates an inconsistent user experience.

### Current Issues

1. **Missing header panel** - Dashboard pages lack the bordered header with glowing title
2. **No phosphor glow** - Titles don't have the characteristic `text-shadow` effect
3. **Simpler panel structure** - Cards don't follow the `.panel` pattern with headers
4. **Missing borders** - Content areas lack the `1px solid var(--border)` styling

### Reference Pattern (from Firehose/Sessions)

```css
/* Header with glowing title */
.page-header {
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  background: var(--surface);
}

.page-title {
  font-family: var(--font-display);
  font-size: var(--font-size-xl);
  color: var(--phosphor);
  text-shadow: 0 0 10px var(--phosphor-glow);
  letter-spacing: 0.1em;
}

/* Panel structure */
.panel {
  border: 1px solid var(--border);
  background: var(--surface);
}

.panel-header {
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-display);
  letter-spacing: 0.1em;
  border-bottom: 1px solid var(--border);
}
```

## Tasks

### Task 1: Update DashboardLayout with header

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardLayout.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardLayout.css`

**Steps:**
1. Add header section with glowing title:
   ```tsx
   <div className="dashboard-header">
     <h1 className="dashboard-title">GROOVE DASHBOARD</h1>
     {/* Tab navigation */}
   </div>
   ```
2. Style header to match Firehose pattern
3. Add phosphor glow to title
4. Run: `just web typecheck`
5. Commit: `refactor(web-ui): add CRT header to dashboard layout`

### Task 2: Update DashboardOverview cards

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardOverview.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardPages.css`
- Modify: `web-ui/src/components/dashboard/TrendCard.tsx`
- Modify: `web-ui/src/components/dashboard/TrendCard.css`

**Steps:**
1. Add panel structure to overview cards
2. Add glowing stat values (like Firehose metrics)
3. Update TrendCard with panel-header pattern
4. Run: `just web test`
5. Commit: `refactor(web-ui): apply CRT styling to overview cards`

### Task 3: Update DashboardLearnings page

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardLearnings.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardLearnings.css`

**Steps:**
1. Wrap content in bordered panel
2. Add panel-header to filter section
3. Apply phosphor glow to key values
4. Run: `just web test`
5. Commit: `refactor(web-ui): apply CRT styling to learnings page`

### Task 4: Update DashboardAttribution page

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardAttribution.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardAttribution.css`

**Steps:**
1. Add panel structure to leaderboard and timeline
2. Apply phosphor glow to scores/rankings
3. Update card borders and backgrounds
4. Run: `just web test`
5. Commit: `refactor(web-ui): apply CRT styling to attribution page`

### Task 5: Update DashboardStrategy page

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardStrategy.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardStrategy.css`

**Steps:**
1. Add panel structure to strategy cards
2. Apply glow effects to weight values
3. Update tab styling to match panel-header
4. Run: `just web test`
5. Commit: `refactor(web-ui): apply CRT styling to strategy page`

### Task 6: Update DashboardHealth page

**Files:**
- Modify: `web-ui/src/pages/dashboard/DashboardHealth.tsx`
- Modify: `web-ui/src/pages/dashboard/DashboardHealth.css`

**Steps:**
1. Add panel structure to health sections
2. Apply glow effects to status indicators
3. Match system status cards with Firehose stat pattern
4. Run: `just web test`
5. Commit: `refactor(web-ui): apply CRT styling to health page`

### Task 7: Update visual regression baselines

**Steps:**
1. Run: `just web visual-update`
2. Review new screenshots for consistency
3. Commit: `test(web-ui): update visual baselines after CRT styling`

## Acceptance Criteria

- [x] Dashboard has header panel with glowing title
- [x] All dashboard pages use bordered panels
- [x] Stat values have phosphor glow effects
- [x] Panel headers use font-display with letter-spacing
- [x] Visual style matches Firehose and Sessions pages
- [x] All tests pass
- [x] Visual baselines updated

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done REFACTOR0069`
3. Commit, push, and create PR

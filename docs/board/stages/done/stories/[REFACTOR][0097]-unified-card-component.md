---
id: REFACTOR0097
title: Unified Card Component with Visual Regression Testing
type: refactor
status: done
priority: high
epics: [web-ui, design-system]
---

# Unified Card Component with Visual Regression Testing

Rename Panel → Card in the design system and add visual regression testing.

## Context

Currently the web-ui has multiple inconsistent card patterns:
- `TrendCard`, `LearningsCard`, `AttributionCard` (custom `.dashboard-card` class)
- `StrategyCard`, `HealthCard` (use `Panel variant="crt"`)
- Settings uses custom `.settings-panel` divs
- SubsystemCard, DistributionCard use custom divs

The design-system already has a `Panel` component with variants, but it's underutilized and poorly named. "Card" is more intuitive.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Approach | Rename Panel → Card (not create new component) |
| Visual testing | Playwright screenshots |
| Migration | Clean break, no backwards-compatible alias |
| Variants | Audit and remove unused variants |

## Component Rename

**Rename `Panel` → `Card`:**
```
design-system/src/primitives/Panel/  →  design-system/src/primitives/Card/
  Panel.tsx         →  Card.tsx
  Panel.module.css  →  Card.module.css
  Panel.test.tsx    →  Card.test.tsx
  Panel.stories.tsx →  Card.stories.tsx
```

## Migration Plan

| Location | Current Usage | Migration |
|----------|---------------|-----------|
| StrategyCard | `<Panel variant="crt">` | → `<Card variant="crt">` |
| HealthCard | `<Panel variant="crt">` | → `<Card variant="crt">` |
| TrendCard | custom `.trend-card` div | → `<Card variant="crt">` |
| LearningsCard | custom `.dashboard-card` div | → `<Card variant="crt">` |
| AttributionCard | custom `.dashboard-card` div | → `<Card variant="crt">` |
| Settings panels | custom `.settings-panel` div | → `<Card variant="crt">` |
| Firehose panels | custom `.panel` div | → `<Card variant="crt">` |
| AssessmentStatus | custom `.status-card` div | → `<Card variant="crt">` |
| SubsystemCard | custom div | → skipped (status-based border variants) |
| DistributionCard | custom article | → skipped (custom header layout) |
| AssessmentHistory | custom `.history-card` div | → skipped (two-element header) |
| Quarantine cards | custom divs | → skipped (custom layouts) |
| Models panel | modal overlay | → skipped (not card pattern) |

## Visual Regression Testing

**Playwright approach:**
```
web-ui/e2e/visual/
  card.spec.ts        # Card component screenshots
  dashboard.spec.ts   # Dashboard page screenshots
  settings.spec.ts    # Settings page screenshots
```

**CI integration:**
- Run Ladle in CI on port 61000
- Run Playwright visual tests against Ladle
- On failure: upload diff images as artifacts
- Threshold: 0.1% pixel difference for anti-aliasing

## Acceptance Criteria

### Card Component Rename
- [x] Rename Panel/ directory to Card/
- [x] Rename component Panel → Card
- [x] Update CSS classes .panel → .card
- [x] Update design-system exports
- [x] Update all tests and stories
- [x] Audit variants: keep elevated/inset (used in stories for documentation)

### Migration
- [x] Migrate StrategyCard, HealthCard (just rename import)
- [x] Migrate TrendCard to use Card
- [x] Migrate LearningsCard to use Card
- [x] Migrate AttributionCard to use Card
- [x] Migrate openworld components (already using Card)
- [x] Migrate Settings panels to use Card (7 panels)
- [x] Migrate Firehose panels to use Card (METRICS, FILTERS)
- [x] Migrate AssessmentStatus panels to use Card (Circuit Breaker, Sampling, Activity, Tier Distribution)
- [~] Migrate SubsystemCard to use Card (skipped: has status-based border variants Card doesn't support)
- [~] Migrate DistributionCard to use Card (skipped: internally consistent, custom header layout)
- [~] Migrate AssessmentHistory to use Card (skipped: custom header layout with two elements)
- [~] Migrate Quarantine stat/trust-level cards (skipped: custom layouts, not standard card pattern)
- [~] Migrate Models details panel (skipped: modal dialog, not card pattern)
- [x] Remove duplicate CSS (TrendCard.css, DashboardCards.css, Settings.css, Firehose.css, AssessmentStatus.css)

### Visual Regression Testing
- [x] Add Playwright visual test config (already existed in e2e-tests/)
- [x] Create Card component visual tests (e2e-tests/tests/card.spec.ts)
- [x] Create baseline snapshots (4 Card snapshots + 2 updated page snapshots)
- [x] Document how to update baselines (`just web visual-update`)
- [x] Add just command for visual tests (`just web visual`)

## Files Changed

**Renamed:**
- `design-system/src/primitives/Panel/` → `Card/`

**Updated imports:**
- `design-system/src/primitives/index.ts`
- `web-ui/src/components/dashboard/StrategyCard.tsx`
- `web-ui/src/components/dashboard/HealthCard.tsx`

**Migrated to Card:**
- `web-ui/src/components/dashboard/TrendCard.tsx`
- `web-ui/src/components/dashboard/LearningsCard.tsx`
- `web-ui/src/components/dashboard/AttributionCard.tsx`
- `web-ui/src/pages/Settings.tsx`
- `web-ui/src/pages/Firehose.tsx`
- `web-ui/src/pages/assessment/AssessmentStatus.tsx`

**Removed/consolidated CSS:**
- `web-ui/src/components/dashboard/TrendCard.css` (removed `.trend-card` styles)
- `web-ui/src/components/dashboard/DashboardCards.css` (removed `.dashboard-card` styles)
- `web-ui/src/pages/Settings.css` (removed `.settings-panel` styles)
- `web-ui/src/pages/Firehose.css` (removed `.panel` styles)
- `web-ui/src/pages/assessment/AssessmentStatus.css` (removed `.status-card` styles)

**New files:**
- `e2e-tests/tests/card.spec.ts`

## Out of Scope

- New Card features beyond what Panel has
- Complex visual diff review UI
- PR comment integration for visual diffs

## Size

L - Large

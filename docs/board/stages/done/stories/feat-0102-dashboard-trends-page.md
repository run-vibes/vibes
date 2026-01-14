---
id: FEAT0102
title: Dashboard Trends Page
type: feat
status: done
priority: medium
epics: [groove]
depends: []
estimate:
created: 2026-01-13
updated: 2026-01-13
---

# Dashboard Trends Page

## Summary

Create the `/groove/trends` page that the TrendCard links to from the Status page. This page should show detailed session trend analytics including historical sparklines, period comparisons, and trend breakdowns.

## Acceptance Criteria

- [x] Route `/groove/trends` renders a TrendsPage component
- [x] Shows expanded sparkline visualization (larger than card version)
- [x] Displays period comparison (e.g., this week vs last week)
- [x] Shows session count trends over time
- [x] Shows improvement percentage trends
- [x] Loading and error states styled consistently with other Groove pages
- [x] Uses Card components with CRT variant
- [x] Uses PageHeader component from design system

## Implementation Notes

- Add route to App.tsx under groove routes
- Create TrendsPage.tsx in pages/groove/
- May need new API endpoint for detailed trends data
- Reuse Sparkline component from charts/
- Add to subnav items if appropriate

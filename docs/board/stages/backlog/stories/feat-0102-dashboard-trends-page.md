---
id: FEAT0102
title: Dashboard Trends Page
type: feat
status: backlog
priority: medium
epics: []
depends: []
estimate:
created: 2026-01-13
updated: 2026-01-13
---

# Dashboard Trends Page

## Summary

Create the `/groove/dashboard/trends` page that the TrendCard links to from the dashboard overview. This page should show detailed session trend analytics including historical sparklines, period comparisons, and trend breakdowns.

## Acceptance Criteria

- [ ] Route `/groove/dashboard/trends` renders a DashboardTrends component
- [ ] Shows expanded sparkline visualization (larger than card version)
- [ ] Displays period comparison (e.g., this week vs last week)
- [ ] Shows session count trends over time
- [ ] Shows improvement percentage trends
- [ ] Loading and error states styled consistently with other dashboard pages
- [ ] Uses Card components with CRT variant

## Implementation Notes

- Add route to DashboardRoutes.tsx
- Create DashboardTrends.tsx page component
- May need new API endpoint for detailed trends data
- Reuse Sparkline component from charts/

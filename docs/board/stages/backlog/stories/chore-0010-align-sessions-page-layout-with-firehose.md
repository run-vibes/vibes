---
id: CHORE0010
title: Align Sessions page layout with Firehose
type: chore
status: backlog
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
---

# Align Sessions page layout with Firehose

## Summary

The `/sessions` page has different spacing and visual structure compared to `/firehose`. Both pages should share consistent layout patterns for visual cohesion across the app.

## Acceptance Criteria

- [ ] Sessions page uses same page padding as Firehose
- [ ] Session cards have consistent spacing with Firehose event list
- [ ] Page header styling matches between pages
- [ ] Grid/list layout uses same gap values

## Implementation Notes

- Compare Sessions.tsx and Firehose.tsx page structures
- Extract shared layout patterns or use consistent CSS variables
- May involve creating shared page layout components

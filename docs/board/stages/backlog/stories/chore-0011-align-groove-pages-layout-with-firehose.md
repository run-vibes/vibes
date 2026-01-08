---
id: CHORE0011
title: Align Groove pages layout with Firehose
type: chore
status: backlog
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
---

# Align Groove pages layout with Firehose

## Summary

The Groove pages (`/groove` Security and `/groove/assessment`) have different spacing and visual structure compared to `/firehose`. All pages should share consistent layout patterns for visual cohesion across the app.

## Acceptance Criteria

- [ ] Security page (`/groove`) uses same page padding as Firehose
- [ ] Assessment page (`/groove/assessment`) uses same page padding as Firehose
- [ ] Section headers and content spacing match Firehose patterns
- [ ] Grid/list layouts use same gap values
- [ ] Page headers styled consistently

## Implementation Notes

- Update Quarantine.tsx and Assessment.tsx page structures
- Use same CSS variables and spacing patterns as Firehose
- Consider extracting shared page layout component if patterns repeat

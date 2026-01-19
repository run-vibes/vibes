---
id: FEAT0017
title: Link +more text to assessment history page
type: feat
status: done
priority: low
scope: plugin-system
depends: []
estimate: 15m
created: 2026-01-08
---

# Link +more text to assessment history page

## Summary

On `/groove/assessment/status`, the "+more" text indicating additional assessments should be a clickable link that navigates to `/groove/assessment/history`.

## Requirements

- Convert "+more" text to an anchor/link element
- Link should navigate to the full history page
- Style should indicate it's clickable (underline, hover state)

## Acceptance Criteria

- [ ] "+more" is a clickable link
- [ ] Clicking navigates to `/groove/assessment/history`
- [ ] Link has appropriate visual affordance

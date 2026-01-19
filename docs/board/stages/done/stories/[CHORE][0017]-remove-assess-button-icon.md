---
id: CHORE0017
title: Remove icon from ASSESS button in stream view
type: chore
status: done
priority: low
scope: plugin-system
depends: []
estimate: 10m
created: 2026-01-08
---

# Remove icon from ASSESS button in stream view

## Summary

On `/groove/assessment/stream`, each event has an "ASSESS" button with an icon to its left. Remove this icon for a cleaner UI.

## Requirements

- Remove the icon element preceding the ASSESS button text
- Keep the button functionality intact

## Acceptance Criteria

- [ ] ASSESS button displays text only, no icon
- [ ] Button click still triggers assessment

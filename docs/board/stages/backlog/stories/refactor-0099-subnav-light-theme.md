---
id: refactor-0099
title: Improve SubnavBar Light Theme Readability
type: refactor
status: pending
priority: medium
epics: [web-ui, design-system]
---

# Improve SubnavBar Light Theme Readability

Redesign the SubnavBar styling in light theme for better readability and visual consistency.

## Context

The SubnavBar in light theme currently uses a warm beige background (`#ebe7dc`) that:
- Doesn't match the rest of the light theme visual language
- Is difficult to read (low contrast)
- Feels disconnected from the main navigation

## Problem

- Text contrast is insufficient for comfortable reading
- Color doesn't harmonize with adjacent UI elements
- Active/hover states may not be clear enough

## Acceptance Criteria

- [ ] Research: review light theme patterns in similar apps (dev tools, dashboards)
- [ ] Design: explore 3-4 alternative approaches with mockups
- [ ] Design: verify WCAG AA contrast ratios for all text states
- [ ] Implementation: update SubnavBar light theme tokens
- [ ] Implementation: ensure active/hover/focus states are clearly distinguishable
- [ ] Implementation: test with both dark and light themes side-by-side
- [ ] Add Ladle story showing SubnavBar in both themes
- [ ] Visual regression test for theme switching

## Design Considerations

Approaches to explore:
1. **Transparent/subtle**: Let content show through, minimal background
2. **High contrast**: White or very light background with darker text
3. **Accent tint**: Subtle tint matching the phosphor/accent color
4. **Match header**: Same background as main nav for visual continuity

Key constraints:
- Must maintain CRT aesthetic in dark theme
- Light theme should feel cohesive but doesn't need to be 1:1 with dark
- Readability is the top priority

## Size

S - Small (CSS/token changes, design exploration)

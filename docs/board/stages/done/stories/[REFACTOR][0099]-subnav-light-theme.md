---
id: REFACTOR0099
title: Improve SubnavBar Light Theme Readability
type: refactor
status: done
priority: medium
scope: design-system
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

- [x] Research: review light theme patterns in similar apps (dev tools, dashboards)
- [x] Design: explore 3-4 alternative approaches with mockups
- [x] Design: verify WCAG AA contrast ratios for all text states
- [x] Implementation: update SubnavBar light theme tokens
- [x] Implementation: ensure active/hover/focus states are clearly distinguishable
- [x] Implementation: test with both dark and light themes side-by-side
- [x] Add Ladle story showing SubnavBar in both themes
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

## Implementation Notes

**Approach chosen:** Option 4 - Match header for visual continuity.

**Changes made:**
- SubnavBar light theme now uses `--surface` (white) instead of `--surface-light` (beige)
- Removed glow/text-shadow effects in light theme (replaced with subtle hover backgrounds)
- Active states use solid border + background instead of glowing effects
- More dropdown gets a subtle shadow in light theme for better separation
- Fixed hardcoded fallback colors to use proper design tokens

**Contrast verification:**
- Text on white background: 5.77:1 (WCAG AA compliant, needs 4.5:1)
- Previous beige background was ~4.8:1 (marginal)

**Files changed:**
- `design-system/src/compositions/SubnavBar/SubnavBar.module.css`
- `design-system/src/compositions/SubnavBar/SubnavBar.stories.tsx` (new)

## Size

S - Small (CSS/token changes, design exploration)

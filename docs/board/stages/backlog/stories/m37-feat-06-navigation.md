---
id: F017
title: Feature: Restyle Navigation with Phosphor Glow
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-04
updated: 2026-01-07
milestone: milestone-37-crt-design-system
---

# Feature: Restyle Navigation with Phosphor Glow

## Problem

The sidebar navigation uses generic styling that doesn't match the CRT aesthetic.

## Goal

Restyle the navigation with phosphor accents, glow effects, and CRT typography.

## Tasks

### Task 1: Restyle Sidebar Container

Apply CRT styling to sidebar:
```css
.sidebar {
  background: var(--screen);
  border-right: 1px solid var(--border);
}
```

### Task 2: Restyle Navigation Items

Apply CRT styling to nav links:
```css
.nav-item {
  color: var(--text-dim);
  font-family: var(--font-display);
  font-size: var(--font-size-lg);
}
.nav-item:hover {
  color: var(--phosphor);
  text-shadow: 0 0 10px var(--phosphor);
}
.nav-item.active {
  color: var(--phosphor);
  border-left: 2px solid var(--phosphor);
}
```

### Task 3: Add Phosphor Glow to Active State

Active nav item gets enhanced glow:
```css
.nav-item.active {
  background: rgba(255, 176, 0, 0.1);
  box-shadow: inset 3px 0 10px rgba(255, 176, 0, 0.2);
}
```

### Task 4: Restyle Logo/Brand Area

Apply CRT styling to logo:
- VT323 font for brand text
- Phosphor color
- Subtle glow animation (optional)

### Task 5: Restyle Footer Links

Apply consistent styling to sidebar footer:
- Settings link
- Help link
- Version info (dim text)

## Acceptance Criteria

- [x] Sidebar has CRT background/border
- [x] Nav items use display font
- [x] Hover shows phosphor glow
- [x] Active state clearly visible with left border
- [x] Logo matches CRT aesthetic
- [x] Footer links styled consistently

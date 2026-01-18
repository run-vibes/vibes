---
id: FEAT0147
title: "Feature: Restyle Core UI Components"
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-04
updated: 2026-01-07
milestone: 37
---

# Feature: Restyle Core UI Components

## Problem

Buttons, cards, inputs, and other core components use generic Tailwind styles that don't match the CRT aesthetic.

## Goal

Restyle core components using design tokens to achieve the CRT Essence look.

## Tasks

### Task 1: Restyle Buttons

Apply CRT styling to buttons:
```css
.btn-primary {
  background: var(--phosphor);
  color: var(--screen);
  font-family: var(--font-display);
  border: 1px solid var(--phosphor-bright);
  box-shadow: 0 0 10px rgba(255, 176, 0, 0.3);
}
.btn-primary:hover {
  background: var(--phosphor-bright);
  box-shadow: 0 0 20px rgba(255, 176, 0, 0.5);
}
```

Secondary/ghost button variants as well.

### Task 2: Restyle Cards

Apply CRT styling to card components:
```css
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}
.card:hover {
  border-color: var(--phosphor-dim);
}
```

### Task 3: Restyle Input Fields

Apply CRT styling to inputs:
```css
input, textarea, select {
  background: var(--screen);
  border: 1px solid var(--border);
  color: var(--text);
  font-family: var(--font-mono);
}
input:focus {
  border-color: var(--phosphor);
  box-shadow: 0 0 5px rgba(255, 176, 0, 0.3);
}
```

### Task 4: Restyle Modal/Dialog

Apply CRT styling to modals:
- Dark surface background
- Phosphor glow on borders
- Vignette within modal viewport

### Task 5: Restyle Badges and Tags

Apply CRT styling to badges:
```css
.badge {
  background: var(--phosphor-dim);
  color: var(--screen);
  font-family: var(--font-display);
  font-size: var(--font-size-xs);
}
```

### Task 6: Update Component Hover States

Ensure all interactive components have phosphor glow on hover/focus.

## Acceptance Criteria

- [x] Buttons use phosphor colors with glow
- [x] Cards have subtle border highlighting
- [x] Inputs have focus glow effect
- [x] Modals match CRT aesthetic
- [x] All components use design tokens (no hardcoded colors)
- [x] Hover/focus states are consistent

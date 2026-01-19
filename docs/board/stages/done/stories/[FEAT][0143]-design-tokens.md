---
id: FEAT0143
title: "Feature: Create Design Token CSS Custom Properties"
type: feat
status: done
priority: medium
scope: models/37-model-management
depends: []
estimate:
created: 2026-01-04
---

# Feature: Create Design Token CSS Custom Properties

## Problem

The web UI has no centralized design token system. Colors, spacing, and typography are scattered across components as hardcoded Tailwind classes or inline styles.

## Goal

Create a foundational CSS file defining all design tokens as CSS custom properties, enabling consistent theming across the entire UI.

## Implementation Notes

Tokens are implemented in the `@vibes/design-system` package at `design-system/src/tokens/` and imported by web-ui via `@import '@vibes/design-system/tokens'`. This follows the existing monorepo architecture where the design-system is a shared package consumed by web-ui.

## Tasks

### Task 1: Create Token File Structure

Updated `design-system/src/tokens/` with:
- `colors.css` - Color palette for dark and light themes
- `typography.css` - Font family, size, and weight tokens
- `spacing.css` - Spacing scale tokens
- `effects.css` - Glow, shadow, and border tokens (NEW)
- `index.css` - Imports all token files
- `index.ts` - Programmatic TypeScript exports

### Task 2: Define Dark Theme Colors

Defined in `:root` (default dark theme):
```css
:root {
  --screen: #0a0908;
  --phosphor: #ffb000;
  --phosphor-dim: #cc8c00;
  --phosphor-bright: #ffc633;
  --text: #e8e0d0;
  --text-dim: #8a8070;
  --surface: #141210;
  --border: #2a2520;
  --scanline: rgba(0,0,0,0.15);
}
```

### Task 3: Define Light Theme Colors

Defined for `[data-theme="light"]`:
```css
[data-theme="light"] {
  --screen: #f4f1e8;
  --phosphor: #c4820a;
  --phosphor-dim: #9a6808;
  --phosphor-bright: #e09810;
  --text: #2c2a26;
  --text-dim: #6b6560;
  --surface: #ffffff;
  --border: #d4d0c8;
  --scanline: rgba(0,0,0,0.03);
}
```

### Task 4: Define Typography Tokens

```css
:root {
  --font-display: 'VT323', monospace;
  --font-mono: 'IBM Plex Mono', monospace;
  --font-size-xs: 0.75rem;
  --font-size-sm: 0.875rem;
  --font-size-base: 1rem;
  --font-size-lg: 1.25rem;
  --font-size-xl: 1.5rem;
  --font-size-2xl: 2rem;
}
```

### Task 5: Define Spacing and Effect Tokens

Spacing scale (4px base) and effect tokens for glow/shadows implemented in `spacing.css` and `effects.css`.

### Task 6: Import Tokens in Main CSS

`design-system/src/tokens/index.css` imports all token files, and `web-ui/src/index.css` already imports via `@import '@vibes/design-system/tokens'`.

## Acceptance Criteria

- [x] Token files created in `design-system/src/tokens/`
- [x] All colors defined for both themes
- [x] Typography tokens match design spec (VT323 + IBM Plex Mono)
- [x] Spacing scale uses 4px grid
- [x] Tokens imported in main CSS entry
- [x] Browser devtools show CSS custom properties on `:root`

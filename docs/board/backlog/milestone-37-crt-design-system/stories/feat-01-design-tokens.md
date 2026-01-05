---
created: 2026-01-04
status: pending
---

# Feature: Create Design Token CSS Custom Properties

## Problem

The web UI has no centralized design token system. Colors, spacing, and typography are scattered across components as hardcoded Tailwind classes or inline styles.

## Goal

Create a foundational CSS file defining all design tokens as CSS custom properties, enabling consistent theming across the entire UI.

## Tasks

### Task 1: Create Token File Structure

Create `web-ui/src/styles/tokens/` directory with:
- `colors.css` - Color palette for dark and light themes
- `typography.css` - Font family, size, and weight tokens
- `spacing.css` - Spacing scale tokens
- `effects.css` - Glow, shadow, and border tokens

### Task 2: Define Dark Theme Colors

Define in `:root` (default dark theme):
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

Define for `[data-theme="light"]`:
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

Spacing scale (4px base) and effect tokens for glow/shadows.

### Task 6: Import Tokens in Main CSS

Update `web-ui/src/index.css` to import token files.

## Acceptance Criteria

- [ ] Token files created in `styles/tokens/`
- [ ] All colors defined for both themes
- [ ] Typography tokens match design spec
- [ ] Spacing scale uses 4px grid
- [ ] Tokens imported in main CSS entry
- [ ] Browser devtools show CSS custom properties on `:root`

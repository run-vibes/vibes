---
created: 2026-01-04
---

# Milestone 37: CRT Design System - Design

> Implement the CRT Essence aesthetic as vibes' unified design system with dark/light themes.

## Overview

The CRT Essence design system brings a warm, retro-futuristic aesthetic inspired by vintage CRT monitors. It creates an immersive, focused environment that reduces eye strain while maintaining excellent readability.

**Reference Prototypes:**
- Dark theme: `docs/design/prototypes/19-crt-essence-v4.html`
- Light theme: `docs/design/prototypes/15-crt-daylight-v2.html`

## Design Tokens

### Colors

#### Dark Theme (CRT Essence)

| Token | Value | Usage |
|-------|-------|-------|
| `--screen` | `#0a0908` | Background (near-black with warm undertone) |
| `--phosphor` | `#ffb000` | Primary accent (amber phosphor glow) |
| `--phosphor-dim` | `#cc8c00` | Muted accent, secondary elements |
| `--phosphor-bright` | `#ffc633` | Highlights, active states |
| `--text` | `#e8e0d0` | Primary text (warm off-white) |
| `--text-dim` | `#8a8070` | Secondary text, placeholders |
| `--scanline` | `rgba(0,0,0,0.15)` | Scanline overlay effect |
| `--surface` | `#141210` | Elevated surfaces (cards, modals) |
| `--border` | `#2a2520` | Subtle borders |

#### Light Theme (CRT Daylight)

| Token | Value | Usage |
|-------|-------|-------|
| `--bg` | `#f4f1e8` | Background (warm parchment) |
| `--accent` | `#c4820a` | Primary accent (sepia amber) |
| `--accent-dim` | `#9a6808` | Muted accent |
| `--accent-bright` | `#e09810` | Highlights, active states |
| `--text` | `#2c2a26` | Primary text (warm charcoal) |
| `--text-dim` | `#6b6560` | Secondary text |
| `--scanline` | `rgba(0,0,0,0.03)` | Subtle scanline overlay |
| `--surface` | `#ffffff` | Elevated surfaces |
| `--border` | `#d4d0c8` | Subtle borders |

### Typography

| Token | Value | Usage |
|-------|-------|-------|
| `--font-display` | `'VT323', monospace` | Headers, titles, UI chrome |
| `--font-mono` | `'IBM Plex Mono', monospace` | Code, data, body text |
| `--font-size-xs` | `0.75rem` | Captions, badges |
| `--font-size-sm` | `0.875rem` | Secondary text |
| `--font-size-base` | `1rem` | Body text |
| `--font-size-lg` | `1.25rem` | Section headers |
| `--font-size-xl` | `1.5rem` | Page titles |
| `--font-size-2xl` | `2rem` | Hero text |

### Effects

#### Scanlines

```css
.scanlines::before {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    var(--scanline) 0px,
    var(--scanline) 1px,
    transparent 1px,
    transparent 2px
  );
  pointer-events: none;
  z-index: 1000;
}
```

#### Vignette

```css
.vignette::after {
  content: '';
  position: fixed;
  inset: 0;
  background: radial-gradient(
    ellipse at center,
    transparent 0%,
    transparent 60%,
    rgba(0,0,0,0.4) 100%
  );
  pointer-events: none;
  z-index: 999;
}
```

#### Glow Utilities

```css
.glow-text {
  text-shadow: 0 0 10px var(--phosphor);
}

.glow-box {
  box-shadow: 0 0 20px rgba(255, 176, 0, 0.3);
}
```

### Spacing

Standard 4px grid:

| Token | Value |
|-------|-------|
| `--space-1` | `0.25rem` (4px) |
| `--space-2` | `0.5rem` (8px) |
| `--space-3` | `0.75rem` (12px) |
| `--space-4` | `1rem` (16px) |
| `--space-6` | `1.5rem` (24px) |
| `--space-8` | `2rem` (32px) |
| `--space-12` | `3rem` (48px) |

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | `2px` | Buttons, inputs |
| `--radius-md` | `4px` | Cards, panels |
| `--radius-lg` | `8px` | Modals, large containers |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Font pairing | VT323 + IBM Plex Mono | VT323 captures CRT aesthetic; IBM Plex Mono is highly readable for code |
| Primary accent | Amber (#ffb000) | Classic phosphor color, warm and focused, reduces blue light |
| Scanline density | 2px period | Visible texture without interfering with readability |
| Vignette strength | 40% dark / 10% light | Creates focus without obscuring content |
| Border radius | Minimal (2-8px) | Retains retro-industrial feel while avoiding harsh edges |

## Architecture

### Implementation Approach

1. **CSS Custom Properties** - All tokens defined as CSS variables in `:root`
2. **Theme Switching** - `data-theme="dark|light"` attribute on `<html>`
3. **Component Classes** - Semantic utility classes (`.btn-primary`, `.card`, `.panel`)
4. **Effect Layers** - Optional scanlines/vignette via `.crt-effects` container

### File Structure

```
web-ui/src/
├── styles/
│   ├── tokens/
│   │   ├── colors.css      # Color tokens (dark + light)
│   │   ├── typography.css  # Font tokens
│   │   ├── spacing.css     # Spacing scale
│   │   └── effects.css     # Glow, scanlines, vignette
│   ├── components/
│   │   ├── button.css
│   │   ├── card.css
│   │   ├── input.css
│   │   └── ...
│   └── index.css           # Main entry, imports all
```

### Theme Toggle

```typescript
function toggleTheme() {
  const html = document.documentElement;
  const current = html.dataset.theme || 'dark';
  html.dataset.theme = current === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', html.dataset.theme);
}
```

## Deliverables

- [ ] Design tokens CSS file with all color/typography/spacing variables
- [ ] Dark theme (CRT Essence) implementation
- [ ] Light theme (CRT Daylight) implementation
- [ ] Theme toggle component with localStorage persistence
- [ ] Scanlines/vignette effect components (optional, toggleable)
- [ ] Core UI components restyled (buttons, cards, inputs, modals)
- [ ] Navigation restyled with new aesthetic
- [ ] Firehose event stream with CRT styling
- [ ] Session cards with phosphor glow effects
- [ ] Documentation of all design tokens

## Migration Strategy

1. Create new CSS token files alongside existing styles
2. Update components incrementally (one at a time)
3. Add theme toggle to settings
4. Default to dark theme (matches current expectation)
5. Remove old ad-hoc styles once migration complete

## References

- [VT323 Font](https://fonts.google.com/specimen/VT323)
- [IBM Plex Mono](https://fonts.google.com/specimen/IBM+Plex+Mono)
- Prototype 19: CRT Essence v4 (dark)
- Prototype 15: CRT Daylight v2 (light)

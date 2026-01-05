# CRT Essence Design System - Design Tokens

This document describes the design token system used throughout the vibes application.

## Overview

The CRT Essence design system uses CSS custom properties (variables) to maintain consistent styling across all components. Tokens are organized into categories: colors, typography, spacing, and effects.

## Importing Tokens

In CSS files:
```css
@import '@vibes/design-system/tokens';
```

In React components, tokens are available globally via `index.css` which imports the design system.

## Color Tokens

### Core Colors

| Token | Dark Theme | Light Theme | Usage |
|-------|-----------|-------------|-------|
| `--screen` | `#0a0908` | `#f4f1e8` | Page background |
| `--surface` | `#141210` | `#ffffff` | Card/panel backgrounds |
| `--surface-light` | `#1e1a16` | `#ebe7dc` | Elevated surfaces |
| `--surface-lighter` | `#2a2520` | `#e0dcd0` | Higher elevation |

### Phosphor (Primary Accent)

| Token | Dark Theme | Light Theme | Usage |
|-------|-----------|-------------|-------|
| `--phosphor` | `#ffb000` | `#c4820a` | Primary accent, active states |
| `--phosphor-bright` | `#ffc633` | `#e09810` | Hover states, emphasis |
| `--phosphor-dim` | `#cc8c00` | `#9a6808` | Secondary accent |
| `--phosphor-faint` | `#4d3500` | `#d4c4a0` | Disabled states |
| `--phosphor-glow` | `rgba(255,176,0,0.4)` | `rgba(196,130,10,0.25)` | Glow effects |
| `--phosphor-subtle` | `rgba(255,176,0,0.15)` | `rgba(196,130,10,0.12)` | Backgrounds |

### Text Colors

| Token | Usage |
|-------|-------|
| `--text` | Primary text |
| `--text-dim` | Secondary text, labels |
| `--text-faint` | Disabled text, placeholders |
| `--text-inverse` | Text on accent backgrounds |

### Border Colors

| Token | Usage |
|-------|-------|
| `--border` | Default borders |
| `--border-subtle` | Subtle dividers |
| `--border-strong` | Emphasized borders |

### Semantic Colors

| Token | Usage |
|-------|-------|
| `--red` / `--red-glow` / `--red-subtle` / `--red-border` | Errors, destructive actions |
| `--green` / `--green-glow` / `--green-subtle` / `--green-border` | Success, positive states |
| `--cyan` / `--cyan-glow` / `--cyan-subtle` / `--cyan-border` | Info, user messages |
| `--gold` / `--gold-glow` | Special highlights |

### Status Colors

For connection states, badges, and indicators:

| Token | Usage |
|-------|-------|
| `--status-disabled` / `--status-disabled-subtle` | Inactive, disabled |
| `--status-starting` / `--status-starting-subtle` | Loading, connecting |
| `--status-connected` / `--status-connected-subtle` | Active, success |
| `--status-failed` / `--status-failed-subtle` | Error, failed |

### Trust Level Colors

For security and trust indicators:

| Token | Usage |
|-------|-------|
| `--trust-local` / `--trust-local-subtle` | Local trust |
| `--trust-private` / `--trust-private-subtle` | Private cloud |
| `--trust-org-verified` / `--trust-org-verified-subtle` | Verified organization |
| `--trust-org-unverified` / `--trust-org-unverified-subtle` | Unverified organization |
| `--trust-public-verified` / `--trust-public-verified-subtle` | Verified public |
| `--trust-public-unverified` / `--trust-public-unverified-subtle` | Unverified public |
| `--trust-quarantined` / `--trust-quarantined-subtle` | Quarantined |

### Terminal Colors

For xterm.js terminal theming:

| Token | Usage |
|-------|-------|
| `--terminal-bg` | Terminal background |
| `--terminal-fg` | Terminal foreground |
| `--terminal-cursor` | Cursor color |
| `--terminal-selection` | Selection background |
| `--terminal-scrollbar` | Scrollbar thumb |
| `--terminal-scrollbar-hover` | Scrollbar hover |

## Typography Tokens

### Font Families

| Token | Value | Usage |
|-------|-------|-------|
| `--font-display` | `'VT323', monospace` | Headers, UI chrome |
| `--font-mono` | `'IBM Plex Mono', ...` | Code, body text |
| `--font-sans` | System sans-serif | Fallback |

### Font Sizes

| Token | Size | Usage |
|-------|------|-------|
| `--font-size-xs` | 0.75rem (12px) | Small labels |
| `--font-size-sm` | 0.875rem (14px) | Body text |
| `--font-size-base` | 1rem (16px) | Default |
| `--font-size-lg` | 1.25rem (20px) | Subheadings |
| `--font-size-xl` | 1.5rem (24px) | Headings |
| `--font-size-2xl` | 2rem (32px) | Page titles |
| `--font-size-3xl` | 2.5rem (40px) | Hero text |

### Line Heights

| Token | Value | Usage |
|-------|-------|-------|
| `--leading-none` | 1 | Single line |
| `--leading-tight` | 1.25 | Compact text |
| `--leading-normal` | 1.5 | Body text |
| `--leading-relaxed` | 1.75 | Loose text |

### Letter Spacing

| Token | Value |
|-------|-------|
| `--tracking-tighter` | -0.05em |
| `--tracking-tight` | -0.02em |
| `--tracking-normal` | 0 |
| `--tracking-wide` | 0.05em |
| `--tracking-wider` | 0.1em |
| `--tracking-widest` | 0.15em |

## Spacing Tokens

Based on a 4px grid system:

| Token | Size |
|-------|------|
| `--space-0` | 0 |
| `--space-px` | 1px |
| `--space-0-5` | 2px |
| `--space-1` | 4px |
| `--space-1-5` | 6px |
| `--space-2` | 8px |
| `--space-2-5` | 10px |
| `--space-3` | 12px |
| `--space-4` | 16px |
| `--space-5` | 20px |
| `--space-6` | 24px |
| `--space-8` | 32px |
| `--space-10` | 40px |
| `--space-12` | 48px |
| `--space-16` | 64px |
| `--space-20` | 80px |
| `--space-24` | 96px |

### Border Radius

| Token | Size |
|-------|------|
| `--radius-none` | 0 |
| `--radius-sm` | 2px |
| `--radius-md` | 4px |
| `--radius-lg` | 8px |
| `--radius-xl` | 12px |
| `--radius-full` | 9999px |

### Transitions

| Token | Duration |
|-------|----------|
| `--transition-fast` | 100ms ease |
| `--transition-normal` | 150ms ease |
| `--transition-slow` | 200ms ease |
| `--transition-slower` | 300ms ease |

### Z-Index Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--z-base` | 0 | Default |
| `--z-dropdown` | 100 | Dropdowns |
| `--z-sticky` | 200 | Sticky headers |
| `--z-overlay` | 500 | Overlays |
| `--z-modal` | 900 | Modals |
| `--z-scanlines` | 1000 | CRT effects |
| `--z-tooltip` | 1100 | Tooltips |

## Effect Tokens

### Glow Effects

| Token | Usage |
|-------|-------|
| `--glow-phosphor` | Standard phosphor glow |
| `--glow-phosphor-bright` | Intense phosphor glow |
| `--glow-phosphor-dim` | Subtle phosphor glow |
| `--glow-red` | Error glow |
| `--glow-green` | Success glow |
| `--glow-cyan` | Info glow |
| `--glow-gold` | Special glow |

### Shadows

| Token | Usage |
|-------|-------|
| `--shadow-sm` | Subtle shadow |
| `--shadow-md` | Medium shadow |
| `--shadow-lg` | Large shadow |
| `--shadow-xl` | Extra large shadow |
| `--shadow-panel` | CRT panel effect |
| `--shadow-modal` | Modal overlay |
| `--shadow-inset-sm` | Subtle inset |
| `--shadow-inset-md` | Medium inset |

### CRT Effects

| Token | Usage |
|-------|-------|
| `--scanline-gradient` | Scanline overlay |
| `--vignette-gradient` | Edge darkening |
| `--selection-bar` | Active item indicator |
| `--focus-ring` | Focus outline |

## Usage Examples

### Basic Component Styling

```css
.my-component {
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

.my-component:hover {
  border-color: var(--phosphor);
  box-shadow: 0 0 10px var(--phosphor-glow);
}
```

### Status Badge

```css
.status-success {
  background-color: var(--green-subtle);
  color: var(--green);
  border: 1px solid var(--green-border);
}
```

### Interactive Button

```css
.button {
  background: var(--phosphor);
  color: var(--screen);
  border: 1px solid var(--phosphor-bright);
  box-shadow: 0 0 10px var(--phosphor-glow);
  transition: all var(--transition-fast);
}

.button:hover {
  background: var(--phosphor-bright);
  box-shadow: var(--glow-phosphor-bright);
}
```

## Theme Customization

The design system supports dark and light themes via the `data-theme` attribute:

```html
<html data-theme="dark">
```

To toggle themes in JavaScript:

```typescript
document.documentElement.setAttribute('data-theme', 'light');
```

All tokens automatically adjust for the current theme.

## Component Styling Patterns

### Cards/Panels
- Use `--surface` for background
- Use `--border` for borders
- Use `--radius-md` or `--radius-lg` for corners
- Add `--shadow-panel` for CRT glow effect

### Text Hierarchy
- `--text` for primary content
- `--text-dim` for secondary/meta info
- `--text-faint` for placeholders/disabled
- `--phosphor` for emphasized/active text

### Interactive Elements
- Default: `--border` border color
- Hover: `--phosphor-dim` border, subtle glow
- Active: `--phosphor` accent, full glow
- Focus: `--focus-ring` outline

### Status Indicators
- Use semantic colors (`--green`, `--red`, `--cyan`)
- Background: use `*-subtle` variants
- Border: use `*-border` variants
- Add `*-glow` for emphasis

## Legacy Aliases

For backward compatibility, these aliases are provided:

| Alias | Maps To |
|-------|---------|
| `--color-bg-base` | `--screen` |
| `--color-bg-surface` | `--surface` |
| `--color-text-primary` | `--text` |
| `--color-text-secondary` | `--text-dim` |
| `--color-accent` | `--phosphor` |
| `--color-success` | `--green` |
| `--color-warning` | `--phosphor` |
| `--color-error` | `--red` |
| `--color-info` | `--cyan` |

Prefer using the primary token names in new code.

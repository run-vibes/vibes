---
id: REFACTOR0100
title: Responsive Header Navigation
type: refactor
status: done
priority: high
epics: [web-ui, design-system]
---

# Responsive Header Navigation

Add responsive behavior to the main header navigation to prevent overflow on smaller screens.

## Context

The main header contains:
- Logo
- Nav items (Sessions, Firehose, Models, Groove)
- Theme toggle button
- Settings button

On smaller viewports (tested on iPhone 14 Pro Max at 430px), these items overflow the header, breaking the layout and making navigation unusable.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Pattern | Slide-out panel from right |
| Breakpoint | 768px |
| Collapsed header | Logo + hamburger only |
| Panel width | 280px |
| Content order | Nav items → divider → theme/settings |

## Component Structure

**Collapsed state (below 768px):**
```
<header class="header">
  <Logo />
  <nav class="desktopNav">...</nav>       <!-- Hidden -->
  <div class="desktopActions">...</div>   <!-- Hidden -->
  <button class="hamburger">☰</button>    <!-- Visible -->
</header>

<aside class="mobileMenu">
  <button class="closeButton">✕</button>
  <nav class="mobileNav">...</nav>
  <div class="mobileActions">
    <ThemeToggle />
    <SettingsLink />
  </div>
</aside>
<div class="overlay" />
```

## Panel Layout

```
┌─────────────────────────────┐
│                         ✕   │
│                             │
│  SESSIONS                   │
│  FIREHOSE                   │
│  MODELS                     │
│  GROOVE              ▾      │
│                             │
│  ─────────────────────────  │
│                             │
│  ◐  THEME                   │
│  ⚙  SETTINGS                │
└─────────────────────────────┘
```

## Styling

**Hamburger Button:**
- 44x44px touch target
- Color: `var(--text-dim)`, hover: `var(--phosphor)` with glow

**Panel:**
- Width: 280px, height: 100vh, fixed position
- Background: `var(--surface)` with `var(--border)` left edge
- Box-shadow: subtle phosphor glow for CRT feel

**Animation:**
- Panel: `translateX(100%)` → `translateX(0)` over 250ms ease-out
- Overlay: opacity 0 → 0.6 over 200ms
- Content: stagger fade-in (50ms per item)

**Overlay:**
- Dark theme: `rgba(0, 0, 0, 0.6)`
- Light theme: `rgba(0, 0, 0, 0.3)`

## Accessibility

- `Escape` key closes menu
- Focus trap within panel when open
- Focus returns to hamburger on close
- Route change auto-closes menu
- `prefers-reduced-motion` respected

**ARIA:**
```tsx
<button aria-label="Open menu" aria-expanded={isOpen} aria-controls="mobile-menu">
<aside id="mobile-menu" role="dialog" aria-modal="true" aria-label="Navigation menu">
```

## Acceptance Criteria

- [x] Define breakpoint: 768px
- [x] Design: hamburger + slide-out panel from right
- [x] Design: Logo + hamburger visible, everything else in menu
- [x] Create MobileMenu component with slide-out panel
- [x] Add hamburger button to Header (hidden above 768px)
- [x] Hide desktop nav/actions below 768px
- [x] Implement open/close animation (250ms ease-out)
- [x] Close on: overlay click, close button, Escape key, route change
- [x] Focus trap and ARIA attributes
- [x] Respect prefers-reduced-motion
- [x] Test on iPhone 14 Pro Max (430px) and tablet (768px)
- [x] Add Ladle stories for mobile viewport

## Files to Change

**Modify:**
- `design-system/src/compositions/Header/Header.tsx`
- `design-system/src/compositions/Header/Header.module.css`
- `design-system/src/compositions/Header/Header.stories.tsx`

**Create:**
- `design-system/src/compositions/Header/MobileMenu.tsx`
- `design-system/src/compositions/Header/MobileMenu.module.css`

## Out of Scope

- Identity/email display (see feat-0101)
- Swipe-to-close gesture

## Size

M - Medium

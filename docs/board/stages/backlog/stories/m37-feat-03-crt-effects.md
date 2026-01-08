---
id: F014
title: Feature: Implement CRT Visual Effects
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

# Feature: Implement CRT Visual Effects

## Problem

The UI lacks the distinctive CRT aesthetic: scanlines, vignette, and phosphor glow that create the retro-futuristic atmosphere.

## Goal

Add optional CRT effects (scanlines, vignette) that can be toggled by users, defaulting to enabled.

## Tasks

### Task 1: Create Scanline Overlay

Add CSS for scanline effect:
```css
.crt-effects::before {
  content: '';
  position: fixed;
  inset: 0;
  background: repeating-linear-gradient(
    0deg,
    var(--scanline) 0px,
    var(--scanline) 1px,
    transparent 1px,
    transparent 2px
  );
  pointer-events: none;
  z-index: 9999;
}
```

### Task 2: Create Vignette Overlay

Add CSS for vignette effect:
```css
.crt-effects::after {
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
  z-index: 9998;
}
```

Light theme uses lower opacity (0.1).

### Task 3: Create CRT Effects Container

Wrap app in container that applies effects:
```tsx
<div className={crtEffectsEnabled ? 'crt-effects' : ''}>
  {children}
</div>
```

### Task 4: Add Effects Toggle Setting

Add toggle in settings panel:
- "CRT Effects" on/off toggle
- Persists to localStorage
- Default: enabled

### Task 5: Create Glow Utility Classes

Add utility classes for phosphor glow:
```css
.glow-text {
  text-shadow: 0 0 10px var(--phosphor);
}
.glow-box {
  box-shadow: 0 0 20px rgba(255, 176, 0, 0.3);
}
```

## Considerations

- Effects may impact performance on low-end devices
- Provide easy toggle for accessibility
- Scanlines can interfere with small text readability

## Acceptance Criteria

- [x] Scanlines visible across entire viewport
- [x] Vignette creates subtle edge darkening
- [x] Effects work in both dark and light themes
- [x] Toggle persists to localStorage
- [x] Disabling removes all effects
- [x] No performance degradation on modern devices

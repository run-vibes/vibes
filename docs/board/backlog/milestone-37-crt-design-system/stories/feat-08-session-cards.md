---
created: 2026-01-04
status: done
---

# Feature: Restyle Session Cards with Glow Effects

## Problem

Session cards use generic styling that doesn't match the CRT aesthetic.

## Goal

Restyle session cards with phosphor glow, CRT typography, and status indicators.

## Tasks

### Task 1: Restyle Card Container

Apply CRT styling to session cards:
```css
.session-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}
.session-card:hover {
  border-color: var(--phosphor-dim);
  box-shadow: 0 0 15px rgba(255, 176, 0, 0.2);
}
```

### Task 2: Style Session Header

Apply CRT typography to header:
- Session name in display font (VT323)
- Phosphor color for active sessions
- Dim text for inactive

### Task 3: Style Session Metadata

Apply styling to session info:
- Created time in dim monospace
- Duration badge
- Event count

### Task 4: Style Status Indicators

Add phosphor glow to active status:
```css
.session-active {
  border-left: 3px solid var(--phosphor);
  box-shadow: inset 3px 0 10px rgba(255, 176, 0, 0.15);
}
.status-dot.active {
  background: var(--phosphor);
  box-shadow: 0 0 8px var(--phosphor);
  animation: pulse 2s infinite;
}
```

### Task 5: Style Quick Actions

Apply CRT styling to card actions:
- Icon buttons with phosphor hover
- Tooltips matching CRT style

### Task 6: Add Pulse Animation for Active

Active sessions get subtle pulsing:
```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
```

## Acceptance Criteria

- [x] Session cards use CRT surface/border
- [x] Active sessions have phosphor glow
- [x] Hover state shows enhanced glow
- [x] Status indicator pulses for active
- [x] Typography matches design spec
- [x] Actions have hover effects

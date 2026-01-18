---
id: FEAT0149
title: "Feature: Apply CRT Styling to Event Firehose"
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

# Feature: Apply CRT Styling to Event Firehose

## Problem

The firehose event stream uses generic styling that doesn't match the CRT aesthetic.

## Goal

Restyle the firehose with CRT colors, typography, and subtle glow effects for events.

## Tasks

### Task 1: Restyle Firehose Container

Apply CRT styling to main container:
```css
.firehose {
  background: var(--screen);
  font-family: var(--font-mono);
}
```

### Task 2: Restyle Event Items

Apply CRT styling to individual events:
```css
.event-item {
  background: var(--surface);
  border: 1px solid var(--border);
  border-left: 3px solid var(--phosphor-dim);
}
.event-item:hover {
  border-left-color: var(--phosphor);
  box-shadow: 0 0 10px rgba(255, 176, 0, 0.1);
}
```

### Task 3: Style Event Metadata

Apply typography to event metadata:
- Timestamp in dim text
- Event type as badge with phosphor accent
- Session ID in dim monospace

### Task 4: Style Event Content

Apply styling to event content area:
- Tool names in phosphor color
- JSON/code in mono font
- Expand/collapse indicators

### Task 5: Style New Event Animation

Add subtle glow animation for new events:
```css
@keyframes event-arrive {
  0% { box-shadow: 0 0 20px rgba(255, 176, 0, 0.5); }
  100% { box-shadow: none; }
}
.event-item.new {
  animation: event-arrive 0.5s ease-out;
}
```

### Task 6: Style Empty State

Apply CRT styling to "no events" state:
- Centered message in display font
- Phosphor accent color
- Subtle pulsing glow (optional)

## Acceptance Criteria

- [x] Firehose uses CRT background/typography
- [x] Event items have phosphor border accent
- [x] Hover state shows glow
- [x] Metadata uses appropriate font sizes
- [x] New events have arrival animation
- [x] Empty state matches aesthetic

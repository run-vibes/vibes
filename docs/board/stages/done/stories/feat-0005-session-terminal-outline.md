---
id: feat-0005-session-terminal-outline
title: "Feature: Apply Terminal Outline to Session Detail Page"
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
---

# Feature: Apply Terminal Outline to Session Detail Page

## Summary

Apply the terminal panel styling from the multi-terminal prototype to the session detail page (`/sessions/[:id]`), giving it the distinctive CRT terminal appearance.

## Design Reference

See prototype: `docs/design/prototypes/21-sessions-terminals.html`

## Requirements

### Terminal Panel Structure

The session detail page should use the terminal outline pattern:

```
┌─────────────────────────────────────────────────────────┐
│ ● session-name   abc123   │ 47 tools  1h 23m │ [ACTIONS]│  ← header
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Session content / transcript                           │  ← body
│                                                         │
├─────────────────────────────────────────────────────────┤
│ ⟩ [input field]                                        │  ← input
└─────────────────────────────────────────────────────────┘
```

### Terminal Header

- **Status indicator** - Colored dot (green=active, dim=idle, red=error)
- **Session name** - Display font, phosphor color
- **Session ID** - Truncated, dim color
- **Metadata** - Tool count, duration
- **Action buttons** - PAUSE, INJECT, LOGS, etc.

### Terminal Body

- Scrollable content area
- Styled output lines:
  - Prompt lines (cyan)
  - Command lines (phosphor)
  - Output lines (dim, with success/error/info variants)
  - Thinking lines (magenta, italic)
  - Tool invocation lines (with left border)

### Terminal Input

- Prompt character (⟩)
- Input field with placeholder
- Blinking cursor animation

### Focus State

- When terminal is focused: bright border, subtle glow, inset shadow

## Technical Implementation

- Create `TerminalPanel` component with header/body/input slots
- Create `TerminalHeader` component
- Create `TerminalLine` component with variants
- Create `TerminalInput` component
- Apply to session detail page layout

## Acceptance Criteria

- [ ] Session detail page uses terminal outline structure
- [ ] Header shows status, name, ID, metadata, and actions
- [ ] Body displays session transcript with styled lines
- [ ] Input section at bottom with prompt and cursor
- [ ] Focus state styling works correctly
- [ ] Follows CRT design system tokens

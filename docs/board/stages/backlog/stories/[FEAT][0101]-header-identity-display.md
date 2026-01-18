---
id: FEAT0101
title: Header Identity Display
type: feat
status: pending
priority: low
epics: [web-ui, design-system]
depends: [refactor-0100]
---

# Header Identity Display

Show authenticated user identity in the header.

## Context

The Header component has an `identity` prop that accepts `{ email: string; provider?: string }` but it's not currently used in the application. Once authentication is implemented, users should see their identity in the header.

## Acceptance Criteria

- [ ] Display user email in header actions area (desktop)
- [ ] Display user email at bottom of mobile menu panel
- [ ] Style: small text, `var(--text-dim)` color
- [ ] Truncate long emails with ellipsis
- [ ] Optional: show provider icon/badge

## Design

**Desktop:** Email appears in actions area, left of theme toggle
**Mobile:** Email appears at bottom of slide-out panel, separated by divider

## Size

S - Small

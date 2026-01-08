---
id: feat-0004-subnav
title: "Feature: Implement Plugin Subnav Bar"
type: feat
status: backlog
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
---

# Feature: Implement Plugin Subnav Bar

## Summary

Implement a collapsible subnav bar that appears when clicking on plugin nav items (like GROOVE). The subnav provides secondary navigation for plugin-specific views.

## Design Reference

See prototype: `docs/design/prototypes/28-crt-essence-v5-subnav.html`

## Requirements

### Subnav Bar Behavior

- Subnav bar slides open when clicking a nav item with `has-subnav` class
- Subnav bar collapses when clicking a non-plugin nav item or logo
- Smooth height transition (0.2s ease-out)
- Shows plugin label on left, nav items in center, optional stats on right

### Groove Subnav Items

1. **Security** - Security monitoring and alerts
2. **Assessment** - Assessment results and analysis

### Styling

- Follows CRT design system tokens
- Plugin-specific accent colors (cyan for groove)
- Active state with border and subtle background
- Icons optional per subnav item

### Technical Implementation

- Add subnav state to router/navigation context
- Create `SubnavBar` component
- Update `NavItem` to support `hasSubnav` prop
- Route subnav items to appropriate views (e.g., `/groove/security`, `/groove/assessment`)

## Acceptance Criteria

- [ ] Clicking GROOVE shows subnav bar with Security and Assessment items
- [ ] Clicking a subnav item navigates to the correct route
- [ ] Clicking another main nav item (DASH, SESS, FIRE) closes subnav
- [ ] Clicking logo closes subnav and returns to home
- [ ] Subnav animates smoothly open/closed
- [ ] Active states work correctly for both main nav and subnav

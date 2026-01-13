---
id: refactor-0100
title: Responsive Header Navigation
type: refactor
status: pending
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

On smaller viewports, these items overflow the header, breaking the layout and making navigation unusable.

## Problem

- Header items overflow on narrow screens
- No responsive breakpoint handling
- Critical navigation becomes inaccessible

## Acceptance Criteria

- [ ] Define breakpoint where collapse should occur
- [ ] Design: hamburger menu icon and expanded state
- [ ] Design: decide what goes in hamburger vs stays visible
- [ ] Implementation: create responsive Header component
- [ ] Implementation: hamburger menu with slide-out or dropdown
- [ ] Implementation: smooth open/close animation
- [ ] Implementation: close menu on route change
- [ ] Implementation: close menu on click outside
- [ ] Implementation: keyboard accessibility (Escape to close, focus trap)
- [ ] Test on various viewport sizes (mobile, tablet, desktop)
- [ ] Add Ladle stories showing collapsed and expanded states

## Design Considerations

What should stay visible vs collapse:
- **Always visible**: Logo (clickable to home)
- **Collapse candidates**: Nav items, theme toggle, settings

Menu patterns to consider:
1. **Hamburger → slide-out panel**: Full-height sidebar from right/left
2. **Hamburger → dropdown**: Menu drops below header
3. **Priority+ pattern**: Show as many items as fit, overflow into "more" menu

Should maintain CRT aesthetic in menu styling.

## Technical Notes

Consider touch targets - minimum 44x44px for mobile.

```tsx
// Rough structure
<Header>
  <Logo />
  <Nav className="desktop-only">...</Nav>
  <HamburgerButton className="mobile-only" />
  <MobileMenu isOpen={open}>
    <Nav />
    <ThemeToggle />
    <SettingsLink />
  </MobileMenu>
</Header>
```

## Size

M - Medium (new component, responsive logic, animations)

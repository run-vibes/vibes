---
id: refactor-0096
title: Simplify Groove Navigation Hierarchy
type: refactor
status: pending
priority: high
epics: [web-ui, groove]
---

# Simplify Groove Navigation Hierarchy

Flatten the 3-level Groove navigation to 2 levels for better usability.

## Context

Currently Groove has 3 levels of navigation:
1. **Main nav**: Sessions, Firehose, Models, Groove
2. **SubnavBar**: Security, Assessment, Dashboard
3. **Dashboard tabs**: Overview, Learnings, Attribution, Strategy, Health, OpenWorld

This creates 9 navigation choices (3 subnav + 6 tabs) which is overwhelming. Users shouldn't need to make this many decisions to find content.

## Problem

- Too many choices at once (cognitive overload)
- Not intuitive which level to look at for specific content
- Dashboard tabs feel like a 4th section rather than children of Dashboard

## Acceptance Criteria

- [ ] Research: document current navigation paths and user flows
- [ ] Design: propose 2-level navigation structure
- [ ] Design: create mockups/wireframes for new hierarchy
- [ ] Implementation: restructure routes to match new hierarchy
- [ ] Implementation: update SubnavBar or remove if unnecessary
- [ ] Implementation: update all internal links/navigation
- [ ] Tests: verify all routes work correctly
- [ ] Ladle: update any affected stories

## Design Considerations

Possible approaches to explore:
1. **Merge subnav + tabs**: Make all 9 items peer-level in one nav
2. **Collapse dashboard**: Move Overview content to Dashboard landing, demote other tabs
3. **Restructure by function**: Group by user task rather than feature area
4. **Progressive disclosure**: Show tabs only when user enters Dashboard section

## Size

M - Medium (routing changes, component updates, testing)

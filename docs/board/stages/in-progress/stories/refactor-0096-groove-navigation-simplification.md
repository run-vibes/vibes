---
id: refactor-0096
title: Simplify Groove Navigation Hierarchy
type: refactor
status: in-progress
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

## Design

### New 2-Level Structure

Eliminate the 3-level hierarchy. Use SubnavBar as the single tab bar directly under the Groove header.

```
Header: Sessions | Firehose | Models | Groove | Streams | Settings
                                   ↓
Groove tabs: Status | Learnings | Security | Stream | Strategy | More ⋯
                                                                   ↓
                                                        OpenWorld | History
```

### Route Changes

| Tab | Route | Content |
|-----|-------|---------|
| Status | `/groove/status` | Combined health view (Overview + Health + Assessment Status) |
| Learnings | `/groove/learnings` | Learning insights + attribution (merged) |
| Security | `/groove/security` | Quarantine page |
| Stream | `/groove/stream` | Live event inspection |
| Strategy | `/groove/strategy` | Configuration |
| More → OpenWorld | `/groove/openworld` | Experimental (demoted) |
| More → History | `/groove/history` | Session history (demoted) |

**Default:** `/groove` redirects to `/groove/status`

**Removed routes:** `/groove/assessment/*` and `/groove/dashboard/*` (no backwards compat needed)

### Component Changes

| File | Change |
|------|--------|
| `SubnavBar.tsx` | Remove label, add overflow "More" menu support |
| `App.tsx` | Update routes and `grooveSubnavItems` |
| `DashboardLayout.tsx` | **Delete** |
| `AssessmentLayout.tsx` | **Delete** |

**New/modified pages:**
- `StatusPage.tsx` — Merge Overview + Health + Assessment Status
- `LearningsPage.tsx` — Merge Learnings + Attribution
- Relocate: Stream, Strategy, OpenWorld, History (minimal changes)

### Implementation Phases

**Phase 1: Route restructure**
- Update routes in App.tsx (flat `/groove/*` paths)
- Update SubnavBar items, remove label prop

**Phase 2: Component consolidation**
- Create StatusPage (merge 3 pages)
- Update LearningsPage (add Attribution)
- Move Stream, Strategy, OpenWorld, History
- Delete DashboardLayout and AssessmentLayout

**Phase 3: More menu**
- Add dropdown to SubnavBar
- Wire OpenWorld and History under More

**Phase 4: Cleanup & testing**
- Update e2e tests for new routes
- Run `just web visual-update`
- Update Ladle stories
- Remove dead code

## Size

M - Medium (routing changes, component updates, testing)

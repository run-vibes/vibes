# Milestone 02: Board Restructure - Design Document

> Reorganize the kanban board hierarchy to Epic > Milestone > Story for clearer traceability.

## Overview

Restructure the board hierarchy from the current Milestone > Epic > Story to Epic > Milestone > Story, matching the Linear model. Add icebox stage and standardize story naming.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Hierarchy** | Epic > Milestone > Story | Matches mental model; epics are initiatives, milestones are increments |
| **Story naming** | `[TYPE][NNNN]-verb-phrase` | Visually distinct type/ID, easy to grep |
| **Icebox stage** | New stage for blocked/deferred | Distinguishes "not now" from "ready to work" |
| **Epic lifecycle** | Can close when complete | Epics are initiatives, not eternal categories |

---

## New Hierarchy

```
Epic (major initiative)
  └── Milestone (shippable increment)
       └── Story (atomic work)
```

### Layer Definitions

| Layer | Purpose | Lifecycle | Naming |
|-------|---------|-----------|--------|
| **Epic** | Major initiative or capability area | Can close when complete | `<slug>`: `coherence-verification` |
| **Milestone** | Shippable increment within an epic | Closes when shipped | `NN-<value-statement>`: `01-artifact-pipeline` |
| **Story** | Atomic unit of work | Moves through stages | `[TYPE][NNNN]-verb-phrase` |

---

## Directory Structure

### Current Structure

```
docs/board/
├── epics/
│   └── <epic>/
│       ├── README.md
│       └── <story>.md -> symlink to stages/
├── milestones/
│   └── NN-<name>/
│       ├── README.md
│       ├── design.md
│       └── <epic> -> symlink to epics/
└── stages/
    ├── backlog/stories/
    ├── in-progress/stories/
    └── done/stories/
```

### New Structure

```
docs/board/
├── epics/
│   └── <epic>/
│       ├── README.md              # Epic overview, status
│       └── milestones/
│           └── NN-<value>/
│               ├── README.md      # Milestone overview
│               └── design.md      # Architecture decisions
├── stages/
│   ├── icebox/stories/            # NEW: blocked/deferred
│   ├── backlog/stories/           # Ready to work
│   ├── in-progress/stories/       # Currently working
│   └── done/stories/              # Completed
└── milestones/                    # DEPRECATED: remove after migration
```

---

## Story Naming Convention

### Format

```
[TYPE][NNNN]-verb-phrase.md
```

### Examples

```
[FEAT][0042]-add-session-export.md
[FIX][0043]-resolve-websocket-timeout.md
[CHORE][0110]-add-mutation-testing-workflow.md
[REFACTOR][0111]-extract-event-projection.md
```

### Types

| Type | Use Case |
|------|----------|
| `FEAT` | New functionality |
| `FIX` | Bug fix |
| `CHORE` | Maintenance, tooling |
| `REFACTOR` | Code restructuring |

---

## Stages

### New Stage: Icebox

Add `stages/icebox/stories/` for work that is:
- Documented but blocked on dependencies
- Good ideas deferred to later
- Low priority items

### Stage Definitions

| Stage | Path | Description |
|-------|------|-------------|
| **icebox** | `stages/icebox/stories/` | Blocked or deferred |
| **backlog** | `stages/backlog/stories/` | Ready to work on |
| **in-progress** | `stages/in-progress/stories/` | Currently being worked on |
| **done** | `stages/done/stories/` | Completed |

---

## Commands

### Updated Commands

| Command | Action |
|---------|--------|
| `just board ice <id>` | Move story to icebox |
| `just board thaw <id>` | Move story from icebox to backlog |
| `just board new story "title" <type>` | Create story with `[TYPE][NNNN]` naming |

### State Transition Rules

| Trigger | Command | Notes |
|---------|---------|-------|
| Starting work | `just board start <id>` | Move to in-progress |
| Work complete | `just board done <id>` | Move to done, update changelog |
| Blocked/deferred | `just board ice <id>` | Move to icebox |
| Blocker resolved | `just board thaw <id>` | Move to backlog |

### Autonomous Operation Rules

When Claude should transition:

| Action | When |
|--------|------|
| `start` | Before writing implementation code |
| `done` | Immediately after PR merged or work complete |
| `ice` | When explicitly deciding to defer |
| `thaw` | When revisiting an iced story |

---

## Story Frontmatter

### Updated Fields

```yaml
---
id: FEAT0042
title: Add session export
type: feat
status: in-progress
milestone: coherence-verification/01-artifact-pipeline  # NEW: parent link
epic: coherence-verification                            # Derived from milestone
priority: medium
depends: []
created: 2026-01-17
updated: 2026-01-17
---
```

---

## Migration Plan

### Phase 1: Add New Structure

1. Create `stages/icebox/stories/`
2. Update `just board new story` to use new naming
3. Add `ice` and `thaw` commands
4. Create new epic structure under `epics/<epic>/milestones/`

### Phase 2: Migrate Existing Stories

1. Rename existing stories to `[TYPE][NNNN]-verb-phrase` format
2. Update frontmatter with `milestone` field
3. Update symlinks

### Phase 3: Deprecate Old Structure

1. Move milestone designs to new location
2. Remove old `milestones/` symlinks
3. Update CONVENTIONS.md

---

## Branch Naming

Standardize branch names to match story IDs:

```
<type>/<nnnn>-<short-desc>

Examples:
feat/0042-session-export
fix/0043-websocket-timeout
chore/0110-mutation-testing
```

---

## Dependencies

No external dependencies - this is board tooling changes.

---

## Testing Strategy

| Component | Test Coverage |
|-----------|---------------|
| `just board ice` | Manual: verify file moves correctly |
| `just board thaw` | Manual: verify file moves correctly |
| Story naming | Manual: verify format is correct |
| Symlink updates | Manual: verify links remain valid |

---

## Deliverables

- [ ] `stages/icebox/stories/` directory
- [ ] Updated `just board new story` command with new naming
- [ ] `just board ice` command
- [ ] `just board thaw` command
- [ ] Epic directory structure under `epics/<epic>/milestones/`
- [ ] Migration script for existing stories
- [ ] Updated CONVENTIONS.md
- [ ] Updated CLAUDE.md

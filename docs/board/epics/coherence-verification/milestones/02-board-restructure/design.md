# Milestone 02: Board Restructure - Design Document

> Reorganize the kanban board hierarchy to Epic > Milestone > Story for clearer traceability.

## Overview

Restructure the board hierarchy from the current Milestone > Epic > Story to Epic > Milestone > Story, matching the Linear model. Add icebox stage and standardize story naming. Migrate 55 existing milestones into their parent epics and rename ~200 stories to the new format.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Hierarchy** | Epic > Milestone > Story | Matches mental model; epics are initiatives, milestones are increments |
| **Story naming** | `[TYPE][NNNN]-verb-phrase` | Visually distinct type/ID, easy to grep |
| **Icebox stage** | New stage for blocked/deferred | Distinguishes "not now" from "ready to work" |
| **Epic lifecycle** | Can close when complete | Epics are initiatives, not eternal categories |
| **Milestone assignment** | Auto-assign to first epic in frontmatter | Move milestones into `epics/<epic>/milestones/` |
| **Milestone-prefixed stories** | Drop prefix, track in frontmatter | `m26-feat-01` → `[FEAT][NNNN]` with `milestone: 26` |
| **Done section** | Show milestones/epics, not stories | Completed work at milestone granularity |
| **Doc sync** | Auto-update on state transitions | Commands update milestone/epic docs automatically |

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

## Board README Layout

The generated README groups milestones under their parent epics:

```markdown
# Planning Board

## In Progress
| Story | Type | Priority | Epic |
|-------|------|----------|------|
| [FEAT][0111]-eval-cli | feat | medium | evals |

## Backlog
| Story | Type | Priority | Epic |
|-------|------|----------|------|
| [CHORE][0087]-use-cranelift | chore | low | dev-environment |

## Icebox
| Story | Type | Priority | Blocked By |
|-------|------|----------|------------|

## Epics

### core (active) — 10 milestones, 8 done
| Milestone | Status | Stories |
|-----------|--------|---------|
| [09-multi-session](epics/core/milestones/09-multi-session/) | done | 3/3 |

### evals (in-progress) — 6 milestones, 0 done
| Milestone | Status | Stories |
|-----------|--------|---------|
| [39-eval-core](epics/evals/milestones/39-eval-core/) | in-progress | 4/6 |

## Done

<details>
<summary>Completed Epics & Milestones</summary>

### [groove](epics/groove/) (done)
- [15-harness-introspection](epics/groove/milestones/15-harness-introspection/) (2024-12-01)
- [29-assessment-framework](epics/groove/milestones/29-assessment-framework/) (2024-12-10)

### [core](epics/core/) — Completed Milestones
- [01-core-proxy](epics/core/milestones/01-core-proxy/) (2024-11-15)

</details>
```

---

## Automatic Doc Sync

State transition commands update related milestone and epic docs:

| Command | Updates |
|---------|---------|
| `just board start <story>` | Updates milestone's `implementation.md` if story is milestone-linked |
| `just board done <story>` | Marks story done in milestone doc; prompts if last story |
| `just board start-milestone <id>` | Updates epic README milestone table |
| `just board done-milestone <id>` | Updates epic README; prompts if all milestones done |
| `just board done-epic <id>` | New command - marks epic as done |

### Milestone Frontmatter

Add `completed` field for tracking:

```yaml
---
id: 01-core-proxy
title: Core Proxy
status: done
epics: [core, networking]
completed: 2024-11-15
---
```

---

## Migration Plan

### Phase 1: Create Missing Epic

1. Create `epics/design-system/` with README

### Phase 2: Move Milestones into Epics

For each milestone in `milestones/`:
1. Read first epic from frontmatter (primary owner)
2. Create `epics/<epic>/milestones/` if needed
3. Move milestone directory
4. Update relative paths in design.md/implementation.md

**Epic-to-Milestone Mapping (55 milestones):**

| Epic | Count | Milestones |
|------|-------|------------|
| core | 10 | 01, 08, 09, 11, 12, 13, 14, 16, 18, 19, 28 |
| groove | 12 | 15, 21-25, 29-34, 36 |
| web-ui | 4 | 04, 17, 26, 27 |
| cli | 4 | 02, 10, 20, 35, 54 |
| tui | 6 | 41-46 |
| evals | 6 | 39, 48-52 |
| plugin-system | 3 | 03, 23, 53 |
| networking | 3 | 05, 06, 07 |
| mobile | 2 | 07, 55 |
| models | 1 | 37 |
| agents | 1 | 38 |
| observability | 1 | 40 |
| design-system | 1 | 47 |

### Phase 3: Rename Stories

1. Scan all stories in `stages/*/stories/`
2. For milestone-prefixed (`m26-feat-01-*`):
   - Extract milestone number, add to frontmatter
   - Assign new global ID
   - Rename to `[TYPE][NNNN]-name.md`
3. For legacy format (`feat-0042-*`):
   - Rename to `[FEAT][0042]-name.md`
4. Update all symlinks in epics

### Phase 4: Update Board Generator

1. Rewrite `generate` task for grouped epic layout
2. Add icebox section
3. Add done milestones/epics section with links
4. Show milestone progress (stories done/total)

### Phase 5: Update State Commands

1. Add milestone doc sync to `start`/`done` commands
2. Add `done-epic` command
3. Add completion prompts
4. Add `completed:` date to milestone frontmatter on done

### Phase 6: Cleanup

1. Remove empty `milestones/` directory
2. Update CONVENTIONS.md
3. Update CLAUDE.md

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

### Infrastructure
- [x] `stages/icebox/stories/` directory (already exists)
- [ ] `epics/design-system/` epic created
- [ ] All 55 milestones moved into `epics/<epic>/milestones/`
- [ ] All stories renamed to `[TYPE][NNNN]-verb-phrase.md` format
- [ ] All epic symlinks updated

### Board Generator
- [ ] Grouped epic sections with milestone tables
- [ ] Icebox section in README
- [ ] Done section shows milestones/epics with links
- [ ] Milestone progress tracking (stories done/total)

### State Commands
- [ ] `just board done-epic <id>` command
- [ ] Auto-sync milestone docs on story state change
- [ ] Auto-sync epic docs on milestone state change
- [ ] `completed:` date added to milestone frontmatter

### Documentation
- [ ] Updated CONVENTIONS.md
- [ ] Updated CLAUDE.md

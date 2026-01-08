---
id: C001
title: Kanban Planning Board - Design Document
type: chore
status: done
priority: medium
epics: [core]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
milestone: milestone-14-kanban-board
---
# Kanban Planning Board - Design Document

> Replace `docs/plans/` with a kanban-style `docs/board/` structure where directory location = workflow status.

## Overview

The current planning system uses numbered directories (`docs/plans/01-core-proxy/`) with a separate `PROGRESS.md` for status tracking. This creates friction:

1. **Hard to see what's active** — Directory names don't show status; requires cross-referencing PROGRESS.md
2. **Status updates are forgotten** — Manual checkbox updates and changelog entries get skipped
3. **Convention drift** — Files end up in wrong places without clear structural guidance

The new kanban structure makes **location = status**, which is self-enforcing. Moving a file IS the status update.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Structure** | Kanban directories | Location = status eliminates sync issues |
| **PROGRESS.md** | Delete (replaced by README.md) | Single source of truth |
| **PLAN.md** | Move to CONVENTIONS.md | Keep conventions, new location |
| **Changelog** | Separate CHANGELOG.md | Updated when items complete |
| **Frontmatter** | Minimal (`created:` only) | Simple, dates useful for staleness |
| **Tooling** | `just board` commands | Easier than remembering conventions |
| **Justfile** | Modular with `mod board` | Keep root justfile clean |

---

## Directory Structure

```
docs/board/
├── README.md                              # Auto-generated kanban view
├── CHANGELOG.md                           # Updated when items complete
├── CONVENTIONS.md                         # Planning conventions (was PLAN.md)
│
├── backlog/                               # Ideas and future work
│   ├── feat-0012-tab-completion.md
│   └── milestone-25-ios-app/
│       └── design.md                      # Design only, not yet scoped
│
├── ready/                                 # Designed, ready to start
│   └── milestone-20-setup-wizards/
│       ├── design.md
│       ├── implementation.md
│       └── stories/
│           ├── feat-0001-tunnel-wizard.md
│           └── feat-0002-auth-wizard.md
│
├── in-progress/                           # Currently being worked on
│   └── milestone-14-continual-learning/
│       ├── design.md
│       ├── implementation.md
│       └── stories/
│           ├── feat-0001-transcript-parser.md
│           └── feat-0002-learning-store.md
│
├── review/                                # Complete, awaiting review/merge
│   └── chore-0007-ci-caching.md
│
└── done/                                  # Completed work
    ├── milestone-01-core-proxy/
    ├── milestone-02-cli/
    └── bug-0001-cwd-propagation.md
```

### Naming Conventions

| Type | Format | Example |
|------|--------|---------|
| Milestone | `milestone-NN-name/` | `milestone-14-continual-learning/` |
| Feature | `feat-NNNN-name.md` | `feat-0012-tab-completion.md` |
| Bug | `bug-NNNN-name.md` | `bug-0001-cwd-propagation.md` |
| Chore | `chore-NNNN-name.md` | `chore-0007-ci-caching.md` |

### When to Use Each Type

| Type | Default Location | Description |
|------|------------------|-------------|
| **Milestone** | Has `design.md` + `implementation.md` | Features needing design and planning |
| **Feature** | Standalone `.md` file | Small features with obvious implementation |
| **Bug** | Standalone `.md` file | Bug fixes |
| **Chore** | Standalone `.md` file | Infrastructure, refactoring, maintenance |

---

## Auto-Generated README.md

The `just board` command scans directories and generates:

```markdown
# Planning Board

> Auto-generated from directory structure. Run `just board` to update.

## In Progress (1 milestone)

### milestone-14-continual-learning
**Phase 4: groove** — Continual learning system
Created: 2025-12-28

Stories:
- [~] feat-0001: Transcript parser
- [ ] feat-0002: Learning store

---

## Ready (1 milestone)

### milestone-20-setup-wizards
Created: 2025-12-30

- [ ] feat-0001: Tunnel wizard
- [ ] feat-0002: Auth wizard

---

## Review (1 item)

- [x] chore-0007: CI caching (2025-12-31)

---

## Backlog (2 items)

- feat-0012: Tab completion
- milestone-25-ios-app *(design only)*

---

## Done (19 milestones, 3 standalone)

<details>
<summary>View completed work</summary>

- milestone-01-core-proxy
- milestone-02-cli
- milestone-03-plugin-foundation
- ...
- bug-0001-cwd-propagation

</details>
```

---

## Tooling

### Modular Justfile

```
vibes/
├── justfile                    # Root - imports modules
├── .justfiles/
│   └── board.just              # Planning board commands
```

Root justfile addition:
```just
# Planning board management
mod board '.justfiles/board.just'
```

### Commands

| Command | Action |
|---------|--------|
| `just board` | Regenerate README.md from current structure |
| `just board new feat "description"` | Create `feat-NNNN-description.md` in backlog |
| `just board new bug "description"` | Create `bug-NNNN-description.md` in backlog |
| `just board new chore "description"` | Create `chore-NNNN-description.md` in backlog |
| `just board new milestone "name"` | Create `milestone-NN-name/` with design.md template in backlog |
| `just board start <item>` | Move backlog/ready → in-progress |
| `just board review <item>` | Move in-progress → review |
| `just board done <item>` | Move → done + prompt for CHANGELOG.md entry |
| `just board status` | Quick summary (counts per column) |
| `just board find "term"` | Search across all items |

### Implementation Notes

- Shell script or Rust binary (TBD)
- ID tracking via scanning existing files for highest number
- `done` command prompts for one-line changelog entry
- All commands auto-run regeneration at the end
- Idempotent where possible (moving to current location = no-op)

---

## File Format

### Standalone Items

Minimal frontmatter with just created date:

```markdown
---
created: 2025-12-28
---

# feat-0012: Tab Completion

## Summary

Add tab completion for CLI commands.

## Implementation

...
```

### Milestone Structure

```
milestone-NN-name/
├── design.md           # Architecture and decisions
├── implementation.md   # Step-by-step tasks
└── stories/            # Optional sub-tasks
    ├── feat-0001-*.md
    └── feat-0002-*.md
```

---

## Migration Plan

### File Moves

| Source | Destination |
|--------|-------------|
| `docs/plans/01-core-proxy/` | `docs/board/done/milestone-01-core-proxy/` |
| `docs/plans/02-cli/` | `docs/board/done/milestone-02-cli/` |
| ... (13 more completed) | `docs/board/done/milestone-NN-*/` |
| `docs/plans/14-continual-learning/` | `docs/board/in-progress/milestone-14-continual-learning/` |
| `docs/plans/2025-12-29-fix-cwd-propagation.md` | `docs/board/done/bug-0001-cwd-propagation.md` |
| Future Phase 5 items | `docs/board/backlog/milestone-20-*/` |

### Documentation Updates

| File | Action |
|------|--------|
| `README.md` (root) | Update links from `docs/plans/` → `docs/board/` |
| `CLAUDE.md` | Replace planning section with board workflow |
| `docs/PLAN.md` | Move to `docs/board/CONVENTIONS.md` |
| `docs/PROGRESS.md` | Delete (replaced by `docs/board/README.md`) |

### Migration Steps

1. Create `docs/board/` structure with empty directories
2. Create `.justfiles/board.just` with commands
3. Add `mod board` to root justfile
4. Move completed milestones to `done/`
5. Move active work to `in-progress/`
6. Create backlog items for planned Phase 5 work
7. Move PLAN.md → CONVENTIONS.md
8. Delete empty `docs/plans/` and `docs/PROGRESS.md`
9. Update CLAUDE.md with new workflow
10. Update root README.md links
11. Run `just board` to generate initial README.md
12. Seed CHANGELOG.md with summary of completed work
13. Single commit: `chore: migrate planning to kanban board structure`

---

## CLAUDE.md Updates

### Planning & Tracking Section

```markdown
## Planning & Tracking

**Use the board to track all work:**

- `just board` — Regenerate the board view
- `just board new feat "description"` — Create new feature
- `just board start <item>` — Begin work (moves to in-progress)
- `just board done <item>` — Complete work (moves to done + changelog)

**Before starting any task:**
1. Check `docs/board/in-progress/` for current work
2. If starting new work, use `just board start` or `just board new`

**After completing work:**
1. Run `just board done <item>`
2. This moves the item and prompts for a changelog entry
```

### Completion Workflow Section

```markdown
## Completing Work

**REQUIRED:** When implementation is complete:

1. **Verify quality:**
   - Run `just test` — all tests pass
   - Run `just pre-commit` — fmt, clippy, tests

2. **Update the board:**
   - Run `just board done <item>`
   - Enter a one-line changelog summary when prompted
   - This moves the item to `done/` and updates CHANGELOG.md

3. **Commit and push:**
   - Commit with conventional commit message
   - Push to origin: `git push -u origin <branch-name>`

4. **Create PR:**
   - `gh pr create --title "<type>: <description>" --body "..."`

**Never leave completed work:**
- Uncommitted or unpushed
- Still in `in-progress/` after merging
```

---

## Deliverables

- [ ] Create `docs/board/` directory structure
- [ ] Create `.justfiles/board.just` with all commands
- [ ] Add `mod board` to root justfile
- [ ] Migrate all `docs/plans/` content
- [ ] Move `docs/PLAN.md` → `docs/board/CONVENTIONS.md`
- [ ] Delete `docs/PROGRESS.md`
- [ ] Update `CLAUDE.md` with board workflow
- [ ] Update root `README.md` links
- [ ] Generate initial `docs/board/README.md`
- [ ] Create initial `docs/board/CHANGELOG.md`

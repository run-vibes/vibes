# Planning Conventions

This document describes the formal planning process and kanban board at `docs/board/`.

## Document Hierarchy

vibes follows a formal software engineering document hierarchy with full traceability:

```
VISION (product)
└── PRD (epic)
    └── SRS + DESIGN (milestone)
        └── Stories
            └── Verification
```

| Level | Document | Purpose | Location |
|-------|----------|---------|----------|
| Product | VISION.md | Product goals and roadmap | `docs/VISION.md` |
| Epic | PRD.md | User needs, business value | `epics/<name>/PRD.md` |
| Milestone | SRS.md | Requirements with verification criteria | `milestones/<id>/SRS.md` |
| Milestone | DESIGN.md | Architecture and implementation | `milestones/<id>/DESIGN.md` |
| Story | `[TYPE][NNNN]-name.md` | Implementable unit of work | `stages/<stage>/stories/` |

---

## Index

### The Board
| Section | Description |
|---------|-------------|
| [Board Structure](#board-structure) | Directory layout and organization |
| [Stages](#stages) | Story lifecycle: backlog, in-progress, done |
| [Epics](#epics) | Large initiatives with PRD |
| [Milestones](#milestones) | Deliverables with SRS and DESIGN |
| [Commands](#commands) | `just board` command reference |

### Planning
| Section | Description |
|---------|-------------|
| [Creating an Epic](#creating-an-epic) | Starting a new initiative |
| [Creating a Milestone](#creating-a-milestone) | Planning a deliverable |
| [Creating Stories](#creating-stories) | Breaking down into work units |
| [Traceability](#traceability) | Linking requirements to verification |

### Execution
| Section | Description |
|---------|-------------|
| [Story Lifecycle](#story-lifecycle) | Moving stories through stages |
| [Completing Work](#completing-work) | Final steps before PR |
| [Using Superpowers](#using-superpowers) | Skills for planning and execution |

### Standards
| Section | Description |
|---------|-------------|
| [Plugin vs Built-in](#architectural-decision-plugin-vs-built-in) | Where new features belong |
| [Best Practices](#best-practices) | Do's and don'ts |

---

# The Board

## Board Structure

```
docs/board/
├── README.md              # Auto-generated board view
├── CHANGELOG.md           # Updated when items complete
├── CONVENTIONS.md         # This file
├── stages/                # Story files organized by status
│   ├── backlog/stories/
│   ├── in-progress/stories/
│   ├── done/stories/
│   └── icebox/stories/
├── epics/                 # Large initiatives
│   └── <epic-name>/
│       ├── README.md      # Navigation and status
│       ├── PRD.md         # Product requirements
│       └── milestones/
│           └── <id>/
│               ├── README.md   # Navigation and status
│               ├── SRS.md      # Software requirements
│               └── DESIGN.md   # Architecture
└── templates/             # Templates for new items
    ├── story.md
    ├── VISION.md
    ├── PRD.md
    ├── SRS.md
    ├── DESIGN.md
    ├── epic-README.md
    └── milestone-README.md
```

### Hierarchy

```
Epic (large initiative with PRD)
└── Milestone (deliverable with SRS + DESIGN)
    └── Story (implementable unit of work)
```

| Level | Scope | Documents | Example |
|-------|-------|-----------|---------|
| **Epic** | Large initiative | PRD.md + README.md | "Coherence Verification" |
| **Milestone** | Deliverable with requirements | SRS.md + DESIGN.md + README.md | "Artifact Pipeline" |
| **Story** | Single implementable unit | Story file | "Add snapshot capture" |

## Stages

Stories live in `stages/<stage>/stories/` and move between stages as work progresses.

| Stage | Path | Description |
|-------|------|-------------|
| **backlog** | `stages/backlog/stories/` | Future work, not yet started |
| **in-progress** | `stages/in-progress/stories/` | Currently being worked on |
| **done** | `stages/done/stories/` | Completed work |
| **icebox** | `stages/icebox/stories/` | Blocked or deferred work |

### Story File Format

Stories use YAML frontmatter for metadata:

```yaml
---
id: FEAT0109
title: Board generator grouped layout
type: feat
status: in-progress
priority: high
scope: coherence-verification/01-artifact-pipeline
depends: []
estimate: 2h
created: 2026-01-17
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (`TYPE` + 4-digit number) |
| `title` | Yes | Human-readable title |
| `type` | Yes | `feat`, `bug`, `chore`, `refactor` |
| `status` | Yes | `backlog`, `in-progress`, `done`, `icebox` |
| `priority` | No | `low`, `medium`, `high`, `critical` |
| `scope` | No | `epic-name/milestone-id` or just `epic-name` |
| `depends` | No | List of story IDs that must complete first |
| `estimate` | No | Time estimate (e.g., `2h`, `1d`) |
| `created` | Yes | Creation date |

### Story Naming

```
[TYPE][NNNN]-verb-phrase.md
```

| Component | Description |
|-----------|-------------|
| `[TYPE]` | Story type in uppercase: `FEAT`, `BUG`, `CHORE`, `REFACTOR` |
| `[NNNN]` | Zero-padded 4-digit ID (auto-generated) |
| `verb-phrase` | Imperative description with hyphens |

## Epics

Epics are large initiatives containing milestones. Each epic has:

- `README.md` — Navigation and status summary
- `PRD.md` — Product requirements document

```
epics/coherence-verification/
├── README.md      # Status, milestones list
├── PRD.md         # Requirements, success criteria
└── milestones/
    ├── 01-artifact-pipeline/
    └── 02-ai-assisted-verification/
```

### Epic README Format

```yaml
---
id: coherence-verification
title: Coherence Verification
status: active
---
```

| Field | Values | Description |
|-------|--------|-------------|
| `status` | `backlog`, `active`, `done` | Epic lifecycle state |

### Epic PRD Format

The PRD defines user needs and requirements:

```markdown
# Epic Title — Product Requirements

> Value proposition

## Problem Statement
## Users
## Requirements
### Functional Requirements
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | ... | must |

### Non-Functional Requirements
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | ... | must |

## Success Criteria
## Milestones
```

Requirements use IDs (FR-01, NFR-01) that milestones reference in their SRS documents.

## Milestones

Milestones are deliverables with requirements and design. Each milestone has:

- `README.md` — Navigation and progress tracking
- `SRS.md` — Software requirements specification
- `DESIGN.md` — Architecture and implementation approach

```
milestones/01-artifact-pipeline/
├── README.md      # Story list, progress
├── SRS.md         # Requirements, verification
└── DESIGN.md      # Architecture, decisions
```

### Milestone README Format

```yaml
---
id: 01-artifact-pipeline
title: Artifact Pipeline
status: in-progress
epic: coherence-verification
---
```

| Field | Values | Description |
|-------|--------|-------------|
| `status` | `backlog`, `in-progress`, `done` | Milestone lifecycle state |

The README includes auto-generated story tables updated by `just board generate`.

### Milestone SRS Format

The SRS defines requirements with verification criteria:

```markdown
# Milestone Title — Software Requirements Specification

> Goal

**Epic:** [Epic Title](../../PRD.md)
**Status:** in-progress

## Scope
## Requirements
### Functional Requirements
| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | ... | FR-01 | test: ... |

## Stories
| Story | Requirements | Status |
|-------|--------------|--------|
| FEAT0042 | SRS-01 | backlog |

## Traceability
```

Key elements:
- **Source** — Links to PRD requirement (FR-01, NFR-01)
- **Verification** — How to verify (test, manual, file exists, glob check)
- **Stories** — Which stories implement which requirements

### Milestone DESIGN Format

The DESIGN documents architecture decisions:

```markdown
# Milestone Title — Design Document

> Summary

**SRS:** [SRS.md](SRS.md)

## Overview
## Architecture
## Key Decisions
| Decision | Choice | Rationale |
|----------|--------|-----------|

## Components
## Data Flow
## Error Handling
```

## Commands

### Story Management

| Command | Action |
|---------|--------|
| `just board new story "title"` | Create story in backlog (interactive type) |
| `just board new story feat "title"` | Create feature story |
| `just board new story bug "title"` | Create bug story |
| `just board start <id>` | Move story to in-progress |
| `just board done <id>` | Move story to done + changelog |
| `just board ice <id>` | Move story to icebox |
| `just board thaw <id>` | Move story from icebox to backlog |

### Epic and Milestone Management

| Command | Action |
|---------|--------|
| `just board new epic "name"` | Create epic with README.md + PRD.md |
| `just board new milestone "name"` | Create milestone with README.md + SRS.md + DESIGN.md |
| `just board start-milestone <id>` | Set milestone to in-progress |
| `just board done-milestone <id>` | Set milestone to done |
| `just board done-epic <id>` | Set epic to done |
| `just board link <story> <milestone>` | Link story to milestone |
| `just board unlink <story> <milestone>` | Remove story from milestone |

### Board Operations

| Command | Action |
|---------|--------|
| `just board` | Show available commands |
| `just board generate` | Regenerate README files with progress |
| `just board status` | Show counts per stage |

### Verification

| Command | Action |
|---------|--------|
| `just verify story <id>` | Run verification for a story |
| `just verify all` | Run all verification |

---

# Planning

## Creating an Epic

1. Run `just board new epic "Epic Name"`
   - Creates `epics/<name>/README.md` from template
   - Creates `epics/<name>/PRD.md` from template

2. Fill in the PRD:
   - Problem statement
   - Users
   - Functional requirements (FR-01, FR-02, ...)
   - Non-functional requirements (NFR-01, ...)
   - Success criteria

3. Run `just board generate` to update the board README

## Creating a Milestone

1. Run `just board new milestone "Milestone Name"`
   - Creates `milestones/<id>/README.md` from template
   - Creates `milestones/<id>/SRS.md` from template
   - Creates `milestones/<id>/DESIGN.md` from template
   - Updates epic README with new milestone

2. Fill in the SRS:
   - Scope
   - Requirements with verification criteria
   - Link requirements to PRD (Source column)
   - Define how each requirement is verified

3. Fill in the DESIGN:
   - Architecture overview
   - Key decisions with rationale
   - Components and data flow

## Creating Stories

1. Run `just board new story "Story Title"` or `just board new story feat "title"`

2. Link to milestone: `just board link <story-id> <milestone-id>`
   - Sets story's `scope` field
   - Creates symlink in milestone directory

3. Fill in the story:
   - Summary
   - Acceptance criteria
   - Implementation notes
   - Link to SRS requirements in story body

## Traceability

The document hierarchy creates full traceability:

```
VISION.md
    └── Epics roadmap
PRD.md (FR-01, FR-02, NFR-01)
    └── Milestones list
SRS.md (SRS-01 → FR-01, SRS-02 → FR-02)
    └── Stories list (Story X → SRS-01)
Story
    └── Verification annotations
verification/report.md
    └── Results
```

### Example Traceability Chain

1. **VISION.md** states goal: "Reduce spec-to-implementation drift"
2. **PRD.md** defines: `FR-01: System captures verification artifacts`
3. **SRS.md** specifies: `SRS-01: Capture screenshots during verification | Source: FR-01 | Verification: test`
4. **Story** implements: `FEAT0042` with `Requirements: SRS-01`
5. **Verification** confirms: `@verify screenshot` annotation runs and passes

---

# Execution

## Story Lifecycle

> **IMPORTANT:** Always use `just board` commands. Never manually move files.

### 1. Create Story

```bash
just board new story feat "Add session export"
```

### 2. Link to Milestone

```bash
just board link FEAT0042 01-artifact-pipeline
```

### 3. Start Work

```bash
just board start FEAT0042
```

When starting the first story of a milestone:
```bash
just board start-milestone 01-artifact-pipeline
```

### 4. Implement

Follow the acceptance criteria. Reference SRS requirements.

### 5. Complete Work

```bash
just board done FEAT0042
```

When completing the last story of a milestone:
```bash
just board done-milestone 01-artifact-pipeline
```

## Completing Work

Before marking a story done:

1. **Verify:** Run `just pre-commit` (fmt + clippy + test)

2. **Refactor:** Run `code-simplifier:code-simplifier` agent
   - Reviews for unnecessary complexity, over-engineering, YAGNI violations
   - Simplify any flagged code

3. **Verify story:** Run `just verify story <id>` if verification annotations exist

4. **Update story:** Ensure frontmatter `status: done`

5. **Move story:** Run `just board done <story-id>`

6. **Commit, push, create PR**

## Using Superpowers

| Skill | When to Use |
|-------|-------------|
| `superpowers:brainstorming` | Before any new feature or architecture decision |
| `superpowers:executing-plans` | When implementing a milestone plan |
| `superpowers:test-driven-development` | Before writing any implementation code |
| `superpowers:systematic-debugging` | When encountering bugs or unexpected behavior |

### Workflow

1. Use `superpowers:brainstorming` to explore options
2. Write PRD for new epic or SRS/DESIGN for new milestone
3. Create stories linked to requirements
4. Use `superpowers:executing-plans` for each story
5. Use `superpowers:test-driven-development` for implementation
6. Run `just pre-commit` and address issues
7. Complete: update story, move via commands, commit, push, PR

---

# Standards

## Architectural Decision: Plugin vs Built-in

When adding new functionality, evaluate whether it should be a plugin.

| Question | Plugin | Built-in |
|----------|--------|----------|
| Is this a first-party core feature? | Maybe | Yes |
| Should users be able to disable it? | Yes | No |
| Is it specific to certain use cases? | Yes | No |
| Would third parties want similar features? | Yes | No |

### Plugin API Capabilities

- **Session lifecycle hooks** - `on_session_created`, `on_turn_complete`, etc.
- **CLI command registration** - `vibes <plugin> <command>`
- **HTTP route registration** - `/api/plugins/<plugin>/...`
- **Configuration** - Persistent key-value store

## Best Practices

### Do

- Define requirements before implementation
- Link stories to SRS requirements
- Include verification criteria in SRS
- Break large features into focused stories
- Document the "why" alongside the "what"
- Follow TDD for testable code
- Use `just board` commands for state changes

### Don't

- Skip PRD/SRS for significant work
- Create stories without linking to requirements
- Leave verification criteria vague
- Create tasks that are too large
- Manually move story files
- Skip verification steps

## Review Checklist

Before implementing:

- [ ] PRD captures user needs and success criteria
- [ ] SRS defines requirements with verification methods
- [ ] DESIGN documents architecture decisions
- [ ] Stories are linked to SRS requirements
- [ ] Each story has clear acceptance criteria

Before completing:

- [ ] `just pre-commit` passes
- [ ] Code reviewed for simplicity
- [ ] Verification annotations present (if applicable)
- [ ] `just verify story <id>` passes (if applicable)
- [ ] Story moved via `just board done`

# Formal Planning Process — Design Document

> Traditional formal document hierarchy with automated progress tracking.

**SRS:** [SRS.md](SRS.md)

## Overview

Replace the ad-hoc planning structure with a formal hierarchy: VISION → PRD → SRS → DESIGN → Stories. Each document type has a clear purpose and templates. Just tasks automatically maintain progress and status in README navigation files.

## Architecture

### Document Hierarchy

```
docs/
├── VISION.md                              # Product vision & roadmap
└── board/
    └── epics/
        └── <epic-name>/
            ├── README.md                  # Navigation + summary
            ├── PRD.md                     # Epic requirements
            └── milestones/
                └── <NN-milestone-name>/
                    ├── README.md          # Navigation + summary
                    ├── SRS.md             # Requirements + verification
                    └── DESIGN.md          # Architecture + design
```

### Document Purposes

| Document | Contains | Answers |
|----------|----------|---------|
| `VISION.md` | Product direction, goals, non-goals, high-level architecture | Where are we going? |
| Epic `PRD.md` | User needs, business value, success criteria, scope | What problem does this epic solve? |
| Epic `README.md` | Links to PRD + milestones, status summary | How do I navigate this epic? |
| `SRS.md` | Functional/non-functional requirements, verification methods | What must be built and how do we verify it? |
| `DESIGN.md` | Architecture decisions, component design, data flow, trade-offs | How do we build it? |
| Milestone `README.md` | Links to SRS + DESIGN, story index, status | How do I navigate this milestone? |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Document naming | Uppercase (PRD.md, SRS.md, DESIGN.md) | Visual distinction for formal docs |
| Keep README.md | Yes | GitHub navigation, summary at a glance |
| PRD per epic | Yes | Epics are natural product boundaries |
| SRS per milestone | Yes | Milestones are implementation units |
| Verification in SRS | Yes | Requirements and verification belong together |
| Replace implementation.md | With SRS.md | SRS lists requirements + links to stories |

## Templates

### VISION.md (Product Level)

```markdown
# [Product Name]

> [One-line vision statement]

## Goals

1. **[Goal 1]** — Description
2. **[Goal 2]** — Description

## Non-Goals

- [What we're explicitly not doing]

## Architecture Overview

[High-level system diagram]

## Epics

| Epic | Status | Description |
|------|--------|-------------|
| [epic-name](board/epics/epic-name/) | active | Brief description |

## Roadmap

[Future direction, phases]
```

### Epic PRD.md

```markdown
# [Epic Name] — Product Requirements

> [One-line value proposition]

## Problem Statement

[What user problem does this solve?]

## Users

[Who benefits from this?]

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | [User can...] | must |
| FR-02 | [System shall...] | should |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | [Performance: ...] | must |

## Success Criteria

- [ ] [Measurable outcome 1]
- [ ] [Measurable outcome 2]

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 01 | [Name](milestones/01-name/) | done |
```

### Epic README.md

```markdown
---
id: <epic-id>
title: <Epic Title>
status: active | done
---

# [Epic Title]

> [One-line from PRD]

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |

## Milestones

| ID | Milestone | Status | Progress |
|----|-----------|--------|----------|
| 01 | [Name](milestones/01-name/) | done | 6/6 |

## Status

**Active Milestone:** NN-name
**Stories In Progress:** N
**Completion:** N/M milestones done
```

### Milestone SRS.md

```markdown
# [Milestone Name] — Software Requirements Specification

> [One-line goal]

**Epic:** [Link to epic PRD](../PRD.md)
**Status:** in-progress

## Scope

[What this milestone delivers]

## Requirements

### Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | [Requirement] | FR-01 | [verification method] |

### Non-Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-NFR-01 | [Requirement] | NFR-01 | [verification method] |

## Stories

| Story | Requirements | Status |
|-------|--------------|--------|
| [ID](link) | SRS-01, SRS-02 | done |

## Traceability

- **Source:** Epic PRD requirements
- **Implements:** Stories
- **Verified by:** `just verify story <ID>`
```

### Milestone DESIGN.md

```markdown
# [Milestone Name] — Design Document

> [One-line summary of approach]

**SRS:** [SRS.md](SRS.md)

## Overview

[How this milestone achieves its requirements]

## Architecture

[Diagrams, component relationships]

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| [Decision] | [Choice] | [Why] |

## Components

### [Component 1]

[Purpose, interface, behavior]

## Data Flow

[How data moves through the system]

## Error Handling

[What can go wrong, how to handle it]
```

### Milestone README.md

```markdown
---
id: <milestone-id>
title: <Milestone Title>
status: backlog | in-progress | done
epic: <epic-id>
---

# [Milestone Title]

> [One-line from SRS]

## Documents

| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [DESIGN.md](DESIGN.md) | Architecture and implementation details |

## Stories

| Story | Description | Status |
|-------|-------------|--------|
| [ID](link) | Description | done |

## Progress

**Requirements:** N/M verified
**Stories:** N/M complete
```

## Just Task Updates

### Commands to Update

| Command | New Behavior |
|---------|--------------|
| `just board generate` | Regenerate epic + milestone READMEs with progress |
| `just board done-milestone` | Update epic README milestone table + progress |
| `just board start-milestone` | Update epic README milestone table |
| `just board done` | Update milestone README story table + progress |
| `just board start` | Update milestone README story table |
| `just board new epic` | Create directory with README.md + PRD.md templates |
| `just board new milestone` | Create directory with README.md + SRS.md + DESIGN.md templates |

### Auto-Updated Fields

| Document | Auto-Updated Fields |
|----------|---------------------|
| Epic README.md | Milestone table (status), Progress section |
| Milestone README.md | Story table (status), Progress section |

## Migration Strategy

### Phase 1: Structure

1. Rename `docs/PRD.md` → `docs/VISION.md`
2. Update all links referencing PRD.md
3. Create templates in `docs/board/templates/`

### Phase 2: Epic Migration

For each epic:
1. Extract requirements from current README.md to new PRD.md
2. Update README.md to new navigation format
3. Preserve existing content where appropriate

### Phase 3: Milestone Migration

For each milestone:
1. Rename `design.md` → `DESIGN.md`
2. Create SRS.md from milestone requirements + implementation.md story list
3. Update README.md to new navigation format
4. Delete `implementation.md` after migration

### Phase 4: Tooling

1. Update `just board new epic` command
2. Update `just board new milestone` command
3. Update `just board generate` for README progress
4. Update story/milestone commands for README sync

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Missing PRD.md in epic | `just board generate` warns, skips progress |
| Missing SRS.md in milestone | `just board generate` warns, skips progress |
| Story links missing in README | Regenerate from frontmatter |

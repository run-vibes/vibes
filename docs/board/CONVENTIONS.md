# Planning Conventions

This document describes how to use the kanban planning board at `docs/board/`.

## Index

### The Board
| Section | Description |
|---------|-------------|
| [Board Structure](#board-structure) | Directory layout and organization |
| [Stages](#stages) | Story lifecycle: backlog, in-progress, done |
| [Epics](#epics) | Grouping related stories |
| [Milestones](#milestones) | Large deliverables with design docs |
| [Commands](#commands) | `just board` command reference |

### Planning
| Section | Description |
|---------|-------------|
| [When to Create a Plan](#when-to-create-a-plan) | Planning vs just doing |
| [Story Lifecycle](#story-lifecycle) | Moving stories through stages |
| [Phase 1: Design Document](#phase-1-design-document) | Architecture and design decisions |
| [Phase 2: Implementation Plan](#phase-2-implementation-plan) | Stories and task breakdown |

### Execution
| Section | Description |
|---------|-------------|
| [Using Plans with Claude Code](#using-plans-with-claude-code) | Superpowers skills for execution |

### Standards
| Section | Description |
|---------|-------------|
| [Architectural Decision: Plugin vs Built-in](#architectural-decision-plugin-vs-built-in) | Where new features belong |
| [Best Practices](#best-practices) | Do's and don'ts |
| [Plan Review Checklist](#plan-review-checklist) | Pre-implementation verification |

---

# The Board

## Board Structure

```
docs/board/
├── README.md              # Auto-generated board view
├── CHANGELOG.md           # Updated when items complete
├── CONVENTIONS.md         # This file
├── stages/                # Story files organized by status
│   ├── backlog/stories/   # Future work
│   ├── in-progress/stories/ # Currently being worked on
│   └── done/stories/      # Completed work
├── epics/                 # Story groupings (symlinks to stories)
│   ├── core/              # Core functionality
│   ├── web-ui/            # Web UI features
│   └── ...
├── milestones/            # Large deliverables (symlinks to epics)
│   ├── milestone-01-core-proxy/
│   └── ...
├── templates/             # Templates for new items
│   ├── story.md
│   ├── epic.md
│   └── milestone.md
├── backlog/               # Legacy: milestone directories (to be migrated)
└── in-progress/           # Legacy: milestone directories (to be migrated)
```

## Stages

Stories live in `stages/<stage>/stories/` and move between stages as work progresses.

| Stage | Path | Description |
|-------|------|-------------|
| **backlog** | `stages/backlog/stories/` | Future work, not yet started |
| **in-progress** | `stages/in-progress/stories/` | Currently being worked on |
| **done** | `stages/done/stories/` | Completed work |

### Story File Format

Stories use YAML frontmatter for metadata:

```yaml
---
id: m26-feat-01-eventlog
title: EventLog Assessment Storage
type: feat
status: in-progress
priority: high
epics: [core]
depends: []
estimate: 2h
created: 2025-01-07
updated: 2025-01-07
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (prefix with milestone: `m26-feat-01`) |
| `title` | Yes | Human-readable title |
| `type` | Yes | `feat`, `bug`, `chore`, `refactor` |
| `status` | Yes | `backlog`, `in-progress`, `done` |
| `priority` | No | `low`, `medium`, `high`, `critical` |
| `epics` | No | List of epic IDs this story belongs to |
| `depends` | No | List of story IDs that must complete first |
| `estimate` | No | Time estimate (e.g., `2h`, `1d`) |
| `created` | Yes | Creation date |
| `updated` | Yes | Last update date |

### Story Naming

Stories use prefixes to indicate type, with optional milestone prefix:

| Pattern | Example | Use Case |
|---------|---------|----------|
| `m<NN>-feat-<NN>-<name>.md` | `m26-feat-01-eventlog.md` | Milestone story |
| `feat-<NNNN>-<name>.md` | `feat-0003-navigation.md` | Standalone feature |
| `bug-<NNNN>-<name>.md` | `bug-0001-cwd-propagation.md` | Standalone bug fix |
| `chore-<NNNN>-<name>.md` | `chore-0001-cleanup.md` | Standalone maintenance |

## Epics

Epics group related stories across milestones. Each epic is a directory in `epics/` containing:

- `README.md` with epic metadata
- Symlinks to stories (pointing to `../../stages/<stage>/stories/<story>.md`)

```
epics/
├── core/
│   ├── README.md
│   ├── m26-feat-01-eventlog.md -> ../../stages/in-progress/stories/m26-feat-01-eventlog.md
│   └── m26-feat-02-processor.md -> ../../stages/in-progress/stories/m26-feat-02-processor.md
└── web-ui/
    ├── README.md
    └── feat-0003-navigation.md -> ../../stages/done/stories/feat-0003-navigation.md
```

### Epic README Format

```yaml
---
id: core
title: Core Functionality
status: active
description: Core vibes functionality and infrastructure
---
```

### Key Properties

- A story can belong to **multiple epics** (via symlinks from each epic)
- Symlinks automatically stay valid when stories move between stages (relative paths)
- Epics provide a cross-cutting view of work by theme

## Milestones

Milestones are large deliverables that span multiple work sessions. They live in `milestones/` and contain:

- `README.md` with milestone metadata
- `design.md` for architecture decisions
- `implementation.md` for story index (optional)
- Symlinks to related epics

```
milestones/
└── milestone-26-assessment-framework/
    ├── README.md
    ├── design.md
    ├── implementation.md
    └── core -> ../../epics/core
```

### Milestone README Format

```yaml
---
id: milestone-26
title: Assessment Framework
status: in-progress
epics: [core]
---
```

### Milestone-Epic Relationship

- Milestones link to epics (not directly to stories)
- This creates a hierarchy: Milestone -> Epic -> Stories
- An epic can be linked to multiple milestones

## Commands

| Command | Action |
|---------|--------|
| `just board` | Show available commands |
| `just board generate` | Regenerate README.md |
| `just board status` | Show counts per stage |
| `just board new story "title"` | Create story in backlog |
| `just board new epic "name"` | Create new epic |
| `just board new milestone "name"` | Create new milestone |
| `just board start <id>` | Move story to in-progress |
| `just board done <id>` | Move story to done + changelog |
| `just board link <story> <epic>` | Link story to epic |
| `just board link-epic <epic> <milestone>` | Link epic to milestone |

---

# Planning

## When to Create a Plan

Create a plan when:

- Adding a new feature or milestone
- Making architectural changes (new crates, trait refactoring)
- Refactoring significant code areas
- Adding new dependencies or external integrations
- Changing the interaction model with Claude Code

Skip planning for:

- Bug fixes with obvious solutions
- Small API additions to existing types
- Documentation updates
- Single-file changes
- Test additions for existing code

## Story Lifecycle

### 1. Create Story

```bash
just board new story "Add session export"
```

This creates a story in `stages/backlog/stories/` using the template.

### 2. Link to Epic (Optional)

```bash
just board link feat-0004-session-export core
```

This creates a symlink in `epics/core/` pointing to the story.

### 3. Start Work

```bash
just board start feat-0004-session-export
```

This moves the story file from `stages/backlog/stories/` to `stages/in-progress/stories/`. Symlinks in epics automatically point to the new location (relative paths).

### 4. Complete Work

```bash
just board done feat-0004-session-export
```

This moves the story to `stages/done/stories/` and updates the changelog.

## Phase 1: Design Document

Before implementation, create a `design.md` that captures architectural decisions.

### Design Document Template

````markdown
# Milestone NN: [Feature Name] - Design Document

> [One-line summary of what this enables]

## Overview

[1-2 paragraphs describing what this feature does and why we're building it]

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Plugin vs Built-in** | [Plugin / Built-in] | [See decision framework] |
| [Decision Area] | [Choice Made] | [Why] |

> **Required:** Every design document must explicitly address the Plugin vs Built-in decision. See [Architectural Decision: Plugin vs Built-in](#architectural-decision-plugin-vs-built-in).

---

## Architecture

[Diagrams using ASCII art or Mermaid]

```
┌──────────────┐     ┌──────────────┐
│  Component A │────▶│  Component B │
└──────────────┘     └──────────────┘
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| [Name] | vibes-core | [What it does] |

---

## Types and Interfaces

```rust
/// Description of the type
pub struct MyType {
    pub field: String,
}
```

---

## API Changes

### HTTP Endpoints (if applicable)

```
GET  /api/resource           # Description
POST /api/resource           # Description
```

---

## Dependencies

```toml
[dependencies]
new-crate = "1.0"            # Purpose
```

---

## Testing Strategy

| Component | Test Coverage |
|-----------|---------------|
| [Name] | [What to test] |

---

## Deliverables

- [ ] Backend implementation
- [ ] Server integration
- [ ] Tests passing
- [ ] Documentation updated
````

### Key Elements

1. **Decisions Table** - Quick reference for all major choices
2. **Rationale** - Explain *why* not just *what*
3. **Trade-offs** - Document what was considered and rejected

### Example: Decision Documentation

```markdown
### Storage Approach

**Choice:** File-backed SQLite

**Considered:**
- Pure in-memory (fast but no persistence)
- File-backed JSON (simple but no queries)
- SQLite (queries + persistence + single file)

**Rationale:** SQLite provides structured queries for history search while maintaining single-file simplicity.
```

## Phase 2: Implementation Plan

After design approval, create an `implementation.md` that breaks the milestone into **stories** - focused deliverables that can be implemented and merged independently.

### Source of Truth

**Story frontmatter is the single source of truth for status.** The `implementation.md` is a navigation index only - it links to stories but does not track their status. Run `just board status` to see current status.

### Implementation Plan Template

The `implementation.md` serves as the entry point and index for the milestone's stories:

```markdown
# Milestone NN: [Name] - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** [One sentence describing the milestone outcome]

**Design:** See [design.md](design.md) for architecture decisions.

---

## Stories

| # | Story | Description |
|---|-------|-------------|
| 1 | [m26-feat-01-types](../../stages/backlog/stories/m26-feat-01-types.md) | Core type definitions |
| 2 | [m26-feat-02-storage](../../stages/backlog/stories/m26-feat-02-storage.md) | Persistence layer |
| 3 | [m26-feat-03-api](../../stages/backlog/stories/m26-feat-03-api.md) | HTTP endpoints |

> **Status:** Check story frontmatter or run `just board status` for current status.

## Dependencies

- Story 2 depends on Story 1 (types must exist before storage)
- Story 3 can run in parallel with Story 2

## Completion Criteria

- [ ] All stories merged
- [ ] Integration tests passing
- [ ] Documentation updated
```

### Story Template

```markdown
---
id: m26-feat-01-types
title: Core Type Definitions
type: feat
status: backlog
priority: high
epics: [core]
depends: []
estimate: 2h
created: 2024-01-15
updated: 2024-01-15
---

# Core Type Definitions

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

[One sentence: what this story delivers]

## Context

[Reference design.md section, key decisions that apply]

## Tasks

Each task ends with a commit:

### Task 1: [Name]

**Files:**
- Create: `path/to/new/file.rs`
- Modify: `path/to/existing.rs`

**Steps:**
1. [Action with expected outcome]
2. [Action with expected outcome]
3. Run tests: `cargo test -p vibes-core module_name`
4. Commit: `feat(module): description`

### Task 2: [Name]

...

## Acceptance Criteria

- [ ] All tests pass
- [ ] Code reviewed and merged
- [ ] [Feature-specific criterion]

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done <story-id>`
3. Commit, push, and create PR
```

### Key Principles

#### 1. Test-Driven Development

For new modules and utilities, follow TDD:

1. Write the failing test first
2. Run test to verify it fails
3. Write the implementation
4. Run test to verify it passes
5. Commit

#### 2. Small, Focused Tasks

Each task should:
- Have a single clear purpose
- Be completable in one sitting
- End with a commit
- Be independently verifiable

#### 3. Explicit Verification

Include expected outcomes for each step:

```markdown
Run: `cargo test -p vibes-core`
Expected: All tests pass
```

#### 4. Commit After Each Task

Every task ends with a commit using conventional commit format.

---

# Execution

## Using Plans with Claude Code

### Creating a Plan

1. Use the brainstorming skill first:
   ```
   /superpowers:brainstorm
   ```

2. Explore the codebase to understand existing patterns

3. Write the design document discussing options

4. Create the implementation plan with stories

### Executing a Plan

Reference the skill at the top of the implementation plan:

```markdown
> **For Claude:** Use superpowers:executing-plans to implement this plan.
```

Then invoke:
```
/superpowers:execute-plan
```

### Completing a Story

After implementing all tasks:

1. **Verify:** Run `just pre-commit` (fmt + clippy + test)
2. **Update story:** Set frontmatter `status: done`
3. **Move story:** Run `just board done <story-id>`
4. **Commit:** Include story status change in commit
5. **Push and PR:** Create PR with conventional commit title

> **Claude must always update the story status and move the story before creating a PR.**

---

# Standards

## Architectural Decision: Plugin vs Built-in

When adding new functionality that could be a separate feature, **always evaluate whether it should be a plugin** before implementing it directly in vibes-cli or vibes-server.

### Decision Framework

| Question | Plugin | Built-in |
|----------|--------|----------|
| Is this a first-party core feature? | Maybe | Yes |
| Should users be able to disable it? | Yes | No |
| Does it need CLI subcommands? | Yes (plugins can register) | No preference |
| Does it need HTTP routes? | Yes (plugins can register) | No preference |
| Is it specific to certain use cases? | Yes | No |
| Would third parties want similar features? | Yes | No |

### Plugin API Capabilities

The `vibes-plugin-api` (v2) supports:

- **Session lifecycle hooks** - `on_session_created`, `on_turn_complete`, `on_hook`, etc.
- **CLI command registration** - `ctx.register_command(CommandSpec { ... })` -> `vibes <plugin> <command>`
- **HTTP route registration** - `ctx.register_route(RouteSpec { ... })` -> `/api/plugins/<plugin>/...`
- **Configuration** - Persistent key-value store with TOML serialization
- **Logging** - Plugin-prefixed logging via tracing

### Example: groove

The **groove** continual learning plugin demonstrates proper plugin architecture:

- **CLI commands** registered via `register_command()` -> `vibes groove init`, `vibes groove status`
- **HTTP routes** registered via `register_route()` -> `/api/plugins/groove/...`
- **Event hooks** - `on_hook()` captures Claude Code events for learning extraction
- **Configuration** - Stores scope and injection preferences

## Best Practices

### Do

- Break large features into multiple stories
- Document the "why" alongside the "what"
- Specify exact file paths
- Include verification steps
- Follow TDD for testable code

### Don't

- Skip the design phase for significant work
- Create tasks that are too large
- Leave decisions implicit
- Forget commit instructions
- Skip verification steps

## Plan Review Checklist

Before implementing, verify:

- [ ] Design document captures all major decisions
- [ ] Trade-offs are documented
- [ ] Stories are small and focused
- [ ] Each task ends with a commit
- [ ] TDD pattern used for testable code
- [ ] Verification steps are explicit
- [ ] File paths are complete and accurate
- [ ] Board item updated when complete

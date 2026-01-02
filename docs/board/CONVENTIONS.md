# Planning Conventions

This document describes how to use the kanban planning board at `docs/board/`.

## Index

### The Board
| Section | Description |
|---------|-------------|
| [Board Structure](#board-structure) | Directory layout and columns |
| [Commands](#commands) | `just board` command reference |
| [Work Item Types](#work-item-types) | Milestones, stories, features, bugs, chores |

### Planning
| Section | Description |
|---------|-------------|
| [When to Create a Plan](#when-to-create-a-plan) | Planning vs just doing |
| [Plan Directory Structure](#plan-directory-structure) | File organization for milestones |
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
├── README.md          # Auto-generated board view
├── CHANGELOG.md       # Updated when items complete
├── CONVENTIONS.md     # This file
├── backlog/           # Future work
├── ready/             # Designed, ready to implement
├── in-progress/       # Currently being worked on
├── review/            # Awaiting review/merge
└── done/              # Completed work
```

## Commands

| Command | Action |
|---------|--------|
| `just board` | Regenerate README.md |
| `just board new feat "desc"` | Create feature in backlog |
| `just board new milestone "name"` | Create milestone in backlog |
| `just board start <item>` | Move to in-progress |
| `just board review <item>` | Move to review |
| `just board done <item>` | Move to done + changelog |
| `just board status` | Show counts per column |

## Work Item Types

The board supports four item types, organized hierarchically:

### Milestones

Large deliverables that span multiple work sessions. Milestones contain a design doc, an implementation plan that indexes the stories, and one or more **stories** that break the work into mergeable chunks.

```
docs/board/backlog/milestone-14-continual-learning/
├── design.md              # Architecture and decisions
├── implementation.md      # Story index with links and sequence
└── stories/               # Child work items (1 or more)
    ├── feat-01-storage.md
    ├── feat-02-capture.md
    └── chore-03-cleanup.md
```

**Create with:** `just board new milestone "Continual Learning"`

### Stories

Focused work items that live within a milestone. Stories break large milestones into reviewable chunks—each story can be implemented and merged independently.

**Naming:** Stories use the same prefixes as standalone items (`feat-`, `bug-`, `chore-`) with a sequence number:

```
stories/
├── feat-01-core-types.md      # New functionality
├── feat-02-api-endpoints.md   # More new functionality
├── bug-03-edge-case.md        # Fix discovered during implementation
└── chore-04-docs.md           # Documentation, cleanup
```

See [Phase 2: Implementation Plan](#phase-2-implementation-plan) for the full story template.

### Features, Bugs, and Chores

Standalone items that don't warrant a full milestone structure.

| Type | Prefix | Use Case |
|------|--------|----------|
| `feat` | `feat-NNNN-` | New functionality, enhancements |
| `bug` | `bug-NNNN-` | Defects, unexpected behavior |
| `chore` | `chore-NNNN-` | Maintenance, refactoring, tooling |

**Create with:** `just board new feat "Add session export"`, `just board new bug "Fix auth timeout"`

These are single markdown files (not directories) unless they grow complex enough to warrant design docs.

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

## Plan Directory Structure

Plans live in `docs/board/` with numbered directories matching milestones.

```
docs/board/
├── 01-core-proxy/
│   ├── design.md           # Architecture and design decisions
│   └── implementation.md   # Step-by-step implementation guide
├── 02-cli/
│   ├── design.md
│   └── implementation.md
├── ...
└── 08-chat-history/        # Next milestone
    ├── design.md
    └── implementation.md
```

**Naming:**
- Prefix with zero-padded number (01, 02, 03...)
- Use kebab-case for the name
- Keep names short but descriptive

## Phase 1: Design Document

Before implementation, create a `design.md` that captures architectural decisions.

### Design Document Template

````markdown
# Milestone X.Y: [Feature Name] - Design Document

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

1. **Decisions Table** — Quick reference for all major choices
2. **Rationale** — Explain *why* not just *what*
3. **Trade-offs** — Document what was considered and rejected

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

After design approval, create an `implementation.md` that breaks the milestone into **stories**—focused deliverables that can be implemented and merged independently.

### Milestone Structure

```
docs/board/<column>/milestone-NN-name/
├── design.md              # Architecture decisions (from Phase 1)
├── implementation.md      # Story index with sequence and links
└── stories/
    ├── feat-01-types.md       # Core type definitions
    ├── feat-02-storage.md     # Persistence layer
    └── feat-03-api.md         # HTTP endpoints
```

### Implementation Plan Template

The `implementation.md` serves as the entry point and index for the milestone's stories:

```markdown
# Milestone X.Y: [Name] - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** [One sentence describing the milestone outcome]

**Design:** See [design.md](design.md) for architecture decisions.

---

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [feat-01-types](stories/feat-01-types.md) | Core type definitions | pending |
| 2 | [feat-02-storage](stories/feat-02-storage.md) | Persistence layer | pending |
| 3 | [feat-03-api](stories/feat-03-api.md) | HTTP endpoints | pending |

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
created: 2024-01-15
status: pending  # pending | in-progress | done
---

# [Type]: [Focused Deliverable]

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

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

### Verification Workflow

After completing implementation:

1. Run `just pre-commit` (fmt + clippy + test)
2. Move board item to done column
3. Create PR with conventional commit title

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

- **Session lifecycle hooks** — `on_session_created`, `on_turn_complete`, `on_hook`, etc.
- **CLI command registration** — `ctx.register_command(CommandSpec { ... })` → `vibes <plugin> <command>`
- **HTTP route registration** — `ctx.register_route(RouteSpec { ... })` → `/api/plugins/<plugin>/...`
- **Configuration** — Persistent key-value store with TOML serialization
- **Logging** — Plugin-prefixed logging via tracing

### Example: groove

The **groove** continual learning plugin demonstrates proper plugin architecture:

- **CLI commands** registered via `register_command()` → `vibes groove init`, `vibes groove status`
- **HTTP routes** registered via `register_route()` → `/api/plugins/groove/...`
- **Event hooks** — `on_hook()` captures Claude Code events for learning extraction
- **Configuration** — Stores scope and injection preferences

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

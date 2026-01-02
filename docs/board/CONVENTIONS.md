# Planning Conventions

This document describes how to use the kanban planning board at `docs/board/`.

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

---

## Work Item Types

The board supports four item types, organized hierarchically:

### Milestones

Large deliverables that span multiple work sessions. Milestones contain design docs, implementation plans, and optionally **stories**.

```
docs/board/backlog/milestone-14-continual-learning/
├── design.md              # Architecture and decisions
├── implementation.md      # Step-by-step tasks (optional)
└── stories/               # Child work items (optional)
    ├── story-01-storage.md
    ├── story-02-capture.md
    └── story-03-inject.md
```

**Create with:** `just board new milestone "Continual Learning"`

### Stories

Focused work items that live within a milestone. Stories break large milestones into reviewable chunks—each story can be implemented and merged independently.

**Structure:** `docs/board/<column>/milestone-XX-name/stories/story-NN-description.md`

**When to use stories:**
- Milestone spans 3+ distinct deliverables
- Work can be parallelized across contributors
- Each piece merits its own PR and review cycle

**Story file template:**
```markdown
---
created: 2024-01-15
status: pending  # pending | in-progress | done
---

# Story: [Description]

## Goal

[What this story delivers]

## Tasks

- [ ] Task 1
- [ ] Task 2

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
```

The board generator lists stories as checklists under their parent milestone.

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

Plugins implement `handle_command()` and `handle_route()` on the `Plugin` trait to respond to registered commands and routes.

### Example: groove

The **groove** continual learning plugin demonstrates proper plugin architecture:

- **CLI commands** registered via `register_command()` → `vibes groove init`, `vibes groove status`, etc.
- **HTTP routes** registered via `register_route()` → `/api/plugins/groove/...`
- **Event hooks** — `on_hook()` captures Claude Code events for learning extraction
- **Configuration** — Stores scope and injection preferences

This pattern should be followed for all new feature plugins.

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
- Number should match milestone (e.g., Milestone 3.1 → 08-xxx)

## Phase 1: Design Document

Before implementation, create a `design.md` that captures architectural decisions.

### Design Document Template

```markdown
# Milestone X.Y: [Feature Name] - Design Document

> [One-line summary of what this enables]

## Overview

[1-2 paragraphs describing what this feature does and why we're building it]

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Plugin vs Built-in** | [Plugin / Built-in] | [See framework above - explain why this is/isn't a plugin] |
| [Decision Area] | [Choice Made] | [Why] |
| ... | ... | ... |

> **Required:** Every design document must explicitly address the Plugin vs Built-in decision. See the [decision framework](#architectural-decision-plugin-vs-built-in) above.

---

## Architecture

### [Component/Flow Name]

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
| ... | ... | ... |

---

## Types and Interfaces

### Core Types

```rust
/// Description of the type
pub struct MyType {
    /// Field description
    pub field: String,
}

impl MyType {
    /// Method description
    pub fn new() -> Self { ... }
}
```

### Trait Design

```rust
#[async_trait]
pub trait MyTrait: Send + Sync {
    async fn method(&self) -> Result<()>;
}
```

---

## API Changes

### HTTP Endpoints (if applicable)

```
GET  /api/resource           # Description
POST /api/resource           # Description
```

### WebSocket Messages (if applicable)

```typescript
// Server → Client
{ "type": "event_name", "data": { ... } }
```

---

## Crate Structure

### New/Modified Files

```
vibes/
├── vibes-core/
│   └── src/
│       └── new_module/      # NEW MODULE
│           ├── mod.rs
│           └── ...
├── vibes-server/
│   └── src/
│       └── ...              # Modified: add routes
└── vibes-cli/
    └── src/
        └── commands/
            └── ...          # Modified: add flag
```

---

## Dependencies

### vibes-core/Cargo.toml

```toml
[dependencies]
new-crate = "1.0"            # Purpose
```

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| [Name] | [What to test] |

### Integration Tests

| Test | Description |
|------|-------------|
| [Name] | [What it verifies] |

---

## Deliverables

### Milestone X.Y Checklist

**Backend (vibes-core):**
- [ ] Item 1
- [ ] Item 2

**Server (vibes-server):**
- [ ] Item 1

**CLI (vibes-cli):**
- [ ] Item 1

**Web UI:**
- [ ] Item 1

**Documentation:**
- [ ] Design document
- [ ] Update board when complete
```

### Key Elements

1. **Decisions Table** — Quick reference for all major choices
2. **Rationale** — Explain *why* not just *what*
3. **Code Examples** — Show actual Rust types and traits
4. **Trade-offs** — Document what was considered and rejected

### Example: Decision Documentation

```markdown
### Storage Approach

**Choice:** File-backed SQLite

**Considered:**
- Pure in-memory (fast but no persistence)
- File-backed JSON (simple but no queries)
- SQLite (queries + persistence + single file)

**Rationale:** SQLite provides structured queries for history search while maintaining single-file simplicity. The `rusqlite` crate is mature and well-tested.
```

---

## Phase 2: Implementation Plan

After design approval, create an `implementation.md` with step-by-step tasks.

### Implementation Plan Template

```markdown
# Milestone X.Y: [Feature Name] - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** [One sentence describing the outcome]

**Architecture:** [Brief recap of key design decisions]

**Tech Stack:** [Relevant technologies]

---

## Task 1: [Task Name]

**Files:**
- Create: `path/to/new/file.rs`
- Modify: `path/to/existing.rs`

**Step 1: [Action]**

[Description or code]

```rust
// Code example
pub struct NewType { ... }
```

**Step 2: [Action]**

...

**Step N: Run tests**

Run: `cargo test -p vibes-core module_name`
Expected: All tests pass

**Step N+1: Commit**

```bash
git add path/to/files
git commit -m "feat(module): description"
```

---

## Task 2: [Next Task]

...

---

## Summary

[What was accomplished, total tasks/commits, next steps]
```

### Key Principles

#### 1. Test-Driven Development

For new modules and utilities, follow TDD:

```markdown
**Step 1: Write the failing test**

```rust
// vibes-core/src/history/store.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = HistoryStore::open(temp_dir.path()).await.unwrap();

        store.save_session(&session).await.unwrap();

        let loaded = store.get_session(&session.id).await.unwrap();
        assert_eq!(loaded.id, session.id);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core history`
Expected: FAIL with "cannot find value `HistoryStore`"

**Step 3: Write the implementation**

```rust
pub struct HistoryStore {
    db: Connection,
}

impl HistoryStore {
    pub async fn open(path: &Path) -> Result<Self> { ... }
    pub async fn save_session(&self, session: &Session) -> Result<()> { ... }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core history`
Expected: PASS
```

#### 2. Small, Focused Tasks

Each task should:
- Have a single clear purpose
- Be completable in one sitting
- End with a commit
- Be independently verifiable

#### 3. Explicit Verification

Include expected outcomes:

```markdown
**Step 5: Run tests**

Run: `cargo test -p vibes-core`
Expected: All tests pass (including new tests)
```

#### 4. Commit After Each Task

```markdown
**Step 6: Commit**

```bash
git add vibes-core/src/history/
git commit -m "feat(history): add HistoryStore with SQLite backend"
```
```

---

## Using Plans with Claude Code

### Creating a Plan

1. Use the brainstorming skill first:
   ```
   /superpowers:brainstorm
   ```

2. **Epic Detection** — Before writing any documents, ask:
   - Will this feature require **3+ internal milestones**?
   - Is the design likely to exceed **30KB**?
   - Will different sub-milestones need **separate brainstorming sessions**?

   If **yes to any**, use the [Multi-Phase Milestones](#multi-phase-milestones-epics) structure from the start.

3. Explore the codebase to understand existing patterns

4. Write the design document discussing options

5. Create the implementation plan with specific tasks

### Executing a Plan

Reference the skill at the top of the implementation plan:

```markdown
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
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

## Rust-Specific Conventions

### Error Handling

Use `thiserror` for library errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("session not found: {0}")]
    NotFound(String),
}
```

Use `anyhow` in binaries (vibes-cli) for context:

```rust
session.save()
    .context("failed to save session")?;
```

### Trait Design

Prefer trait objects for extensibility:

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
}

// Implementations
pub struct FileStorage { ... }
pub struct MemoryStorage { ... }
```

### Testing Patterns

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_fn().await;
    assert!(result.is_ok());
}
```

Use `tempfile` for filesystem tests:

```rust
#[tokio::test]
async fn test_file_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = Store::new(temp_dir.path());
    // ... test ...
}
```

Use `MockBackend` for Claude interaction tests:

```rust
#[tokio::test]
async fn test_session_flow() {
    let backend = MockBackend::new(vec![
        MockResponse::text("Hello"),
        MockResponse::complete(),
    ]);
    let session = Session::with_backend(backend);
    // ... test ...
}
```

### Module Organization

Follow this pattern for new modules:

```
vibes-core/src/new_feature/
├── mod.rs          # Module exports and documentation
├── types.rs        # Type definitions
├── store.rs        # Persistence (if applicable)
├── service.rs      # Business logic
└── error.rs        # Feature-specific errors (optional)
```

Each file should have `#[cfg(test)] mod tests { ... }` at the bottom.

---

## Best Practices

### Do

- Break large features into multiple tasks
- Include Rust code snippets for complex patterns
- Document the "why" alongside the "what"
- Specify exact file paths
- Include verification steps (cargo commands)
- Follow TDD for testable code
- Add to workspace dependencies when adding crates

### Don't

- Skip the design phase for significant work
- Create tasks that are too large
- Leave decisions implicit
- Forget commit instructions
- Skip verification steps
- Add dependencies without documenting why

---

## Plan Review Checklist

Before implementing, verify:

- [ ] Design document captures all major decisions
- [ ] Trade-offs are documented
- [ ] Implementation tasks are small and focused
- [ ] Each task ends with a commit
- [ ] TDD pattern used for testable code
- [ ] Verification steps are explicit
- [ ] File paths are complete and accurate
- [ ] New dependencies are documented
- [ ] Board item updated when complete

---

## Milestone Numbering

| Phase | Milestone | Plan Directory |
|-------|-----------|----------------|
| 1 | 1.1 Core Proxy | 01-core-proxy |
| 1 | 1.2 CLI | 02-cli |
| 1 | 1.3 Plugin Foundation | 03-plugin-foundation |
| 1 | 1.4 Server + Web UI | 04-server-web-ui |
| 2 | 2.1 Cloudflare Tunnel | 05-cloudflare-tunnel |
| 2 | 2.2 Cloudflare Access | 06-cloudflare-access |
| 2 | 2.3 Push Notifications | 07-push-notifications |
| 3 | 3.1 Chat History | 08-chat-history |
| 3 | 3.2 Multi-Session Support | 09-multi-session |
| 3 | 3.3 CLI - Web Mirroring | 10-cli-web-mirroring |
| 3 | 3.4 PTY Backend | 12-pty-backend |
| 4 | 4.1-4.9 Continual Learning | 14-continual-learning |
| 4 | 4.1 Harness Introspection | 15-harness-introspection |
| 5 | 5.1 Setup Wizards | (planned) |
| 5 | 5.2 Default Plugins | (planned) |
| 5 | 5.3 CLI Enhancements | (planned) |
| 5 | 5.4 iOS App | (planned) |

When starting a new milestone:
1. Create the directory under `docs/board/`
2. Write `design.md` first with architecture decisions
3. Get design approved (PR or discussion)
4. Write `implementation.md` with step-by-step tasks
5. Reference any new ADRs added to `docs/PRD.md`

---

## Multi-Phase Milestones (Epics)

Some milestones are too large to fit the standard `design.md` + `implementation.md` pattern. These "epic" milestones contain multiple internal sub-milestones that each require their own implementation planning.

### When to Use Multi-Phase Structure

Use this structure when:
- A feature spans **3+ internal milestones**
- The design document exceeds 30KB
- Different sub-milestones require different technical decisions
- Sub-milestones may be executed by different people or in parallel

### Multi-Phase Directory Structure

```
docs/board/NN-epic-name/
├── design.md                          # Unified design covering ALL sub-milestones
├── milestone-X.Y-decisions.md         # Brainstorm decisions for sub-milestone X.Y
├── milestone-X.Y-implementation.md    # Implementation plan for sub-milestone X.Y
├── milestone-X.Z-decisions.md         # (optional) for next sub-milestone
└── milestone-X.Z-implementation.md    # Implementation plan for X.Z
```

### File Naming Conventions

| File | Purpose | When to Create |
|------|---------|----------------|
| `design.md` | Comprehensive architecture for the epic | At epic start |
| `milestone-X.Y-decisions.md` | Captures decisions from brainstorming session | After brainstorming sub-milestone |
| `milestone-X.Y-implementation.md` | Step-by-step tasks for sub-milestone | When starting implementation |

### Example: vibes groove (Milestone 14)

groove is an epic spanning milestones 4.1-4.9:

```
docs/board/14-continual-learning/
├── design.md                              # 98KB covering all 10 sub-milestones
├── milestone-4.2-decisions.md             # Storage decisions from brainstorm
└── milestone-4.2-implementation.md        # 20 tasks for storage foundation
```

The unified `design.md` links to sections via anchors (e.g., `design.md#42-storage-foundation`).

### Multi-Phase Workflow

1. **Create unified design.md** covering the entire epic architecture
2. **For each sub-milestone:**
   a. Use `superpowers:brainstorming` to explore options
   b. Capture decisions in `milestone-X.Y-decisions.md`
   c. Write `milestone-X.Y-implementation.md` with tasks
   d. Execute using `superpowers:executing-plans`
3. **Track progress** in board columns with sub-milestone granularity

### Design Document Sections for Multi-Phase

The unified design.md should include:
- **Milestone Index** - Table linking to each sub-milestone section
- **Level Progression** - Dependencies between sub-milestones
- **Core Types** - Shared types used across sub-milestones
- **Architecture Diagram** - How sub-milestones connect

Each sub-milestone section in design.md should have an anchor (e.g., `## 4.2 Storage Foundation {#42-storage-foundation}`) for linking from board items

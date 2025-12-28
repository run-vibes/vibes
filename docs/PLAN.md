# Planning Conventions

This document describes how to create design and implementation plans for the vibes project when working with Claude Code.

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

Plans live in `docs/plans/` with numbered directories matching milestones.

```
docs/plans/
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
| [Decision Area] | [Choice Made] | [Why] |
| ... | ... | ... |

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
- [ ] Update PROGRESS.md
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

2. Explore the codebase to understand existing patterns

3. Write the design document discussing options

4. Create the implementation plan with specific tasks

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
2. Update [PROGRESS.md](PROGRESS.md) with completed items
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
- [ ] PROGRESS.md update is included

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
| 3 | 3.3 CLI ↔ Web Mirroring | 10-cli-web-mirroring |
| 3 | 3.4 PTY Backend | 12-pty-backend |
| 4 | 4.1-4.6 Continual Learning | 14-continual-learning |
| 5 | 5.1 Setup Wizards | 15-setup-wizards |
| 5 | 5.2 Default Plugins | 16-default-plugins |
| 5 | 5.3 CLI Enhancements | 17-cli-enhancements |
| 5 | 5.4 iOS App | 18-ios-app |

When starting a new milestone:
1. Create the directory under `docs/plans/`
2. Write `design.md` first with architecture decisions
3. Get design approved (PR or discussion)
4. Write `implementation.md` with step-by-step tasks
5. Reference any new ADRs added to `docs/PRD.md`

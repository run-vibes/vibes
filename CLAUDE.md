# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**vibes** - Remote control for your Claude Code sessions

A utility application that proxies Claude Code with remote access capabilities, plugin system, and cross-platform support (CLI, native GUI, web UI). See [docs/PRD.md](docs/PRD.md) for full product requirements.

### Key Architecture

- **vibes-core**: Shared Rust library (sessions, events, plugins, auth, tunnel, notifications)
- **vibes-server**: HTTP/WebSocket server (axum-based)
- **vibes-cli**: CLI binary consuming vibes-core, connects to daemon via WebSocket
- **vibes-plugin-api**: Published crate for plugin authors
- **vibes-introspection**: Harness detection and capability discovery (Level 0 for groove)
- **vibes-groove**: Continual learning plugin (in development)
- **web-ui**: TanStack frontend embedded in binary via rust-embed

### Current State

**Phases 1-3 are complete** (11 milestones). See [docs/PROGRESS.md](docs/PROGRESS.md) for details.

Key capabilities:
- `vibes claude` proxies Claude Code with PTY-based terminal (full CLI parity)
- Daemon architecture: server owns state, CLI + Web UI are WebSocket clients
- Plugin system with dynamic native Rust library loading
- Cloudflare Tunnel integration for remote access
- Cloudflare Access JWT authentication with localhost bypass
- Web Push notifications for session events
- Persistent chat history with FTS5 full-text search
- Multi-session support with real-time updates
- CLI ↔ Web UI mirroring with source attribution

**Current: Phase 4 (vibes groove)** - The continual learning system:
- Milestone 4.1 (Harness Introspection) ✓ complete
- Milestone 4.2 (Storage Foundation) in progress
- See [groove design](docs/plans/14-continual-learning/design.md) and [branding](docs/groove/BRANDING.md)

## Development Environment

The project uses a **Nix flake** for reproducible development environments with **direnv** for automatic shell loading.

```bash
cd vibes              # direnv auto-loads the Nix shell
direnv allow          # First time only
just setup-hooks      # Enable pre-commit hooks
```

### Tooling

| Tool | Purpose |
|------|---------|
| `just` | Task runner (prefer over raw cargo commands) |
| `cargo-nextest` | Fast parallel test runner |
| `cargo-mutants` | Mutation testing |
| `cargo-watch` | Watch mode for development |

## Git Submodules

This project uses a git submodule for Apache Iggy (message streaming):

```
vendor/iggy/  → github.com/apache/iggy
```

### First-Time Setup

```bash
# If you cloned without --recursive:
git submodule update --init --recursive
```

### Building

```bash
just build-all   # Builds vibes + iggy-server
```

The `iggy-server` binary is copied to `target/release/` alongside vibes.

### Updating Iggy

```bash
cd vendor/iggy
git fetch --tags
git checkout server-0.7.0  # New version
cd ../..
git add vendor/iggy
git commit -m "chore: update iggy to server-0.7.0"
```

### How It Works

When `vibes serve` starts, it automatically:
1. Looks for `iggy-server` next to the vibes binary
2. Spawns it as a subprocess if not running
3. Connects using the Iggy client SDK
4. Falls back to in-memory storage if Iggy unavailable

## Development Commands

**Always prefer `just` commands over direct cargo commands.**

```bash
just              # List all available commands
just setup-hooks  # Enable git pre-commit hooks (run once)
just dev          # Watch mode (cargo watch -x check)
just test         # Run tests (cargo nextest run)
just test-all     # Run tests including integration tests
just clippy       # Lint
just fmt          # Format code
just pre-commit   # Run all checks before committing
just mutants      # Mutation testing
just build        # Build the project
```

## Continuous Integration

CI runs on GitHub Actions for all PRs and pushes to main. The workflow uses **Nix** to ensure the CI environment matches local development exactly.

**CI checks** (same as `just pre-commit`):
- `just fmt-check` — Formatting
- `just clippy` — Linting
- `just test` — Unit tests

### Integration Tests

Integration tests (`just test-integration`) require **Claude CLI installed** and are **not run in CI**. Run them locally before submitting changes that affect Claude interaction:

```bash
just test-integration    # Requires Claude CLI
just test-all            # Unit + integration tests
```

## Planning Process

**See [docs/PLAN.md](docs/PLAN.md) for full planning conventions.**

Plans are required for new milestones and significant features. Skip planning for bug fixes, small changes, and documentation updates.

### When to Create a Plan

| Create Plan | Skip Planning |
|-------------|---------------|
| New milestone | Bug fixes with obvious solutions |
| New crate or module | Single-file changes |
| Architectural changes | Documentation updates |
| External integrations | Test additions for existing code |

### Plan Directory Structure

```
docs/plans/
├── 01-core-proxy/              # Standard milestone (design.md + implementation.md)
├── ...
├── 14-continual-learning/      # Multi-phase epic (see PLAN.md)
│   ├── design.md               # Unified design for all sub-milestones
│   ├── milestone-4.2-decisions.md
│   └── milestone-4.2-implementation.md
└── 15-harness-introspection/   # Standard milestone
    ├── design.md
    └── implementation.md
```

**Note:** For large epics with 3+ internal sub-milestones, see "Multi-Phase Milestones" in [docs/PLAN.md](docs/PLAN.md).

### Planning Workflow

1. **Brainstorm first**: Use `superpowers:brainstorming` skill to explore options
2. **Write design.md**: Capture architecture decisions, types, and trade-offs
3. **Get approval**: PR or discussion before implementation
4. **Write implementation.md**: Step-by-step tasks with TDD pattern
5. **Execute plan**: Use `superpowers:executing-plans` skill

## Development Workflow

**Always use these superpowers skills when developing:**

| Skill | When to Use |
|-------|-------------|
| `superpowers:brainstorming` | **FIRST** - Before any new feature or architecture decision |
| `superpowers:executing-plans` | When implementing a milestone plan |
| `superpowers:test-driven-development` | Before writing any implementation code |
| `superpowers:systematic-debugging` | When encountering bugs, test failures, or unexpected behavior |

### Workflow for New Features

1. **Brainstorm**: Use `superpowers:brainstorming` to explore options and trade-offs
2. **Plan**: Write `design.md` then `implementation.md` (see [docs/PLAN.md](docs/PLAN.md))
3. **Execute**: Use `superpowers:executing-plans` skill with the plan
4. **Implement with TDD**: Use `superpowers:test-driven-development` for each task
5. **Fix issues**: Use `superpowers:systematic-debugging` - never guess at fixes
6. **Review**: Run `just pre-commit` and address any issues
7. **Complete with PR**: Push and create a Pull Request

### Workflow for Bug Fixes

1. **Debug**: Use `superpowers:systematic-debugging` to investigate
2. **Fix with TDD**: Write a failing test that reproduces the bug, then fix
3. **Verify**: Run `just pre-commit`
4. **Commit**: Single commit with `fix:` prefix

**This workflow is mandatory.** Do not skip skills or try to implement without following this process.

## Completing Implementation Work

**REQUIRED:** When implementation is complete, always create a Pull Request:

1. Verify all tests pass: `just test`
2. Run pre-commit checks: `just pre-commit`
3. Commit changes with conventional commit message
4. Push to origin: `git push -u origin <branch-name>`
5. Create PR: `gh pr create --title "<type>: <description>" --body "..."`

**Never leave completed work uncommitted or unpushed.** All implementation work should result in a PR for review.

## Testing Philosophy (TDD)

**REQUIRED:** Use the `superpowers:test-driven-development` skill when implementing any feature or fix. Invoke it with the Skill tool before writing implementation code.

We follow Test-Driven Development for component and utility code:

1. **Write the failing test first** — Define expected behavior before implementation
2. **Run the test to verify it fails** — Confirms the test is actually testing something
3. **Write minimal code to pass** — Only implement what's needed to make the test green
4. **Run tests to verify they pass** — Confirm implementation is correct
5. **Commit** — Small, frequent commits after each passing test

**What to test:**
- Public API behavior (trait implementations, public functions)
- Error conditions and edge cases
- Serialization/deserialization (TOML, JSON)
- Async behavior with `#[tokio::test]`
- File persistence with `tempfile` crate

**What NOT to test:**
- Private implementation details
- Third-party library behavior
- Simple getters/setters without logic

## Verification Before Completing Work

**REQUIRED:** Always run these verification steps before marking work complete:

1. **`just check`** — Run all code quality checks:

2. **`just test`** — Run all unit tests:

All checks must pass before work is considered done.

## Progress Tracking

When completing work that corresponds to a roadmap item:
1. Update [docs/PROGRESS.md](docs/PROGRESS.md) to mark the item complete
2. Change `[ ]` to `[x]` for completed items, or `[~]` for in-progress items
3. Add an entry to the Changelog table at the bottom with the date and change

## Git Commit Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>: <description>

[optional body]
```

**Types:**
- `feat:` — New feature or functionality
- `fix:` — Bug fix
- `docs:` — Documentation changes only
- `style:` — Formatting, whitespace (no code change)
- `refactor:` — Code restructuring (no behavior change)
- `test:` — Adding or updating tests
- `chore:` — Build, tooling, dependencies

**Guidelines:**
- Use imperative mood: "add feature" not "added feature"
- Keep subject line under 72 characters
- No period at end of subject line
- Separate subject from body with blank line
- Body explains *what* and *why*, not *how*

**Do NOT include:**
- "Generated with Claude Code" footers
- "Co-Authored-By" trailers

**Examples:**
```
feat: add user authentication flow

fix: prevent form submission on empty input

refactor: extract validation logic to shared utility
```

## Pull Request Conventions

**Title format:** Same as commit message (`<type>: <description>`)

**Body structure:**
```markdown
## Summary
- Bullet points describing what changed (2-4 items)

## Test Plan
- [ ] Verification steps as checklist
```

**Guidelines:**
- Title should describe the overall change, not individual commits
- Summary bullets focus on *what* changed, not *how*
- Test plan includes both automated checks and manual verification
- Link related issues with "Fixes #123" or "Closes #123"

**Example:**
```markdown
## Summary
- Add login form with email/password validation
- Implement session persistence with secure cookies
- Add logout button to navigation

## Test Plan
- [x] All unit tests passing (`just test`)
- [x] Lint checks pass (`just check`, `just clippy`, `just fmt-check`)
- [ ] Manual test: login flow works end-to-end
- [ ] Manual test: session persists across page refresh
```


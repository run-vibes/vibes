# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**vibes** — The vibe engineering mech suit.

vibes augments *you*—the human developer—with AI-powered superpowers: remote session control, persistent context, and a learning system that remembers what works. You stay in command; vibes amplifies your reach.

See [docs/PRD.md](docs/PRD.md) for full product requirements.

### Key Architecture

- **vibes-core**: Shared Rust library (sessions, events, plugins, auth, tunnel, notifications)
- **vibes-server**: HTTP/WebSocket server (axum-based)
- **vibes-cli**: CLI binary consuming vibes-core, connects to daemon via WebSocket
- **vibes-iggy**: EventLog implementation backed by Apache Iggy message streaming
- **vibes-plugin-api**: Published crate for plugin authors
- **vibes-introspection**: Harness detection and capability discovery
- **vibes-groove**: Continual learning plugin (in development, under `plugins/`)
- **web-ui**: TanStack frontend embedded in binary via rust-embed

### Current State

**Phases 1-3 complete.** See [docs/board/README.md](docs/board/README.md) for details.

Core capabilities:
- `vibes claude` proxies Claude Code with PTY-based terminal (full CLI parity)
- Daemon architecture: server owns state, CLI + Web UI are WebSocket clients
- Plugin system with dynamic native Rust library loading
- Cloudflare Tunnel integration for remote access
- Cloudflare Access JWT authentication with localhost bypass
- Web Push notifications for session events
- EventLog persistence via Apache Iggy
- Multi-session support with real-time updates
- CLI ↔ Web UI mirroring with source attribution

**Current: Phase 4 (groove)** — The continual learning system:
- 4.1 Harness Introspection ✓
- 4.2 Storage Foundation ✓
- 4.3 Capture & Inject ✓
- 4.4 Assessment Framework 🔄 in progress
- See [groove design](docs/board/in-progress/milestone-14-continual-learning/design.md) and [branding](docs/groove/BRANDING.md)

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
just build           # Debug build (vibes + iggy-server)
just build-release   # Release build (vibes + iggy-server)
```

The `iggy-server` binary is copied to `target/debug/` or `target/release/` alongside vibes.

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

## Planning & Tracking

**Use the board to track all work:**

```bash
just board                        # Regenerate board view
just board new feat "description" # Create new feature
just board new milestone "name"   # Create new milestone
just board start <item>           # Begin work (→ in-progress)
just board review <item>          # Ready for review (→ review)
just board done <item>            # Complete (→ done + changelog)
just board status                 # Show counts per column
```

**Before starting any task:**
1. Check `docs/board/in-progress/` for current work
2. If starting new work, use `just board start` or `just board new`

**Board structure:**
```
docs/board/
├── backlog/       # Future work and ideas
├── ready/         # Designed, ready to implement
├── in-progress/   # Currently being worked on
├── review/        # Awaiting review/merge
└── done/          # Completed work
```

See [docs/board/CONVENTIONS.md](docs/board/CONVENTIONS.md) for full planning conventions.

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
2. **Plan**: Write `design.md` then `implementation.md` (see [docs/board/CONVENTIONS.md](docs/board/CONVENTIONS.md))
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
1. Use `just board done <item>` to move the item to done and update changelog
2. See [docs/board/README.md](docs/board/README.md) for overall project status

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


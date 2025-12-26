# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**vibes** - Vibe coding swiss army knife of enhancements

A utility application that proxies Claude Code with remote access capabilities, plugin system, and cross-platform support (CLI, native GUI, web UI). See [docs/PRD.md](docs/PRD.md) for full product requirements.

### Key Architecture

- **vibes-core**: Shared Rust library (sessions, events, plugins, server)
- **vibes-cli**: CLI binary consuming vibes-core
- **vibes-plugin-api**: Published crate for plugin authors
- **web-ui**: TanStack frontend embedded in binary

### Current State

**Milestone 1.1 (Core proxy) is complete.** The vibes-core crate provides:
- Session and SessionManager for managing Claude Code interactions
- EventBus trait with MemoryEventBus for real-time event distribution
- ClaudeBackend trait with PrintModeBackend (spawns `claude -p`) and MockBackend implementations
- Stream-JSON parser for Claude Code's output format
- ClaudeEvent and VibesEvent types for typed event handling

## Development Environment

The project uses a **Nix flake** for reproducible development environments with **direnv** for automatic shell loading.

```bash
cd vibes              # direnv auto-loads the Nix shell
direnv allow          # First time only
```

### Tooling

| Tool | Purpose |
|------|---------|
| `just` | Task runner (prefer over raw cargo commands) |
| `cargo-nextest` | Fast parallel test runner |
| `cargo-mutants` | Mutation testing |
| `cargo-watch` | Watch mode for development |

## Development Commands

**Always prefer `just` commands over direct cargo commands.**

```bash
just              # List all available commands
just dev          # Watch mode (cargo watch -x check)
just test         # Run tests (cargo nextest run)
just test-all     # Run tests including integration tests
just clippy       # Lint
just fmt          # Format code
just pre-commit   # Run all checks before committing
just mutants      # Mutation testing
just build        # Build the project
```

## Milestone Plans

Design and implementation plans are stored in `docs/plans/` following this convention:

```
docs/plans/
├── 01-core-proxy/
│   ├── design.md           # Architecture and design decisions
│   └── implementation.md   # Step-by-step implementation guide
├── 02-cli/
│   ├── design.md
│   └── implementation.md
└── ...
```

**Naming convention:** `XX-milestone-name/` where XX is the milestone number (01, 02, etc.)

When starting a new milestone:
1. Create the directory under `docs/plans/`
2. Write `design.md` first with architecture decisions
3. Write `implementation.md` with step-by-step tasks
4. Reference any new ADRs added to `docs/PRD.md`

## Development Workflow

**Always use these superpowers skills when developing:**

| Skill | When to Use |
|-------|-------------|
| `superpowers:executing-plans` | When implementing a milestone plan |
| `superpowers:test-driven-development` | Before writing any implementation code |
| `superpowers:systematic-debugging` | When encountering bugs, test failures, or unexpected behavior |
| `superpowers:brainstorming` | When designing new features or making architecture decisions |

**Workflow for implementing a milestone:**

1. **Start implementation**: Use `superpowers:executing-plans` skill with the `implementation.md` plan
2. **Write each feature**: Use `superpowers:test-driven-development` - write tests first, then implementation
3. **Fix issues**: Use `superpowers:systematic-debugging` - never guess at fixes, investigate first
4. **Review before commit**: Run `just pre-commit` and address any issues

**This is mandatory.** Do not skip these skills or try to implement without following this workflow.

## Testing Philosophy (TDD)

**REQUIRED:** Use the `superpowers:test-driven-development` skill when implementing any feature or fix. Invoke it with the Skill tool before writing implementation code.

We follow Test-Driven Development for component and utility code:

1. **Write the failing test first** — Define expected behavior before implementation
2. **Run the test to verify it fails** — Confirms the test is actually testing something
3. **Write minimal code to pass** — Only implement what's needed to make the test green
4. **Run tests to verify they pass** — Confirm implementation is correct
5. **Commit** — Small, frequent commits after each passing test

**What to test:**
- Component rendering and variants
- User interactions (clicks, input)
- Conditional rendering
- Props pass-through and className merging

**What NOT to test:**
- Implementation details (internal state, private methods)
- Styling (covered by visual review in Ladle)
- Third-party library behavior

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


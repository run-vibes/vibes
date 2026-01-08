# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Project Overview

**vibes** — The vibe engineering mech suit.

vibes augments *you*—the human developer—with AI-powered superpowers: remote session control, persistent context, and a learning system that remembers what works. You stay in command; vibes amplifies your reach.

### Architecture

| Crate | Purpose |
|-------|---------|
| **vibes-core** | Shared library (sessions, events, plugins, auth, tunnel) |
| **vibes-server** | HTTP/WebSocket server (axum-based) |
| **vibes-cli** | CLI binary, connects to daemon via WebSocket |
| **vibes-iggy** | EventLog backed by Apache Iggy |
| **vibes-plugin-api** | Published crate for plugin authors |
| **vibes-introspection** | Harness detection and capability discovery |
| **vibes-groove** | Continual learning plugin (under `plugins/`) |
| **web-ui** | TanStack frontend embedded via rust-embed |

See [docs/PRD.md](docs/PRD.md) for product requirements and [docs/board/README.md](docs/board/README.md) for project status.

## Setup

**Nix flake** with **direnv** for reproducible environments:

```bash
cd vibes                                    # direnv auto-loads Nix shell
direnv allow                                # First time only
git submodule update --init --recursive     # Initialize Iggy submodule
just setup-hooks                            # Enable pre-commit hooks
just build                                  # Build vibes + iggy-server
```

The `iggy-server` binary is copied alongside vibes in `target/`.

## Commands

**Always use `just` over raw cargo commands.**

### Top-Level Commands

| Command | Purpose |
|---------|---------|
| `just` | List all available commands |
| `just setup` | Full setup for new developers |
| `just build` | Debug build (vibes + iggy-server) |
| `just build-release` | Release build |
| `just pre-commit` | All checks before committing |

### Module Commands

Commands are organized into modules. Use `just <module>` to see available subcommands.

| Module | Commands | Examples |
|--------|----------|----------|
| `just tests` | `run`, `all`, `integration`, `watch`, `one <name>` | `just tests run` |
| `just quality` | `check`, `clippy`, `fmt`, `fmt-check`, `mutants` | `just quality clippy` |
| `just coverage` | `report`, `html`, `summary`, `lcov`, `package <pkg>` | `just coverage summary` |
| `just builds` | `debug`, `release`, `dev` | `just builds dev` |
| `just web` | `build`, `typecheck`, `test`, `install`, `e2e`, `e2e-setup` | `just web build` |
| `just plugin` | `list`, `install-groove`, `uninstall-groove` | `just plugin list` |
| `just board` | `status`, `generate`, `new`, `start`, `done`, `link` | `just board status` |

### Board Commands

| Command | Purpose |
|---------|---------|
| `just board` | Show available commands |
| `just board generate` | Regenerate board README.md |
| `just board status` | Show board status |
| `just board new story "title"` | Create new story |
| `just board new epic "name"` | Create new epic |
| `just board new milestone "name"` | Create new milestone |
| `just board start <id>` | Move story to in-progress |
| `just board done <id>` | Move story to done |

## Workflow

**Always use these superpowers skills:**

| Skill | When to Use |
|-------|-------------|
| `superpowers:brainstorming` | Before any new feature or architecture decision |
| `superpowers:executing-plans` | When implementing a milestone plan |
| `superpowers:test-driven-development` | Before writing any implementation code |
| `superpowers:systematic-debugging` | When encountering bugs or unexpected behavior |

### New Features

1. Check `docs/board/stages/in-progress/stories/` for current work
2. Use `superpowers:brainstorming` to explore options
3. Write `design.md` then `implementation.md`
4. Use `superpowers:executing-plans` with the plan
5. Use `superpowers:test-driven-development` for each task
6. Run `just pre-commit` and address issues
7. Complete: update story/board, commit, push, create PR

### Design Document Location (IMPORTANT)

**Override for `superpowers:brainstorming` skill:** Do NOT write designs to `docs/plans/`. That path doesn't exist in this project.

All designs go in `docs/board/` following CONVENTIONS.md:

| Size | Structure |
|------|-----------|
| **Small feature** | Story file: `docs/board/stages/backlog/stories/<type>-NNNN-name.md` |
| **Large feature** | Milestone directory: `docs/board/milestones/NN-name/design.md` |

Use `just board new story "name"` or `just board new milestone "name"` to create the correct structure.

### Bug Fixes

1. Use `superpowers:systematic-debugging` to investigate
2. Write failing test, then fix
3. Run `just pre-commit`
4. Commit with `fix:` prefix

See [docs/board/CONVENTIONS.md](docs/board/CONVENTIONS.md) for detailed planning conventions.

### Story State Changes (IMPORTANT)

**ALWAYS use `just board` commands to change story state. NEVER manually move files or update symlinks.**

| Action | Command |
|--------|---------|
| Start working | `just board start <story-id>` |
| Complete work | `just board done <story-id>` |

These commands handle file moves, symlink updates, and changelog entries automatically.

### Completing Work

**REQUIRED before marking work done:**

1. Run `just pre-commit` — all checks pass

2. Update the board:
   - Check acceptance criteria in story file
   - Set frontmatter `status: done`
   - Run `just board done <story-id>` (moves file, updates symlinks, adds changelog)

3. Commit, push, create PR

## Git Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>
```

| Type | Use |
|------|-----|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `refactor:` | Code restructuring (no behavior change) |
| `test:` | Adding or updating tests |
| `chore:` | Build, tooling, dependencies |

**Guidelines:**
- Imperative mood: "add feature" not "added feature"
- Under 72 characters, no trailing period
- Do NOT include "Generated with Claude Code" or "Co-Authored-By"

### Pull Requests

**Title:** `<type>: <description>`

**Body:**
```markdown
## Summary
- Bullet points describing what changed

## Test Plan
- [ ] Verification steps as checklist
```

## CI

CI runs on GitHub Actions using **Nix** to match local environment. Runs `just pre-commit` (fmt, clippy, test).

Integration tests require Claude CLI and are not run in CI—run `just test-integration` locally.

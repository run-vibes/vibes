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

See [docs/VISION.md](docs/VISION.md) for product vision and [docs/board/README.md](docs/board/README.md) for project status.

### Event Sourcing (CRITICAL)

**vibes is a fully event-sourced system.** All state derives from events stored in Apache Iggy.

| Principle | Description |
|-----------|-------------|
| **Events are the source of truth** | State is derived from replaying events, never stored directly |
| **Iggy is the event store** | All domain events go through vibes-iggy |
| **Projections for queries** | SQLite/other stores are read-optimized projections, rebuilt from events |

**Before implementing any storage:**
1. Define the events that capture state changes
2. Store events in Iggy
3. Build projections as needed for query performance

**ASK before making architectural decisions** like:
- Choosing to store state directly instead of as events
- Adding a new database or storage mechanism
- Changing how events flow through the system
- Deviating from the event-sourced pattern

## Setup

**Nix flake** with **direnv** for reproducible environments:

```bash
cd vibes                                    # direnv auto-loads Nix shell
direnv allow                                # First time only
just setup-hooks                            # Enable git hooks (pre-commit + post-checkout)
just build                                  # Build vibes + iggy-server
```

Submodules are initialized automatically by the `post-checkout` hook when creating worktrees or cloning.

### Shared Build Cache

All worktrees share build caches for faster builds:

| Cache | Location | Purpose |
|-------|----------|---------|
| Rust artifacts | `~/.cache/cargo-target/vibes/` | Cargo compilation cache |
| Turbo cache | `~/.cache/turbo/vibes/` | npm workspace build cache |

This means:
- Fresh worktrees reuse compiled artifacts from other worktrees
- Incremental compilation is shared across worktrees
- Web-ui `dist/` is restored from turbo cache on checkout (~80ms)

**Binary Isolation:** `just build` copies final binaries (`vibes`, `iggy-server`) to `./target/debug/` in each worktree. Tests automatically find these worktree-local binaries, preventing cross-worktree binary clobbering.

**WARNING:** `cargo clean` will delete Rust artifacts for ALL worktrees. Use with caution.

## Commands

**Always use `just` over raw cargo commands.**

### Top-Level Commands

| Command | Purpose |
|---------|---------|
| `just` | List all available commands |
| `just setup` | Full setup for new developers |
| `just build` | Debug build (vibes + iggy-server) |
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
| `just board ice <id>` | Move story to icebox (blocked/deferred) |
| `just board thaw <id>` | Move story from icebox to backlog |
| `just board start-milestone <id>` | Set milestone to in-progress |
| `just board done-milestone <id>` | Set milestone to done |
| `just board done-epic <id>` | Set epic to done |

### Verification Commands

Visual verification captures screenshots and videos to document system behavior.

| Command | Purpose |
|---------|---------|
| `just verify` | Show available verify commands |
| `just verify snapshots` | Tier 1: Capture key screen PNGs (~30s) |
| `just verify checkpoints` | Tier 2: Capture interaction sequences (~2min) |
| `just verify videos` | Tier 3: Record CLI + Web videos (~5min) |
| `just verify all` | Run all tiers and generate report |
| `just verify report` | Generate verification/report.md |
| `just verify summary` | Show artifact counts |
| `just verify clean` | Remove all artifacts |
| `just verify ai <story-id>` | Run AI verification for a story |
| `just verify ai <story-id> --model "ollama:model"` | AI verification with model override |

**Run `just verify all` before creating PRs** to capture visual documentation.

Artifacts are gitignored except `verification/report.md`. Define what to capture in:
- `verification/snapshots.json` - Pages to screenshot
- `verification/checkpoints.json` - Interaction sequences

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
| **Small feature** | Story file: `docs/board/stages/backlog/stories/[TYPE][NNNN]-name.md` |
| **Large feature** | Milestone directory: `docs/board/epics/<epic>/milestones/NN-name/design.md` |

Use `just board new story "name"` or `just board new milestone "name"` to create the correct structure.

### Design System Workflow (IMPORTANT)

**NEVER create ad-hoc CSS classes in web-ui. Build reusable components in the design system first.**

When adding new UI patterns or styled elements:

1. **Check design-system first:** Look in `design-system/src/` for existing components
2. **Create in design-system:** If the pattern doesn't exist, add it:
   - Component: `design-system/src/primitives/<Name>/<Name>.tsx`
   - Styles: `design-system/src/primitives/<Name>/<Name>.module.css`
   - Story: `design-system/src/primitives/<Name>/<Name>.stories.tsx`
   - Export: Update `design-system/src/primitives/index.ts`
3. **View in Ladle:** Run `just web ladle` to preview and iterate on the component
4. **Use in web-ui:** Import from `@vibes/design-system` in web-ui pages

**Examples of design system components:**
- `PageHeader` — Standard page title with optional left/right content
- `Card` — Content containers with CRT styling
- `Badge` — Status indicators
- `StreamView` — Event stream display
- `SubnavBar` — Secondary navigation with overflow menu

Run `just web ladle` to browse all available components.

### Design Prototypes Workflow (IMPORTANT)

**When exploring visual directions or UI layouts, create static HTML/CSS prototypes.**

Prototypes live in `docs/design/prototypes/` organized by phase:

```
docs/design/prototypes/
├── README.md
├── phase-01-exploration/     # Initial style explorations
├── phase-02-art-deco/        # Art Deco direction
└── phase-NN-description/     # Future phases
```

**When creating prototypes:**

1. **Create a new phase directory** if starting a new design direction
2. **Include a shared.css** for design tokens and reusable styles
3. **Number files sequentially** (01-xxx.html, 02-xxx.html)
4. **Use iterations** for refinements (v11, v12, v13 suffixes)

**Viewing prototypes:**
```bash
cd docs/design/prototypes/phase-NN-xxx
python -m http.server 8765
```

**When to create prototypes vs components:**
- **Prototypes**: Exploring layouts, visual directions, new page designs
- **Design system**: Reusable components ready for implementation

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

### Milestone Management

**When starting the FIRST story of a milestone:**

1. Run `just board start-milestone <id>`
2. Commit with message: `chore(board): start milestone NN-name`

**When completing the LAST story of a milestone:**

1. Run `just board done-milestone <id>`
2. Commit with message: `chore(board): complete milestone NN-name`

Milestone files live in `docs/board/epics/<epic>/milestones/<id>/`.

### Epic Management

**When completing all milestones of an epic:**

1. Run `just board done-epic <id>`
2. Commit with message: `chore(board): complete epic <name>`

### Completing Work

**REQUIRED before marking work done:**

1. Run `just pre-commit` — all checks pass

2. **Refactor pass:** Run `code-simplifier:code-simplifier` agent on changes
   - Reviews for unnecessary complexity, over-engineering, YAGNI violations
   - Simplify any flagged code before proceeding

3. Update the board:
   - Check acceptance criteria in story file
   - Set frontmatter `status: done`
   - Run `just board done <story-id>` (moves file, updates symlinks, adds changelog)

4. Commit, push, create PR

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

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

This project is in its initial state with no source code yet. As development begins, update this file with:
- Module organization as it takes shape

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

## Progress Tracking

When completing work that corresponds to a roadmap item:
1. Update [docs/PROGRESS.md](docs/PROGRESS.md) to mark the item complete
2. Change `[ ]` to `[x]` for completed items, or `[~]` for in-progress items
3. Add an entry to the Changelog table at the bottom with the date and change

## Git Conventions

**Commit messages:**
- Use imperative mood: "Add feature" not "Added feature"
- First line: concise summary (50 chars or less ideal)
- Body: explain what and why, not how
- No "Generated with Claude Code" or "Co-Authored-By" footers

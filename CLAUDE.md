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
- Build and test commands once `Cargo.toml` is created
- Module organization as it takes shape

## Tooling Intentions (from .gitignore)

The project is configured to support:
- **Mutation testing**: `cargo mutants` (mutants.out directories are gitignored)
- **Formatting**: `rustfmt` (backup files are gitignored)

## Development Commands

Commands will be added once the Cargo project is initialized. Expected:
```bash
cargo build          # Build the project
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint
cargo mutants        # Mutation testing
```

## Git Conventions

**Commit messages:**
- Use imperative mood: "Add feature" not "Added feature"
- First line: concise summary (50 chars or less ideal)
- Body: explain what and why, not how
- No "Generated with Claude Code" or "Co-Authored-By" footers

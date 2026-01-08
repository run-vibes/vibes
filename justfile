# vibes task runner

# ─── Modules ─────────────────────────────────────────────────────────────────

# Planning board management
mod board '.justfiles/board.just'

# Testing commands (use: just tests run, just tests all)
mod tests '.justfiles/test.just'

# Code quality commands (use: just quality check, just quality clippy)
mod quality '.justfiles/quality.just'

# Coverage commands (use: just coverage report, just coverage summary)
mod coverage '.justfiles/coverage.just'

# Build commands (use: just builds debug, just builds release)
mod builds '.justfiles/build.just'

# Web UI commands (use: just web build, just web test)
mod web '.justfiles/web.just'

# Plugin management (use: just plugin list, just plugin install-groove)
mod plugin '.justfiles/plugin.just'

# ─── Top-Level Commands ──────────────────────────────────────────────────────

# Default: show available commands
default:
    @just --list

# Setup git hooks (run once after clone)
setup-hooks:
    git config core.hooksPath .githooks
    @echo "✓ Git hooks configured"

# Full setup for new developers
setup: setup-hooks
    #!/usr/bin/env bash
    set -euo pipefail

    # Initialize submodules if needed
    if [[ ! -f vendor/iggy/Cargo.toml ]]; then
        echo "Initializing git submodules..."
        git submodule update --init --recursive
    fi

    # Install npm deps
    npm ci

    echo "✓ Setup complete. Run 'just build' to build."

# Run all checks (pre-commit)
pre-commit: fmt-check clippy test web::typecheck web::test
    @echo "✓ All pre-commit checks passed"

# ─── Convenience Aliases ─────────────────────────────────────────────────────
# Backwards compatible top-level commands

# Development watch mode
dev:
    cargo watch -x check

# Run unit tests
test:
    cargo nextest run

# Run all tests including integration
test-all:
    cargo nextest run --run-ignored all

# Integration tests only (requires Claude CLI)
test-integration:
    cargo nextest run --run-ignored ignored-only

# Watch mode for tests
test-watch:
    cargo watch -x 'nextest run'

# Run a specific test by name
test-one NAME:
    cargo nextest run {{NAME}}

# Type check all targets
check:
    cargo check --all-targets

# Lint with clippy
clippy:
    cargo clippy --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Mutation testing
mutants:
    cargo mutants

# Build debug (vibes + iggy)
build: web::build builds::_check-submodules
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/debug
    cp vendor/iggy/target/debug/iggy-server target/debug/
    @echo "✓ Built: vibes (debug), iggy-server (debug)"

# Build release (vibes + iggy)
build-release: web::build builds::_check-submodules
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/release
    cp vendor/iggy/target/release/iggy-server target/release/
    @echo "✓ Built: vibes (release), iggy-server (release)"

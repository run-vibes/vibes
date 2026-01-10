# vibes task runner

# ─── Modules ─────────────────────────────────────────────────────────────────

# Planning board management
mod board '.justfiles/board.just'

# Testing commands
mod tests '.justfiles/test.just'

# Code quality commands
mod quality '.justfiles/quality.just'

# Coverage commands
mod coverage '.justfiles/coverage.just'

# Build commands
mod builds '.justfiles/build.just'

# Web UI commands
mod web '.justfiles/web.just'

# Plugin management
mod plugin '.justfiles/plugin.just'

# CLI recording commands
mod cli '.justfiles/cli.just'

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
pre-commit: quality::fmt-check quality::clippy tests::run web::typecheck web::test
    @echo "✓ All pre-commit checks passed"

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

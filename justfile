# vibes task runner

set unstable

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

# Verification commands
mod verify '.justfiles/verify.just'

# ─── Top-Level Commands ──────────────────────────────────────────────────────

# Show available commands
[group('common')]
default:
    @just --list

# Setup git hooks (run once after clone)
[group('setup')]
setup-hooks:
    git config core.hooksPath .githooks
    @echo "✓ Git hooks configured"

# Full setup for new developers
[group('setup')]
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
[group('common')]
pre-commit: quality::fmt-check quality::clippy tests::run web::typecheck web::test
    @echo "✓ All pre-commit checks passed"

# Build debug (vibes + iggy)
# Compiles to $CARGO_TARGET_DIR, copies binaries to ./target/debug/ for worktree isolation
[group('common')]
build: web::build builds::_check-submodules
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    just builds _copy-binaries debug
    echo "✓ Built: vibes (debug), iggy-server (debug)"
    echo "  Local: ./target/debug/"


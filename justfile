# vibes task runner

# Planning board management
mod board '.justfiles/board.just'

# Default: show available commands
default:
    @just --list

# Setup git hooks (run once after clone)
setup-hooks:
    git config core.hooksPath .githooks
    @echo "✓ Git hooks configured"

# Development
dev:
    cargo watch -x check

# Testing
test:
    cargo nextest run

test-all:
    cargo nextest run --run-ignored all

# Integration tests only (requires Claude CLI)
test-integration:
    cargo nextest run --run-ignored ignored-only

test-watch:
    cargo watch -x 'nextest run'

test-one NAME:
    cargo nextest run {{NAME}}

# Code quality
check:
    cargo check --all-targets

clippy:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

# Mutation testing
mutants:
    cargo mutants

# Build (vibes + iggy for debug)
build: build-web _check-submodules
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/debug
    cp vendor/iggy/target/debug/iggy-server target/debug/
    @echo "✓ Built: vibes (debug), iggy-server (debug)"

# Build release (vibes + iggy for release)
build-release: build-web _check-submodules
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/release
    cp vendor/iggy/target/release/iggy-server target/release/
    @echo "✓ Built: vibes (release), iggy-server (release)"

# Build web-ui (required for server)
build-web:
    npm run build

# Install npm dependencies (uses workspaces)
npm-install:
    npm ci

# E2E tests with Playwright
test-e2e:
    npm run test:e2e

# E2E tests in headed mode (visible browser)
test-e2e-headed:
    npm run test:e2e:headed

# E2E tests in debug mode
test-e2e-debug:
    npm run test:e2e -- --debug

# Install Playwright browsers
e2e-setup:
    npx playwright install chromium --with-deps

# Run all checks (pre-commit)
pre-commit: fmt-check clippy test

# ─── Submodule Management ────────────────────────────────────────────────────

# Check that submodules are initialized
_check-submodules:
    #!/usr/bin/env bash
    if [[ ! -f vendor/iggy/Cargo.toml ]]; then
        echo "Error: Git submodules not initialized."
        echo "Run: git submodule update --init --recursive"
        exit 1
    fi

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

# ─── Plugin Management ───────────────────────────────────────────────────────

# Install groove plugin (build, copy, and enable)
install-groove: build-release
    #!/usr/bin/env bash
    set -euo pipefail

    PLUGIN_DIR="${HOME}/.config/vibes/plugins/groove"
    mkdir -p "${PLUGIN_DIR}"

    # Detect platform and copy the appropriate library
    if [[ "$(uname)" == "Darwin" ]]; then
        cp target/release/libvibes_groove.dylib "${PLUGIN_DIR}/groove.dylib"
        echo "✓ Installed groove.dylib to ${PLUGIN_DIR}"
    else
        cp target/release/libvibes_groove.so "${PLUGIN_DIR}/groove.so"
        echo "✓ Installed groove.so to ${PLUGIN_DIR}"
    fi

    # Enable the plugin in the registry
    ./target/release/vibes plugin enable groove
    echo "✓ Plugin enabled in registry"

    echo ""
    echo "✓ Groove plugin installed and enabled"
    echo ""
    echo "Run 'vibes plugin list' to verify."

# Uninstall groove plugin
uninstall-groove:
    #!/usr/bin/env bash
    set -euo pipefail

    # Disable the plugin first (ignore errors if not enabled)
    ./target/release/vibes plugin disable groove 2>/dev/null || true

    # Remove the plugin directory
    rm -rf "${HOME}/.config/vibes/plugins/groove"
    echo "✓ Groove plugin uninstalled"

# List installed plugins
plugin-list:
    cargo run --release -p vibes-cli -- plugin list

# Build commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available build commands
default:
    @echo "Build commands:"
    @echo "  just builds debug    Debug build (vibes + iggy-server)"
    @echo "  just builds release  Release build (vibes + iggy-server)"
    @echo "  just builds dev      Development watch mode"

# Debug build (vibes + iggy-server)
# Both binaries go to $CARGO_TARGET_DIR (shared across worktrees)
debug: _check-submodules
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    echo "✓ Built: vibes (debug), iggy-server (debug)"
    echo "  Location: ${CARGO_TARGET_DIR:-target}/debug/"

# Release build (vibes + iggy-server)
# Both binaries go to $CARGO_TARGET_DIR (shared across worktrees)
release: _check-submodules
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p server
    echo "✓ Built: vibes (release), iggy-server (release)"
    echo "  Location: ${CARGO_TARGET_DIR:-target}/release/"

# Development watch mode
dev:
    cargo watch -x check

# ─── Internal Helpers ────────────────────────────────────────────────────────

# Check that submodules are initialized
_check-submodules:
    #!/usr/bin/env bash
    if [[ ! -f vendor/iggy/Cargo.toml ]]; then
        echo "Error: Git submodules not initialized."
        echo "Run: git submodule update --init --recursive"
        exit 1
    fi

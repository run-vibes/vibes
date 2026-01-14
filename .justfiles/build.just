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
# Compiles to $CARGO_TARGET_DIR, copies binaries to ./target/debug/ for worktree isolation
debug: _check-submodules
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    just builds _copy-binaries debug
    echo "✓ Built: vibes (debug), iggy-server (debug)"
    echo "  Local: ./target/debug/"

# Release build (vibes + iggy-server)
# Compiles to $CARGO_TARGET_DIR, copies binaries to ./target/release/ for worktree isolation
release: _check-submodules
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p server
    just builds _copy-binaries release
    echo "✓ Built: vibes (release), iggy-server (release)"
    echo "  Local: ./target/release/"

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

# Copy binaries from shared target to worktree-local target
# This ensures each worktree has its own binaries for test isolation
_copy-binaries profile:
    #!/usr/bin/env bash
    set -euo pipefail
    SRC="${CARGO_TARGET_DIR:-target}/{{profile}}"
    DEST="./target/{{profile}}"
    mkdir -p "$DEST"
    cp "$SRC/vibes" "$DEST/" 2>/dev/null || true
    cp "$SRC/iggy-server" "$DEST/" 2>/dev/null || true

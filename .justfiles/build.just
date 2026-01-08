# Build commands for vibes

# Default: show available build commands
default:
    @just build --list

# Debug build (vibes + iggy-server)
debug: _check-submodules
    cargo build
    cargo build --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/debug
    cp vendor/iggy/target/debug/iggy-server target/debug/
    @echo "✓ Built: vibes (debug), iggy-server (debug)"

# Release build (vibes + iggy-server)
release: _check-submodules
    cargo build --release
    cargo build --release --manifest-path vendor/iggy/Cargo.toml -p server
    mkdir -p target/release
    cp vendor/iggy/target/release/iggy-server target/release/
    @echo "✓ Built: vibes (release), iggy-server (release)"

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

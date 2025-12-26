# vibes task runner

# Default: show available commands
default:
    @just --list

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

# Build
build:
    cargo build

build-release:
    cargo build --release

# Run all checks (pre-commit)
pre-commit: fmt-check clippy test

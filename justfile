# vibes task runner

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

# Build
build: build-web
    cargo build

build-release: build-web
    cargo build --release

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

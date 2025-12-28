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
build: build-web
    cargo build

build-release: build-web
    cargo build --release

# Build web-ui (required for server)
build-web:
    cd web-ui && npm run build

# E2E tests with Playwright
test-e2e:
    cd e2e-tests && npm test

# E2E tests in headed mode (visible browser)
test-e2e-headed:
    cd e2e-tests && npm run test:headed

# E2E tests in debug mode
test-e2e-debug:
    cd e2e-tests && npm run test:debug

# Install Playwright browsers
e2e-setup:
    cd e2e-tests && npm install && npx playwright install chromium

# Run all checks (pre-commit)
pre-commit: fmt-check clippy test

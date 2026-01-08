# Code quality commands for vibes

# Default: show available quality commands
default:
    @just quality --list

# Type check all targets
check:
    cargo check --all-targets

# Run clippy linter
clippy:
    cargo clippy --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Mutation testing
mutants:
    cargo mutants

# Run all quality checks (pre-commit)
pre-commit: fmt-check clippy
    @echo "✓ Code quality checks passed"

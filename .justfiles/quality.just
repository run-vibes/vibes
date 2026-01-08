# Code quality commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available quality commands
default:
    @echo "Code quality commands:"
    @echo "  just quality check      Type check all targets"
    @echo "  just quality clippy     Run clippy linter"
    @echo "  just quality fmt        Format code"
    @echo "  just quality fmt-check  Check formatting without modifying files"
    @echo "  just quality mutants    Mutation testing"

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

# Testing commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available test commands
default:
    @echo "Testing commands:"
    @echo "  just tests run           Run unit tests"
    @echo "  just tests all           Run all tests including integration"
    @echo "  just tests integration   Integration tests only (requires Claude CLI)"
    @echo "  just tests watch         Watch mode for tests"
    @echo "  just tests one <name>    Run a specific test by name"

# Run unit tests
run:
    cargo nextest run

# Run all tests including integration
all:
    cargo nextest run --run-ignored all

# Integration tests only (requires Claude CLI)
integration:
    cargo nextest run --run-ignored ignored-only

# Watch mode for tests
watch:
    cargo watch -x 'nextest run'

# Run a specific test by name
one NAME:
    cargo nextest run {{NAME}}

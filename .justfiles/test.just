# Testing commands for vibes

# Default: show available test commands
default:
    @just test --list

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

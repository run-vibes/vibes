# CLI recording commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available CLI commands
default:
    @echo "CLI recording commands:"
    @echo "  just cli record       Generate all CLI recordings"
    @echo "  just cli record-one   Generate a specific recording"
    @echo "  just cli verify       Verify CLI output matches expected"

# Generate all CLI recordings
record:
    #!/usr/bin/env bash
    set -euo pipefail
    cd cli/recordings
    for tape in tapes/*.tape; do
        name=$(basename "$tape" .tape)
        echo "Recording: $name"
        vhs "$tape"
    done
    echo "✓ All recordings generated in cli/recordings/output/"

# Generate a specific recording
record-one name:
    #!/usr/bin/env bash
    set -euo pipefail
    cd cli/recordings
    if [[ ! -f "tapes/{{name}}.tape" ]]; then
        echo "Error: tapes/{{name}}.tape not found"
        exit 1
    fi
    vhs "tapes/{{name}}.tape"
    echo "✓ Generated: output/{{name}}.gif"

# Verify CLI output matches expected
verify:
    #!/usr/bin/env bash
    set -euo pipefail
    failed=0

    # Check help output
    if [[ -f cli/recordings/expected/help.txt ]]; then
        echo "Verifying: vibes --help"
        if ! ./target/debug/vibes --help | diff -q cli/recordings/expected/help.txt - > /dev/null 2>&1; then
            echo "  ✗ Output differs from expected"
            failed=1
        else
            echo "  ✓ Matches expected"
        fi
    fi

    # Check version output
    if [[ -f cli/recordings/expected/version.txt ]]; then
        echo "Verifying: vibes --version"
        if ! ./target/debug/vibes --version | diff -q cli/recordings/expected/version.txt - > /dev/null 2>&1; then
            echo "  ✗ Output differs from expected"
            failed=1
        else
            echo "  ✓ Matches expected"
        fi
    fi

    if [[ $failed -eq 1 ]]; then
        echo "✗ Some verifications failed"
        exit 1
    fi
    echo "✓ All verifications passed"

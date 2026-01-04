#!/bin/bash
# Test script for vibes-hook-send.sh
# Verifies that session_id is extracted from input JSON and passed to --session

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

# Set up paths
MOCK_VIBES="$TEST_DIR/vibes"
CAPTURED_ARGS="$TEST_DIR/captured_args"

# Create a mock vibes command that captures arguments
cat > "$MOCK_VIBES" << EOF
#!/bin/bash
# Mock vibes command - capture all arguments
echo "\$@" >> "$CAPTURED_ARGS"
EOF
chmod +x "$MOCK_VIBES"

run_test() {
    local test_name="$1"
    local input_json="$2"
    local expected_pattern="$3"

    # Clear captured args
    : > "$CAPTURED_ARGS"

    # Run the hook script with mock vibes
    export VIBES_BIN="$MOCK_VIBES"
    echo "$input_json" | "$SCRIPT_DIR/vibes-hook-send.sh" user_prompt_submit

    # Check if expected pattern is in captured args
    if grep -qF -- "$expected_pattern" "$CAPTURED_ARGS" 2>/dev/null; then
        echo "PASS: $test_name"
        return 0
    else
        echo "FAIL: $test_name"
        echo "  Expected pattern: $expected_pattern"
        echo "  Captured args: $(cat "$CAPTURED_ARGS" 2>/dev/null || echo '<empty>')"
        return 1
    fi
}

echo "=== Testing vibes-hook-send.sh ==="
echo

FAILED=0

# Test 1: session_id in input JSON should be passed as --session
run_test "session_id from JSON passed as --session" \
    '{"session_id":"test-session-123","prompt":"hello"}' \
    '--session test-session-123' || FAILED=$((FAILED + 1))

# Test 2: null session_id should not add --session
run_test "null session_id does not add --session" \
    '{"session_id":null,"prompt":"hello"}' \
    'event send --type hook' || FAILED=$((FAILED + 1))

# Test 3: missing session_id should not add --session
run_test "missing session_id does not add --session" \
    '{"prompt":"hello"}' \
    'event send --type hook' || FAILED=$((FAILED + 1))

# Test 4: Auto-detect vibes from config file when VIBES_BIN is unset
# This tests that the script reads ~/.config/vibes/bin_path when:
# - VIBES_BIN is not set
# - vibes is not in PATH
test_autodetect() {
    local test_name="auto-detect vibes from config file"

    # Create a temporary HOME to isolate config
    local FAKE_HOME="$TEST_DIR/home"
    mkdir -p "$FAKE_HOME/.config/vibes"

    # Write config file pointing to our mock vibes
    echo "$MOCK_VIBES" > "$FAKE_HOME/.config/vibes/bin_path"

    # Clear captured args
    : > "$CAPTURED_ARGS"

    # Run WITHOUT VIBES_BIN set, with PATH that doesn't contain vibes
    # Use subshell to isolate env changes
    # Keep essential tools but ensure 'vibes' command isn't found
    (
        unset VIBES_BIN
        export HOME="$FAKE_HOME"
        # Keep standard paths for tools like cat, jq, but vibes won't be in any of them
        export PATH="/usr/bin:/bin"
        echo '{"session_id":"auto-test","prompt":"hello"}' | "$SCRIPT_DIR/vibes-hook-send.sh" user_prompt_submit
    )

    # Check if vibes was called with session flag
    if grep -qF -- '--session auto-test' "$CAPTURED_ARGS" 2>/dev/null; then
        echo "PASS: $test_name"
        return 0
    else
        echo "FAIL: $test_name"
        echo "  Expected vibes to be called via config file auto-detection"
        echo "  Captured args: $(cat "$CAPTURED_ARGS" 2>/dev/null || echo '<empty>')"
        return 1
    fi
}

test_autodetect || FAILED=$((FAILED + 1))

echo
if [ $FAILED -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "$FAILED test(s) failed"
    exit 1
fi

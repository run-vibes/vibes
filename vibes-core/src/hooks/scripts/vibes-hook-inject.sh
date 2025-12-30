#!/bin/bash
# vibes-hook-inject.sh - Send hook data to vibes daemon and receive response
#
# This script is called by injection hooks (SessionStart, UserPromptSubmit)
# to forward Claude Code events to the vibes daemon and receive a response
# with additional context to inject into the conversation.
#
# Usage: vibes-hook-inject.sh <hook-type>
#   Reads JSON data from stdin and wraps it with type information.
#   Outputs JSON response from vibes for Claude to process.
#
# Environment:
#   VIBES_SOCKET_PATH - Unix socket path (default: /tmp/vibes-hooks.sock)
#   VIBES_HOOK_PORT   - TCP port for Windows (default: 7744)
#   VIBES_SESSION_ID  - Session ID to include in events

set -e

HOOK_TYPE="${1:-unknown}"
SOCKET_PATH="${VIBES_SOCKET_PATH:-/tmp/vibes-hooks.sock}"
TCP_PORT="${VIBES_HOOK_PORT:-7744}"
TIMEOUT="${VIBES_HOOK_TIMEOUT:-5}"

# Read input JSON from stdin
INPUT_JSON=$(cat)

# Build the event JSON with type wrapper
# Add session_id if available
if [ -n "$VIBES_SESSION_ID" ]; then
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c ". + {session_id: \"$VIBES_SESSION_ID\"} | {type: \"$HOOK_TYPE\"} + .")
else
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c "{type: \"$HOOK_TYPE\"} + .")
fi

# Function to send and receive response
send_and_receive() {
    local response=""

    if [ -S "$SOCKET_PATH" ]; then
        # Unix socket exists - use it with bidirectional communication
        response=$(echo "$EVENT_JSON" | nc -U "$SOCKET_PATH" 2>/dev/null) || true
    elif command -v nc &>/dev/null; then
        # Try TCP fallback with response
        response=$(echo "$EVENT_JSON" | nc -w "$TIMEOUT" 127.0.0.1 "$TCP_PORT" 2>/dev/null) || true
    fi

    echo "$response"
}

# Get response from vibes
RESPONSE=$(send_and_receive)

# Output response if we got one, otherwise output empty object
if [ -n "$RESPONSE" ]; then
    if echo "$RESPONSE" | jq -e . >/dev/null 2>&1; then
        echo "$RESPONSE"
    else
        # Log invalid JSON response to stderr for debugging
        echo "[vibes-hook-inject] Warning: Received invalid JSON response, returning empty object" >&2
        echo "[vibes-hook-inject] Raw response was: $RESPONSE" >&2
        echo '{}'
    fi
else
    # No response received (daemon not running or connection failed)
    echo '[vibes-hook-inject] Debug: No response from vibes daemon' >&2
    echo '{}'
fi

exit 0

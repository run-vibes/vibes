#!/bin/bash
# vibes-hook-send.sh - Send hook data to vibes daemon
#
# This script is called by hook scripts to forward Claude Code events
# to the vibes daemon via Unix socket (Linux/macOS) or TCP (Windows).
#
# Usage: vibes-hook-send.sh <hook-type>
#   Reads JSON data from stdin and wraps it with type information.
#
# Environment:
#   VIBES_SOCKET_PATH - Unix socket path (default: /tmp/vibes-hooks.sock)
#   VIBES_HOOK_PORT   - TCP port for Windows (default: 7744)
#   VIBES_SESSION_ID  - Session ID to include in events

set -e

HOOK_TYPE="${1:-unknown}"
SOCKET_PATH="${VIBES_SOCKET_PATH:-/tmp/vibes-hooks.sock}"
TCP_PORT="${VIBES_HOOK_PORT:-7744}"

# Read input JSON from stdin
INPUT_JSON=$(cat)

# Build the event JSON with type wrapper
# Add session_id if available
if [ -n "$VIBES_SESSION_ID" ]; then
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c ". + {session_id: \"$VIBES_SESSION_ID\"} | {type: \"$HOOK_TYPE\"} + .")
else
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c "{type: \"$HOOK_TYPE\"} + .")
fi

# Send to socket (Unix) or TCP (Windows/fallback)
if [ -S "$SOCKET_PATH" ]; then
    # Unix socket exists - use it
    echo "$EVENT_JSON" | nc -U "$SOCKET_PATH" 2>/dev/null || true
elif command -v nc &>/dev/null; then
    # Try TCP fallback
    echo "$EVENT_JSON" | nc -w 1 127.0.0.1 "$TCP_PORT" 2>/dev/null || true
fi

# Always exit successfully - hooks shouldn't block Claude
exit 0

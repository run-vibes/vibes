#!/bin/bash
# vibes-hook-send.sh - Send hook data to Iggy via vibes CLI
#
# This script is called by hook scripts to forward Claude Code events
# to Iggy message streaming via the vibes CLI.
#
# Usage: vibes-hook-send.sh <hook-type>
#   Reads JSON data from stdin and wraps it with type information.
#
# Environment:
#   VIBES_SESSION_ID  - Session ID to include in events

set -e

HOOK_TYPE="${1:-unknown}"

# Read input JSON from stdin
INPUT_JSON=$(cat)

# Build the event JSON with type wrapper
# Add session_id if available
if [ -n "$VIBES_SESSION_ID" ]; then
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c ". + {session_id: \"$VIBES_SESSION_ID\"} | {type: \"$HOOK_TYPE\"} + .")
else
    EVENT_JSON=$(echo "$INPUT_JSON" | jq -c "{type: \"$HOOK_TYPE\"} + .")
fi

# Send to Iggy via vibes CLI
# The CLI handles authentication and connection details
if command -v vibes &>/dev/null; then
    vibes event send --type hook --data "$EVENT_JSON" ${VIBES_SESSION_ID:+--session "$VIBES_SESSION_ID"} 2>/dev/null || true
fi

# Always exit successfully - hooks shouldn't block Claude
exit 0

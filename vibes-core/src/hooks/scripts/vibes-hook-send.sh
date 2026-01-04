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
#   VIBES_SESSION_ID  - Session ID override (optional, defaults to JSON input)

set -e

HOOK_TYPE="${1:-unknown}"

# Read input JSON from stdin
INPUT_JSON=$(cat)

# Extract session_id from input JSON (Claude Code provides this)
# Use VIBES_SESSION_ID env var as override if set
if [ -n "$VIBES_SESSION_ID" ]; then
    SESSION_ID="$VIBES_SESSION_ID"
else
    # Extract from JSON - returns empty string if null or missing
    SESSION_ID=$(echo "$INPUT_JSON" | jq -r '.session_id // empty')
fi

# Build the event JSON with type wrapper
EVENT_JSON=$(echo "$INPUT_JSON" | jq -c "{type: \"$HOOK_TYPE\"} + .")

# Send to Iggy via vibes CLI
# Resolution order:
# 1. VIBES_BIN env var (for development)
# 2. vibes in PATH
# 3. Path from config file ~/.config/vibes/bin_path
VIBES_CMD=""
if [ -n "$VIBES_BIN" ]; then
    VIBES_CMD="$VIBES_BIN"
elif command -v vibes &>/dev/null; then
    VIBES_CMD="vibes"
elif [ -f "$HOME/.config/vibes/bin_path" ]; then
    VIBES_CMD="$(cat "$HOME/.config/vibes/bin_path")"
fi

if [ -n "$VIBES_CMD" ] && { [ -x "$VIBES_CMD" ] || command -v "$VIBES_CMD" &>/dev/null; }; then
    "$VIBES_CMD" event send --type hook --data "$EVENT_JSON" ${SESSION_ID:+--session "$SESSION_ID"} 2>/dev/null || true
fi

# Always exit successfully - hooks shouldn't block Claude
exit 0

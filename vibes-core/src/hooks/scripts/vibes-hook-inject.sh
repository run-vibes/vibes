#!/bin/bash
# vibes-hook-inject.sh - Send hook data to Iggy and return response for context injection
#
# This script is called by injection hooks (SessionStart, UserPromptSubmit)
# to forward Claude Code events to the vibes event log (Iggy) and return
# a response with additional context to inject into the conversation.
#
# Usage: vibes-hook-inject.sh <hook-type>
#   Reads JSON data from stdin and wraps it with type information.
#   Sends event to Iggy via CLI and outputs JSON response for Claude.
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

# Send event to Iggy via vibes CLI (fire-and-forget for event logging)
# Use VIBES_BIN if set (for development), otherwise fall back to PATH
VIBES_CMD="${VIBES_BIN:-vibes}"
if [ -x "$VIBES_CMD" ] || command -v "$VIBES_CMD" &>/dev/null; then
    "$VIBES_CMD" event send --type hook --data "$EVENT_JSON" ${SESSION_ID:+--session "$SESSION_ID"} 2>/dev/null || true
fi

# TODO: Future enhancement - query vibes daemon for additionalContext response
# For now, return empty response (no context injection)
echo '{}'

exit 0

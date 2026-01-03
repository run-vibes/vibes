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

# Send event to Iggy via vibes CLI (fire-and-forget for event logging)
if command -v vibes &>/dev/null; then
    vibes event send --type hook --data "$EVENT_JSON" ${VIBES_SESSION_ID:+--session "$VIBES_SESSION_ID"} 2>/dev/null || true
fi

# TODO: Future enhancement - query vibes daemon for additionalContext response
# For now, return empty response (no context injection)
echo '{}'

exit 0

#!/bin/bash
# subagent-stop.sh - Claude Code SubagentStop hook
#
# Called when a subagent (spawned task) completes. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "subagent_id": "agent-42",
#   "reason": "completed"
# }
#
# This hook forwards the event to vibes for logging.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "subagent_stop"

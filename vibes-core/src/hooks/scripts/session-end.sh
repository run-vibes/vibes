#!/bin/bash
# session-end.sh - Claude Code SessionEnd hook
#
# Called when a Claude Code session ends. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "reason": "user_exit"
# }
#
# This hook forwards the event to vibes for logging and cleanup.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "session_end"

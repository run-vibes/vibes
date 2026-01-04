#!/bin/bash
# pre-compact.sh - Claude Code PreCompact hook
#
# Called before context compaction occurs. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id"
# }
#
# This hook forwards the event to vibes for logging.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "pre_compact"

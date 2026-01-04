#!/bin/bash
# notification.sh - Claude Code Notification hook
#
# Called when Claude sends a notification. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "title": "Build Complete",
#   "message": "Your build finished successfully"
# }
#
# This hook forwards the event to vibes for logging.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "notification"

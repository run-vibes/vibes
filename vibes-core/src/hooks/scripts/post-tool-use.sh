#!/bin/bash
# post-tool-use.sh - Claude Code PostToolUse hook
#
# Called after Claude executes a tool. Receives JSON via stdin:
# {
#   "tool_name": "Bash",
#   "output": "...",
#   "success": true,
#   "duration_ms": 150
# }
#
# This hook forwards the event to vibes for monitoring.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "post_tool_use"

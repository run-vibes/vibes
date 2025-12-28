#!/bin/bash
# pre-tool-use.sh - Claude Code PreToolUse hook
#
# Called before Claude executes a tool. Receives JSON via stdin:
# {
#   "tool_name": "Bash",
#   "input": "{\"command\": \"ls -la\"}"
# }
#
# This hook forwards the event to vibes for monitoring.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "pre_tool_use"

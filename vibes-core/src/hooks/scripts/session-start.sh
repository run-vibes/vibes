#!/bin/bash
# session-start.sh - Claude Code SessionStart hook
#
# Called when a new Claude Code session starts. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "project_path": "/path/to/project"
# }
#
# This hook can inject additional context into the session by returning
# a JSON response with an "additionalContext" field.
#
# Returns JSON:
# {
#   "additionalContext": "Markdown content to inject into conversation"
# }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-inject.sh" "session_start"

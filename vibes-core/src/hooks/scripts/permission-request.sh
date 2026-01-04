#!/bin/bash
# permission-request.sh - Claude Code PermissionRequest hook
#
# Called when Claude requests permission for a dangerous operation. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "tool_name": "Bash",
#   "input": "{\"command\": \"rm -rf /\"}"
# }
#
# This hook can block or modify the request by returning a JSON response.
#
# Returns JSON:
# {
#   "decision": "allow" | "deny" | "modify",
#   "reason": "optional explanation"
# }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-inject.sh" "permission_request"

#!/bin/bash
# user-prompt-submit.sh - Claude Code UserPromptSubmit hook
#
# Called when a user submits a prompt. Receives JSON via stdin:
# {
#   "session_id": "optional-session-id",
#   "prompt": "User's prompt text"
# }
#
# This hook can inject additional context into the conversation by returning
# a JSON response with an "additionalContext" field.
#
# Returns JSON:
# {
#   "additionalContext": "Markdown content to inject into conversation"
# }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-inject.sh" "user_prompt_submit"

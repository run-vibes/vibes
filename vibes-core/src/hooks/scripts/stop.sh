#!/bin/bash
# stop.sh - Claude Code Stop hook
#
# Called when Claude session ends. Receives JSON via stdin:
# {
#   "transcript_path": "/path/to/transcript.jsonl",
#   "reason": "user"
# }
#
# This hook forwards the event to vibes for cleanup/archival.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/vibes-hook-send.sh" "stop"

---
id: CHORE0015
title: CLI help text audit
type: chore
status: done
priority: low
scope: cli
depends: []
estimate: 1h
created: 2026-01-08
---

# CLI help text audit

## Summary

Review all CLI help text for consistency, accuracy, and completeness. The CLI has grown organically and some commands may have outdated or inconsistent help descriptions.

## Scope

All commands under `vibes`:
- `vibes start/stop/status`
- `vibes session *`
- `vibes events *`
- `vibes assess *`
- `vibes tunnel *`
- `vibes config *`

## Acceptance Criteria

- [x] All commands have `#[clap(about = "...")]` descriptions
- [x] Descriptions are consistent in style (imperative mood)
- [x] Examples provided for complex commands
- [x] No placeholder or TODO text in help output
- [x] `vibes --help` shows clean, organized output

## Implementation Notes

### Style Guide

- Use imperative mood: "Start the daemon" not "Starts the daemon"
- Keep descriptions under 80 characters
- Add examples for commands with many options
- Group related subcommands logically

### Audit Script

```bash
# Check all help text
for cmd in start stop status session events assess tunnel config; do
  echo "=== vibes $cmd ==="
  vibes $cmd --help
done
```

### Key Files

- `vibes-cli/src/main.rs` - Top-level CLI structure
- `vibes-cli/src/commands/*.rs` - Individual command definitions

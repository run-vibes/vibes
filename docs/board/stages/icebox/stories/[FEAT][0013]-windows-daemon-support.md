---
id: FEAT0013
title: Windows daemon support
type: feat
status: icebox
priority: low
epics: [cli, cross-platform]
depends: []
estimate: 1w
created: 2026-01-08
updated: 2026-01-18
---

# Windows daemon support

## Summary

Add Windows support for the vibes daemon. Currently, the daemon uses Unix-specific features (daemonize crate, Unix signals, PTY). Windows support would broaden the user base.

## Current Blockers

1. **daemonize crate**: Unix-only, need Windows service equivalent
2. **PTY support**: `portable-pty` may work but needs testing
3. **Signal handling**: SIGINT/SIGTERM need Windows equivalents
4. **Socket paths**: Unix sockets not available, need named pipes or TCP

## Acceptance Criteria

- [ ] `vibes start` works on Windows
- [ ] Daemon runs as background process
- [ ] `vibes stop` cleanly terminates daemon
- [ ] PTY sessions function correctly
- [ ] CI includes Windows build verification

## Implementation Notes

### Approach Options

1. **Windows Service**: Native integration, complex to implement
2. **Background Process**: Simpler, use `windows-service` crate
3. **Named Pipes**: Replace Unix sockets for IPC

### Key Files to Modify

- `vibes-server/src/daemon.rs` - Platform-specific daemon logic
- `vibes-cli/src/commands/start.rs` - Windows startup logic
- `build.rs` - Feature flags for Windows

### Dependencies

- Consider `windows-service` crate
- Test `portable-pty` on Windows
- May need `windows-sys` for native APIs

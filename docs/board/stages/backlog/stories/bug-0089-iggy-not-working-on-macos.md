---
id: BUG0089
title: Iggy doesn't return events on macOS
type: bug
status: pending
priority: high
epics: [cross-platform]
depends: []
estimate: 4h
created: 2026-01-12
---

# Iggy doesn't return events on macOS

## Summary

Iggy event streaming doesn't work on macOS ARM64 - no events are returned. The daemon appears to start but events are never delivered.

## Symptoms

- Iggy server starts without errors
- Events are sent but never received
- No errors in logs
- Works correctly on Linux

## Compilation Warnings (Fixed)

The following warnings appeared when compiling on macOS:

```
warning: constant `MIN_MEMLOCK_BYTES` is never used
warning: function `format_bytes` is never used
warning: constant `MEMLOCK_HELP` is never used
```

**Fixed:** Added `#[cfg(target_os = "linux")]` to Linux-specific code in `vibes-iggy/src/preflight.rs`.

## Investigation Findings

### Ruled Out: thread_pool_limit Issue

The upstream Iggy code in `bootstrap.rs` has a known issue where `thread_pool_limit(0)` causes hangs on macOS due to compio needing a thread pool for filesystem operations. However, **this is already conditionally skipped for macOS ARM64**:

```rust
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
proactor.thread_pool_limit(0);
```

Since the user is on ARM64 macOS, this is NOT the cause.

### Potential Areas to Investigate

1. **File flush visibility**: The `flush_to_disk()` method is documented as critical for io_uring consistency. Test if calling this before polling helps.

2. **Compio kqueue backend**: The server uses compio for networking and file I/O. On macOS, compio uses kqueue instead of io_uring. There may be differences in event notification timing.

3. **SDK vs Server runtime**: The Iggy SDK uses tokio (standard async), while the server uses compio. Cross-runtime communication might have edge cases.

4. **Timing/race conditions**: Integration tests include sleep delays before polling. macOS might need longer delays or explicit flushes.

## Key Finding: Integration Tests Pass

The `vibes-iggy` integration tests pass on macOS ARM64:
```bash
cargo nextest run -p vibes-iggy --test integration
```

This means basic Iggy event flow (append, poll, seek) works correctly. **The issue is specific to how the daemon uses Iggy**, not Iggy itself.

## Debugging Steps

1. Compare daemon Iggy usage vs integration test usage
2. Check daemon's consumer setup (seek position, consumer group)
3. Enable trace logging: `RUST_LOG=vibes_iggy=trace,iggy=trace`
4. Check if `flush_to_disk()` is being called before polling in daemon
5. Verify the daemon's event consumer is seeking to the correct position

## Next Steps

1. Identify where daemon's Iggy usage differs from integration tests
2. Add detailed tracing to the daemon's event consumption path
3. Check if the issue is in consumer initialization or polling

## Acceptance Criteria

- [x] No unused code warnings on macOS (fixed in preflight.rs)
- [ ] Root cause identified
- [ ] Events flow correctly on macOS
- [ ] Linux functionality unchanged
- [ ] Tests pass on both platforms

---
id: CHORE0016
title: proper shutdown signal coordination for plugin manager
type: chore
status: done
priority: low
epics: [core]
depends: []
estimate: S
created: 2026-01-08
updated: 2026-01-08
---

# Proper shutdown signal coordination for plugin manager

## Summary

Replace the `std::future::pending::<()>().await` placeholder in `vibes-server/src/lib.rs:294`
with proper shutdown signal coordination. Currently the spawned task that keeps the plugin
manager alive never terminates - it should listen for the shutdown signal and gracefully
stop the plugin event consumers.

## Context

In `VibesServer::start_plugin_consumer()`, a task is spawned to keep the `ConsumerManager`
alive. The current implementation uses `pending()` which blocks forever:

```rust
tokio::spawn(async move {
    let _manager = manager; // Keep manager alive
    let _shutdown = shutdown; // Keep shutdown token alive
    std::future::pending::<()>().await;
});
```

This works but doesn't participate in graceful shutdown - when the server stops, plugin
consumers aren't notified and may have in-flight work.

## Acceptance Criteria

- [ ] Spawned task awaits the shutdown signal instead of pending()
- [ ] ConsumerManager is dropped gracefully when shutdown signal received
- [ ] Plugin event consumers complete in-flight work before stopping
- [ ] Shutdown logs indicate plugin consumer cleanup
- [ ] No behavior change during normal operation

## Implementation Notes

1. The `shutdown` token is already passed in - use `shutdown.cancelled()` instead of `pending()`
2. Consider adding a `ConsumerManager::shutdown()` method if cleanup is needed
3. May need timeout to avoid hanging on stuck consumers

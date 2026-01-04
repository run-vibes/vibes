---
created: 2026-01-03
status: pending
---

# Refactor: Add Plugin `on_ready()` Lifecycle Method

> **For Claude:** Use superpowers:brainstorming if design needs refinement.

## Problem

The assessment consumer is started in `vibes-server/src/consumers/assessment.rs`, but it uses types from `vibes-groove`. This means:

1. `vibes-server` depends on `vibes-groove` internals
2. The groove plugin isn't self-contained
3. Plugin enable/disable requires modifying server code

## Solution

Add a generic `on_ready()` lifecycle method to the `Plugin` trait. This method is called after the server is fully initialized, with runtime dependencies injected.

```rust
trait Plugin {
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError>;

    /// Called after server is fully initialized with runtime dependencies.
    /// Default implementation does nothing.
    fn on_ready(&mut self, runtime: &mut PluginRuntime) -> Result<(), PluginError> {
        Ok(())
    }
}

struct PluginRuntime {
    /// Event log for consuming/producing events
    pub event_log: Arc<dyn EventLog<StoredEvent>>,
    /// Shutdown signal
    pub shutdown: CancellationToken,
    /// Iggy manager (if available) for persistent storage
    pub iggy_manager: Option<Arc<IggyManager>>,
}
```

## Tasks

- [ ] Add `PluginRuntime` struct to `vibes-plugin-api`
- [ ] Add `on_ready()` method to `Plugin` trait with default no-op
- [ ] Update `vibes-server` to call `on_ready()` after initialization
- [ ] Move assessment consumer startup from `vibes-server` to `GroovePlugin.on_ready()`
- [ ] Remove `vibes-groove` dependency from `vibes-server` consumer code
- [ ] Update tests

## Acceptance Criteria

- [ ] `vibes-server` has no knowledge of groove internals
- [ ] Groove plugin starts its own assessment consumer in `on_ready()`
- [ ] Existing functionality unchanged (assessment events still flow)
- [ ] `just pre-commit` passes

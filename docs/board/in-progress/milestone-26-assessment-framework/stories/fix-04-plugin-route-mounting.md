---
created: 2026-01-03
status: done
---

# Fix: Plugin Routes Return HTML Instead of JSON

> **For Claude:** Use superpowers:systematic-debugging if the fix isn't straightforward.

## Problem

API routes like `http://localhost:44991/api/groove/policy` return HTML (the SPA index.html) instead of JSON responses from the groove plugin.

## Root Cause

In `vibes-server/src/http/mod.rs`, the `plugin_router()` defined in `plugins.rs` is never mounted. The main router goes straight to the static file fallback:

```rust
// Current (broken)
Router::new()
    .route("/api/health", get(api::health))
    // ... other routes ...
    .fallback(static_files::static_handler)  // Catches /api/groove/* too!
```

The `plugin_router()` at `plugins.rs:111-113` handles `/api/{plugin}/*` routes but is never nested into the main router.

## Fix

Mount the plugin router before the static fallback:

```rust
Router::new()
    .route("/api/health", get(api::health))
    // ... other explicit routes ...
    .nest("/", plugins::plugin_router())  // Plugin routes checked before fallback
    .fallback(static_files::static_handler)
```

Or use a more explicit nest path if preferred.

## Tasks

- [ ] Mount `plugin_router()` in `create_router()`
- [ ] Verify `/api/groove/policy` returns JSON
- [ ] Add test for plugin route handling

## Acceptance Criteria

- `curl http://localhost:PORT/api/groove/policy` returns JSON, not HTML
- Existing static file serving still works for the SPA
- Plugin routes with path parameters work (e.g., `/api/groove/quarantine/:id`)

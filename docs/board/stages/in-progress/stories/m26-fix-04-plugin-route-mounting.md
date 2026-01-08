---
id: B003
title: Fix: Plugin Routes Return HTML Instead of JSON
type: fix
status: done
priority: medium
epics: [core, cli, plugin-system]
depends: []
estimate:
created: 2026-01-03
updated: 2026-01-07
milestone: milestone-26-assessment-framework
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

- [x] Mount `plugin_router()` in `create_router()`
- [x] Add test for plugin route handling (`test_plugin_routes_return_json_not_html`)
- [x] Add test for groove routes when loaded (`test_groove_routes_work_when_plugin_loaded`)
- [x] Load plugins on server startup via `load_plugins()` method

## Acceptance Criteria

- `curl http://localhost:PORT/api/groove/policy` returns JSON, not HTML
- Existing static file serving still works for the SPA
- Plugin routes with path parameters work (e.g., `/api/groove/quarantine/:id`)

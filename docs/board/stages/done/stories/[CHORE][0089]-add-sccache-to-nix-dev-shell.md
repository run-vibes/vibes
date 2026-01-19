---
id: CHORE0089
title: Add sccache to Nix dev shell
type: chore
status: done
priority: medium
scope: dev-environment
depends: []
estimate: 2h
created: 2026-01-13
---

# Add sccache to Nix dev shell

## Summary

Add sccache to the Nix flake development shell to cache Rust compilation artifacts across builds. This reduces rebuild times by caching intermediate compilation results.

## Context

sccache is Mozilla's shared compilation cache that works with Rust. It caches compiled artifacts and can significantly speed up incremental builds, especially when switching branches or doing clean builds. It integrates transparently with cargo via the `RUSTC_WRAPPER` environment variable.

## Tasks

### Task 1: Add sccache to Nix flake

**Steps:**
1. Add `sccache` to the `devShells.default` packages in `flake.nix`
2. Set `RUSTC_WRAPPER = "sccache"` in the shell hook or environment
3. Optionally configure sccache cache directory via `SCCACHE_DIR`

### Task 2: Verify integration

**Steps:**
1. Run `direnv reload` to pick up changes
2. Build the project with `just build`
3. Verify sccache is being used with `sccache --show-stats`
4. Run a clean build and check cache hits on second build

## Acceptance Criteria

- [ ] sccache is available in the Nix dev shell
- [ ] Cargo uses sccache as the rustc wrapper
- [ ] `sccache --show-stats` shows cache activity after builds
- [ ] Second build of the same code shows cache hits

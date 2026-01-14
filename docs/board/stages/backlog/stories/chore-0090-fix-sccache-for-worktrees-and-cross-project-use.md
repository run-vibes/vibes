---
id: CHORE0090
title: fix sccache for worktrees and cross-project use
type: chore
status: backlog
priority: high
epics: [dev-environment]
depends: []
estimate: 2h
created: 2026-01-13
updated: 2026-01-13
---

# fix sccache for worktrees and cross-project use

## Summary

sccache is configured but not working effectively. Current stats show **0% Rust cache hit rate** and the cache size is stuck at 10 GiB instead of the configured 24G. CozoDB recompiles from source regularly, wasting significant build time.

## Problem Analysis

### Current State

From `sccache --show-stats`:
- Cache hits rate (Rust): **0.00%**
- Max cache size: **10 GiB** (should be 24G)
- Non-cacheable compilations: 82 due to `incremental` mode

### Root Causes

1. **Incremental compilation conflict**: Cargo's default incremental compilation mode is incompatible with sccache. Artifacts compiled incrementally cannot be cached.

2. **Cache size not persisted**: `SCCACHE_CACHE_SIZE=24G` in `flake.nix` only takes effect if sccache server starts *after* entering the nix shell. If the server was started earlier (e.g., from another project), it uses the 10G default.

3. **No system-wide config**: Without `~/.config/sccache/config`, sccache settings reset on each server restart.

4. **Cross-project interference**: Both vibes and rialo set `RUSTC_WRAPPER=sccache` but with different (or no) cache size settings. The first project to start the sccache server determines the cache size.

## Tasks

### Task 1: Disable incremental compilation when using sccache

Add `CARGO_INCREMENTAL=0` to the nix shell environment. This trades off incremental build speed for cross-session caching—net positive for worktrees where incremental caches don't persist anyway.

**Files:** `flake.nix`

### Task 2: Create system-wide sccache config

Create `~/.config/sccache/config` with persistent settings:

```toml
[cache.disk]
dir = "/home/alex/.cache/sccache"
size = 25769803776  # 24 GiB in bytes
```

This ensures consistent behavior regardless of which project starts sccache first.

**Note:** This is a user-level change, not a repo change. Document it in the README or setup guide.

### Task 3: Add sccache server restart to shell hook

When entering the nix shell, stop any existing sccache server so it restarts with correct settings:

```bash
sccache --stop-server 2>/dev/null || true
```

**Files:** `flake.nix` (shellHook)

### Task 4: Verify cache hits after fix

After implementing:
1. Clear cache: `sccache --zero-stats`
2. Build: `just build`
3. Clean and rebuild: `cargo clean && just build`
4. Check stats: `sccache --show-stats`

Expected: Rust cache hit rate > 50% on rebuild

## Acceptance Criteria

- [ ] `CARGO_INCREMENTAL=0` set in nix shell
- [ ] `sccache --show-stats` shows Max cache size: 24 GiB
- [ ] Rust cache hit rate > 0% after clean rebuild
- [ ] CozoDB cache hits on subsequent builds
- [ ] Works consistently across git worktrees

## Implementation Notes

### Trade-offs

Disabling incremental compilation means first builds in a session are slower, but:
- Cross-session/worktree builds are faster (cache hits)
- CozoDB and other large deps compile once and cache
- CI-like behavior in dev—catches issues faster

### Alternative: Selective incremental

Could use incremental for workspace crates and sccache for deps:
```toml
# Cargo.toml
[profile.dev.package."*"]
incremental = false
```

This is more complex but preserves incremental for local code. Consider for future if needed.

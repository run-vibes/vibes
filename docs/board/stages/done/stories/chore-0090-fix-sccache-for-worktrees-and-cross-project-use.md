---
id: CHORE0090
title: fix sccache for worktrees and cross-project use
type: chore
status: done
priority: high
epics: [dev-environment]
depends: []
estimate: 2h
created: 2026-01-13
updated: 2026-01-13
---

# fix sccache for worktrees and cross-project use

## Summary

sccache was configured but not working effectively. Stats showed **0% Rust cache hit rate** and cache size stuck at 10 GiB instead of configured 24G. CozoDB was recompiling from source regularly.

## Problem Analysis

### Root Causes

1. **Incremental compilation conflict**: Cargo's default incremental mode is incompatible with sccache—artifacts can't be cached.

2. **Cache size not persisted**: No `~/.config/sccache/config` file meant settings reset on server restart.

3. **Stale TMPDIR**: Nix-shell sets ephemeral TMPDIR that disappears after shell exits, breaking sccache server.

## Implementation

### Changes Made

1. **`flake.nix`**: Added `CARGO_INCREMENTAL = "0"` to disable incremental compilation

2. **`.envrc`**: Added `export TMPDIR=/var/tmp` before `use flake` to ensure sccache uses persistent temp storage

3. **`~/.config/sccache/config`** (user-level): Created with 24 GiB cache size:
   ```toml
   [cache.disk]
   dir = "/home/alex/.cache/sccache"
   size = 25769803776
   ```

### What We Didn't Do

- **Server restart in shellHook**: Initially planned but removed—it disrupts other shells running builds.

## Results

After fix, `sccache --show-stats` shows:
- **Rust cache hit rate: 57.43%** (was 0%)
- **C/C++ cache hit rate: 85.40%** (CozoDB/RocksDB)
- **Max cache size: 24 GiB** (was 10 GiB)
- **Clean rebuild time: ~16s** (was minutes)

## Acceptance Criteria

- [x] `CARGO_INCREMENTAL=0` set in nix shell
- [x] `sccache --show-stats` shows Max cache size: 24 GiB
- [x] Rust cache hit rate > 0% after clean rebuild
- [x] CozoDB cache hits on subsequent builds
- [x] Works consistently across git worktrees

## Notes

### Trade-offs

Disabling incremental compilation means first builds in a session compile more, but:
- Cross-session/worktree builds are much faster (cache hits)
- CozoDB and large deps compile once and cache
- CI-like behavior catches issues earlier

### For Other Machines

To replicate on a new machine, create `~/.config/sccache/config`:
```toml
[cache.disk]
dir = "/home/<user>/.cache/sccache"
size = 25769803776
```

---
id: FEAT0108
title: Improve Git Worktree DX for Rust
type: feat
status: done
priority: medium
epics: []
depends: []
estimate: S
created: 2026-01-14
updated: 2026-01-14
---

# Improve Git Worktree DX for Rust

## Summary

Git worktrees in vibes have poor developer experience: each worktree requires manual submodule initialization and triggers full Rust recompilation. This story adds a git hook for automatic setup and shared target directory for build artifact reuse across worktrees.

## Problem

| Issue | Impact |
|-------|--------|
| Submodules not auto-initialized | Builds fail until manual `git submodule update` |
| Separate target dir per worktree | Full recompile (~5-10 min) on fresh worktree |
| iggy-server rebuilt per worktree | Slowest part of build, repeated unnecessarily |

## Solution (Implemented)

### 1. Git Post-Checkout Hook

Used git's native `post-checkout` hook (`.githooks/post-checkout`) instead of a separate script. This runs automatically after `git worktree add` and branch checkouts:

```bash
#!/usr/bin/env bash
set -e

# Only run on branch checkouts, not file checkouts
if [[ "$3" != "1" ]]; then exit 0; fi

# Initialize submodules if needed (idempotent)
if [[ ! -f "vendor/iggy/Cargo.toml" ]]; then
    git submodule update --init --recursive
fi

# Allow direnv if available
if [[ -f ".envrc" ]] && command -v direnv &> /dev/null; then
    direnv allow
fi
```

### 2. Shared Target Directory

Added to `flake.nix` shellHook:

```bash
export CARGO_TARGET_DIR="$HOME/.cargo-target/vibes"
```

All worktrees write to `~/.cargo-target/vibes/`:
- Compiled deps shared across worktrees
- iggy-server built once, reused everywhere
- Incremental compilation state preserved

### 3. Binary Lookup Fix

Updated `vibes-iggy/src/config.rs` to check `CARGO_TARGET_DIR` when finding iggy-server binary. Without this, tests would fail because they couldn't locate the binary.

### 4. Build Script Cleanup

Removed iggy-server binary copy from `.justfiles/build.just` since both binaries now go to the same shared directory.

### 5. Documentation

Updated CLAUDE.md with:
- Shared build cache section
- Warning about `cargo clean` affecting all worktrees

## Expected Impact

| Metric | Before | After |
|--------|--------|-------|
| Fresh worktree first build | ~5-10 min | ~30 sec (incremental) |
| Submodule init | Manual | Automatic |
| iggy-server rebuild | Every worktree | Once, shared |

## Acceptance Criteria

- [x] Git hook initializes submodules automatically on worktree creation
- [x] `flake.nix` sets `CARGO_TARGET_DIR` to shared location
- [x] Fresh worktree build uses cached artifacts from main
- [x] iggy-server not rebuilt when already present in shared target
- [x] CLAUDE.md documents `cargo clean` footgun

## Files Changed

- `flake.nix` — Added `CARGO_TARGET_DIR` environment variable
- `.githooks/post-checkout` — New git hook for worktree setup
- `.justfiles/build.just` — Removed binary copy, added location echo
- `vibes-iggy/src/config.rs` — Check `CARGO_TARGET_DIR` for binary lookup
- `CLAUDE.md` — Documented shared target behavior and warnings

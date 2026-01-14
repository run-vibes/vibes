---
id: FEAT0108
title: Improve Git Worktree DX for Rust
type: feat
status: backlog
priority: medium
epics: []
depends: []
estimate: S
created: 2026-01-14
updated: 2026-01-14
---

# Improve Git Worktree DX for Rust

## Summary

Git worktrees in vibes have poor developer experience: each worktree requires manual submodule initialization and triggers full Rust recompilation. This story adds a project hook for automatic setup and shared target directory for build artifact reuse across worktrees.

## Problem

| Issue | Impact |
|-------|--------|
| Submodules not auto-initialized | Builds fail until manual `git submodule update` |
| Separate target dir per worktree | Full recompile (~5-10 min) on fresh worktree |
| iggy-server rebuilt per worktree | Slowest part of build, repeated unnecessarily |

## Solution

### 1. Project Hook (`.worktree-setup.sh`)

Executable script in repo root, runs after worktree creation:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Setting up vibes worktree..."

# Initialize submodules (iggy-server source)
git submodule update --init --recursive

# Allow direnv to load nix environment
if command -v direnv &> /dev/null; then
    direnv allow
fi

# Show sccache stats for visibility
if command -v sccache &> /dev/null; then
    echo "sccache stats:"
    sccache --show-stats | grep -E "(Compile requests|Cache hits|Cache misses)"
fi

echo "Worktree setup complete. Run 'just build' to compile."
```

### 2. Shared Target Directory

Change `flake.nix` to set shared target location:

```nix
CARGO_TARGET_DIR = "${builtins.getEnv "HOME"}/.cargo-target/vibes";
```

All worktrees write to `~/.cargo-target/vibes/`:
- Compiled deps shared across worktrees
- iggy-server built once, reused everywhere
- Incremental compilation state preserved

### 3. Justfile Updates

Remove iggy-server binary copy from `.justfiles/build.just` since both vibes and iggy-server now use the same target directory.

### 4. Documentation

Add warning to CLAUDE.md about `cargo clean` affecting all worktrees.

## Expected Impact

| Metric | Before | After |
|--------|--------|-------|
| Fresh worktree first build | ~5-10 min | ~30 sec (incremental) |
| Submodule init | Manual | Automatic |
| iggy-server rebuild | Every worktree | Once, shared |

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Concurrent builds | Cargo file locking handles this |
| Different commits | Shared deps cached; changed crates rebuild |
| `cargo clean` | Cleans shared dir for ALL worktrees |
| Different toolchains | Artifacts are toolchain-stamped; no conflict |

## Acceptance Criteria

- [ ] `.worktree-setup.sh` exists and initializes submodules
- [ ] `flake.nix` sets `CARGO_TARGET_DIR` to shared location
- [ ] Fresh worktree build uses cached artifacts from main
- [ ] iggy-server not rebuilt when already present in shared target
- [ ] CLAUDE.md documents `cargo clean` footgun

## Files to Change

- `flake.nix` — Add `CARGO_TARGET_DIR` environment variable
- `.worktree-setup.sh` — New file (project hook)
- `.justfiles/build.just` — Remove binary copy step
- `CLAUDE.md` — Document shared target behavior

## Implementation Notes

The `git-worktree` skill from compound-engineering detects `.worktree-setup.sh` and runs it automatically after worktree creation. No changes needed to the skill itself.

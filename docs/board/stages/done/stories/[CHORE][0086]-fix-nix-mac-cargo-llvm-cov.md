---
id: CHORE0086
title: Fix Nix dev shell on Mac
type: chore
status: done
priority: medium
scope: dev-environment
depends: []
estimate: 2h
created: 2026-01-12
---

# Fix Nix dev shell on Mac

## Summary

Fix the Nix dev shell on macOS where `cargo-llvm-cov` is marked as broken.

## Context

The Nix flake includes `cargo-llvm-cov` for code coverage, but it's currently marked as broken on macOS. This prevents Mac users from using the full dev shell.

## Options

1. **Use dev Cargo.toml dependency** - Add `cargo-llvm-cov` as a dev dependency installed via cargo instead of Nix
2. **Conditional Nix package** - Only include `cargo-llvm-cov` on Linux in the flake
3. **Fix upstream** - Check if there's an updated nixpkgs with a working version

## Tasks

### Task 1: Implement fix

**Steps:**
1. Test current behavior on Mac
2. Implement chosen solution (likely option 1 or 2)
3. Verify coverage still works on Linux
4. Verify dev shell works on Mac
5. Update documentation if needed
6. Commit: `chore(nix): fix dev shell on macOS`

## Acceptance Criteria

- [ ] `direnv allow` works on macOS without errors
- [ ] `just coverage` still works on Linux
- [ ] Coverage tooling available on Mac (via cargo install if needed)

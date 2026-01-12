---
id: CHORE0087
title: Use Cranelift for faster builds
type: chore
status: pending
priority: medium
epics: [dev-environment]
depends: []
estimate: 1h
created: 2026-01-12
---

# Use Cranelift for faster builds

## Summary

Configure Cranelift as the codegen backend for debug builds to improve compilation speed.

## Context

Cranelift is an alternative codegen backend for rustc that produces code faster than LLVM, though with less optimization. For debug builds, this tradeoff is beneficial since we prioritize iteration speed over runtime performance.

## Tasks

### Task 1: Configure Cranelift

**Steps:**
1. Add `.cargo/config.toml` with Cranelift backend for dev profile:
   ```toml
   [profile.dev]
   codegen-backend = "cranelift"
   ```
2. Ensure `rustup component add rustc-codegen-cranelift-preview` is in setup
3. Update Nix flake to include Cranelift component
4. Benchmark build times before/after
5. Verify all tests still pass
6. Update CLAUDE.md with any relevant notes
7. Commit: `chore(build): use Cranelift for faster debug builds`

## Acceptance Criteria

- [ ] Debug builds use Cranelift backend
- [ ] Measurable improvement in build times
- [ ] All tests pass
- [ ] Release builds still use LLVM

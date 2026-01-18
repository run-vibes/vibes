---
id: CHORE0087
title: Use Cranelift for faster builds
type: chore
status: icebox
priority: low
epics: [dev-environment]
depends: []
estimate: 4h
created: 2026-01-12
---

# Use Cranelift for faster builds

## Summary

Configure Cranelift as the codegen backend for debug builds to improve compilation speed.

## Context

Cranelift is an alternative codegen backend for rustc that produces code faster than LLVM, though with less optimization. For debug builds, this tradeoff is beneficial since we prioritize iteration speed over runtime performance.

## Blockers

**Requires nightly Rust:** Cranelift codegen backend needs `-Z codegen-backend` flag which is only available on nightly.

**C++ compilation issues:** When switching to nightly, RocksDB (via CozoDB) fails to compile with the nightly toolchain. The cc-rs crate's g++ invocation fails. Needs investigation:
- May need to pin a specific nightly version that works
- May need to set CC/CXX environment variables to use clang
- May be a compatibility issue between nightly Rust and the C++ dependencies

## Tasks

### Task 1: Resolve Nightly Compatibility

**Steps:**
1. Investigate RocksDB/CozoDB compilation failure with nightly
2. Test with pinned nightly versions to find a working one
3. Consider setting CC=clang CXX=clang++ to avoid g++ issues

### Task 2: Configure Cranelift

**Steps:**
1. Update Nix flake to use nightly with `rustc-codegen-cranelift` extension:
   ```nix
   rust = pkgs.rust-bin.nightly.latest.default.override {
     extensions = [ "rust-src" "rust-analyzer" "llvm-tools" "rustc-codegen-cranelift" ];
   };
   ```
2. Add `.cargo/config.toml`:
   ```toml
   [unstable]
   codegen-backend = true

   [profile.dev]
   codegen-backend = "cranelift"

   [profile.dev.package."*"]
   codegen-backend = "llvm"  # Use LLVM for dependencies
   ```
3. Benchmark build times before/after
4. Verify all tests still pass
5. Commit: `chore(build): use Cranelift for faster debug builds`

## Acceptance Criteria

- [ ] Nightly toolchain builds CozoDB/RocksDB successfully
- [ ] Debug builds use Cranelift backend
- [ ] Measurable improvement in build times
- [ ] All tests pass
- [ ] Release builds still use LLVM

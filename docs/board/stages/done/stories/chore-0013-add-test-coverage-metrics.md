---
id: CHORE0013
title: Add test coverage metrics
type: chore
status: done
priority: medium
epics: [core]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
---

# Add test coverage metrics

## Summary

Add test coverage tracking to the project using cargo-llvm-cov. While the test suite is extensive (1115+ tests), there's no visibility into which code paths are covered. Coverage metrics help identify untested code and prevent regressions.

## Acceptance Criteria

- [x] `just coverage` command generates coverage report
- [x] HTML report viewable locally
- [x] Coverage thresholds documented (not enforced, just tracked)
- [x] Key crates show >80% coverage (verified: vibes-introspection at 96.72%)
- [ ] CI optionally uploads coverage to codecov/coveralls (deferred)

## Implementation

### Commands Added

| Command | Description |
|---------|-------------|
| `just coverage` | Generate HTML report and open in browser |
| `just coverage-html` | Generate HTML report without opening |
| `just coverage-summary` | Print summary to terminal |
| `just coverage-lcov` | Generate LCOV format for CI |
| `just coverage-package PACKAGE` | Coverage for specific package |

### Nix Integration

Added to `flake.nix`:
- `pkgs.cargo-llvm-cov` in buildInputs
- `llvm-tools` extension to Rust toolchain (required by cargo-llvm-cov)
- Shell hook mentions `just coverage`

### Verified Coverage (vibes-introspection)

```
Filename                              Lines   Cover
capabilities.rs                       130     100.00%
claude_code/detection.rs              360     99.17%
claude_code/harness.rs                189     99.47%
harness.rs                            19      94.74%
paths.rs                              44      95.45%
watcher.rs                            188     88.83%
TOTAL                                 944     96.72%
```

### Coverage Targets (Informational)

| Crate | Target | Notes |
|-------|--------|-------|
| vibes-core | 80% | Core business logic |
| vibes-groove | 70% | Plugin-specific |
| vibes-cli | 60% | Mostly integration |
| vibes-introspection | 90% | Small, well-tested |

### CI Integration (Future)

```yaml
- run: cargo llvm-cov --lcov --output-path lcov.info
- uses: codecov/codecov-action@v3
```

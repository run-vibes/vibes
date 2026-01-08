---
id: CHORE0013
title: Add test coverage metrics
type: chore
status: backlog
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

- [ ] `just coverage` command generates coverage report
- [ ] HTML report viewable locally
- [ ] Coverage thresholds documented (not enforced, just tracked)
- [ ] Key crates show >80% coverage
- [ ] CI optionally uploads coverage to codecov/coveralls

## Implementation Notes

### Setup

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Add to justfile
coverage:
    cargo llvm-cov --html --open
```

### Nix Integration

Add to `flake.nix` dev shell:
```nix
packages = [ pkgs.cargo-llvm-cov ];
```

### Coverage Targets

| Crate | Target | Notes |
|-------|--------|-------|
| vibes-core | 80% | Core business logic |
| vibes-groove | 70% | Plugin-specific |
| vibes-cli | 60% | Mostly integration |

### CI Integration (Optional)

```yaml
- run: cargo llvm-cov --lcov --output-path lcov.info
- uses: codecov/codecov-action@v3
```

---
id: CHORE0088
title: Add Codecov coverage reporting to CI
type: chore
status: done
priority: medium
scope: dev-environment
depends: []
estimate: 1h
created: 2026-01-12
---

# Add Codecov coverage reporting to CI

## Summary

Integrate Codecov with GitHub Actions to track test coverage over time and display a coverage badge in the README.

## Context

We now have working coverage reports via `just coverage`. This story adds CI integration to:
- Automatically generate coverage on every PR
- Upload to Codecov for tracking trends
- Display a coverage badge in README.md
- Optionally fail PRs that decrease coverage

## Tasks

### Task 1: Add Coverage Job to CI

**Steps:**
1. Add new `coverage` job to `.github/workflows/ci.yml`:
   ```yaml
   coverage:
     runs-on: ubuntu-latest
     needs: check
     steps:
       - uses: actions/checkout@v4
         with:
           submodules: recursive

       - uses: DeterminateSystems/nix-installer-action@main
       - uses: DeterminateSystems/magic-nix-cache-action@main

       - name: Cache cargo
         uses: Swatinem/rust-cache@v2
         with:
           cache-all-crates: true
           shared-key: "rust-cache"
           workspaces: |
             . -> target
             vendor/iggy -> target

       - name: Generate coverage
         run: nix develop --command just coverage lcov

       - name: Upload to Codecov
         uses: codecov/codecov-action@v4
         with:
           files: target/lcov.info
           fail_ci_if_error: false
   ```
2. Commit: `chore(ci): add coverage job to CI workflow`

### Task 2: Setup Codecov

**Steps:**
1. Sign up at codecov.io with GitHub account
2. Enable the vibes repository
3. Note: For public repos, no token is needed
4. For private repos, add `CODECOV_TOKEN` secret to GitHub repo settings

### Task 3: Add Coverage Badge to README

**Steps:**
1. Add coverage badge to README.md after the title:
   ```markdown
   [![codecov](https://codecov.io/gh/run-vibes/vibes/graph/badge.svg)](https://codecov.io/gh/run-vibes/vibes)
   ```
2. Commit: `docs: add Codecov coverage badge to README`

### Task 4: Configure Codecov (Optional)

**Steps:**
1. Create `codecov.yml` in repo root:
   ```yaml
   coverage:
     status:
       project:
         default:
           # Don't fail PRs for coverage changes
           informational: true
       patch:
         default:
           # Require new code to have some coverage
           target: 50%

   ignore:
     - "vendor/**"
     - "web-ui/**"
   ```
2. Commit: `chore: add Codecov configuration`

## Acceptance Criteria

- [ ] Coverage job runs on every PR
- [ ] Coverage reports uploaded to Codecov
- [ ] Coverage badge displays in README.md
- [ ] Badge shows current coverage percentage

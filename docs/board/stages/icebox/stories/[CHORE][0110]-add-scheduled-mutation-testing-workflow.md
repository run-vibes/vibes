---
id: CHORE0110
title: Add scheduled mutation testing workflow
type: chore
status: icebox
scope:
priority: medium
depends: []
estimate:
created: 2026-01-17
---

# Add scheduled mutation testing workflow

## Summary

Create a GitHub Actions workflow that runs mutation testing (`cargo mutants`) on a daily schedule and uploads the results as an artifact.

Mutation testing is expensive and shouldn't run on every commit, but running it periodically helps identify weak spots in test coverage.

## Acceptance Criteria

- [ ] New workflow file `.github/workflows/mutants.yml`
- [ ] Runs daily on a schedule (e.g., 6am UTC)
- [ ] Supports manual trigger via `workflow_dispatch`
- [ ] Generates mutation testing report
- [ ] Uploads report as GitHub artifact
- [ ] Uses Nix environment for consistency with other CI jobs

## Implementation Notes

- Use `cargo mutants` with appropriate timeout flags to prevent runaway tests
- Consider scoping to specific packages if full run is too slow
- Reuse cargo cache setup from existing CI workflow
- Artifact retention ~7 days to avoid storage bloat

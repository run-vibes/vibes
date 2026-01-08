---
id: DOCS0001
title: Fix broken documentation links
type: docs
status: backlog
priority: medium
epics: [core]
depends: []
estimate: 30m
created: 2026-01-08
updated: 2026-01-08
---

# Fix broken documentation links

## Summary

Audit and fix broken internal documentation links discovered during project review. Several docs reference non-existent paths like `docs/plans/` or have incorrect relative paths.

## Known Issues

1. **docs/PRD.md**: References `docs/plans/` which doesn't exist (should be `docs/board/`)
2. **docs/PLUGINS.md**: Links to `groove/BRANDING.md` instead of `docs/groove/BRANDING.md`
3. **Various milestone docs**: May reference paths using old directory structure

## Acceptance Criteria

- [ ] All internal documentation links resolve correctly
- [ ] `docs/plans/` references updated to `docs/board/`
- [ ] Relative path references validated
- [ ] CI check added to prevent future broken links (optional)

## Implementation Notes

1. Search for `docs/plans/` pattern across all files
2. Search for relative links in markdown files
3. Validate each link manually
4. Update incorrect paths
5. Consider adding a link checker script or CI step

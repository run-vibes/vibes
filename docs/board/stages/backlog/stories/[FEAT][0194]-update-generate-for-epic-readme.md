---
id: FEAT0194
title: Update generate for epic README
type: feat
status: backlog
priority: medium
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191]
estimate: 3h
created: 2026-01-18
---

# Update Generate for Epic README

## Summary

Update `just board generate` to regenerate epic README files with current milestone progress and status.

## Acceptance Criteria

- [ ] Epic README milestone table shows current status from milestone frontmatter
- [ ] Epic README shows progress (stories complete / total)
- [ ] Epic README shows active milestone name
- [ ] Epic README shows stories in progress count
- [ ] Generation is idempotent (running twice produces same result)

## Implementation Notes

- Modify `generate` in `.justfiles/board.just`
- Parse milestone README frontmatter for status
- Count stories by scanning milestone scope in stages/
- Update epic README in place (preserve non-generated sections)

## Requirements

- SRS-08: `just board generate` updates epic README with milestone progress
- SRS-NFR-02: README progress updates are idempotent

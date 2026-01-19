---
id: FEAT0195
title: Update generate for milestone README
type: feat
status: backlog
priority: medium
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191]
estimate: 3h
created: 2026-01-18
---

# Update Generate for Milestone README

## Summary

Update `just board generate` to regenerate milestone README files with current story progress and status.

## Acceptance Criteria

- [ ] Milestone README story table shows current status from story frontmatter
- [ ] Milestone README shows progress (requirements verified / total)
- [ ] Milestone README shows stories complete count
- [ ] Generation is idempotent (running twice produces same result)
- [ ] Stories are discovered by scope field matching milestone

## Implementation Notes

- Modify `generate` in `.justfiles/board.just`
- Parse story frontmatter for status and scope
- Match stories to milestones via `scope: epic/milestone` format
- Update milestone README in place (preserve non-generated sections)

## Requirements

- SRS-09: `just board generate` updates milestone README with story progress
- SRS-NFR-02: README progress updates are idempotent

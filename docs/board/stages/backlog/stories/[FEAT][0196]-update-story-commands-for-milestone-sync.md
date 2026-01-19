---
id: FEAT0196
title: Update story commands for milestone sync
type: feat
status: backlog
priority: medium
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0195]
estimate: 2h
created: 2026-01-18
---

# Update Story Commands for Milestone Sync

## Summary

Update `just board start` and `just board done` to sync milestone README when story state changes.

## Acceptance Criteria

- [ ] `just board start <id>` updates milestone README story table
- [ ] `just board done <id>` updates milestone README story table
- [ ] Milestone README progress section updates on story completion
- [ ] Commands work for stories with scope field

## Implementation Notes

- Modify `start` and `done` in `.justfiles/board.just`
- Parse story scope to find milestone
- Trigger milestone README regeneration after state change
- Reuse logic from FEAT0195

## Requirements

- SRS-10: Story state changes update milestone README

---
id: FEAT0205
title: Add just learn reflect command
type: feat
status: backlog
priority: medium
scope: coherence-verification/05-learnings-capture
depends: [CHORE0202]
estimate: 1h
created: 2026-01-19
---

# Add just learn reflect command

## Summary

Create `just learn reflect` command for ad-hoc learning capture outside of story/milestone completion.

## Acceptance Criteria

- [ ] `just learn reflect` prompts for topic
- [ ] Prompts through reflection questions (same as story completion)
- [ ] Creates `docs/learnings/YYYY-MM-DD-topic.md` file
- [ ] File uses structured learning template
- [ ] Handles multiple reflections on same day (adds suffix)

## Implementation Notes

**SRS Requirements:** SRS-06, SRS-07

**Files:**
- Create: `.justfiles/learn.just`
- Modify: `justfile` (import learn.just)
- Create: `docs/learnings/` directory


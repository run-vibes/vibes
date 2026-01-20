---
id: FEAT0203
title: Implement learning capture on story completion
type: feat
status: done
priority: high
scope: coherence-verification/05-learnings-capture
depends: [CHORE0202]
estimate: 2h
created: 2026-01-19
---

# Implement learning capture on story completion

## Summary

Modify `just board done` to prompt for learnings before completing a story.

## Acceptance Criteria

- [ ] `just board done <id>` prompts: "What went well?"
- [ ] Prompts: "What was harder than expected?"
- [ ] Prompts: "What would you do differently?"
- [ ] All prompts can be skipped with Enter
- [ ] If any input provided, generates structured learning entry
- [ ] Learning appended to story file under `## Learnings`
- [ ] Story still completes if all prompts skipped

## Implementation Notes

**SRS Requirements:** SRS-02, SRS-03

**Files:**
- Modify: `.justfiles/board.just` (done recipe)

See [DESIGN.md](../../epics/coherence-verification/milestones/05-learnings-capture/DESIGN.md) for prompt flow.


## Learnings

### L001: Interactive prompts integrate smoothly with done workflow

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Interactive prompts integrate smoothly with done workflow | **Harder than expected:** Counting existing learnings to generate sequential IDs | **Would do differently:** Use a more robust ID generation approach |
| **Suggested Action** | Use a more robust ID generation approach |
| **Applies To** | (to be determined) |
| **Applied** | |

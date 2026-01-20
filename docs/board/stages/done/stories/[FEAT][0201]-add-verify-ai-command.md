---
id: FEAT0201
title: Add just verify ai command
type: feat
status: done
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: [FEAT0200]
estimate: 2h
created: 2026-01-19
---

# Add just verify ai command

## Summary

Create the main entry point script and add the `just verify ai` command to run AI verification for a story.

## Acceptance Criteria

- [ ] `just verify ai <story-id>` runs AI verification
- [ ] `just verify ai <story-id> --model "ollama:model"` overrides model
- [ ] Graceful error if story not found
- [ ] Graceful error if Ollama not running
- [ ] Graceful error if model not available
- [ ] Individual criterion failures don't block report generation
- [ ] Documentation added to CLAUDE.md

## Implementation Notes

**SRS Requirements:** SRS-09, SRS-NFR-01, SRS-NFR-02

**Files:**
- Create: `verification/scripts/ai-verify.ts`
- Modify: `.justfiles/verify.just` (add `ai` command)
- Modify: `CLAUDE.md` (document new command)

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for error handling table.

## Learnings

### L001: Just command integrates cleanly with existing verify workflo

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Just command integrates cleanly with existing verify workflow | **Harder than expected:** Ensuring proper error handling for missing config | **Would do differently:** Add a --dry-run flag to preview what would be verified |
| **Suggested Action** | Add a --dry-run flag to preview what would be verified |
| **Applies To** | (to be determined) |
| **Applied** | |

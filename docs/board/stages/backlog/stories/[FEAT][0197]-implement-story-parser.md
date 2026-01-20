---
id: FEAT0197
title: Implement story parser for AI verification
type: feat
status: backlog
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: [CHORE0146]
estimate: 2h
created: 2026-01-19
---

# Implement story parser for AI verification

## Summary

Create TypeScript module to parse acceptance criteria and verify annotations from story markdown files.

## Acceptance Criteria

- [ ] Parser extracts acceptance criteria from story markdown
- [ ] Parser extracts `<!-- verify: type:name -->` annotations
- [ ] Parser extracts optional hints from annotations `<!-- verify: snapshot:foo | should show X -->`
- [ ] Unit tests cover parsing logic

## Implementation Notes

**SRS Requirements:** SRS-01, SRS-02

**Files:**
- Create: `verification/scripts/lib/parser.ts`
- Create: `verification/scripts/lib/parser.test.ts`

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for interface definitions.

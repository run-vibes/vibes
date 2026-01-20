---
id: FEAT0200
title: Implement AI report generator
type: feat
status: backlog
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: [FEAT0199]
estimate: 2h
created: 2026-01-19
---

# Implement AI report generator

## Summary

Create TypeScript module to generate detailed markdown reports from AI verification results.

## Acceptance Criteria

- [ ] Report includes story metadata (ID, title, scope, model used)
- [ ] Report includes summary table (pass/fail/needs-review counts)
- [ ] Report shows each criterion with artifact, verdict, confidence, evidence
- [ ] Failed criteria include suggested fixes
- [ ] Low confidence criteria marked as "Needs Review"
- [ ] Report saved to `verification/reports/<scope>/<id>-ai.md`

## Implementation Notes

**SRS Requirements:** SRS-08

**Files:**
- Create: `verification/scripts/lib/report.ts`

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for report format.

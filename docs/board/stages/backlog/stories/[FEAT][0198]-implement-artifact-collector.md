---
id: FEAT0198
title: Implement artifact collector
type: feat
status: backlog
priority: high
scope: coherence-verification/04-ai-assisted-verification
depends: [FEAT0197]
estimate: 2h
created: 2026-01-19
---

# Implement artifact collector

## Summary

Create TypeScript module to collect referenced artifacts (snapshots, checkpoints, videos) from the verification directory.

## Acceptance Criteria

- [ ] Collector gathers snapshots from `verification/snapshots/`
- [ ] Collector gathers checkpoints from `verification/checkpoints/`
- [ ] Collector gathers videos from `verification/videos/`
- [ ] Returns artifact type (image/video) and data buffer
- [ ] Unit tests cover collection logic

## Implementation Notes

**SRS Requirements:** SRS-03

**Files:**
- Create: `verification/scripts/lib/collector.ts`
- Create: `verification/scripts/lib/collector.test.ts`

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for interface definitions.

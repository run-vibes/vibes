---
id: FEAT0198
title: Implement artifact collector
type: feat
status: done
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

- [x] Collector gathers snapshots from `verification/snapshots/`
- [x] Collector gathers checkpoints from `verification/checkpoints/`
- [x] Collector gathers videos from `verification/videos/`
- [x] Returns artifact type (image/video) and data buffer
- [x] Unit tests cover collection logic

## Implementation Notes

**SRS Requirements:** SRS-03

**Files:**
- Create: `verification/scripts/lib/collector.ts`
- Create: `verification/scripts/lib/collector.test.ts`

See [DESIGN.md](../../epics/coherence-verification/milestones/04-ai-assisted-verification/DESIGN.md) for interface definitions.

---
id: 01-verification-artifact-pipeline
title: Verification Artifact Pipeline
status: planned
epic: coherence-verification
created: 2026-01-17
---

# Milestone 01: Verification Artifact Pipeline

> Capture system behavior as visual artifacts at three tiers of fidelity.

## Value Statement

Developers can run `just verify all` to capture screenshots, interaction sequences, and stitched CLI+Web videos, producing a report that links artifacts to story acceptance criteria.

## Done Criteria

- [ ] `just verify snapshots` captures key screen PNGs
- [ ] `just verify checkpoints` captures interaction sequence PNGs
- [ ] `just verify videos` records CLI (VHS) + Web (Playwright) and stitches with ffmpeg
- [ ] `just verify report` generates `verification/report.md`
- [ ] `just pre-commit` includes `verify all` step
- [ ] Report links artifacts to story acceptance criteria

## Design

See [design.md](design.md) for architecture and implementation details.

## Stories

| ID | Story | Status |
|----|-------|--------|
| TBD | Create verification directory structure | planned |
| TBD | Implement snapshot capture command | planned |
| TBD | Implement checkpoint capture command | planned |
| TBD | Implement video recording and stitching | planned |
| TBD | Implement report generation | planned |
| TBD | Integrate with pre-commit | planned |

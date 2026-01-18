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

- [x] `just verify snapshots` captures key screen PNGs
- [x] `just verify checkpoints` captures interaction sequence PNGs
- [x] `just verify videos` records CLI (VHS) + Web (Playwright) and stitches with ffmpeg
- [x] `just verify report` generates `verification/report.md`
- [ ] `just pre-commit` includes `verify all` step (deferred - adds 7+ min to commits)
- [ ] Report links artifacts to story acceptance criteria (planned for phase 2)

## Design

See [design.md](design.md) for architecture and implementation details.

## Stories

| ID | Story | Status |
|----|-------|--------|
| FEAT0191 | Create verification definition files | done |
| FEAT0192 | Implement snapshots.spec.ts | done |
| FEAT0193 | Implement checkpoints.spec.ts | done |
| FEAT0194 | Implement videos.spec.ts | done |
| CHORE0195 | Add pre-commit integration | deferred |
| DOCS0196 | Document verification commands | done |

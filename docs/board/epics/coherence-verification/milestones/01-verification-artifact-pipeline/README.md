---
id: 01-verification-artifact-pipeline
title: Verification Artifact Pipeline
status: in-progress
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

## Work Items

> Work tracked inline during implementation (no separate story files).

| Item | Status |
|------|--------|
| Create verification definition files (snapshots.json, checkpoints.json) | done |
| Implement snapshots.spec.ts | done |
| Implement checkpoints.spec.ts | done |
| Implement videos.spec.ts with CLI+Web stitching | done |
| Add pre-commit integration | deferred |
| Document verification commands in CLAUDE.md | done |

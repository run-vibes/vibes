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

### Phase 1: Artifact Capture
- [x] `just verify snapshots` captures key screen PNGs
- [x] `just verify checkpoints` captures interaction sequence PNGs
- [x] `just verify videos` records CLI (VHS) + Web (Playwright) and stitches with ffmpeg
- [x] `just verify report` generates `verification/report.md`

### Phase 2: Story-Scoped Verification
- [x] Story frontmatter uses `scope` field (replaces `epics` + `milestone`)
- [x] All stories migrated to new schema
- [x] CONVENTIONS.md documents new schema
- [ ] Verification annotations in acceptance criteria (`<!-- verify: type:name -->`)
- [ ] `just verify story <ID>` captures artifacts for a story
- [ ] `just verify story-report <ID>` generates story-specific report
- [ ] Reports committed at `verification/reports/<scope>/<id>.md`

## Design

See [design.md](design.md) for architecture and implementation details.

## Work Items

### Phase 1 (Complete)

| Item | Status |
|------|--------|
| Create verification definition files (snapshots.json, checkpoints.json) | done |
| Implement snapshots.spec.ts | done |
| Implement checkpoints.spec.ts | done |
| Implement videos.spec.ts with CLI+Web stitching | done |
| Document verification commands in CLAUDE.md | done |

### Phase 2 (In Progress)

| Item | Status |
|------|--------|
| Design story-scoped verification | done |
| Update story frontmatter schema (scope field) | done |
| Migrate all stories to new schema | done |
| Update CONVENTIONS.md | done |
| Implement verification annotation parsing | pending |
| Implement `just verify story` command | pending |
| Update .gitignore for reports directory | pending |

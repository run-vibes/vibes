---
id: coherence-verification
title: Coherence Verification System
status: active
description: Reduce spec-to-implementation drift through visual artifacts and traceable history
created: 2026-01-17
---

# Coherence Verification System

## Vision

Build a verification system that captures system behavior as visual artifacts (screenshots, videos), connects them back to story specifications, and enables early detection of drift between intent and implementation.

## Goals

1. **Visual Artifacts** — Capture CLI and web UI behavior as screenshots and videos
2. **Traceable History** — Connect any feature from idea → design → implementation → verification
3. **Autonomous Operation** — Clear enough rules that Claude can manage the board correctly
4. **Reduced Feedback Latency** — Catch coherence issues before PR, not after deploy

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 01 | [Artifact Pipeline](milestones/01-artifact-pipeline/) | planned |
| 02 | [Board Restructure](milestones/02-board-restructure/) | in-progress |
| 03 | AI-Assisted Verification | planned |

## Success Criteria

- [ ] `just verify all` captures snapshots, checkpoints, and stitched videos
- [ ] `verification/report.md` links artifacts to story acceptance criteria
- [x] Board hierarchy is Epic > Milestone > Story with clear lifecycles
- [x] Icebox stage exists for blocked/deferred work
- [x] Story naming follows `[TYPE][NNNN]-verb-phrase` convention

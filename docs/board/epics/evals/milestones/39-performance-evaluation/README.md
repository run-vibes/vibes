---
id: 39-performance-evaluation
title: Performance Evaluation
status: in-progress
epics: [evals]
---

# Performance Evaluation

## Overview

First milestone of the Evals epic. Establishes the foundation for evaluation: metrics definitions, storage for benchmark and longitudinal data, and study lifecycle management.

## Goals

- Metrics definitions (session, task, agent, swarm, learning)
- Storage schema for benchmark results and longitudinal studies
- Time-series data collection infrastructure
- Study lifecycle (start, checkpoint, stop)

## Key Deliverables

- `vibes-evals` crate skeleton
- `LongitudinalMetrics` and related types
- `EvalStorage` with benchmark and study tables
- Study management (create, checkpoint, stop)
- `vibes eval study` CLI commands

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0115](../../../../stages/backlog/stories/[FEAT][0115]-eval-web-ui.md) | Eval Web UI | backlog |
| 2 | [FEAT0158](../../../../stages/done/stories/[FEAT][0158]-evals-crate-skeleton.md) | vibes-evals crate skeleton | done |
| 3 | [FEAT0159](../../../../stages/done/stories/[FEAT][0159]-longitudinal-metrics.md) | LongitudinalMetrics types | done |
| 4 | [FEAT0160](../../../../stages/done/stories/[FEAT][0160]-eval-storage.md) | EvalStorage schema and implementation | done |
| 5 | [FEAT0161](../../../../stages/done/stories/[FEAT][0161]-study-lifecycle.md) | Study lifecycle management | done |
| 6 | [FEAT0190](../../../../stages/in-progress/stories/[FEAT][0190]-eval-cli.md) | vibes eval study CLI commands | in-progress |

## Progress

**Requirements:** 0/0
0 verified
**Stories:** 4/6 complete


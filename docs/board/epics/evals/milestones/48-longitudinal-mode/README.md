---
id: 48-longitudinal-mode
title: Long-term Studies
status: planned
epics: [evals]
---

# Long-term Studies

## Overview

Continuous measurement of real-world performance across sessions, agents, and groove learnings. Builds on M39's study infrastructure to collect and aggregate metrics over extended periods.

## Goals

- Consume session, agent, and groove events from Iggy
- Compute LongitudinalMetrics at configurable checkpoint intervals
- Correlate groove learning activations with session outcomes
- Track which sessions contribute to each study checkpoint

## Key Deliverables

- `MetricsCollector` — Event consumer that aggregates metrics by study window
- Checkpoint scheduler with configurable intervals (hourly, daily, weekly)
- Session-to-study correlation tracking
- Groove event integration (learning activations, strategy selections)
- CLI: `vibes eval checkpoint` commands

## Architecture

```
Iggy Streams              MetricsCollector              EvalStorage
─────────────             ────────────────              ───────────
sessions ─────┐
agents ───────┼──────────▶ aggregate by    ──────────▶ CheckpointRecorded
groove ───────┘            study window                 event → Turso
```

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m48-feat-01 | MetricsCollector consumer | high | 4h |
| m48-feat-02 | Session event aggregation | high | 3h |
| m48-feat-03 | Agent event aggregation | high | 3h |
| m48-feat-04 | Groove event correlation | high | 4h |
| m48-feat-05 | Checkpoint scheduler | medium | 3h |
| m48-feat-06 | CLI checkpoint commands | medium | 2h |

## Dependencies

- M39 (Eval Core) — Study lifecycle, EvalStorage, LongitudinalMetrics types

## Epics

- [evals](../../epics/evals)

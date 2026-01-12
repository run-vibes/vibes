---
id: 39-eval-core
title: Eval Core
status: planned
epics: [evals]
---

# Eval Core

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

## Epics

- [evals](../../epics/evals)

## Design

See [../../epics/evals/README.md](../../epics/evals/README.md) for architecture.

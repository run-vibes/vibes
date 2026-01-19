---
id: 40-distributed-tracing
title: Distributed Tracing
status: done
epics: [observability]
---

# Distributed Tracing

## Overview

First milestone of the Observability epic. Establishes OpenTelemetry-based distributed tracing for understanding request flows across sessions, agents, and swarms.

## Goals

- OpenTelemetry tracing integration
- Automatic span instrumentation
- Trace context propagation (session, agent, swarm IDs)
- Export to console, file, Jaeger, OTLP

## Key Deliverables

- `vibes-observe` crate skeleton
- OpenTelemetry tracer setup
- `TraceContext` with vibes-specific context
- `#[instrument]` macros on key functions
- Configurable export targets
- `vibes observe traces` CLI command

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0162](../../../../stages/done/stories/[FEAT][0162]-observe-crate-skeleton.md) | vibes-observe crate skeleton | done |
| 2 | [FEAT0163](../../../../stages/done/stories/[FEAT][0163]-opentelemetry-setup.md) | OpenTelemetry tracer setup | done |
| 3 | [FEAT0164](../../../../stages/done/stories/[FEAT][0164]-trace-context.md) | TraceContext with vibes-specific attributes | done |
| 4 | [FEAT0165](../../../../stages/done/stories/[FEAT][0165]-instrument-key-functions.md) | Instrument key functions with spans | done |
| 5 | [FEAT0166](../../../../stages/done/stories/[FEAT][0166]-export-targets.md) | Export targets configuration | done |
| 6 | [FEAT0167](../../../../stages/done/stories/[FEAT][0167]-observe-cli.md) | vibes observe traces CLI command | done |
| 7 | [FEAT0168](../../../../stages/done/stories/[FEAT][0168]-observe-web-ui.md) | Observe Web UI | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 7/7 complete


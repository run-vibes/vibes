---
id: 40-observability-tracing
title: Observability Tracing
status: planned
epics: [observability]
---

# Observability Tracing

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

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m40-feat-01 | vibes-observe crate skeleton | high | 2h |
| m40-feat-02 | OpenTelemetry tracer setup | high | 4h |
| m40-feat-03 | TraceContext with vibes-specific attributes | high | 3h |
| m40-feat-04 | Instrument key functions with spans | high | 4h |
| m40-feat-05 | Export targets configuration | high | 3h |
| m40-feat-06 | vibes observe traces CLI command | medium | 3h |
| m40-feat-07 | Observe Web UI | medium | 4h |

## Epics

- [observability](../../epics/observability)

## Design

See [../../epics/observability/README.md](../../epics/observability/README.md) for architecture.

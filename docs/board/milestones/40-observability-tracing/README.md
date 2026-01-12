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

## Epics

- [observability](../../epics/observability)

## Design

See [../../epics/observability/README.md](../../epics/observability/README.md) for architecture.

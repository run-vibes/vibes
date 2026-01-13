---
id: 31-attribution-engine
title: Attribution Engine
status: done
epics: [groove]
---

# Attribution Engine

## Overview

Milestone 31: Attribution Engine - Determine which learnings help or hurt sessions.

Uses a 4-layer architecture:
- **Layer 1**: Activation detection (did the learning influence behavior?)
- **Layer 2**: Temporal correlation (positive/negative signals near activation)
- **Layer 3**: Ablation testing (A/B experiments for uncertain learnings)
- **Layer 4**: Value aggregation (combine signals, auto-deprecate harmful learnings)

Each layer is trait-based for future extensibility.

## Epics

- [plugin-system](epics/plugin-system)

## Design

See [design.md](design.md) for architecture decisions.

## Implementation

See [implementation.md](implementation.md) for stories and execution plan.

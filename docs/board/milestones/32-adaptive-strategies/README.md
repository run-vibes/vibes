---
id: 32-adaptive-strategies
title: Adaptive Strategies
status: planned
epics: [plugin-system]
---

# Adaptive Strategies

## Overview

Milestone 32: Adaptive Strategies - Learn which injection strategies work best via Thompson sampling.

Uses hierarchical distributions:
- **Category-level priors**: Cold-start behavior for new learnings
- **Learning specialization**: Individual learnings diverge after threshold sessions
- **Full parameter tuning**: Strategy type + all parameters (timing, agents, callbacks)

Extension points for future novelty detection.

## Epics

- [plugin-system](epics/plugin-system)

## Design

See [design.md](design.md) for architecture decisions.

## Implementation

See [implementation.md](implementation.md) for stories and execution plan.

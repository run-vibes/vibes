---
id: 09-smart-recommendations
title: Smart Recommendations
status: done
epics: [groove]
---

# Smart Recommendations

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

See [DESIGN.md](DESIGN.md) for architecture and implementation details.

## Implementation

See [SRS.md](SRS.md) for stories and execution plan.

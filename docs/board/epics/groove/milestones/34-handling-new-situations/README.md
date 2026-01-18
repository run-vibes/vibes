---
id: 34-handling-new-situations
title: Handling New Situations
status: done
epics: [groove]
---

# Handling New Situations

## Overview

Milestone 34: Open World Adaptation - Detect unknown patterns and surface capability gaps.

Closes the loop on the learning system:
- **NoveltyDetector**: Embedding similarity + incremental DBSCAN clustering
- **CapabilityGapDetector**: Combined signals (failures + attribution + confidence)
- **GraduatedResponse**: Monitor → cluster → auto-adjust → surface
- **SolutionGenerator**: Templates + pattern analysis

Integrates with M32 via NoveltyHook trait, creating a closed feedback loop.

## Epics

- [plugin-system](epics/plugin-system)

## Design

See [design.md](design.md) for architecture decisions.

## Implementation

See [implementation.md](implementation.md) for stories and execution plan.

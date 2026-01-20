---
id: 05-image-understanding
title: Image Understanding
status: in-progress
epics: [web-ui, plugin-system, design-system]
---

# Image Understanding

> **"Spoke"** — Your hub. Your data. Your command. The wheel that connects everything.

## Overview

This milestone evolved from a home page redesign into a comprehensive product vision exploration. It now encompasses:

1. **Visual Depth System** — A physics-inspired aesthetic hierarchy (cosmic → luxury → mechanical → subatomic)
2. **Brand Evolution** — Potential rename from "vibes" to "spoke" (spoke.sh)
3. **Lakehouse Architecture** — Unified data infrastructure design
4. **Command Modes** — Survey/Command/Deep Dive across the product
5. **Design Language** — Art Deco aesthetic with spoke/wheel motifs

## Key Artifacts

### Completed
- [Visual Depth System](visual-depth-system.md) — Documents the aesthetic hierarchy (cosmic → luxury → mechanical → subatomic)
- [Biological Layer](biological-layer.md) — Iggy as the nervous system, neural pathway visualization, the Ka metaphor
- [Biological Design Language](biological-design-language.md) — CSS/animation patterns for organic, living UI elements
- [Command Modes System](command-modes-system.md) — Survey/Command/Deep Dive as vessel configurations
- [Lakehouse Architecture](lakehouse-architecture.md) — Hybrid execution, object-first storage, multi-modal queries
- [Spoke Brand](spoke-brand.md) — Brand positioning, messaging, logo concepts
- [Design Document](DESIGN.md) — Original "Phosphor Command" vision
- Direction prototypes:
  - `12-spoke-direction-a-mechanical.html` — Blueprint/technical aesthetic
  - `12-spoke-direction-b-luxurious.html` — Warm Art Deco aesthetic
  - `12-spoke-direction-c-hybrid.html` — Context-dependent blend
  - `13-spoke-logo-prototypes.html` — Logo variations (12 icons, wordmarks, sizes, animations)
  - `14-command-palette-refined.html` — Cmd+K with hub-and-spoke nav, vessel modes, lakehouse commands
  - `15-smart-palette.html` — Adaptive palette with mindset detection and Groove integration

### Reference Prototypes
- `10-skill-tree-v6.html` — Luxury aesthetic reference
- `10-skill-tree-v7.html` — Cosmic aesthetic reference
- `01-full-dashboard.html` — Dashboard luxury reference

## Open Questions

### Lakehouse (See [lakehouse-architecture.md](lakehouse-architecture.md))
- [x] Hybrid execution model (DataFusion local + remote, Arrow Flight)
- [x] Object-first storage (object_store crate)
- [x] Multi-modal query interface (SQL, vector, graph, NL)
- [ ] Graph query model — Cypher support, compilation strategy, storage model
- [ ] Key schema — geo-distribution, content-addressable option
- [ ] Tiering policy — triggers for hot/warm/cold movement
- [ ] Cache strategy — eviction policy, prefetching
- [ ] Federation vs ingestion — external source query patterns

### Command Modes (See [command-modes-system.md](command-modes-system.md))
- [x] Product-wide posture definition
- [x] Vessel/explorer metaphor
- [x] Accessibility as vessel customization
- [ ] Mode switching UX details
- [ ] Keyboard shortcuts per mode
- [ ] Mode memory per page

### Cmd+K Palette (See `14-command-palette-refined.html`, `15-smart-palette.html`)
- [x] Command categories and prefixes (hub-and-spoke: s: a: g: d: q: v: l: /)
- [x] Contextual commands based on current view (vessel mode integration)
- [x] Smart palette with mindset detection (Groove-powered suggestions)
- [ ] Search scope and filtering implementation details

### Spoke Rename (See [spoke-brand.md](spoke-brand.md), `13-spoke-logo-prototypes.html`)
- [x] Brand implications (Explore. Connect. Amplify.)
- [x] Logo design (Direction B: 6 spokes, dashed inner ring, hub ring + core)
- [x] spoke.sh domain usage (www.spoke.sh canonical, subdomains defined)
- [x] Messaging and positioning (developer, data teams, leaders, curious)

### Multiplayer
- [ ] Commander abstraction refinement
- [ ] Authority levels
- [ ] Handoff modes
- [ ] Presence indicators

## Epics

- `web-ui` — Core UI implementation
- `plugin-system` — Widget and extension APIs
- `design-system` — Component library and visual language

## Design

See [DESIGN.md](DESIGN.md) for the original "Phosphor Command" dashboard architecture.
See [visual-depth-system.md](visual-depth-system.md) for the aesthetic hierarchy documentation.

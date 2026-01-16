---
id: 47-home-page-redesign
title: Home Page Redesign
status: done
epics: [web-ui, plugin-system]
---

# Home Page Redesign

> "Phosphor Mission Control" — A data-driven dashboard that shows what's happening now and what needs your attention next.

## Overview

Redesign the home page from a basic navigation hub to a comprehensive mission control dashboard. The new design features:

- **Attention Zone**: High-priority action items needing human response
- **Primary Zone**: Session, agent, and plugin-contributed widgets
- **Activity Stream**: Real-time unified event feed
- **Metrics Zone**: System-wide statistics and health indicators

Key innovation: **Plugin extensibility** — plugins can register dashboard widgets and contribute attention items.

## Epics

- `web-ui` — Core dashboard implementation
- `plugin-system` — Widget registration API

## Stories

1. **Design System Components**
   - `AttentionBanner` - Action items with priority sorting
   - `DashboardWidget` - Base widget with header/content/footer
   - `ActivityFeed` - Unified event stream
   - `MetricTile` - Small metric display

2. **Web UI Implementation**
   - `HomePage` - Zone-based layout replacing StreamsPage
   - `SessionsWidget` / `AgentsWidget` - Compact list widgets
   - `useAttentionItems` / `useSystemMetrics` hooks

3. **Plugin API**
   - Widget registration in PluginContext
   - Attention item contribution
   - `/api/dashboard/widgets` endpoint

4. **Groove Integration**
   - `GroovePulseWidget` plugin-contributed widget
   - Learning review attention items

## Design

See [design.md](design.md) for detailed architecture and component specifications.

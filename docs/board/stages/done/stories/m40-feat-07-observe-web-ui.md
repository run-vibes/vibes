---
id: m40-feat-07
title: Observe Web UI
type: feat
status: done
priority: medium
epics: [observability]
depends: [m40-feat-06]
estimate: 4h
milestone: 40-observability-tracing
---

# Observe Web UI

## Summary

Add observability and tracing to the web UI. Users can view traces, filter by session/agent, and configure tracing settings from the dashboard.

## Features

### Traces Page

A dedicated `/traces` route showing live traces:

- Trace list with tree visualization
- Span hierarchy display
- Timing information per span
- Color coding by status (success, error, warning)
- Filter by session, agent, or level

### Trace Detail View

Clicking a trace shows:

- Full span tree with timing
- Span attributes and metadata
- Token counts and tool calls
- Error details if applicable
- Related session/agent links

### Live Streaming

Real-time trace viewing:

- WebSocket subscription for new traces
- Auto-scroll with pause option
- Trace buffer with configurable size
- Clear traces button

### Configuration Panel

Settings drawer for tracing config:

- Enable/disable tracing
- Sample rate slider
- Exporter list with status
- Add/remove exporters

## Implementation

1. Add `/traces` route to web-ui
2. Create `TraceList` component with tree view
3. Create `TraceDetail` component
4. Create `TracingConfig` drawer component
5. Add WebSocket handlers for trace streaming
6. Implement span tree visualization
7. Add filter controls

## Acceptance Criteria

- [ ] Traces page shows live traces
- [ ] Trace tree renders spans correctly
- [ ] Session/agent filters work
- [ ] Level filter works
- [ ] Config panel updates settings
- [ ] Real-time streaming updates

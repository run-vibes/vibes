---
id: m39-feat-06
title: Eval Web UI
type: feat
status: backlog
priority: medium
epics: [evals]
depends: [m39-feat-05]
estimate: 4h
milestone: 39-eval-core
---

# Eval Web UI

## Summary

Add evaluation and study management to the web UI. Users can view metrics, manage longitudinal studies, and generate reports from the dashboard.

## Features

### Evals Page

A dedicated `/evals` route with two tabs:

**Studies Tab:**
- List of longitudinal studies with name, status, started date
- Quick actions (stop, checkpoint, report)
- Start new study button

**Metrics Tab:**
- Current session metrics overview
- Success rate, cost, token usage
- Trend indicators

### Study Detail View

Clicking a study shows:

- Study metadata (ID, name, period, checkpoint interval)
- Progress timeline with checkpoints
- Key metrics over time (charts)
- Latest checkpoint summary
- Generate report button

### Study Creation

Modal for starting a new study:

- Study name
- Period duration (days, weeks, months)
- Checkpoint interval configuration
- Start button

### Report View

Generated study reports showing:

- Summary statistics
- Trend analysis with visualizations
- Insights and recommendations
- Export options (markdown, JSON)

## Implementation

**Note:** Uses the event-sourced architecture from m39-feat-03/04.

1. Add `/evals` route to web-ui
2. Create `StudyList` component with status indicators
3. Create `StudyDetail` component with charts
4. Create `StartStudyModal` component
5. Create `ReportView` component
6. Add WebSocket handlers for study events:
   - Mutations send commands → server emits events
   - Subscribe to eval event stream for real-time updates
7. Queries read from Turso projection via API
8. Integrate charting library for visualizations

## Acceptance Criteria

- [ ] Evals page shows studies and metrics
- [ ] Study detail shows checkpoints and charts
- [ ] Start study modal creates new studies
- [ ] Stop/checkpoint actions work
- [ ] Report generation displays correctly
- [ ] Charts render metric trends

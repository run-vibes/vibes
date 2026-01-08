---
id: F009
title: "Feature: Wire IggyAssessmentLog with Firehose"
type: feat
status: done
priority: medium
epics: [core, cli, plugin-system]
depends: []
estimate:
created: 2026-01-03
updated: 2026-01-07
milestone: 26-assessment-framework
---

# Feature: Wire IggyAssessmentLog with Firehose

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Persist assessment events to Iggy and expose them via a dedicated WebSocket firehose, enabling visualization and debugging of the assessment pipeline.

## Context

The assessment processor now emits `AssessmentEvent`s (Lightweight, Medium, Heavy), but they're stored in-memory and lost on restart. This story:

1. Completes the `IggyAssessmentLog` stub to persist events
2. Adds a `/ws/assessment` firehose endpoint
3. Adds a Web UI tab to visualize assessment events

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         vibes-server                                │
│                                                                     │
│  ┌──────────────┐    ┌─────────────────────┐    ┌────────────────┐ │
│  │ EventLog     │───▶│ AssessmentConsumer  │───▶│ Processor      │ │
│  │ (vibes.events)│    │ (reads StoredEvent) │    │ (extracts IDs) │ │
│  └──────────────┘    └─────────────────────┘    └───────┬────────┘ │
│                                                          │          │
│                                                          ▼          │
│                                              ┌───────────────────┐  │
│                                              │ IggyAssessmentLog │  │
│                                              │ (3 topics)        │  │
│                                              └─────────┬─────────┘  │
│                                                        │            │
│  ┌──────────────┐                                      │            │
│  │ /ws/assessment│◀─────────────────────────────────────┘            │
│  │ (firehose)   │                                                   │
│  └──────────────┘                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

## Iggy Topics

**Stream:** `groove.assessment`

| Topic | Events | Retention | Volume |
|-------|--------|-----------|--------|
| `groove.assessment.lightweight` | Per-message signals, EMA | 24 hours | High |
| `groove.assessment.medium` | Checkpoint summaries | 7 days | Medium |
| `groove.assessment.heavy` | Session outcomes | Forever | Low |

## Tasks

### Task 1: Update processor to use StoredEvent

**Files:** `plugins/vibes-groove/src/assessment/processor.rs`, `consumer.rs`

**Changes:**
- Change `process_event(&VibesEvent)` → `process_event(&StoredEvent)`
- Update consumer to pass full `StoredEvent`
- Capture `stored.event_id` for triggering context

**Commit:** `refactor(groove): processor takes StoredEvent for event_id access`

### Task 2: Add event_id fields to assessment types

**File:** `plugins/vibes-groove/src/assessment/types.rs`

**Changes:**
- Add `triggering_event_id: EventId` to `LightweightEvent`
- Add `event_ids_in_segment: Vec<EventId>` to `MediumEvent`
- Update processor to populate these fields

**Commit:** `feat(groove): add triggering_event_id to assessment events`

### Task 3: Complete IggyAssessmentLog

**File:** `plugins/vibes-groove/src/assessment/iggy/log.rs`

**Changes:**
- Create Iggy client connection
- Create stream and topics if they don't exist
- Implement `append()` to route events to correct topic
- Implement `read_session()` and `read_range()` queries
- Add reconnect buffer (like main EventLog)

**Commit:** `feat(groove): implement IggyAssessmentLog`

### Task 4: Wire IggyAssessmentLog in server

**File:** `vibes-server/src/consumers/assessment.rs`

**Changes:**
- Create `IggyAssessmentLog` with `IggyManager`
- Replace `InMemoryAssessmentLog`
- Pass to processor

**Commit:** `feat(server): use IggyAssessmentLog for persistence`

### Task 5: Add /ws/assessment endpoint

**Files:** `vibes-server/src/ws/assessment.rs`, `vibes-server/src/ws/mod.rs`

**Query params:**
- `tiers`: Filter by tier (comma-separated: `lightweight,medium,heavy`)
- `session`: Filter by session_id

**Messages:**
- Server → Client: `Event { tier, event }`, `Batch { events }`
- Client → Server: `FetchOlder { before_offset, limit }`

**Commit:** `feat(server): add /ws/assessment firehose endpoint`

### Task 6: Add Web UI Assessment tab

**Files:** `web-ui/src/components/AssessmentFirehose.tsx`, etc.

**Changes:**
- Add "Assessment" tab to firehose view
- Connect to `/ws/assessment`
- Display events with tier badges
- Make `triggering_event_id` clickable (links to main firehose event)
- Add tier filter checkboxes

**Commit:** `feat(web-ui): add assessment firehose tab`

### Task 7: E2E tests

**Changes:**
- CLI test: Run session with frustrating patterns, verify assessment topics populated
- Browser test (Playwright): Connect to `/ws/assessment`, verify events appear

**Commit:** `test(e2e): add assessment firehose tests`

## Acceptance Criteria

- [x] Assessment events persisted to Iggy (survive restart)
- [x] Three topics with correct tier routing
- [x] `/ws/assessment` streams events with tier filtering
- [x] Web UI shows assessment events with links to triggering events
- [x] No latency impact on main event flow (fire-and-forget maintained)
- [x] E2E: CLI session generates assessment events
- [x] E2E: Browser receives and displays assessment events

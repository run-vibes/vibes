---
id: BUG0002
title: "bug-0002: Firehose Not Loading Historical Events"
type: bug
status: done
priority: medium
epics: [core]
depends: []
estimate:
created: 2026-01-01
updated: 2026-01-07
milestone: 
---

# bug-0002: Firehose Not Loading Historical Events

## Summary

The firehose view in the web UI doesn't show any events. Need to investigate whether it's:
1. Not consuming from the EventLog at all
2. Seeking to `Latest` and only receiving new events (missing historical data)

## Expected Behavior

The firehose should:
- **On load:** Display the last N events from the EventLog (e.g., last 100)
- **After load:** Stream new events as they arrive in real-time

This leverages Iggy's persistent log—there's no reason to lose historical context.

## Investigation Steps

1. Check if firehose WebSocket handler consumes from EventLog
2. Check the `SeekPosition` used when creating the consumer
3. Verify events are being published to the EventLog

## Implementation

### If not consuming from EventLog:
- Wire up EventLog consumer to firehose WebSocket handler

### If seeking to Latest:
- Change to seek to `Offset(N)` from the end or `First` with a limit
- Implement a "load last N events" query before starting the stream

## Acceptance Criteria

- [x] Firehose shows last N historical events on page load
- [x] New events stream in real-time after initial load
- [x] Works correctly after daemon restart (events persist in Iggy)

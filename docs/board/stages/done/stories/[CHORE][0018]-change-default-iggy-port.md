---
id: CHORE0018
title: Change default Iggy port to 7431
type: chore
status: done
priority: high
epics: [core]
depends: []
estimate: 30m
created: 2026-01-08
updated: 2026-01-08
milestone: 29-assessment-framework
---

# Change default Iggy port to 7431

## Summary

Iggy currently starts on port 3001 by default, which conflicts with common development tools (Next.js dev server, other services). Change to port 7431 to avoid conflicts.

## Requirements

- Change default Iggy TCP port from 3001 to 7431
- Update any hardcoded port references
- Ensure configuration still allows overriding the port

## Acceptance Criteria

- [x] Iggy starts on port 7431 by default
- [x] Existing port override configuration still works
- [x] Documentation updated if needed (N/A - port is configurable via env vars)

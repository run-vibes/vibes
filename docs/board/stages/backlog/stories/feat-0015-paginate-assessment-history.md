---
id: FEAT0015
title: Paginate assessment history endpoint
type: feat
status: backlog
priority: medium
epics: [core,plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
milestone: 29-assessment-framework
---

# Paginate assessment history endpoint

## Summary

The `/groove/assessment/history` endpoint currently returns all assessments in a single response, resulting in a huge scrollable list. Add pagination support to improve performance and UX.

## Requirements

- Add `page` and `per_page` query parameters to the history endpoint
- Return pagination metadata (total count, current page, total pages)
- Default to reasonable page size (e.g., 20-50 items)
- Update CLI `vibes groove assess history` to support pagination flags
- Web UI should load pages on demand or use infinite scroll

## Acceptance Criteria

- [ ] HTTP endpoint accepts `?page=N&per_page=M` parameters
- [ ] Response includes pagination metadata
- [ ] CLI supports `--page` and `--per-page` flags
- [ ] Large history lists load quickly

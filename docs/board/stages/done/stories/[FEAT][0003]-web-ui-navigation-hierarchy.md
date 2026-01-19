---
id: FEAT0003
title: "feat-0003: Web UI Navigation Hierarchy"
type: feat
status: done
priority: medium
scope: core
depends: []
estimate:
created: 2026-01-02
---

# feat-0003: Web UI Navigation Hierarchy

## Summary

Restructure the web UI navigation to reflect actual usage patterns. The Streams page becomes the entry point, sessions are accessible from there, and status information moves to Settings.

## Current State

```
Nav: [Sessions] [Streams] [Groove]              [Settings]

Routes:
  /           → placeholder home page
  /claude     → sessions list (Claude-specific naming)
  /claude/$id → session detail
  /streams    → stream index (firehose, debug, sessions, status)
  /status     → tunnel + notifications
  /settings   → about section with wrong links
```

## Proposed Changes

### 1. Make Streams the Home Page

- `/streams` content moves to `/`
- Remove the placeholder home page
- Update any internal links pointing to `/streams`

### 2. Rename Claude Routes to Sessions

- `/claude` → `/sessions` (generic, harness-agnostic)
- `/claude/$sessionId` → `/sessions/$sessionId`
- Update page titles and copy to be harness-agnostic (not "Claude sessions")
- File renames: `ClaudeSessions.tsx` → `Sessions.tsx`, `ClaudeSession.tsx` → `Session.tsx`

### 3. Remove Sessions from Top Nav

- Remove "Sessions" nav item
- Sessions accessible via card on home page (formerly Streams)
- Keep the route functional, just not in primary nav

### 4. Remove Status Page

- Delete `/status` route and `Status.tsx`
- Move Tunnel section to Settings page (under a "Tunnel" heading)
- Notifications already duplicated in Settings

### 5. Fix About Links in Settings

Current:
- `https://github.com/anthropics/vibes`
- `https://github.com/anthropics/vibes/issues`

Should be:
- `https://github.com/run-vibes/vibes`
- `https://github.com/run-vibes/vibes/issues`

## Resulting Structure

```
Nav: [Streams] [Groove]                         [Settings]

Routes:
  /            → streams index (firehose, debug, sessions)
  /sessions    → sessions list (harness-agnostic)
  /sessions/$id → session detail
  /firehose    → event firehose
  /debug       → debug stream
  /settings    → notifications, tunnel, about (fixed links)
```

## Tasks

### Task 1: Fix About Links
- Update Settings.tsx: `anthropics` → `run-vibes`
- Commit: `fix(web-ui): correct GitHub links in settings`

### Task 2: Move Tunnel to Settings
- Add Tunnel section to Settings.tsx (reuse `useTunnelStatus` hook and `StatusBadge`)
- Remove status link from Streams page
- Commit: `refactor(web-ui): move tunnel status to settings`

### Task 3: Delete Status Page
- Remove `/status` route from App.tsx
- Delete `Status.tsx`
- Update `TunnelBadge.tsx` to link to `/settings` instead of `/status`
- Commit: `refactor(web-ui): remove status page`

### Task 4: Rename Claude to Sessions
- Rename files: `ClaudeSessions.tsx` → `Sessions.tsx`, `ClaudeSession.tsx` → `Session.tsx`
- Update routes: `/claude` → `/sessions`, `/claude/$sessionId` → `/sessions/$sessionId`
- Update imports in App.tsx
- Update internal links (Streams.tsx, Session.tsx back link, SessionCard.tsx)
- Update page copy to be harness-agnostic
- Commit: `refactor(web-ui): rename claude routes to sessions`

### Task 5: Make Streams the Home Page
- Move StreamsPage content to be the index route (`/`)
- Remove old placeholder HomePage
- Remove "status" card from streams (it's now in settings)
- Update nav: remove "Sessions", keep "Streams" pointing to `/`
- Commit: `feat(web-ui): make streams the home page`

### Task 6: Update Nav Items
- Remove "Sessions" from navItems array
- Update "Streams" to point to `/` with appropriate active state logic
- Commit: `refactor(web-ui): simplify navigation`

## Acceptance Criteria

- [x] Visiting `/` shows streams index (firehose, debug, sessions cards)
- [x] `/sessions` shows harness-agnostic session list
- [x] Top nav has only: Groove (and Settings icon) — Streams is now home
- [x] Tunnel status visible in Settings page
- [x] No `/status` route exists
- [x] About links point to `run-vibes/vibes`
- [x] All internal navigation works correctly

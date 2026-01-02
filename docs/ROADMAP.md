# Roadmap

vibes development is organized into phases, each delivering a cohesive set of capabilities.

## Overview

| Phase | Description | Status |
|-------|-------------|--------|
| **1. Foundation** | Claude Code proxy, plugin system, local web UI | ✅ Complete |
| **2. Remote Access** | Cloudflare Tunnel, authentication, push notifications | ✅ Complete |
| **3. Multi-Client** | PTY backend, xterm.js UI, multi-session, mirroring | ✅ Complete |
| **4. Continual Learning** | Self-improving assistant that learns from every session | 🔄 In Progress |
| **5. Polish** | Setup wizards, default plugins, iOS app | ⏳ Planned |

---

## Phase 1: Foundation ✅

Established the core proxy architecture and plugin system.

**Milestones:**
- Core proxy with Claude Code passthrough
- CLI with session management
- Plugin foundation with dynamic loading
- Local web UI with embedded server

---

## Phase 2: Remote Access ✅

Enabled secure remote access from any device.

**Milestones:**
- Cloudflare Tunnel integration for secure exposure
- Cloudflare Access JWT authentication
- Web Push notifications for session events
- Persistent chat history with full-text search

---

## Phase 3: Multi-Client ✅

Delivered full terminal parity across clients.

**Milestones:**
- PTY-based backend replacing ANSI simulation
- xterm.js web terminal with full escape sequence support
- Multi-session support with session switching
- CLI ↔ Web mirroring with source attribution

---

## Phase 4: Continual Learning 🔄

Building **groove**—the system that makes every AI coding session better than the last.

**Milestones:**
- 4.1 Harness Introspection ✅ — Detect AI harness capabilities
- 4.2 Storage Foundation ✅ — CozoDB, learning types, adaptive parameters
- 4.3 Capture & Inject ✅ — End-to-end learning pipeline
- 4.4 Assessment Framework 🔄 — Tiered outcome measurement
- 4.5 Learning Extraction — Semantic analysis, embeddings
- 4.6 Attribution Engine — Impact measurement
- 4.7 Adaptive Strategies — Thompson sampling
- 4.8 Dashboard — Observability UI
- 4.9 Open-World Adaptation — Novelty detection

See [groove branding guide](groove/BRANDING.md) for the philosophy behind continual learning.

---

## Phase 5: Polish ⏳

Refining the experience for everyday use.

**Planned:**
- Setup wizards for first-run configuration
- Default plugins (analytics, history)
- CLI enhancements
- iOS companion app

---

## Tracking

- [Planning Board](board/README.md) — Kanban board with detailed milestone tracking
- [Changelog](board/CHANGELOG.md) — History of completed work

---

## Contributing

Interested in contributing to a milestone? See [CLAUDE.md](../CLAUDE.md) for development workflow and conventions.

# Current Focus: Milestone 4.3 Capture & Inject MVP

> **Last Updated:** 2025-12-29

## Active Milestone

**4.3 Capture & Inject** — Build the end-to-end learning pipeline: capture session signals, extract learnings, and inject them into future sessions.

## Quick Links

| Document | Purpose |
|----------|---------|
| [milestone-4.3-design.md](./milestone-4.3-design.md) | Full design document |
| [design.md#43-capture--inject-mvp](./design.md#43-capture--inject-mvp) | Overview in main design |

## Status

- **Tasks:** ~20 (see design document)
- **Crate:** `vibes-groove` (extending existing)
- **Dependencies:** 4.1, 4.2, 4.2.5, 4.2.6 (all complete)

## Key Components

| Component | Purpose |
|-----------|---------|
| `VibesEvent::Hook` | EventBus extension for hook events |
| `SessionCollector` | Per-session event buffering |
| `TranscriptParser` | JSONL transcript parsing |
| `LearningExtractor` | MVP pattern extraction |
| `LearningFormatter` | HTML comment format with markers |
| `ClaudeCodeInjector` | CLAUDE.md + hooks injection |

## Injection Channels

| Channel | When | Mechanism |
|---------|------|-----------|
| CLAUDE.md | Before session | `@import` to learnings.md |
| SessionStart | Session begins | `additionalContext` |
| UserPromptSubmit | Each prompt | `additionalContext` |

## How to Continue

```bash
# Execute the implementation plan (once created)
/superpowers:executing-plans docs/plans/14-continual-learning/milestone-4.3-implementation.md
```

## Completed Milestones in This Epic

| Milestone | Status | Notes |
|-----------|--------|-------|
| 4.1 Harness Introspection | ✅ Complete | `vibes-introspection` crate |
| 4.2 Storage Foundation | ✅ Complete | CozoDB, Learning types, AdaptiveParam |
| 4.2.5 Security Foundation | ✅ Complete | Trust, provenance, RBAC, quarantine, audit |
| 4.2.6 Plugin API Extension | ✅ Complete | CLI/route registration for plugins |
| 4.3 Capture & Inject | 🔄 In Progress | This milestone |
| 4.4 Assessment Framework | ⏳ Not Started | — |
| 4.5 Learning Extraction | ⏳ Not Started | — |
| 4.6 Attribution Engine | ⏳ Not Started | — |
| 4.7 Adaptive Strategies | ⏳ Not Started | — |
| 4.8 groove Dashboard | ⏳ Not Started | — |
| 4.9 Open-World Adaptation | ⏳ Not Started | — |
| Future: Enterprise Scope | ⏳ Not Started | System-level integration |

---

*Update this file when moving to the next sub-milestone.*

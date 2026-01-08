# Milestone 4.3: Capture & Inject MVP

> **Status:** Design Complete
> **Last Updated:** 2025-12-29

## Overview

Build the foundational capture and injection pipeline that enables groove's end-to-end learning loop. This milestone creates the infrastructure for capturing session signals, extracting learnings, storing them, and injecting them back into future sessions.

### Goals

1. **Capture**: Subscribe to hook events via EventBus, collect session data
2. **Extract**: Parse transcripts for explicit preferences, positive feedback, tool patterns
3. **Store**: Persist learnings using existing vibes-groove storage layer
4. **Inject**: Write learnings to CLAUDE.md via `@import` pattern and hooks

### Success Criteria

- User runs a session expressing a preference ("I always use pytest")
- groove captures the session, extracts the preference
- Next session, the preference appears in Claude's context
- User can verify injection via `vibes groove list`

### Out of Scope (Deferred)

| Item | Milestone | Description |
|------|-----------|-------------|
| Semantic extraction | 4.5 | LLM-powered pattern extraction, embedding-based matching |
| Correction detection | 4.5 | Analyze "No, actually..." patterns, error recovery chains |
| Attribution engine | 4.6 | Track which learnings were activated, measure impact |
| Adaptive strategies | 4.7 | Thompson sampling for injection strategy selection |
| Context-aware injection | 4.7 | Semantic search to inject only relevant learnings per prompt |
| Dashboard | 4.8 | Visualize learnings, trends, attribution insights |
| Enterprise/System scope | Future | Integration with enterprise admin console |

### Dependencies

- `vibes-introspection` (complete) — harness detection
- `vibes-groove` storage (complete) — learning persistence
- `vibes-groove` security (complete) — trust, provenance, scanning

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            vibes daemon                                  │
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │ HookReceiver │───▶│   EventBus   │───▶│     groove plugin        │  │
│  │ (extended)   │    │ + HookEvents │    │                          │  │
│  └──────────────┘    └──────────────┘    │  ┌────────────────────┐  │  │
│         ▲                                │  │  SessionCollector  │  │  │
│         │                                │  │  (buffers events)  │  │  │
│  ┌──────────────┐                        │  └─────────┬──────────┘  │  │
│  │ Claude Code  │                        │            │             │  │
│  │   Hooks      │                        │            ▼             │  │
│  │ ┌──────────┐ │                        │  ┌────────────────────┐  │  │
│  │ │SessionSt.│─┼────inject─────────────▶│  │  CaptureAdapter    │  │  │
│  │ │PromptSub.│─┼────inject─────────────▶│  │                    │  │  │
│  │ │  Stop    │─┼────capture────────────▶│  └─────────┬──────────┘  │  │
│  │ └──────────┘ │                        │            │             │  │
│  └──────────────┘                        │            ▼             │  │
│                                          │  ┌────────────────────┐  │  │
│                                          │  │ TranscriptParser   │  │  │
│                                          │  │ + LearningExtract  │  │  │
│                                          │  └─────────┬──────────┘  │  │
│                                          │            │             │  │
│                                          │            ▼             │  │
│                                          │  ┌────────────────────┐  │  │
│                                          │  │   LearningStore    │  │  │
│                                          │  │     (CozoDB)       │  │  │
│                                          │  └─────────┬──────────┘  │  │
│                                          │            │             │  │
│                                          │            ▼             │  │
│                                          │  ┌────────────────────┐  │  │
│                                          │  │ InjectionAdapter   │  │  │
│                                          │  │  (learnings.md)    │  │  │
│                                          │  └────────────────────┘  │  │
│                                          └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Components

| Component | Purpose |
|-----------|---------|
| `VibesEvent::Hook` | New EventBus variant for unified hook event stream |
| `SessionCollector` | Buffers events per-session, processes on Stop |
| `CaptureAdapter` | Trait for abstract capture (Claude Code implementation) |
| `TranscriptParser` | Parses JSONL transcripts, extracts signals |
| `LearningExtractor` | Converts signals to Learning objects |
| `LearningFormatter` | Formats learnings with HTML comment markers |
| `InjectionAdapter` | Trait for abstract injection (CLAUDE.md + hooks) |

---

## Injection Channels

Three channels for injecting learnings into Claude's context:

| Channel | When | Mechanism | Best For |
|---------|------|-----------|----------|
| **CLAUDE.md** | Before session | `@import` to learnings.md | Stable, high-confidence learnings |
| **SessionStart hook** | Session begins | `additionalContext` in stdout | User preferences, session setup |
| **UserPromptSubmit hook** | Each prompt | `additionalContext` in stdout | Context-relevant learnings |

### Channel Details

#### 1. CLAUDE.md via @import

groove maintains a `learnings.md` file that is imported into the user's CLAUDE.md:

```markdown
# CLAUDE.md (user's file)
(user content here)

@.vibes/plugins/groove/learnings.md
```

**Pros:**
- Persistent across sessions
- User can review/edit
- Works with existing Claude Code behavior

**Cons:**
- Requires modifying user's CLAUDE.md (one-time setup)
- Static until next sync

#### 2. SessionStart Hook

Injects learnings when a new Claude Code session starts:

```json
{
  "additionalContext": "## groove Learnings\n\n<!-- groove:01JFX7... -->\n..."
}
```

**Pros:**
- Dynamic injection
- No file modification during session
- Can filter based on project context

**Cons:**
- Requires hook to be installed
- Transient (not persisted in transcript)

#### 3. UserPromptSubmit Hook

Injects context-relevant learnings before each prompt is processed:

```json
{
  "additionalContext": "## Relevant Learnings\n\n<!-- groove:01JFX8... -->\n..."
}
```

**Pros:**
- Per-prompt relevance filtering (future: semantic search)
- Most dynamic channel
- Can react to prompt content

**Cons:**
- Highest overhead (runs on every prompt)
- Requires hook to be installed

---

## Learning Format

Unified format across all injection channels using HTML comment markers:

```markdown
<!-- groove:01JFX7K3N2P confidence:0.85 scope:project category:preference -->
- Always use `pytest` instead of `unittest`
<!-- /groove:01JFX7K3N2P -->
```

### Marker Fields

| Field | Description | Example |
|-------|-------------|---------|
| `id` | UUIDv7 for tracking/updates | `01JFX7K3N2P` |
| `confidence` | 0.0-1.0 score | `0.85` |
| `scope` | global/user/project | `project` |
| `category` | Learning type | `preference` |

### Benefits

- **Surgical updates**: Add/remove individual learnings by ID
- **Attribution tracking**: Detect which learnings Claude "saw"
- **Metadata visibility**: Confidence scores available if needed
- **Clean rendering**: HTML comments hidden in markdown preview
- **Parseable**: Easy regex extraction for reading back

---

## Capture Pipeline

### EventBus Extension

Add `VibesEvent::Hook` variant for unified event stream:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VibesEvent {
    // ... existing variants ...

    /// Hook event from Claude Code
    Hook {
        session_id: Option<String>,
        event: HookEvent,
    },
}
```

### SessionCollector

Buffers events per-session and processes when session ends:

```rust
pub struct SessionCollector {
    /// Active session buffers (session_id -> events)
    sessions: HashMap<String, SessionBuffer>,
    /// Learning storage
    store: Arc<dyn LearningStore>,
    /// Transcript parser
    parser: TranscriptParser,
    /// Learning extractor
    extractor: LearningExtractor,
}

pub struct SessionBuffer {
    session_id: String,
    project_path: Option<PathBuf>,
    tool_events: Vec<ToolEvent>,
    start_time: DateTime<Utc>,
}
```

### Ordering Guarantees

- Events processed sequentially within a session
- Sessions processed in parallel (no cross-session dependencies)
- Stop event triggers batch processing of buffered events

---

## Learning Extraction (MVP)

### TranscriptParser

Parses Claude Code JSONL transcripts:

```rust
pub struct TranscriptParser {
    supported_versions: Vec<String>,
}

pub struct ParsedTranscript {
    pub session_id: String,
    pub messages: Vec<TranscriptMessage>,
    pub tool_uses: Vec<ToolUse>,
    pub metadata: TranscriptMetadata,
}
```

### LearningExtractor

Extracts learnings using regex patterns (MVP):

| Signal | Pattern | Example |
|--------|---------|---------|
| Explicit preference | `(?i)(always\|never\|prefer\|use)\s+.{5,50}` | "always use pytest" |
| Positive feedback | `(?i)(perfect\|exactly\|great\|thanks)` | "Perfect!" |
| Tool preference | Tool X succeeds consistently | "prefers `just` over `make`" |

### Confidence Scoring (MVP)

```rust
fn compute_confidence(signal: &Signal) -> f64 {
    match signal {
        Signal::ExplicitPreference { .. } => 0.85,
        Signal::PositiveFeedback { .. } => 0.70,
        Signal::ToolPattern { success_rate, .. } => *success_rate,
    }
}
```

**Future (4.5):** Semantic analysis, LLM-powered extraction, embedding-based matching.

---

## File Paths

### Cross-Platform Support (via `dirs` crate)

| Platform | User Scope Path |
|----------|-----------------|
| Linux | `~/.local/share/vibes/plugins/groove/` |
| macOS | `~/Library/Application Support/vibes/plugins/groove/` |
| Windows | `%APPDATA%\vibes\plugins\groove\` |

### Scope Paths

| Scope | Path |
|-------|------|
| **User** | `{data_dir}/vibes/plugins/groove/learnings.md` |
| **Project** | `.vibes/plugins/groove/learnings.md` |
| **Global** | `{data_dir}/vibes/plugins/groove/global/learnings.md` |

---

## First-Run Setup

### Hybrid Approach (Option D)

1. **Auto-create directories**: Safe, no user files modified
2. **Prompt for CLAUDE.md modification**: Requires explicit action

### CLI Commands

```bash
# Initialize groove for current project
vibes groove init

# Initialize groove for user scope
vibes groove init --user

# List current learnings
vibes groove list

# Show injection status
vibes groove status
```

---

## Hook Management

### Single Vibes Hook Pattern

Vibes installs one hook per type that dispatches to all interested plugins:

```
~/.claude/hooks/
├── session-start.sh      → calls vibes daemon
├── user-prompt-submit.sh → calls vibes daemon
├── pre-tool-use.sh       → calls vibes daemon
├── post-tool-use.sh      → calls vibes daemon
└── stop.sh               → calls vibes daemon
```

### Hook Response Flow

```rust
impl HookReceiver {
    async fn handle_hook(&self, event: HookEvent) -> Option<HookResponse> {
        // 1. Publish to EventBus for all subscribers
        self.event_bus.publish(VibesEvent::Hook {
            session_id: event.session_id(),
            event: event.clone(),
        }).await;

        // 2. For injection hooks, collect responses from plugins
        if event.supports_response() {
            return self.collect_plugin_responses(&event).await;
        }

        None
    }
}
```

---

## Implementation Tasks (TDD)

### Phase 1: Core Infrastructure

```
1.1 VibesEvent::Hook variant
    ├── Test: VibesEvent::Hook serialization roundtrip
    ├── Test: HookEvent session_id extraction
    └── Implement: Add Hook variant to VibesEvent enum

1.2 Extended HookInstaller
    ├── Test: install_hook creates SessionStart script
    ├── Test: install_hook creates UserPromptSubmit script
    └── Implement: Add new hook types to HookInstaller

1.3 Hook response collection
    ├── Test: HookReceiver returns response for injection hooks
    ├── Test: HookReceiver publishes to EventBus
    └── Implement: Add response collection to HookReceiver

1.4 GroovePaths
    ├── Test: learnings_file returns correct path per scope
    ├── Test: cross-platform paths use dirs crate
    └── Implement: GroovePaths struct
```

### Phase 2: Capture Pipeline

```
2.1 SessionCollector
    ├── Test: buffers events by session_id
    ├── Test: processes buffer on Stop event
    ├── Test: handles concurrent sessions independently
    └── Implement: SessionCollector struct

2.2 TranscriptParser
    ├── Test: parses valid JSONL transcript
    ├── Test: extracts user and assistant messages
    ├── Test: handles malformed lines gracefully
    └── Implement: TranscriptParser

2.3 LearningExtractor
    ├── Test: extracts "always use X" as preference
    ├── Test: extracts "perfect" as positive feedback
    ├── Test: computes confidence scores correctly
    └── Implement: LearningExtractor with regex patterns

2.4 EventBus subscription
    ├── Test: groove receives Hook events from EventBus
    └── Implement: Wire plugin to EventBus
```

### Phase 3: Injection Pipeline

```
3.1 LearningFormatter
    ├── Test: format_one produces correct HTML comment markers
    ├── Test: format_all groups by category
    ├── Test: parse extracts learnings from formatted content
    └── Implement: LearningFormatter

3.2 InjectionAdapter trait
    ├── Test: trait is object-safe
    └── Implement: InjectionAdapter trait definition

3.3 ClaudeCodeInjector
    ├── Test: sync_to_file writes learnings.md correctly
    ├── Test: sync_to_file preserves learning IDs on update
    ├── Test: sync_to_file removes deleted learnings
    └── Implement: ClaudeCodeInjector

3.4 Hook injection responses
    ├── Test: SessionStart hook returns formatted learnings
    ├── Test: UserPromptSubmit hook returns context-relevant learnings
    └── Implement: Hook response handlers
```

### Phase 4: Setup & CLI

```
4.1 GrooveSetup
    ├── Test: ensure_directories creates user data dir
    ├── Test: init_project creates .vibes/plugins/groove/
    └── Implement: GrooveSetup

4.2 `vibes groove init`
    ├── Test: init creates learnings.md
    ├── Test: init --user modifies ~/.claude/CLAUDE.md
    └── Implement: CLI command

4.3 `vibes groove list`
    ├── Test: list shows learnings with metadata
    └── Implement: CLI command

4.4 `vibes groove status`
    ├── Test: status shows injection channels enabled
    └── Implement: CLI command
```

### Phase 5: Integration & Documentation

```
5.1 End-to-end integration test
    ├── Test: preference in transcript → stored → injected in next session
    └── Implement: Integration test with mock hooks

5.2 Update vibes-groove/README.md
5.3 Update docs/PROGRESS.md
```

---

## Future Milestones

### Enterprise/System Scope (New Milestone)

Deferred to future milestone:

- [ ] System-level learnings path (`/etc/vibes/groove/`)
- [ ] Integration with Claude.ai admin console API
- [ ] Org-wide learning sync
- [ ] Admin approval workflow for shared learnings
- [ ] Compliance API integration for audit

### Related Milestones

| Milestone | Extends 4.3 With |
|-----------|------------------|
| 4.5 Learning Extraction | Semantic analysis, LLM extraction, embeddings |
| 4.6 Attribution Engine | Track learning activation, measure impact |
| 4.7 Adaptive Strategies | Thompson sampling, context-aware injection |
| 4.8 Dashboard | Visualize learnings, trends, insights |

---

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| End-to-end scope | Full demo flow | Validates entire architecture |
| Hook injection | SessionStart + UserPromptSubmit | Dynamic injection with `additionalContext` |
| EventBus integration | Add `VibesEvent::Hook` | Unified event stream |
| Ordering | Per-session buffering | Sequential within session, parallel across |
| Hook management | Single vibes hook per type | Avoids conflicts, enables plugin dispatch |
| CLAUDE.md format | `@import` to learnings.md | Clean separation, user file untouched |
| Learnings path | `~/.local/share/vibes/plugins/groove/` | XDG-compliant, plugin-scoped |
| Learning markers | HTML comments with metadata | Surgical updates, attribution tracking |
| Format consistency | Same format all channels | Unified parsing, metadata everywhere |
| First-run setup | Hybrid (auto dirs, prompt for CLAUDE.md) | Respects user files |
| Enterprise scope | Deferred | Focus on core learning loop first |

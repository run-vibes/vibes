# vibes-groove

> **groove** — The continual learning system that finds your coding rhythm.

groove is a vibes plugin that captures what works in your Claude Code sessions and injects those learnings into future sessions—automatically and without friction.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            vibes daemon                                  │
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────┐  │
│  │ HookReceiver │───▶│   EventLog   │───▶│     groove plugin        │  │
│  │ (extended)   │    │  (via Iggy)  │    │                          │  │
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

## Injection Channels

groove uses three channels to inject learnings into Claude's context:

| Channel | When | Mechanism | Best For |
|---------|------|-----------|----------|
| **CLAUDE.md** | Before session | `@import` to learnings.md | Stable, high-confidence learnings |
| **SessionStart hook** | Session begins | `additionalContext` | User preferences, session setup |
| **UserPromptSubmit hook** | Each prompt | `additionalContext` | Context-relevant learnings |

### 1. CLAUDE.md via @import

groove maintains a `learnings.md` file imported into your CLAUDE.md:

```markdown
# Your CLAUDE.md
(your content here)

@.vibes/plugins/groove/learnings.md
```

**Pros:** Persistent, user-reviewable, works with existing Claude Code behavior
**Cons:** Requires one-time setup, static until next sync

### 2. SessionStart Hook

Injects learnings when a new Claude Code session starts.

**Pros:** Dynamic, no file modification, project-context aware
**Cons:** Requires hook installation, transient

### 3. UserPromptSubmit Hook

Injects context-relevant learnings before each prompt is processed.

**Pros:** Per-prompt relevance filtering, most dynamic
**Cons:** Runs on every prompt, requires hook installation

## Learning Format

All channels use a unified format with HTML comment markers:

```markdown
<!-- groove:01JFX7K3N2P confidence:0.85 scope:project category:preference -->
- Always use `pytest` instead of `unittest`
<!-- /groove:01JFX7K3N2P -->
```

### Marker Fields

| Field | Description | Example |
|-------|-------------|---------|
| `id` | UUIDv7 for tracking | `01JFX7K3N2P` |
| `confidence` | 0.0-1.0 score | `0.85` |
| `scope` | global/user/project | `project` |
| `category` | Learning type | `preference` |

### Benefits

- **Surgical updates**: Add/remove learnings by ID
- **Attribution tracking**: Know which learnings Claude "saw"
- **Metadata visibility**: Confidence scores embedded
- **Clean rendering**: HTML comments hidden in preview
- **Parseable**: Easy to read back programmatically

## File Paths

### Cross-Platform (via `dirs` crate)

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

## CLI Commands

```bash
# Initialize groove for current project
vibes groove init

# Initialize groove for user scope
vibes groove init --user

# List current learnings
vibes groove list

# Show injection status
vibes groove status

# Trust/security management
vibes groove trust <learning-id> <level>
vibes groove policy show
vibes groove quarantine list
```

## Crate Structure

```
vibes-groove/
├── src/
│   ├── lib.rs              # Plugin entry point
│   ├── plugin.rs           # GroovePlugin implementation
│   ├── config.rs           # Configuration types
│   ├── error.rs            # Error types
│   │
│   ├── types/              # Core domain types
│   │   ├── learning.rs     # Learning, LearningCategory
│   │   ├── params.rs       # AdaptiveParam, SystemParam
│   │   ├── scope.rs        # Scope enum
│   │   ├── relations.rs    # LearningRelation, RelationType
│   │   └── usage.rs        # UsageStats
│   │
│   ├── store/              # Storage layer
│   │   ├── traits.rs       # LearningStore, ParamStore traits
│   │   ├── cozo.rs         # CozoDB implementation
│   │   └── schema.rs       # Database schema & migrations
│   │
│   ├── security/           # Security subsystem
│   │   ├── trust.rs        # TrustLevel, TrustContext
│   │   ├── provenance.rs   # ContentHash, Provenance
│   │   ├── rbac.rs         # OrgRole, Permissions
│   │   ├── scanning/       # Content security scanning
│   │   ├── quarantine/     # Quarantine management
│   │   ├── policy/         # Policy engine
│   │   └── audit/          # Audit logging
│   │
│   ├── capture/            # Capture pipeline (4.3)
│   │   ├── collector.rs    # SessionCollector
│   │   ├── parser.rs       # TranscriptParser
│   │   └── extractor.rs    # LearningExtractor
│   │
│   ├── inject/             # Injection pipeline (4.3)
│   │   ├── formatter.rs    # LearningFormatter
│   │   └── injector.rs     # ClaudeCodeInjector
│   │
│   └── paths.rs            # GroovePaths (cross-platform)
│
└── Cargo.toml
```

## Development

### Building

```bash
just build
```

### Testing

```bash
just test
```

### Running with vibes

groove is loaded as a plugin by the vibes daemon:

```bash
vibes daemon start
```

## Design Documents

- [Branding Guide](../docs/groove/BRANDING.md)
- [Continual Learning Design](../docs/plans/14-continual-learning/design.md)
- [Milestone 4.3 Design](../docs/plans/14-continual-learning/milestone-4.3-design.md)

## Milestones

| Milestone | Status | Description |
|-----------|--------|-------------|
| 4.1 Harness Introspection | Complete | `vibes-introspection` crate |
| 4.2 Storage Foundation | Complete | CozoDB, Learning types, AdaptiveParam |
| 4.2.5 Security Foundation | Complete | Trust, provenance, RBAC, quarantine |
| 4.2.6 Plugin API Extension | Complete | CLI/route registration for plugins |
| 4.3 Capture & Inject | Complete | End-to-end learning pipeline |
| 4.4 Assessment Framework | Not Started | Tiered outcome measurement |
| 4.5 Learning Extraction | Not Started | Semantic analysis, embeddings |
| 4.6 Attribution Engine | Not Started | Impact measurement |
| 4.7 Adaptive Strategies | Not Started | Thompson sampling |
| 4.8 Dashboard | Not Started | Observability UI |
| 4.9 Open-World Adaptation | Not Started | Novelty detection |

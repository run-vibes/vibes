# Learnings Capture — Design Document

> Architecture and implementation details for the learnings capture system.

**Milestone:** [05-learnings-capture](README.md)
**SRS:** [SRS.md](SRS.md)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Learnings Capture System                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Triggers:                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ just board   │  │ just board   │  │ just learn   │          │
│  │ done <id>    │  │ done-        │  │ reflect      │          │
│  │              │  │ milestone    │  │              │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                    │
│         ▼                 ▼                 ▼                    │
│  ┌────────────────────────────────────────────────────┐         │
│  │              Learning Capture Prompt                │         │
│  │  (AI-guided reflection on what was learned)         │         │
│  └────────────────────────────┬───────────────────────┘         │
│                               │                                  │
│                               ▼                                  │
│  ┌────────────────────────────────────────────────────┐         │
│  │              Storage Layer                          │         │
│  │  ┌────────────┐ ┌─────────────┐ ┌──────────────┐  │         │
│  │  │ Story file │ │ Milestone   │ │ docs/        │  │         │
│  │  │ ## Learn.. │ │ LEARNINGS.md│ │ learnings/   │  │         │
│  │  └────────────┘ └─────────────┘ └──────────────┘  │         │
│  └────────────────────────────────────────────────────┘         │
│                               │                                  │
│                               ▼                                  │
│  ┌────────────────────────────────────────────────────┐         │
│  │              Propagation Engine                     │         │
│  │  (AI suggests where to apply, user decides)         │         │
│  └────────────────────────────────────────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Learning Template

Structured format for all learnings:

```markdown
### L001: [Brief title]

| Field | Value |
|-------|-------|
| **Category** | process / architecture / verification / code |
| **Context** | What triggered this learning |
| **Insight** | What we learned |
| **Suggested Action** | Concrete next step |
| **Applies To** | Where this should propagate |
| **Applied** | (empty until propagated) |
```

### 2. Story Learning Capture

Added to `just board done`:

```bash
# Prompt flow
echo "What went well with this story? (Enter to skip)"
read well
echo "What was harder than expected? (Enter to skip)"
read hard
echo "What would you do differently? (Enter to skip)"
read different

# If any input provided, append to story file
if [[ -n "$well" || -n "$hard" || -n "$different" ]]; then
    # Generate structured learning and append to story
fi
```

### 3. Milestone Learnings Aggregation

Created by `just board done-milestone`:

```markdown
# Milestone Learnings: 05-learnings-capture

> Aggregated learnings from completed stories.

## Story Learnings

### From FEAT0203: Implement story learning capture

[Copy of learnings from story file]

## Synthesis

### ML001: [Milestone-level insight]

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Completing the learnings capture milestone |
| **Insight** | [User-provided synthesis] |
| **Suggested Action** | [Concrete next step] |
| **Applies To** | [Target files/conventions] |
```

### 4. Ad-hoc Reflection

`just learn reflect` creates files in `docs/learnings/`:

```
docs/learnings/
├── 2026-01-19-verification-hints.md
├── 2026-01-20-test-patterns.md
└── ...
```

### 5. Propagation Engine

`just learn apply` workflow:

1. Scan all learnings (stories, milestones, ad-hoc) for unapplied entries
2. For each, AI suggests concrete changes to target files
3. Present diff-style preview to user
4. User: accept / reject / edit
5. Apply accepted changes, mark learning as applied

## Commands

| Command | Description |
|---------|-------------|
| `just board done <id>` | (Modified) Prompts for learnings before completing |
| `just board done-milestone <id>` | (Modified) Aggregates learnings, prompts for synthesis |
| `just learn reflect` | Interactive ad-hoc learning capture |
| `just learn apply` | Suggest and apply pending learnings |
| `just learn list` | Show all learnings with status |

## File Locations

| Type | Location |
|------|----------|
| Story learnings | `docs/board/stages/done/stories/[TYPE][NNNN]-name.md` (## Learnings section) |
| Milestone learnings | `docs/board/epics/*/milestones/*/LEARNINGS.md` |
| Ad-hoc learnings | `docs/learnings/YYYY-MM-DD-topic.md` |

## Learning ID Format

| Scope | Format | Example |
|-------|--------|---------|
| Story | `L001`, `L002` | Per-story sequential |
| Milestone | `ML001`, `ML002` | Per-milestone sequential |
| Ad-hoc | Filename | `2026-01-19-verification-hints` |

## Applied Status Tracking

When a learning is propagated:

```markdown
| **Applied** | CLAUDE.md (2026-01-20), CONVENTIONS.md (2026-01-20) |
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| User skips all prompts | No learnings added, story/milestone still completes |
| Learning file missing | Create with header, then append |
| Propagation target doesn't exist | Warn and skip that target |
| User rejects all suggestions | Mark as "reviewed" not "applied" |


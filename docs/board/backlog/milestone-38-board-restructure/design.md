# Milestone 38: Board Restructure

> Design document for restructuring the kanban board to support epics and flexible story relationships.

## Motivation

The current board structure embeds stories inside milestone directories. This creates two problems:

1. **No thematic grouping** — Stories can't be grouped by theme (CLI, Web UI, Core) independently of release milestones
2. **No cross-cutting stories** — A story that touches multiple areas must live in one milestone only

## Design

### Directory Structure

```
docs/board/
├── README.md              # Auto-generated board view
├── CHANGELOG.md           # Release history
├── CONVENTIONS.md         # Usage guide
├── templates/
│   ├── story.md
│   ├── epic.md
│   └── milestone.md
├── stages/
│   ├── backlog/
│   │   └── stories/       # Story files live here
│   ├── in-progress/
│   │   └── stories/
│   └── done/
│       └── stories/
├── epics/
│   ├── README.md          # Epic index (auto-generated)
│   ├── core/
│   │   ├── README.md      # Epic metadata + description
│   │   └── F001-name.md   # Symlinks to story files
│   ├── cli/
│   │   └── README.md
│   ├── web-ui/
│   │   └── README.md
│   ├── plugin-system/
│   │   └── README.md
│   ├── networking/
│   │   └── README.md
│   └── mobile/
│       └── README.md
└── milestones/
    ├── README.md          # Milestone index (auto-generated)
    └── NN-name/
        ├── README.md      # Milestone metadata + description
        └── epic-name/     # Symlinks to epic directories
```

### Key Concepts

- **Stories** are files in `stages/<stage>/stories/`
- **Epics** contain symlinks to story files (e.g., `epics/cli/F001-name.md → ../../stages/backlog/stories/F001-name.md`)
- **Milestones** contain symlinks to epic directories (e.g., `milestones/26-assessment/cli → ../../epics/cli`)
- Stories can belong to multiple epics
- Epics can belong to multiple milestones

### Frontmatter Schema

**Story** (`stages/<stage>/stories/F001-name.md`):
```yaml
---
id: F001
title: CLI assess queries
type: feat        # feat | bug | chore | refactor | docs
status: backlog   # backlog | in-progress | done
priority: high    # critical | high | medium | low
epics: [cli, core]
depends: [F000]   # Story IDs this depends on (optional)
estimate: 2h      # Time estimate (optional)
created: 2025-01-07
updated: 2025-01-07
---
```

**Epic** (`epics/<name>/README.md`):
```yaml
---
id: cli
title: CLI Experience
status: active    # active | complete | paused
description: Command-line interface and user experience
---
```

**Milestone** (`milestones/<name>/README.md`):
```yaml
---
id: 26-assessment-framework
title: Assessment Framework
status: in-progress   # planned | in-progress | done
epics: [core, cli, plugin-system]
---
```

### Story Lifecycle

**Creation:**
1. Create story file in `stages/backlog/stories/F001-name.md`
2. Create symlink in relevant epic(s)
3. Optionally link epic to milestone

**Movement:**
```
stages/backlog/stories/F001-name.md
        ↓ (just board start F001)
stages/in-progress/stories/F001-name.md
        ↓ (just board done F001)
stages/done/stories/F001-name.md
```

When a story moves stages, tooling automatically updates symlinks in epics.

### Justfile Commands

**Help & generation:**
| Command | Action |
|---------|--------|
| `just board` | Show available commands |
| `just board generate` | Regenerate README.md |
| `just board status` | Show counts per stage + blocked stories |

**Story management:**
| Command | Action |
|---------|--------|
| `just board new story "title"` | Create story in backlog |
| `just board start <id>` | Move to in-progress |
| `just board done <id>` | Move to done + changelog |

**Epic/Milestone management:**
| Command | Action |
|---------|--------|
| `just board new epic "name"` | Create epic |
| `just board new milestone "name"` | Create milestone |
| `just board link <story> <epic>` | Add story to epic |
| `just board unlink <story> <epic>` | Remove story from epic |
| `just board link-epic <epic> <milestone>` | Add epic to milestone |

**Utility:**
| Command | Action |
|---------|--------|
| `just board list epics` | Show epics with story counts |
| `just board list milestones` | Show milestones with progress |
| `just board show <id>` | Display story details |

## Migration Plan

### Phase 1: Structure
1. Create new directory structure (`stages/`, `epics/`, `milestones/`, `templates/`)
2. Create the 6 epics (core, cli, web-ui, plugin-system, networking, mobile)

### Phase 2: Story Extraction
1. Extract stories from existing milestone directories into `stages/<stage>/stories/`
2. Assign each story to appropriate epic(s)
3. Create symlinks from epics to stories

### Phase 3: Milestone Conversion
1. Convert milestone directories to new format
2. Create symlinks from milestones to relevant epics

### Phase 4: Tooling
1. Rewrite `scripts/board.sh` with new commands
2. Update justfile recipes
3. Update CONVENTIONS.md and CLAUDE.md

### Phase 5: Cleanup
1. Remove old directory structure
2. Run `just board generate` to verify
3. Commit

## Epics

Initial epic structure based on codebase themes:

| Epic | Description |
|------|-------------|
| `core` | Proxy, PTY, event system, storage, sessions |
| `cli` | Commands, user experience, output formatting |
| `web-ui` | Dashboard, firehose, CRT design system |
| `plugin-system` | Plugin API, lifecycle, loading |
| `networking` | Tunnels, auth, security |
| `mobile` | iOS app |

Individual plugins (groove, etc.) become milestones that link to relevant epics.

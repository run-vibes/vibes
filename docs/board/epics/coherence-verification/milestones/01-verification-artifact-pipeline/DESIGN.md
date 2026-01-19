# Milestone 01: Artifact Pipeline - Design Document

> Capture system behavior as visual artifacts that can be traced back to story specifications.

## Overview

The artifact pipeline generates verification artifacts at three tiers of fidelity and produces a report linking artifacts to story acceptance criteria.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Artifact storage** | Local only, report committed | Keep repo lean; artifacts reviewed locally |
| **Verification pyramid** | 3 tiers (snapshots, checkpoints, videos) | Balance speed vs comprehensiveness |
| **CLI + Web stitching** | ffmpeg | CLI recordings via VHS, web via Playwright, stitch together |
| **AI verification** | Phase 2 | Build intuition with human review first |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     VERIFICATION PIPELINE                        │
│                                                                   │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐            │
│  │   Tier 1    │   │   Tier 2    │   │   Tier 3    │            │
│  │  Snapshots  │   │ Checkpoints │   │   Videos    │            │
│  │   (~30s)    │   │   (~2min)   │   │   (~5min)   │            │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘            │
│         │                 │                 │                    │
│         └─────────────────┼─────────────────┘                    │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │  Report Gen   │                              │
│                   │ (report.md)   │                              │
│                   └───────────────┘                              │
│                           │                                      │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │    Commit     │                              │
│                   │  report.md    │                              │
│                   └───────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| Snapshot capture | `.justfiles/verify.just` | Playwright screenshots of key screens |
| Checkpoint capture | `.justfiles/verify.just` | Playwright sequences with screenshots at key moments |
| Video capture (CLI) | `verification/tapes/` | VHS tape recordings of CLI workflows |
| Video capture (Web) | `e2e-tests/` | Playwright video recordings |
| Video stitcher | `.justfiles/verify.just` | ffmpeg combining CLI + Web videos |
| Report generator | `.justfiles/verify.just` | Markdown report linking artifacts to stories |

---

## Directory Structure

```
verification/                    # .gitignored except report.md
├── .gitignore                   # Ignore all except report.md
├── report.md                    # COMMITTED: Summary linking to stories
├── snapshots/                   # Tier 1: Key screen PNGs
│   ├── dashboard.png
│   ├── session-list.png
│   └── ...
├── checkpoints/                 # Tier 2: Interaction sequence PNGs
│   ├── 01-navigate-sessions.png
│   ├── 02-create-session.png
│   └── ...
└── videos/                      # Tier 3: Full flow recordings
    ├── cli/                     # VHS recordings (webm)
    │   └── help.webm
    ├── web/                     # Playwright recordings (webm)
    │   └── dashboard-walkthrough.webm
    └── stitched/                # Combined CLI + Web (side-by-side)
        └── combined.webm
```

---

## Commands

### Justfile Module

New module: `.justfiles/verify.just`

| Command | Description |
|---------|-------------|
| `just verify` | Show available verify commands |
| `just verify snapshots` | Tier 1: Capture key screen snapshots |
| `just verify checkpoints` | Tier 2: Capture interaction sequences |
| `just verify videos` | Tier 3: Record and stitch CLI + Web videos |
| `just verify all` | Run all tiers |
| `just verify report` | Generate report.md from captured artifacts |
| `just verify clean` | Remove all artifacts |

## Snapshot Definitions

Define which screens to capture in `verification/snapshots.json`:

```json
{
  "snapshots": [
    {
      "name": "dashboard",
      "url": "/groove/status",
      "waitFor": "[data-testid='status-indicator']"
    },
    {
      "name": "session-list",
      "url": "/sessions",
      "waitFor": ".session-row"
    }
  ]
}
```

---

## Checkpoint Definitions

Define interaction sequences in `verification/checkpoints.json`:

```json
{
  "checkpoints": [
    {
      "name": "create-session",
      "steps": [
        {"action": "goto", "url": "/sessions", "screenshot": "01-sessions-page"},
        {"action": "click", "selector": "[data-testid='new-session']", "screenshot": "02-after-click"},
        {"action": "waitFor", "selector": ".session-created", "screenshot": "03-session-created"}
      ]
    }
  ]
}
```

---

## Video Stitching

### CLI Recording (VHS)

VHS tapes in `verification/tapes/` output webm directly at 1280x720:

```tape
Output verification/videos/cli/help.webm
Set FontSize 18
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
```

### Web Recording (Playwright)

Configure Playwright to record:

```typescript
// playwright.config.ts
use: {
  video: 'on',
  videoSize: { width: 1280, height: 720 }
}
```

### Stitching with ffmpeg

Both CLI and Web videos are 1280x720 webm. Stitch side-by-side for 2560x720 output:

```bash
# Side-by-side (CLI left, Web right)
ffmpeg -i cli/help.webm -i web/dashboard-walkthrough.webm \
  -filter_complex "[0:v][1:v]hstack=inputs=2" \
  -c:v libvpx-vp9 stitched/combined.webm
```

---

## Report Format

See [`verification/templates/report.md`](../../../../../verification/templates/report.md) for the global report template.

Generated at `verification/report.md`.

---

## PR Integration

### PR Template

See [`.github/PULL_REQUEST_TEMPLATE.md`](../../../../../.github/PULL_REQUEST_TEMPLATE.md) for the GitHub PR template.

GitHub auto-populates this template when creating PRs.

---

## Dependencies

```toml
# No new Rust dependencies - this is tooling
```

### External Tools

| Tool | Purpose | Install |
|------|---------|---------|
| Playwright | Web screenshots/video | `npm install` (existing) |
| VHS | CLI recordings | `nix develop` (existing) |
| ffmpeg | Video stitching | `nix develop` (existing) |

---

## Testing Strategy

| Component | Test Coverage |
|-----------|---------------|
| Snapshot capture | Manual verification (visual output) |
| Checkpoint capture | Manual verification (visual output) |
| Video stitching | Manual verification (video plays correctly) |
| Report generation | Unit test for markdown format |

---

## Deliverables

- [x] `verification/` directory structure with `.gitignore`
- [x] `.justfiles/verify.just` module with all commands
- [x] `verification/snapshots.json` definition file
- [x] `verification/checkpoints.json` definition file
- [x] Report generation script
- [x] Documentation in CLAUDE.md

---

# Phase 2: Story-Scoped Verification

> Link artifacts to story acceptance criteria to complete the coherence loop.

## Overview

Phase 1 captures artifacts globally. Phase 2 adds story-scoped verification where each story's acceptance criteria can reference specific artifacts, and the report shows coverage.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Verification location** | Story file (inline) | Single source of truth; criteria and verification together |
| **Scope field** | Single `scope` replacing `epics` + `milestone` | Simpler hierarchy; one field for report path |
| **Report per story** | `reports/<scope>/<id>.md` | Organized by epic/milestone; matches board hierarchy |

---

## Story Frontmatter Schema

Replace `epics` array and `milestone` field with single `scope` field:

```yaml
---
id: FEAT0109
title: Board generator grouped layout
type: feat
status: done
priority: high
scope: coherence-verification/01-artifact-pipeline  # epic/milestone or just epic
depends: []
estimate: 2h
created: 2026-01-17
---
```

**Scope values:**
| Value | Meaning |
|-------|---------|
| `epic/milestone` | Story belongs to epic and milestone |
| `epic` | Story belongs to epic (no milestone) |
| (empty) | Ad-hoc story |

---

## Verification Annotations

Add HTML comments to acceptance criteria referencing artifacts:

```markdown
## Acceptance Criteria

- [ ] Sessions page displays list <!-- verify: snapshot:sessions -->
- [ ] Clicking row opens detail <!-- verify: checkpoint:view-session-detail -->
- [ ] CLI help shows commands <!-- verify: video:cli/help -->
```

**Annotation format:** `<!-- verify: <type>:<name> -->`

| Type | Lookup |
|------|--------|
| `snapshot:<name>` | `snapshots.json` by name |
| `checkpoint:<name>` | `checkpoints.json` by name |
| `video:<path>` | Video file at path |

---

## Commands

| Command | Description |
|---------|-------------|
| `just verify story <ID>` | Capture artifacts for story, generate report |
| `just verify story-report <ID>` | Generate report only (artifacts exist) |

**Story command flow:**
1. Find story file by ID
2. Parse `scope` from frontmatter
3. Extract criteria with `<!-- verify: -->` annotations
4. Capture only referenced artifacts
5. Generate report at `verification/reports/<scope>/<id>.md`

---

## Directory Structure (Updated)

```
verification/
├── snapshots.json
├── checkpoints.json
├── report.md                           # Global report
├── reports/                            # Story-specific reports (COMMITTED)
│   ├── coherence-verification/
│   │   └── 01-artifact-pipeline/
│   │       └── FEAT0109.md
│   └── core/
│       └── BUG0001.md
├── snapshots/
├── checkpoints/
└── videos/
```

---

## Story Report Format

See [`verification/templates/story-report.md`](../../../../../verification/templates/story-report.md) for the report template.

Reports are generated at `verification/reports/<scope>/<id>.md`.

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Artifact not in definitions | Report shows `❌ (not defined)` |
| Artifact capture fails | Report shows `❌ (capture failed)` |
| No verify annotations | Report shows "No verification annotations found" |
| Story file not found | Command fails with clear error |

---

## Deliverables (Phase 2)

- [x] Update story frontmatter schema (`scope` replaces `epics` + `milestone`)
- [x] Migrate all existing stories to new schema
- [x] Update CONVENTIONS.md with new schema
- [x] Add verification annotation parsing
- [x] Implement `just verify story <ID>` command
- [x] Implement `just verify story-report <ID>` command
- [x] Update `.gitignore` to commit `reports/**/*.md`

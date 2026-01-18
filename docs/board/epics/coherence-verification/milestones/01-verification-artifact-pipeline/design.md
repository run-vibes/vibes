# Milestone 01: Artifact Pipeline - Design Document

> Capture system behavior as visual artifacts that can be traced back to story specifications.

## Overview

The artifact pipeline generates verification artifacts at three tiers of fidelity, runs as part of `just pre-commit`, and produces a report linking artifacts to story acceptance criteria.

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
| Video capture (CLI) | `cli/recordings/` | VHS tape recordings of CLI workflows |
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
    ├── cli/                     # VHS recordings
    │   └── session-create.mp4
    ├── web/                     # Playwright recordings
    │   └── session-flow.webm
    └── stitched/                # Combined CLI + Web
        └── full-session-flow.mp4
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

### Pre-commit Integration

Update `just pre-commit` to include verification:

```bash
# Current
fmt-check → clippy → tests → typecheck

# New
fmt-check → clippy → tests → typecheck → verify all
```

---

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

Existing VHS tapes in `cli/recordings/tapes/` produce GIFs. Modify to also produce MP4:

```tape
Output session-create.mp4
...
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

Combine side-by-side or sequentially:

```bash
# Side-by-side (CLI left, Web right)
ffmpeg -i cli/session-create.mp4 -i web/session-flow.webm \
  -filter_complex "[0:v][1:v]hstack=inputs=2" \
  stitched/full-session-flow.mp4

# Sequential (CLI first, then Web)
ffmpeg -f concat -i videos.txt -c copy stitched/full-session-flow.mp4
```

---

## Report Format

`verification/report.md`:

```markdown
# Verification Report

Generated: 2026-01-17T14:30:00Z
Branch: feat/0042-session-export
Stories: [FEAT][0042]-add-session-export

## Summary

| Tier | Count | Status |
|------|-------|--------|
| Snapshots | 5 | ✅ |
| Checkpoints | 3 | ✅ |
| Videos | 2 | ✅ |

## Artifacts

### Snapshots (Tier 1)
- dashboard.png
- session-list.png
- session-detail.png

### Checkpoints (Tier 2)
- create-session (3 steps)
- export-session (4 steps)

### Videos (Tier 3)
- full-session-flow.mp4 (CLI + Web stitched)

## Story Coverage

### [FEAT][0042]-add-session-export

| Criterion | Artifact | Status |
|-----------|----------|--------|
| User can navigate to session list | 01-sessions-page.png | ✅ |
| Export button visible | 02-after-click.png | ✅ |
| CSV download works | *manual verification* | ⚠️ |

Coverage: 2/3 (67%)
```

---

## PR Integration

### PR Description Template

When creating a PR, include verification summary:

```markdown
## Summary
- Added session export feature

## Verification Report
See [verification/report.md](verification/report.md) for full details.

### Coverage
- ✅ 3/4 acceptance criteria verified
- 📸 12 snapshots captured
- 🎬 2 flow videos recorded

### Uncovered
- [ ] CSV download works (manual verification needed)

## Test Plan
- [x] `just pre-commit` passes
- [x] Verification artifacts reviewed locally
```

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

- [ ] `verification/` directory structure with `.gitignore`
- [ ] `.justfiles/verify.just` module with all commands
- [ ] `verification/snapshots.json` definition file
- [ ] `verification/checkpoints.json` definition file
- [ ] Pre-commit integration
- [ ] Report generation script
- [ ] Documentation in CLAUDE.md

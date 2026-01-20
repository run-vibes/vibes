---
id: CHORE0121
title: Kanban Planning Board - Implementation Plan
type: chore
status: done
priority: medium
scope: core/07-visual-project-planning
depends: []
estimate:
created: 2026-01-07
---
# Kanban Planning Board - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate from `docs/plans/` to kanban-style `docs/board/` where directory location = workflow status.

**Architecture:** Shell-based `just board` commands manage items. Auto-generated README.md serves as board view. Minimal tooling, maximum clarity.

**Tech Stack:** Just (task runner), Bash scripts, Markdown

**Design:** [2026-01-01-kanban-board-design.md](2026-01-01-kanban-board-design.md)

---

## Task 1: Create Board Module Structure

**Files:**
- Create: `.justfiles/board.just`
- Modify: `justfile`

**Step 1: Create .justfiles directory**

```bash
mkdir -p .justfiles
```

**Step 2: Create board.just with core commands**

Create `.justfiles/board.just`:

```just
# Board management commands
# Usage: just board, just board new, just board start, etc.

board_dir := "docs/board"

# Default: regenerate the board README
default:
    @just board generate

# Generate README.md from directory structure
generate:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "{{board_dir}}"

    echo "# Planning Board"
    echo ""
    echo "> Auto-generated from directory structure. Run \`just board\` to update."
    echo ""

    # In Progress
    echo "## In Progress"
    echo ""
    if [[ -d in-progress ]] && [[ -n "$(ls -A in-progress 2>/dev/null)" ]]; then
        for item in in-progress/*/; do
            [[ -d "$item" ]] || continue
            name=$(basename "$item")
            echo "### $name"
            if [[ -f "${item}design.md" ]]; then
                # Extract first non-empty, non-heading line as description
                desc=$(grep -m1 "^>" "${item}design.md" 2>/dev/null | sed 's/^> *//' || echo "")
                [[ -n "$desc" ]] && echo "$desc"
            fi
            # List stories if present
            if [[ -d "${item}stories" ]]; then
                echo ""
                echo "Stories:"
                for story in "${item}stories"/*.md; do
                    [[ -f "$story" ]] || continue
                    sname=$(basename "$story" .md)
                    echo "- [ ] $sname"
                done
            fi
            echo ""
        done
        for item in in-progress/*.md; do
            [[ -f "$item" ]] || continue
            name=$(basename "$item" .md)
            echo "- [~] $name"
        done
    else
        echo "*No items in progress*"
    fi
    echo ""
    echo "---"
    echo ""

    # Ready
    echo "## Ready"
    echo ""
    if [[ -d ready ]] && [[ -n "$(ls -A ready 2>/dev/null)" ]]; then
        for item in ready/*/; do
            [[ -d "$item" ]] || continue
            name=$(basename "$item")
            echo "### $name"
            if [[ -d "${item}stories" ]]; then
                for story in "${item}stories"/*.md; do
                    [[ -f "$story" ]] || continue
                    sname=$(basename "$story" .md)
                    echo "- [ ] $sname"
                done
            fi
            echo ""
        done
        for item in ready/*.md; do
            [[ -f "$item" ]] || continue
            name=$(basename "$item" .md)
            echo "- [ ] $name"
        done
    else
        echo "*No items ready*"
    fi
    echo ""
    echo "---"
    echo ""

    # Review
    echo "## Review"
    echo ""
    if [[ -d review ]] && [[ -n "$(ls -A review 2>/dev/null)" ]]; then
        for item in review/*.md review/*/; do
            [[ -e "$item" ]] || continue
            if [[ -d "$item" ]]; then
                name=$(basename "$item")
            else
                name=$(basename "$item" .md)
            fi
            echo "- [x] $name"
        done
    else
        echo "*No items in review*"
    fi
    echo ""
    echo "---"
    echo ""

    # Backlog
    echo "## Backlog"
    echo ""
    if [[ -d backlog ]] && [[ -n "$(ls -A backlog 2>/dev/null)" ]]; then
        for item in backlog/*.md backlog/*/; do
            [[ -e "$item" ]] || continue
            if [[ -d "$item" ]]; then
                name=$(basename "$item")
                if [[ ! -f "${item}implementation.md" ]]; then
                    echo "- $name *(design only)*"
                else
                    echo "- $name"
                fi
            else
                name=$(basename "$item" .md)
                echo "- $name"
            fi
        done
    else
        echo "*Backlog empty*"
    fi
    echo ""
    echo "---"
    echo ""

    # Done (collapsed)
    echo "## Done"
    echo ""
    echo "<details>"
    echo "<summary>View completed work</summary>"
    echo ""
    if [[ -d done ]] && [[ -n "$(ls -A done 2>/dev/null)" ]]; then
        for item in done/*.md done/*/; do
            [[ -e "$item" ]] || continue
            if [[ -d "$item" ]]; then
                name=$(basename "$item")
            else
                name=$(basename "$item" .md)
            fi
            echo "- $name"
        done
    else
        echo "*No completed items*"
    fi
    echo ""
    echo "</details>"

# Show board status summary
status:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    echo "Planning Board Status:"
    echo "  In Progress: $(find in-progress -maxdepth 1 -mindepth 1 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Ready:       $(find ready -maxdepth 1 -mindepth 1 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Review:      $(find review -maxdepth 1 -mindepth 1 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Backlog:     $(find backlog -maxdepth 1 -mindepth 1 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Done:        $(find done -maxdepth 1 -mindepth 1 2>/dev/null | wc -l | tr -d ' ')"
```

**Step 3: Add mod statement to root justfile**

Add after line 1 in `justfile`:

```just
# Planning board management
mod board '.justfiles/board.just'
```

**Step 4: Verify module loads**

Run: `just board status`
Expected: Shows "Planning Board Status:" with counts (will show 0s until directories created)

**Step 5: Commit**

```bash
git add .justfiles/board.just justfile
git commit -m "feat: add modular board.just for planning management"
```

---

## Task 2: Create Board Directory Structure

**Files:**
- Create: `docs/board/` with subdirectories
- Create: `docs/board/CHANGELOG.md`

**Step 1: Create directory structure**

```bash
mkdir -p docs/board/{backlog,ready,in-progress,review,done}
```

**Step 2: Create initial CHANGELOG.md**

Create `docs/board/CHANGELOG.md`:

```markdown
# Changelog

> Updated when items move to done. Most recent first.

| Date | Item | Summary |
|------|------|---------|
| 2026-01-01 | Initial migration | Migrated 19 milestones from docs/plans/ to kanban board structure |
```

**Step 3: Verify structure**

Run: `ls -la docs/board/`
Expected: Shows backlog, ready, in-progress, review, done directories and CHANGELOG.md

**Step 4: Commit**

```bash
git add docs/board/
git commit -m "feat: create docs/board/ kanban structure"
```

---

## Task 3: Add Board Item Management Commands

**Files:**
- Modify: `.justfiles/board.just`

**Step 1: Add next-id helper function**

Append to `.justfiles/board.just`:

```just
# Get next available ID for a type (feat, bug, chore)
_next-id TYPE:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    max=0
    for f in $(find . -name "{{TYPE}}-*.md" -o -type d -name "{{TYPE}}-*" 2>/dev/null); do
        num=$(basename "$f" | sed -n 's/^{{TYPE}}-\([0-9]*\).*/\1/p')
        if [[ -n "$num" ]] && [[ "$num" -gt "$max" ]]; then
            max=$num
        fi
    done
    printf "%04d" $((max + 1))

# Get next milestone number
_next-milestone:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    max=0
    for d in $(find . -type d -name "milestone-*" 2>/dev/null); do
        num=$(basename "$d" | sed -n 's/^milestone-\([0-9]*\).*/\1/p')
        if [[ -n "$num" ]] && [[ "$num" -gt "$max" ]]; then
            max=$num
        fi
    done
    printf "%02d" $((max + 1))
```

**Step 2: Add new command**

Append to `.justfiles/board.just`:

```just
# Create new item: just board new <type> "description"
# Types: feat, bug, chore, milestone
new TYPE DESC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    type="{{TYPE}}"
    desc="{{DESC}}"
    slug=$(echo "$desc" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')

    case "$type" in
        feat|bug|chore)
            id=$(just board _next-id "$type")
            filename="backlog/${type}-${id}-${slug}.md"
            cat > "$filename" << EOF
---
created: $(date +%Y-%m-%d)
---

# ${type}-${id}: ${desc}

## Summary

[Description of the ${type}]

## Implementation

[Steps to implement]
EOF
            echo "✓ Created: $filename"
            ;;
        milestone)
            id=$(just board _next-milestone)
            dirname="backlog/milestone-${id}-${slug}"
            mkdir -p "$dirname"
            cat > "${dirname}/design.md" << EOF
---
created: $(date +%Y-%m-%d)
---

# Milestone ${id}: ${desc} - Design

> [One-line summary]

## Overview

[Description of the milestone]

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| | | |

## Architecture

[Technical approach]

## Deliverables

- [ ] Item 1
- [ ] Item 2
EOF
            echo "✓ Created: ${dirname}/design.md"
            ;;
        *)
            echo "Error: Unknown type '$type'. Use: feat, bug, chore, milestone"
            exit 1
            ;;
    esac

    just board generate > README.md
    echo "✓ Board updated"
```

**Step 3: Add move commands**

Append to `.justfiles/board.just`:

```just
# Find item by name pattern and return its current path
_find-item PATTERN:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    # Search in order: backlog, ready, in-progress, review
    for dir in backlog ready in-progress review; do
        # Check for directory match
        match=$(find "$dir" -maxdepth 1 -type d -name "*{{PATTERN}}*" 2>/dev/null | head -1)
        [[ -n "$match" ]] && echo "$match" && exit 0
        # Check for file match
        match=$(find "$dir" -maxdepth 1 -type f -name "*{{PATTERN}}*.md" 2>/dev/null | head -1)
        [[ -n "$match" ]] && echo "$match" && exit 0
    done
    echo ""

# Start work on an item (move to in-progress)
start ITEM:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path=$(just board _find-item "{{ITEM}}")
    if [[ -z "$path" ]]; then
        echo "Error: Item '{{ITEM}}' not found in backlog or ready"
        exit 1
    fi

    name=$(basename "$path")
    dest="in-progress/$name"

    if [[ -e "$dest" ]]; then
        echo "Error: '$name' is already in progress"
        exit 1
    fi

    mv "$path" "in-progress/"
    echo "✓ Started: $name"

    just board generate > README.md
    echo "✓ Board updated"

# Move item to review
review ITEM:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path="in-progress/{{ITEM}}"
    if [[ ! -e "$path" ]]; then
        # Try pattern match
        path=$(find in-progress -maxdepth 1 \( -type d -o -type f \) -name "*{{ITEM}}*" 2>/dev/null | head -1)
    fi

    if [[ -z "$path" ]] || [[ ! -e "$path" ]]; then
        echo "Error: Item '{{ITEM}}' not found in in-progress"
        exit 1
    fi

    name=$(basename "$path")
    mv "$path" "review/"
    echo "✓ Moved to review: $name"

    just board generate > README.md
    echo "✓ Board updated"

# Complete item (move to done + changelog entry)
done ITEM:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    # Find in review first, then in-progress
    path=""
    for dir in review in-progress; do
        found=$(find "$dir" -maxdepth 1 \( -type d -o -type f \) -name "*{{ITEM}}*" 2>/dev/null | head -1)
        if [[ -n "$found" ]]; then
            path="$found"
            break
        fi
    done

    if [[ -z "$path" ]] || [[ ! -e "$path" ]]; then
        echo "Error: Item '{{ITEM}}' not found in review or in-progress"
        exit 1
    fi

    name=$(basename "$path" .md)
    [[ -d "$path" ]] && name=$(basename "$path")

    # Prompt for changelog entry
    echo "Enter changelog summary for '$name' (one line):"
    read -r summary

    # Add to changelog
    date=$(date +%Y-%m-%d)
    # Insert after header row in changelog table
    sed -i "4a\\| $date | $name | $summary |" CHANGELOG.md

    mv "$path" "done/"
    echo "✓ Completed: $name"
    echo "✓ Added to CHANGELOG.md"

    just board generate > README.md
    echo "✓ Board updated"
```

**Step 4: Test new command**

Run: `just board new feat "test feature"`
Expected: Creates `docs/board/backlog/feat-0001-test-feature.md`

Run: `rm docs/board/backlog/feat-0001-test-feature.md`

**Step 5: Commit**

```bash
git add .justfiles/board.just
git commit -m "feat: add board item management commands (new, start, review, done)"
```

---

## Task 4: Migrate Completed Milestones

**Files:**
- Move: `docs/plans/01-*` through `docs/plans/13-*` → `docs/board/done/`
- Move: `docs/plans/15-*` through `docs/plans/19-*` → `docs/board/done/`

**Step 1: Move completed milestones to done**

```bash
cd docs

# Phase 1 milestones (complete)
mv plans/01-core-proxy board/done/milestone-01-core-proxy
mv plans/02-cli board/done/milestone-02-cli
mv plans/03-plugin-foundation board/done/milestone-03-plugin-foundation
mv plans/04-server-web-ui board/done/milestone-04-server-web-ui

# Phase 2 milestones (complete)
mv plans/05-cloudflare-tunnel board/done/milestone-05-cloudflare-tunnel
mv plans/06-cloudflare-access board/done/milestone-06-cloudflare-access
mv plans/07-push-notifications board/done/milestone-07-push-notifications

# Phase 3 milestones (complete)
mv plans/08-chat-history board/done/milestone-08-chat-history
mv plans/09-multi-session board/done/milestone-09-multi-session
mv plans/10-cli-web-mirroring board/done/milestone-10-cli-web-mirroring
mv plans/11-test-coverage board/done/milestone-11-test-coverage
mv plans/12-pty-backend board/done/milestone-12-pty-backend
mv plans/13-scrollback-simplification board/done/milestone-13-scrollback-simplification

# Infrastructure milestones (complete)
mv plans/15-harness-introspection board/done/milestone-15-harness-introspection
mv plans/16-iggy-bundling board/done/milestone-16-iggy-bundling
mv plans/17-web-ui-modernization board/done/milestone-17-web-ui-modernization
mv plans/18-eventlog-wiring board/done/milestone-18-eventlog-wiring
mv plans/18-iggy-sdk-integration board/done/milestone-18-iggy-sdk-integration
mv plans/19-event-cli board/done/milestone-19-event-cli

# Standalone bug fix
mv plans/2025-12-29-fix-cwd-propagation.md board/done/bug-0001-cwd-propagation.md

cd ..
```

**Step 2: Verify migration**

Run: `ls docs/board/done/ | wc -l`
Expected: 20 items

**Step 3: Commit**

```bash
git add docs/plans docs/board/done
git commit -m "chore: migrate completed milestones to docs/board/done/"
```

---

## Task 5: Migrate In-Progress Work

**Files:**
- Move: `docs/plans/14-continual-learning/` → `docs/board/in-progress/`

**Step 1: Move continual-learning milestone**

```bash
mv docs/plans/14-continual-learning docs/board/in-progress/milestone-14-continual-learning
```

**Step 2: Move design documents to in-progress**

The kanban board design docs should stay with the migration until complete:

```bash
mv docs/plans/2026-01-01-kanban-board-design.md docs/board/in-progress/
mv docs/plans/2026-01-01-kanban-board-implementation.md docs/board/in-progress/
```

**Step 3: Verify in-progress has content**

Run: `ls docs/board/in-progress/`
Expected: milestone-14-continual-learning directory and kanban design files

**Step 4: Commit**

```bash
git add docs/plans docs/board/in-progress
git commit -m "chore: migrate in-progress work to docs/board/in-progress/"
```

---

## Task 6: Create Backlog Items for Future Work

**Files:**
- Create: `docs/board/backlog/milestone-20-setup-wizards/`
- Create: `docs/board/backlog/milestone-21-default-plugins/`
- Create: `docs/board/backlog/milestone-22-cli-enhancements/`
- Create: `docs/board/backlog/milestone-23-ios-app/`

**Step 1: Create future milestone stubs**

```bash
mkdir -p docs/board/backlog/milestone-20-setup-wizards
cat > docs/board/backlog/milestone-20-setup-wizards/design.md << 'EOF'
---
created: 2026-01-01
---

# Milestone 20: Setup Wizards - Design

> Interactive setup wizards for tunnel and authentication configuration.

## Overview

Phase 5.1 from roadmap. Guide users through cloudflared installation,
tunnel configuration, and Cloudflare Access setup.

## Deliverables

- [ ] Interactive `vibes tunnel setup` wizard
- [ ] Interactive `vibes auth setup` wizard
- [ ] Auto-detect team/AUD from tunnel config
- [ ] Connectivity testing and validation
EOF

mkdir -p docs/board/backlog/milestone-21-default-plugins
cat > docs/board/backlog/milestone-21-default-plugins/design.md << 'EOF'
---
created: 2026-01-01
---

# Milestone 21: Default Plugins - Design

> Built-in plugins for analytics, templates, and export.

## Overview

Phase 5.2 from roadmap. Ship useful plugins out of the box.

## Deliverables

- [ ] analytics plugin (session stats, token usage)
- [ ] templates plugin (prompt templates/snippets)
- [ ] export plugin (session export to markdown/JSON)
EOF

mkdir -p docs/board/backlog/milestone-22-cli-enhancements
cat > docs/board/backlog/milestone-22-cli-enhancements/design.md << 'EOF'
---
created: 2026-01-01
---

# Milestone 22: CLI Enhancements - Design

> Polish CLI with tab completion and interactive pickers.

## Overview

Phase 5.3 from roadmap.

## Deliverables

- [ ] Tab completion for commands and arguments
- [ ] Interactive session picker
EOF

mkdir -p docs/board/backlog/milestone-23-ios-app
cat > docs/board/backlog/milestone-23-ios-app/design.md << 'EOF'
---
created: 2026-01-01
---

# Milestone 23: iOS App - Design

> Native iOS app for mobile access.

## Overview

Phase 5.4 from roadmap.

## Deliverables

- [ ] Swift native app with xterm.js WebView
- [ ] Push notification integration
- [ ] Session list and attach
EOF
```

**Step 2: Verify backlog**

Run: `ls docs/board/backlog/`
Expected: 4 milestone directories

**Step 3: Commit**

```bash
git add docs/board/backlog/
git commit -m "chore: create backlog items for Phase 5 milestones"
```

---

## Task 7: Move PLAN.md to CONVENTIONS.md

**Files:**
- Move: `docs/PLAN.md` → `docs/board/CONVENTIONS.md`
- Delete: `docs/PROGRESS.md`

**Step 1: Move and update PLAN.md**

```bash
mv docs/PLAN.md docs/board/CONVENTIONS.md
```

**Step 2: Update references in CONVENTIONS.md**

Edit `docs/board/CONVENTIONS.md` to update the header and remove obsolete references to the old structure. Change the opening to reference the board:

```markdown
# Planning Conventions

This document describes how to use the kanban planning board at `docs/board/`.

## Board Structure

```
docs/board/
├── README.md          # Auto-generated board view
├── CHANGELOG.md       # Updated when items complete
├── CONVENTIONS.md     # This file
├── backlog/           # Future work
├── ready/             # Designed, ready to implement
├── in-progress/       # Currently being worked on
├── review/            # Awaiting review/merge
└── done/              # Completed work
```

## Commands

| Command | Action |
|---------|--------|
| `just board` | Regenerate README.md |
| `just board new feat "desc"` | Create feature in backlog |
| `just board new milestone "name"` | Create milestone in backlog |
| `just board start <item>` | Move to in-progress |
| `just board review <item>` | Move to review |
| `just board done <item>` | Move to done + changelog |
| `just board status` | Show counts per column |

---

[... rest of existing content about design docs, implementation plans, etc. ...]
```

**Step 3: Delete PROGRESS.md**

```bash
rm docs/PROGRESS.md
```

**Step 4: Remove empty plans directory**

```bash
rmdir docs/plans
```

**Step 5: Commit**

```bash
git add docs/
git commit -m "chore: move PLAN.md to CONVENTIONS.md, delete PROGRESS.md"
```

---

## Task 8: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update planning section**

Replace the "Development Workflow" and related sections in CLAUDE.md with:

```markdown
## Planning & Tracking

**Use the board to track all work:**

```bash
just board                        # Regenerate board view
just board new feat "description" # Create new feature
just board new milestone "name"   # Create new milestone
just board start <item>           # Begin work (→ in-progress)
just board review <item>          # Ready for review (→ review)
just board done <item>            # Complete (→ done + changelog)
just board status                 # Show counts per column
```

**Before starting any task:**
1. Check `docs/board/in-progress/` for current work
2. If starting new work, use `just board start` or `just board new`

**Board structure:**
```
docs/board/
├── backlog/       # Future work and ideas
├── ready/         # Designed, ready to implement
├── in-progress/   # Currently being worked on
├── review/        # Awaiting review/merge
└── done/          # Completed work
```

See [docs/board/CONVENTIONS.md](docs/board/CONVENTIONS.md) for full planning conventions.
```

**Step 2: Update completion workflow**

Update the "Completing Implementation Work" section:

```markdown
## Completing Work

**REQUIRED:** When implementation is complete:

1. **Verify quality:**
   - Run `just test` — all tests pass
   - Run `just pre-commit` — fmt, clippy, tests

2. **Update the board:**
   - Run `just board done <item>`
   - Enter a one-line changelog summary when prompted
   - This moves the item to `done/` and updates CHANGELOG.md

3. **Commit and push:**
   - Commit with conventional commit message
   - Push to origin: `git push -u origin <branch-name>`

4. **Create PR:**
   - `gh pr create --title "<type>: <description>" --body "..."`

**Never leave completed work:**
- Uncommitted or unpushed
- Still in `in-progress/` after merging
```

**Step 3: Update references**

Search and replace in CLAUDE.md:
- `docs/PROGRESS.md` → `docs/board/README.md`
- `docs/PLAN.md` → `docs/board/CONVENTIONS.md`
- `docs/plans/` → `docs/board/`

**Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with board workflow"
```

---

## Task 9: Update Root README.md

**Files:**
- Modify: `README.md`

**Step 1: Find and update planning references**

Search for references to docs/plans and PROGRESS.md in README.md and update them:
- `docs/plans/` → `docs/board/`
- `PROGRESS.md` → `docs/board/README.md`
- Add link to planning board if not present

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README.md links to board structure"
```

---

## Task 10: Generate Initial Board and Finalize

**Files:**
- Generate: `docs/board/README.md`

**Step 1: Generate board README**

```bash
just board > docs/board/README.md
```

**Step 2: Verify board content**

Run: `cat docs/board/README.md | head -50`
Expected: Shows "# Planning Board" with In Progress, Ready, Review, Backlog, Done sections

**Step 3: Verify just board status**

Run: `just board status`
Expected: Shows counts (In Progress: 3, Backlog: 4, Done: 20, etc.)

**Step 4: Move kanban design docs to done**

Now that migration is complete, move the kanban design docs to done:

```bash
just board done "kanban-board"
# Enter summary: "Migrate planning to kanban board structure"
```

**Step 5: Final commit**

```bash
git add docs/board/README.md docs/board/CHANGELOG.md
git commit -m "chore: generate initial board view, complete migration"
```

---

## Summary

After completing all tasks:

- **Created:** `.justfiles/board.just` with full command set
- **Created:** `docs/board/` kanban structure
- **Migrated:** 20 completed items to `done/`
- **Migrated:** 1 in-progress milestone + design docs
- **Created:** 4 backlog items for Phase 5
- **Updated:** CLAUDE.md with board workflow
- **Updated:** README.md references
- **Deleted:** `docs/plans/`, `docs/PROGRESS.md`
- **Moved:** `docs/PLAN.md` → `docs/board/CONVENTIONS.md`

Total commits: 10

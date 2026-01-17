# Board Restructure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate the kanban board to support epics, symlinked stories, and flexible relationships.

**Architecture:** Stories live in stage directories as canonical files. Epics and milestones use symlinks to reference them. Tooling manages symlink maintenance automatically.

**Tech Stack:** Bash scripts, justfile recipes, YAML frontmatter

---

## Migration Scope

- 26 completed milestones
- 1 in-progress milestone (10 stories)
- 11 backlog milestones (9 stories in one)
- 3 standalone done items
- ~105 markdown files total

---

## Task 1: Create New Directory Structure

**Files:**
- Create: `docs/board/stages/backlog/stories/.gitkeep`
- Create: `docs/board/stages/in-progress/stories/.gitkeep`
- Create: `docs/board/stages/done/stories/.gitkeep`
- Create: `docs/board/epics/README.md`
- Create: `docs/board/milestones/README.md`
- Create: `docs/board/templates/story.md`
- Create: `docs/board/templates/epic.md`
- Create: `docs/board/templates/milestone.md`

**Step 1: Create stage directories**

```bash
cd docs/board
mkdir -p stages/backlog/stories
mkdir -p stages/in-progress/stories
mkdir -p stages/done/stories
touch stages/backlog/stories/.gitkeep
touch stages/in-progress/stories/.gitkeep
touch stages/done/stories/.gitkeep
```

**Step 2: Create epic and milestone directories**

```bash
mkdir -p epics milestones templates
```

**Step 3: Create epics index README**

```bash
cat > epics/README.md << 'EOF'
---
generated: true
---

# Epics

> Auto-generated index. Run `just board generate` to update.

| Epic | Status | Stories |
|------|--------|---------|
EOF
```

**Step 4: Create milestones index README**

```bash
cat > milestones/README.md << 'EOF'
---
generated: true
---

# Milestones

> Auto-generated index. Run `just board generate` to update.

| Milestone | Status | Progress |
|-----------|--------|----------|
EOF
```

**Step 5: Commit structure**

```bash
git add stages/ epics/ milestones/ templates/
git commit -m "chore(board): create new directory structure for epics/milestones"
```

---

## Task 2: Create Templates

**Files:**
- Create: `docs/board/templates/story.md`
- Create: `docs/board/templates/epic.md`
- Create: `docs/board/templates/milestone.md`

**Step 1: Create story template**

```bash
cat > templates/story.md << 'EOF'
---
id: ${ID}
title: ${TITLE}
type: ${TYPE}
status: backlog
priority: medium
epics: []
depends: []
estimate:
created: ${DATE}
updated: ${DATE}
---

# ${TITLE}

## Summary

[Description of the work]

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2

## Implementation Notes

[Technical approach]
EOF
```

**Step 2: Create epic template**

```bash
cat > templates/epic.md << 'EOF'
---
id: ${ID}
title: ${TITLE}
status: active
description: ${DESCRIPTION}
---

# ${TITLE}

${DESCRIPTION}

## Stories

> Auto-generated from symlinks. Stories are linked via `just board link <story> <epic>`.

EOF
```

**Step 3: Create milestone template**

```bash
cat > templates/milestone.md << 'EOF'
---
id: ${ID}
title: ${TITLE}
status: planned
epics: []
---

# ${TITLE}

## Overview

[Description of the milestone]

## Epics

> Epics linked to this milestone via `just board link-epic <epic> <milestone>`.

## Design

See [design.md](design.md) for architecture decisions.
EOF
```

**Step 4: Commit templates**

```bash
git add templates/
git commit -m "chore(board): add story, epic, and milestone templates"
```

---

## Task 3: Create Initial Epics

**Files:**
- Create: `docs/board/epics/core/README.md`
- Create: `docs/board/epics/cli/README.md`
- Create: `docs/board/epics/web-ui/README.md`
- Create: `docs/board/epics/plugin-system/README.md`
- Create: `docs/board/epics/networking/README.md`
- Create: `docs/board/epics/mobile/README.md`

**Step 1: Create core epic**

```bash
mkdir -p epics/core
cat > epics/core/README.md << 'EOF'
---
id: core
title: Core Infrastructure
status: active
description: Proxy, PTY, event system, storage, sessions
---

# Core Infrastructure

Foundation systems: proxy server, PTY backend, event bus, storage layer, session management.
EOF
```

**Step 2: Create cli epic**

```bash
mkdir -p epics/cli
cat > epics/cli/README.md << 'EOF'
---
id: cli
title: CLI Experience
status: active
description: Commands, user experience, output formatting
---

# CLI Experience

Command-line interface: commands, flags, output formatting, interactive features.
EOF
```

**Step 3: Create web-ui epic**

```bash
mkdir -p epics/web-ui
cat > epics/web-ui/README.md << 'EOF'
---
id: web-ui
title: Web UI
status: active
description: Dashboard, firehose, CRT design system
---

# Web UI

Web dashboard: session views, firehose, navigation, CRT visual design.
EOF
```

**Step 4: Create plugin-system epic**

```bash
mkdir -p epics/plugin-system
cat > epics/plugin-system/README.md << 'EOF'
---
id: plugin-system
title: Plugin System
status: active
description: Plugin API, lifecycle, loading
---

# Plugin System

Plugin architecture: API contracts, lifecycle management, dynamic loading.
EOF
```

**Step 5: Create networking epic**

```bash
mkdir -p epics/networking
cat > epics/networking/README.md << 'EOF'
---
id: networking
title: Networking
status: active
description: Tunnels, auth, security
---

# Networking

Network layer: Cloudflare tunnels, authentication, access control, security.
EOF
```

**Step 6: Create mobile epic**

```bash
mkdir -p epics/mobile
cat > epics/mobile/README.md << 'EOF'
---
id: mobile
title: Mobile
status: active
description: iOS app
---

# Mobile

Mobile applications: iOS app, push notifications, mobile-specific features.
EOF
```

**Step 7: Commit epics**

```bash
git add epics/
git commit -m "chore(board): create initial epics (core, cli, web-ui, plugin-system, networking, mobile)"
```

---

## Task 4: Write Migration Script

**Files:**
- Create: `docs/board/migrate.sh`

**Step 1: Write migration script**

This script will:
1. Extract stories from milestone directories
2. Convert to new frontmatter format
3. Create symlinks in epics
4. Convert milestones to new format

```bash
cat > docs/board/migrate.sh << 'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

# Board migration script
# Migrates from nested milestone structure to flat stories with symlinks

BOARD_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BOARD_DIR"

echo "=== Board Migration ==="
echo ""

# Epic assignments based on milestone content
# Format: milestone_pattern:epic1,epic2
declare -A EPIC_MAP=(
    ["core-proxy"]="core,networking"
    ["cli"]="cli"
    ["plugin"]="plugin-system"
    ["web-ui"]="web-ui"
    ["tunnel"]="networking"
    ["access"]="networking"
    ["push-notification"]="mobile,networking"
    ["chat-history"]="core"
    ["multi-session"]="core"
    ["mirroring"]="cli,web-ui"
    ["test-coverage"]="core"
    ["pty"]="core"
    ["scrollback"]="core"
    ["kanban"]="core"
    ["introspection"]="core"
    ["iggy"]="core"
    ["modernization"]="web-ui"
    ["eventlog"]="core"
    ["event-cli"]="cli,core"
    ["storage"]="core"
    ["security"]="networking"
    ["api-extension"]="plugin-system"
    ["capture-inject"]="core"
    ["assessment"]="core,cli,plugin-system"
    ["firehose"]="web-ui"
    ["crt"]="web-ui"
    ["learning"]="plugin-system"
    ["attribution"]="plugin-system"
    ["adaptive"]="plugin-system"
    ["groove"]="plugin-system"
    ["open-world"]="plugin-system"
    ["setup-wizard"]="cli"
    ["default-plugin"]="plugin-system"
    ["enhancement"]="cli"
    ["ios"]="mobile"
)

# Get epic for a milestone name
get_epics() {
    local name="$1"
    for pattern in "${!EPIC_MAP[@]}"; do
        if [[ "$name" == *"$pattern"* ]]; then
            echo "${EPIC_MAP[$pattern]}"
            return
        fi
    done
    echo "core"  # Default
}

# Extract story ID from filename
get_story_id() {
    local file="$1"
    local base=$(basename "$file" .md)
    # Extract type and number: feat-01, bug-02, chore-03
    if [[ "$base" =~ ^(feat|bug|chore|fix|refactor|docs)-([0-9]+) ]]; then
        local type="${BASH_REMATCH[1]}"
        local num="${BASH_REMATCH[2]}"
        # Normalize fix -> bug, docs -> chore for ID
        [[ "$type" == "fix" ]] && type="bug"
        [[ "$type" == "docs" ]] && type="chore"
        echo "${type^^}$(printf '%03d' $num)"
    else
        # Generate from filename
        echo "STORY-$RANDOM"
    fi
}

# Process a single story file
process_story() {
    local src="$1"
    local stage="$2"
    local milestone_name="$3"

    local base=$(basename "$src" .md)
    local dest="stages/$stage/stories/$base.md"
    local epics=$(get_epics "$milestone_name")
    local story_id=$(get_story_id "$src")

    echo "  Processing: $base -> $dest"

    # Read existing content
    local content=$(cat "$src")

    # Check if it already has frontmatter
    if [[ "$content" == ---* ]]; then
        # Update existing frontmatter
        cp "$src" "$dest"
        # Add epics field if not present
        if ! grep -q "^epics:" "$dest"; then
            sed -i "/^---$/,/^---$/ { /^status:/a epics: [$epics]
            }" "$dest"
        fi
    else
        # Add new frontmatter
        cat > "$dest" << EOF
---
id: $story_id
title: $base
type: $(echo "$base" | cut -d'-' -f1)
status: $stage
priority: medium
epics: [$epics]
depends: []
estimate:
created: $(date +%Y-%m-%d)
updated: $(date +%Y-%m-%d)
---

$content
EOF
    fi

    # Create symlinks in each epic
    IFS=',' read -ra epic_list <<< "$epics"
    for epic in "${epic_list[@]}"; do
        local link="epics/$epic/$base.md"
        local rel_target="../../stages/$stage/stories/$base.md"
        ln -sf "$rel_target" "$link"
        echo "    Linked: $link"
    done
}

# Process milestones in a stage
process_stage() {
    local stage="$1"
    local src_dir="$2"

    echo ""
    echo "=== Processing $stage ==="

    [[ ! -d "$src_dir" ]] && return

    for milestone_dir in "$src_dir"/milestone-*/; do
        [[ ! -d "$milestone_dir" ]] && continue
        local milestone_name=$(basename "$milestone_dir")
        echo ""
        echo "Milestone: $milestone_name"

        # Process stories
        if [[ -d "${milestone_dir}stories" ]]; then
            for story in "${milestone_dir}stories"/*.md; do
                [[ ! -f "$story" ]] && continue
                process_story "$story" "$stage" "$milestone_name"
            done
        fi

        # Convert milestone to new format
        local dest_milestone="milestones/$milestone_name"
        mkdir -p "$dest_milestone"

        # Copy design.md if exists
        [[ -f "${milestone_dir}design.md" ]] && cp "${milestone_dir}design.md" "$dest_milestone/"
        [[ -f "${milestone_dir}implementation.md" ]] && cp "${milestone_dir}implementation.md" "$dest_milestone/"
        [[ -f "${milestone_dir}decisions.md" ]] && cp "${milestone_dir}decisions.md" "$dest_milestone/"

        # Create milestone README with epic symlinks
        local epics=$(get_epics "$milestone_name")
        local milestone_status="done"
        [[ "$stage" == "in-progress" ]] && milestone_status="in-progress"
        [[ "$stage" == "backlog" ]] && milestone_status="planned"

        cat > "$dest_milestone/README.md" << EOF
---
id: $milestone_name
title: $(echo "$milestone_name" | sed 's/milestone-[0-9]*-//' | tr '-' ' ' | sed 's/\b\w/\u&/g')
status: $milestone_status
epics: [$epics]
---

# $(echo "$milestone_name" | sed 's/milestone-[0-9]*-//' | tr '-' ' ' | sed 's/\b\w/\u&/g')

See [design.md](design.md) for details.
EOF

        # Create symlinks to epics
        IFS=',' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            ln -sf "../../epics/$epic" "$dest_milestone/$epic"
            echo "  Linked epic: $epic"
        done
    done

    # Process standalone items (bug-*, feat-*, chore-*)
    for item in "$src_dir"/*.md; do
        [[ ! -f "$item" ]] && continue
        local base=$(basename "$item" .md)
        echo ""
        echo "Standalone: $base"
        process_story "$item" "$stage" "$base"
    done
}

# Main migration
process_stage "done" "done"
process_stage "in-progress" "in-progress"
process_stage "backlog" "backlog"

echo ""
echo "=== Migration Complete ==="
echo ""
echo "Next steps:"
echo "1. Review migrated files"
echo "2. Update board.just with new commands"
echo "3. Run: just board generate"
echo "4. Remove old directories"
SCRIPT
chmod +x docs/board/migrate.sh
```

**Step 2: Commit migration script**

```bash
git add docs/board/migrate.sh
git commit -m "chore(board): add migration script for board restructure"
```

---

## Task 5: Run Migration

**Step 1: Run migration script**

```bash
cd docs/board
./migrate.sh
```

**Step 2: Verify migration**

```bash
# Check story counts
find stages -name "*.md" -type f | wc -l

# Check symlinks in epics
find epics -type l | wc -l

# Check milestone symlinks
find milestones -type l | wc -l
```

**Step 3: Commit migrated content**

```bash
git add stages/ epics/ milestones/
git commit -m "chore(board): migrate existing content to new structure"
```

---

## Task 6: Rewrite Board Tooling

**Files:**
- Modify: `.justfiles/board.just`

**Step 1: Rewrite board.just with new commands**

The new tooling needs to:
- Show help on `just board`
- Generate README on `just board generate`
- Manage stories, epics, milestones
- Handle symlink maintenance

```bash
cat > .justfiles/board.just << 'EOF'
# Board management commands
# Usage: just board, just board new story, just board start, etc.

board_dir := source_directory() / ".." / "docs" / "board"

# Default: show available commands
default:
    @echo "Board Commands:"
    @echo ""
    @echo "  just board                  Show this help"
    @echo "  just board generate         Regenerate README.md"
    @echo "  just board status           Show counts per stage"
    @echo ""
    @echo "Story Management:"
    @echo "  just board new story \"title\"    Create story in backlog"
    @echo "  just board start <id>           Move story to in-progress"
    @echo "  just board done <id>            Move story to done"
    @echo ""
    @echo "Epic/Milestone Management:"
    @echo "  just board new epic \"name\"      Create epic"
    @echo "  just board new milestone \"name\" Create milestone"
    @echo "  just board link <story> <epic>  Link story to epic"
    @echo "  just board unlink <story> <epic> Unlink story from epic"
    @echo "  just board link-epic <epic> <milestone> Link epic to milestone"
    @echo ""
    @echo "Utilities:"
    @echo "  just board list epics       Show all epics"
    @echo "  just board list milestones  Show all milestones"
    @echo "  just board show <id>        Show story details"

# Generate README.md from directory structure
generate:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    # Helper: count stories in a stage
    count_stories() {
        find "stages/$1/stories" -name "*.md" -type f 2>/dev/null | wc -l | tr -d ' '
    }

    # Helper: get frontmatter value
    get_fm() {
        grep -m1 "^$2:" "$1" 2>/dev/null | sed "s/$2: *//" | tr -d '[]' || echo ""
    }

    {
        echo "# Planning Board"
        echo ""
        echo "> Auto-generated. Run \`just board generate\` to update."
        echo ""

        # In Progress
        echo "## 🔨 In Progress"
        echo ""
        if [[ $(count_stories "in-progress") -gt 0 ]]; then
            echo "| Story | Type | Priority | Epics |"
            echo "|-------|------|----------|-------|"
            for f in stages/in-progress/stories/*.md; do
                [[ -f "$f" ]] || continue
                id=$(get_fm "$f" "id")
                title=$(get_fm "$f" "title")
                type=$(get_fm "$f" "type")
                priority=$(get_fm "$f" "priority")
                epics=$(get_fm "$f" "epics")
                name=$(basename "$f" .md)
                echo "| [$name](stages/in-progress/stories/$name.md) | $type | $priority | $epics |"
            done
        else
            echo "*No stories in progress*"
        fi
        echo ""

        # Backlog
        echo "## 📥 Backlog"
        echo ""
        if [[ $(count_stories "backlog") -gt 0 ]]; then
            echo "| Story | Type | Priority | Epics |"
            echo "|-------|------|----------|-------|"
            for f in stages/backlog/stories/*.md; do
                [[ -f "$f" ]] || continue
                id=$(get_fm "$f" "id")
                title=$(get_fm "$f" "title")
                type=$(get_fm "$f" "type")
                priority=$(get_fm "$f" "priority")
                epics=$(get_fm "$f" "epics")
                name=$(basename "$f" .md)
                echo "| [$name](stages/backlog/stories/$name.md) | $type | $priority | $epics |"
            done
        else
            echo "*Backlog empty*"
        fi
        echo ""

        # Done (collapsed)
        echo "## ✅ Done"
        echo ""
        echo "<details>"
        echo "<summary>View completed ($(count_stories done) stories)</summary>"
        echo ""
        if [[ $(count_stories "done") -gt 0 ]]; then
            for f in stages/done/stories/*.md; do
                [[ -f "$f" ]] || continue
                name=$(basename "$f" .md)
                echo "- [$name](stages/done/stories/$name.md)"
            done
        fi
        echo ""
        echo "</details>"
        echo ""

        # Epics
        echo "## 📚 Epics"
        echo ""
        echo "| Epic | Status | Stories |"
        echo "|------|--------|---------|"
        for d in epics/*/; do
            [[ -d "$d" ]] || continue
            name=$(basename "$d")
            [[ "$name" == "README.md" ]] && continue
            readme="$d/README.md"
            status=$(get_fm "$readme" "status")
            count=$(find "$d" -maxdepth 1 -type l | wc -l | tr -d ' ')
            echo "| [$name](epics/$name/) | $status | $count |"
        done
        echo ""

        # Milestones
        echo "## 🎯 Milestones"
        echo ""
        echo "| Milestone | Status | Epics |"
        echo "|-----------|--------|-------|"
        for d in milestones/milestone-*/; do
            [[ -d "$d" ]] || continue
            name=$(basename "$d")
            readme="$d/README.md"
            [[ -f "$readme" ]] || continue
            status=$(get_fm "$readme" "status")
            epics=$(get_fm "$readme" "epics")
            echo "| [$name](milestones/$name/) | $status | $epics |"
        done

    } > README.md
    echo "✓ Board README.md updated"

# Show board status summary
status:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    echo "Board Status:"
    echo "  In Progress: $(find stages/in-progress/stories -name '*.md' -type f 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Backlog:     $(find stages/backlog/stories -name '*.md' -type f 2>/dev/null | wc -l | tr -d ' ')"
    echo "  Done:        $(find stages/done/stories -name '*.md' -type f 2>/dev/null | wc -l | tr -d ' ')"
    echo ""
    echo "Epics:"
    for d in epics/*/; do
        [[ -d "$d" ]] || continue
        name=$(basename "$d")
        count=$(find "$d" -maxdepth 1 -type l 2>/dev/null | wc -l | tr -d ' ')
        echo "  $name: $count stories"
    done

# Create new story
_new-story TITLE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    title="{{TITLE}}"

    # Prompt for type
    echo "Story type:"
    echo "  1) feat - New feature"
    echo "  2) bug  - Bug fix"
    echo "  3) chore - Maintenance"
    read -p "Choice [1]: " type_choice
    case "${type_choice:-1}" in
        1) type="feat" ;;
        2) type="bug" ;;
        3) type="chore" ;;
        *) type="feat" ;;
    esac

    # Get next ID
    max=0
    for f in $(find stages -name "${type}-*.md" 2>/dev/null); do
        num=$(basename "$f" | sed -n "s/^${type}-\([0-9]*\).*/\1/p")
        [[ -n "$num" && "$num" -gt "$max" ]] && max=$num
    done
    id=$(printf "%02d" $((max + 1)))

    slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    filename="stages/backlog/stories/${type}-${id}-${slug}.md"

    # Prompt for epics
    echo ""
    echo "Available epics:"
    for d in epics/*/; do
        [[ -d "$d" ]] && echo "  $(basename "$d")"
    done
    read -p "Epics (comma-separated): " epics_input
    epics=$(echo "$epics_input" | tr -d ' ')

    # Create story
    cat > "$filename" << EOF
---
id: ${type^^}${id}
title: $title
type: $type
status: backlog
priority: medium
epics: [$epics]
depends: []
estimate:
created: $(date +%Y-%m-%d)
updated: $(date +%Y-%m-%d)
---

# $title

## Summary

[Description]

## Acceptance Criteria

- [ ] Criterion 1
EOF

    # Create symlinks
    IFS=',' read -ra epic_list <<< "$epics"
    for epic in "${epic_list[@]}"; do
        if [[ -d "epics/$epic" ]]; then
            ln -sf "../../../stages/backlog/stories/${type}-${id}-${slug}.md" "epics/$epic/${type}-${id}-${slug}.md"
        fi
    done

    echo "✓ Created: $filename"
    just board generate

# Create new epic
_new-epic NAME:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"
    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')

    mkdir -p "epics/$slug"
    cat > "epics/$slug/README.md" << EOF
---
id: $slug
title: $name
status: active
description: [Description]
---

# $name

[Description]
EOF

    echo "✓ Created: epics/$slug/"
    just board generate

# Create new milestone
_new-milestone NAME:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"

    # Get next number
    max=0
    for d in milestones/milestone-*/; do
        [[ -d "$d" ]] || continue
        num=$(basename "$d" | sed -n 's/milestone-\([0-9]*\).*/\1/p')
        [[ -n "$num" && "$num" -gt "$max" ]] && max=$num
    done
    id=$(printf "%02d" $((max + 1)))

    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    dirname="milestones/milestone-${id}-${slug}"

    mkdir -p "$dirname"
    cat > "$dirname/README.md" << EOF
---
id: milestone-${id}-${slug}
title: $name
status: planned
epics: []
---

# $name

## Overview

[Description]
EOF

    cat > "$dirname/design.md" << EOF
# Milestone ${id}: $name - Design

> [Summary]

## Overview

[Description]
EOF

    echo "✓ Created: $dirname/"
    just board generate

# Router for 'new' subcommand
new TYPE NAME:
    #!/usr/bin/env bash
    case "{{TYPE}}" in
        story) just board _new-story "{{NAME}}" ;;
        epic) just board _new-epic "{{NAME}}" ;;
        milestone) just board _new-milestone "{{NAME}}" ;;
        *) echo "Unknown type: {{TYPE}}. Use: story, epic, milestone" && exit 1 ;;
    esac

# Find story by pattern
_find-story PATTERN:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    for stage in backlog in-progress done; do
        match=$(find "stages/$stage/stories" -name "*{{PATTERN}}*" -type f 2>/dev/null | head -1)
        [[ -n "$match" ]] && echo "$match" && exit 0
    done

# Start work on a story
start ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    src=$(just board _find-story "{{ID}}")
    if [[ -z "$src" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    name=$(basename "$src")
    current_stage=$(echo "$src" | sed 's|stages/\([^/]*\)/.*|\1|')

    if [[ "$current_stage" == "in-progress" ]]; then
        echo "Story is already in progress"
        exit 0
    fi

    dest="stages/in-progress/stories/$name"
    mv "$src" "$dest"

    # Update frontmatter status
    sed -i 's/^status: .*/status: in-progress/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    # Update symlinks in epics
    epics=$(grep "^epics:" "$dest" | sed 's/epics: *//' | tr -d '[]' | tr ',' ' ')
    for epic in $epics; do
        epic=$(echo "$epic" | tr -d ' ')
        [[ -z "$epic" ]] && continue
        link="epics/$epic/$name"
        [[ -L "$link" ]] && rm "$link"
        ln -sf "../../../stages/in-progress/stories/$name" "$link"
    done

    echo "✓ Started: $name"
    just board generate

# Complete a story
done ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    src=$(just board _find-story "{{ID}}")
    if [[ -z "$src" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    name=$(basename "$src")
    dest="stages/done/stories/$name"
    mv "$src" "$dest"

    # Update frontmatter status
    sed -i 's/^status: .*/status: done/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    # Update symlinks in epics
    epics=$(grep "^epics:" "$dest" | sed 's/epics: *//' | tr -d '[]' | tr ',' ' ')
    for epic in $epics; do
        epic=$(echo "$epic" | tr -d ' ')
        [[ -z "$epic" ]] && continue
        link="epics/$epic/$name"
        [[ -L "$link" ]] && rm "$link"
        ln -sf "../../../stages/done/stories/$name" "$link"
    done

    # Add to changelog
    echo "Enter changelog summary:"
    read -r summary
    date=$(date +%Y-%m-%d)
    sed -i "4a\\| $date | $(basename "$name" .md) | $summary |" CHANGELOG.md

    echo "✓ Completed: $name"
    just board generate

# Link story to epic
link STORY EPIC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    src=$(just board _find-story "{{STORY}}")
    if [[ -z "$src" ]]; then
        echo "Error: Story '{{STORY}}' not found"
        exit 1
    fi

    if [[ ! -d "epics/{{EPIC}}" ]]; then
        echo "Error: Epic '{{EPIC}}' not found"
        exit 1
    fi

    name=$(basename "$src")
    stage=$(echo "$src" | sed 's|stages/\([^/]*\)/.*|\1|')

    # Create symlink
    ln -sf "../../../stages/$stage/stories/$name" "epics/{{EPIC}}/$name"

    # Update frontmatter
    current_epics=$(grep "^epics:" "$src" | sed 's/epics: *//' | tr -d '[]')
    if [[ -z "$current_epics" ]]; then
        new_epics="{{EPIC}}"
    else
        new_epics="$current_epics, {{EPIC}}"
    fi
    sed -i "s/^epics: .*/epics: [$new_epics]/" "$src"

    echo "✓ Linked $(basename "$src" .md) to {{EPIC}}"

# Unlink story from epic
unlink STORY EPIC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    src=$(just board _find-story "{{STORY}}")
    if [[ -z "$src" ]]; then
        echo "Error: Story '{{STORY}}' not found"
        exit 1
    fi

    name=$(basename "$src")
    link="epics/{{EPIC}}/$name"

    if [[ -L "$link" ]]; then
        rm "$link"

        # Update frontmatter
        current_epics=$(grep "^epics:" "$src" | sed 's/epics: *//' | tr -d '[]')
        new_epics=$(echo "$current_epics" | sed "s/{{EPIC}}//" | sed 's/^, //' | sed 's/, $//' | sed 's/, ,/, /')
        sed -i "s/^epics: .*/epics: [$new_epics]/" "$src"

        echo "✓ Unlinked $(basename "$src" .md) from {{EPIC}}"
    else
        echo "Story not linked to {{EPIC}}"
    fi

# Link epic to milestone
link-epic EPIC MILESTONE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    if [[ ! -d "epics/{{EPIC}}" ]]; then
        echo "Error: Epic '{{EPIC}}' not found"
        exit 1
    fi

    milestone_dir=$(find milestones -maxdepth 1 -type d -name "*{{MILESTONE}}*" | head -1)
    if [[ -z "$milestone_dir" ]]; then
        echo "Error: Milestone '{{MILESTONE}}' not found"
        exit 1
    fi

    ln -sf "../../epics/{{EPIC}}" "$milestone_dir/{{EPIC}}"

    # Update milestone frontmatter
    readme="$milestone_dir/README.md"
    current_epics=$(grep "^epics:" "$readme" | sed 's/epics: *//' | tr -d '[]')
    if [[ -z "$current_epics" ]]; then
        new_epics="{{EPIC}}"
    else
        new_epics="$current_epics, {{EPIC}}"
    fi
    sed -i "s/^epics: .*/epics: [$new_epics]/" "$readme"

    echo "✓ Linked {{EPIC}} to $(basename "$milestone_dir")"

# List epics
_list-epics:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    echo "Epics:"
    for d in epics/*/; do
        [[ -d "$d" ]] || continue
        name=$(basename "$d")
        count=$(find "$d" -maxdepth 1 -type l 2>/dev/null | wc -l | tr -d ' ')
        status=$(grep "^status:" "$d/README.md" 2>/dev/null | sed 's/status: *//' || echo "unknown")
        echo "  $name ($status) - $count stories"
    done

# List milestones
_list-milestones:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    echo "Milestones:"
    for d in milestones/milestone-*/; do
        [[ -d "$d" ]] || continue
        name=$(basename "$d")
        status=$(grep "^status:" "$d/README.md" 2>/dev/null | sed 's/status: *//' || echo "unknown")
        epics=$(grep "^epics:" "$d/README.md" 2>/dev/null | sed 's/epics: *//' | tr -d '[]' || echo "")
        echo "  $name ($status)"
        [[ -n "$epics" ]] && echo "    epics: $epics"
    done

# Router for 'list' subcommand
list TYPE:
    #!/usr/bin/env bash
    case "{{TYPE}}" in
        epics) just board _list-epics ;;
        milestones) just board _list-milestones ;;
        *) echo "Unknown type: {{TYPE}}. Use: epics, milestones" && exit 1 ;;
    esac

# Show story details
show ID:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    src=$(just board _find-story "{{ID}}")
    if [[ -z "$src" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    echo "=== $(basename "$src" .md) ==="
    echo "File: $src"
    echo ""
    head -20 "$src"
EOF
```

**Step 2: Commit new tooling**

```bash
git add .justfiles/board.just
git commit -m "feat(board): rewrite board.just with epic/milestone commands"
```

---

## Task 7: Update Documentation

**Files:**
- Modify: `docs/board/CONVENTIONS.md`
- Modify: `CLAUDE.md`

**Step 1: Update CONVENTIONS.md**

Replace the existing CONVENTIONS.md with documentation for the new structure.

**Step 2: Update CLAUDE.md**

Update the board commands reference in CLAUDE.md.

**Step 3: Commit documentation**

```bash
git add docs/board/CONVENTIONS.md CLAUDE.md
git commit -m "docs(board): update documentation for new board structure"
```

---

## Task 8: Cleanup and Verify

**Step 1: Remove old directories**

```bash
cd docs/board
rm -rf backlog/ in-progress/ ready/ review/ done/
rm -f migrate.sh
```

**Step 2: Verify board generation**

```bash
just board generate
just board status
just board list epics
just board list milestones
```

**Step 3: Final commit**

```bash
git add -A
git commit -m "chore(board): remove old structure, migration complete"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Create directory structure |
| 2 | Create templates |
| 3 | Create initial epics |
| 4 | Write migration script |
| 5 | Run migration |
| 6 | Rewrite board tooling |
| 7 | Update documentation |
| 8 | Cleanup and verify |

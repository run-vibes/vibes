# Board management commands for epic-based structure
# Usage: just board, just board new story, just board start, etc.

# Use parent of .justfiles directory (project root) + docs/board
board_dir := source_directory() / ".." / "docs" / "board"

# Default: show help
default:
    #!/usr/bin/env bash
    echo "Board Management Commands"
    echo ""
    echo "Help & generation:"
    echo "  just board              Show this help"
    echo "  just board generate     Regenerate README.md"
    echo "  just board status       Show counts per stage + blocked stories"
    echo ""
    echo "Story management:"
    echo "  just board new story \"title\" [type] [epics] [milestone]"
    echo "                              Create story (type: feat|bug|chore|refactor)"
    echo "  just board start <id>       Move story to in-progress"
    echo "  just board done <id> [summary]"
    echo "                              Move story to done + changelog"
    echo "  just board ice <id>         Move story to icebox (blocked/deferred)"
    echo "  just board thaw <id>        Move story from icebox to backlog"
    echo ""
    echo "Epic/Milestone management:"
    echo "  just board new epic \"name\" [description]"
    echo "                              Create epic directory"
    echo "  just board new milestone \"name\""
    echo "                              Create milestone directory"
    echo "  just board start-milestone <id>"
    echo "                              Set milestone to in-progress"
    echo "  just board done-milestone <id>"
    echo "                              Set milestone to done"
    echo "  just board done-epic <id>       Set epic to done"
    echo "  just board link <story> <epic>   Add story to epic"
    echo "  just board unlink <story> <epic> Remove story from epic"
    echo "  just board link-epic <epic> <milestone>  Add epic to milestone"
    echo ""
    echo "Utility:"
    echo "  just board list epics      Show all epics with story counts"
    echo "  just board list milestones Show all milestones with progress"
    echo "  just board show <id>       Display story details"
    echo ""
    echo "Examples:"
    echo "  just board new story \"Add login\" feat \"web-ui\" \"27-crt-design-system\""
    echo "  just board new epic \"performance\" \"Speed improvements\""
    echo "  just board done feat-01 \"Added user authentication\""

# Generate README.md from directory structure
generate:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    # Helper: get frontmatter field value
    get_field() {
        local file="$1"
        local field="$2"
        grep -m1 "^${field}:" "$file" 2>/dev/null | sed "s/^${field}: *//" | tr -d ' ' || echo ""
    }

    # Helper: get status from frontmatter
    get_status() {
        get_field "$1" "status"
    }

    # Generate README.md
    {
    echo "# Planning Board"
    echo ""
    echo "> [!NOTE]"
    echo "> Auto-generated from directory structure. Run \`just board generate\` to update."
    echo ""

    # In Progress stories
    echo "## In Progress"
    echo ""
    if [[ -d stages/in-progress/stories ]] && ls stages/in-progress/stories/*.md &>/dev/null; then
        echo "| Story | Type | Priority | Epic |"
        echo "|-------|------|----------|------|"
        for story in stages/in-progress/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            type=$(get_field "$story" "type")
            priority=$(get_field "$story" "priority")
            epics=$(get_field "$story" "epics" | tr -d '[]')
            echo "| [$name](stages/in-progress/stories/$name.md) | $type | $priority | $epics |"
        done
    else
        echo "*No stories in progress*"
    fi
    echo ""

    # Backlog stories
    echo "## Backlog"
    echo ""
    if [[ -d stages/backlog/stories ]] && ls stages/backlog/stories/*.md &>/dev/null; then
        echo "| Story | Type | Priority | Epic |"
        echo "|-------|------|----------|------|"
        for story in stages/backlog/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            type=$(get_field "$story" "type")
            priority=$(get_field "$story" "priority")
            epics=$(get_field "$story" "epics" | tr -d '[]')
            echo "| [$name](stages/backlog/stories/$name.md) | $type | $priority | $epics |"
        done
    else
        echo "*Backlog empty*"
    fi
    echo ""

    # Icebox stories
    echo "## Icebox"
    echo ""
    if [[ -d stages/icebox/stories ]] && ls stages/icebox/stories/*.md &>/dev/null 2>&1; then
        echo "| Story | Type | Priority | Blocked By |"
        echo "|-------|------|----------|------------|"
        for story in stages/icebox/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            type=$(get_field "$story" "type")
            priority=$(get_field "$story" "priority")
            depends=$(get_field "$story" "depends" | tr -d '[]')
            echo "| [$name](stages/icebox/stories/$name.md) | $type | $priority | $depends |"
        done
    else
        echo "*No stories in icebox*"
    fi
    echo ""

    # Epics section - group milestones under each epic
    echo "## Epics"
    echo ""
    if [[ -d epics ]]; then
        # Collect epics with active/in-progress milestones first
        for epic_dir in epics/*/; do
            [[ -d "$epic_dir" ]] || continue
            epic_name=$(basename "$epic_dir")
            [[ -f "${epic_dir}README.md" ]] || continue

            epic_status=$(get_field "${epic_dir}README.md" "status")

            # Check if this epic has milestones
            if [[ -d "${epic_dir}milestones" ]]; then
                # Count milestones and done milestones
                total_ms=0
                done_ms=0
                in_progress_ms=0
                for ms_dir in "${epic_dir}milestones"/*/; do
                    [[ -d "$ms_dir" ]] || continue
                    [[ -f "${ms_dir}README.md" ]] || continue
                    total_ms=$((total_ms + 1))
                    ms_status=$(get_field "${ms_dir}README.md" "status")
                    if [[ "$ms_status" == "done" ]]; then
                        done_ms=$((done_ms + 1))
                    elif [[ "$ms_status" == "in-progress" ]]; then
                        in_progress_ms=$((in_progress_ms + 1))
                    fi
                done

                # Skip epics with no milestones
                [[ $total_ms -eq 0 ]] && continue

                # Skip if all milestones are done (will appear in Done section)
                [[ $done_ms -eq $total_ms ]] && continue

                echo "### $epic_name ($epic_status) - $total_ms milestones, $done_ms done"
                echo ""
                echo "| Milestone | Status |"
                echo "|-----------|--------|"

                # List milestones sorted by numeric prefix
                for ms_dir in $(ls -d "${epic_dir}milestones"/*/ 2>/dev/null | sort -t'/' -k5 -V); do
                    [[ -d "$ms_dir" ]] || continue
                    ms_name=$(basename "$ms_dir")
                    [[ -f "${ms_dir}README.md" ]] || continue
                    ms_status=$(get_field "${ms_dir}README.md" "status")
                    echo "| [$ms_name](epics/$epic_name/milestones/$ms_name/) | $ms_status |"
                done
                echo ""
            fi
        done
    fi

    # Done section - completed epics and milestones
    echo "## Done"
    echo ""
    echo "<details>"
    echo "<summary>Completed Epics & Milestones</summary>"
    echo ""

    # First, show epics where ALL milestones are done
    if [[ -d epics ]]; then
        for epic_dir in epics/*/; do
            [[ -d "$epic_dir" ]] || continue
            epic_name=$(basename "$epic_dir")
            [[ -f "${epic_dir}README.md" ]] || continue

            epic_status=$(get_field "${epic_dir}README.md" "status")

            if [[ -d "${epic_dir}milestones" ]]; then
                # Count milestones
                total_ms=0
                done_ms=0
                for ms_dir in "${epic_dir}milestones"/*/; do
                    [[ -d "$ms_dir" ]] || continue
                    [[ -f "${ms_dir}README.md" ]] || continue
                    total_ms=$((total_ms + 1))
                    ms_status=$(get_field "${ms_dir}README.md" "status")
                    if [[ "$ms_status" == "done" ]]; then
                        done_ms=$((done_ms + 1))
                    fi
                done

                # Show epic if ALL milestones are done
                if [[ $total_ms -gt 0 ]] && [[ $done_ms -eq $total_ms ]]; then
                    echo "### [$epic_name](epics/$epic_name/) (done)"
                    echo ""
                    for ms_dir in $(ls -d "${epic_dir}milestones"/*/ 2>/dev/null | sort -t'/' -k5 -V); do
                        [[ -d "$ms_dir" ]] || continue
                        ms_name=$(basename "$ms_dir")
                        echo "- [$ms_name](epics/$epic_name/milestones/$ms_name/)"
                    done
                    echo ""
                # Show completed milestones from active epics
                elif [[ $done_ms -gt 0 ]]; then
                    echo "### [$epic_name](epics/$epic_name/) - Completed Milestones"
                    echo ""
                    for ms_dir in $(ls -d "${epic_dir}milestones"/*/ 2>/dev/null | sort -t'/' -k5 -V); do
                        [[ -d "$ms_dir" ]] || continue
                        ms_name=$(basename "$ms_dir")
                        [[ -f "${ms_dir}README.md" ]] || continue
                        ms_status=$(get_field "${ms_dir}README.md" "status")
                        if [[ "$ms_status" == "done" ]]; then
                            echo "- [$ms_name](epics/$epic_name/milestones/$ms_name/)"
                        fi
                    done
                    echo ""
                fi
            fi
        done
    fi

    echo "</details>"
    } > README.md

    echo "Generated README.md"

# Show board status summary
status:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    # Count stories in each stage
    icebox=$(find stages/icebox/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')
    in_progress=$(find stages/in-progress/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')
    backlog=$(find stages/backlog/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')
    done_count=$(find stages/done/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')

    echo "Board Status"
    echo "============"
    echo ""
    echo "Stories by Stage:"
    echo "  In Progress: $in_progress"
    echo "  Backlog:     $backlog"
    echo "  Icebox:      $icebox"
    echo "  Done:        $done_count"
    echo ""

    # Count epics and milestones
    epic_count=$(find epics -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')
    milestone_count=$(find milestones -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')

    echo "Organization:"
    echo "  Epics:      $epic_count"
    echo "  Milestones: $milestone_count"
    echo ""

    # Show blocked stories (stories with unmet depends)
    echo "Blocked Stories:"
    blocked=0
    for story in stages/backlog/stories/*.md stages/in-progress/stories/*.md; do
        [[ -f "$story" ]] || continue
        depends=$(grep -m1 "^depends:" "$story" 2>/dev/null | sed 's/^depends: *//' | tr -d '[]' || echo "")
        if [[ -n "$depends" ]] && [[ "$depends" != "[]" ]]; then
            name=$(basename "$story" .md)
            echo "  - $name (depends: $depends)"
            blocked=$((blocked + 1))
        fi
    done
    [[ $blocked -eq 0 ]] && echo "  (none)"

# Get next available ID for a type (feat, bug, chore, fix, refactor)
_next-id TYPE:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    type="{{TYPE}}"
    type_upper="${type^^}"
    max=0

    # Search for new format: [TYPE][NNNN]-*.md
    for f in $(find stages -name "\[${type_upper}\]\[*\]*.md" 2>/dev/null); do
        # Extract sequence number from [TYPE][NNNN] format
        num=$(basename "$f" .md | grep -oE "\[${type_upper}\]\[[0-9]+\]" | grep -oE "[0-9]+")
        [[ -z "$num" ]] && continue
        num=$((10#$num))
        if [[ "$num" -gt "$max" ]]; then
            max=$num
        fi
    done

    # Also search old format for backwards compatibility: type-NNNN-*.md
    for f in $(find stages -name "${type}-*.md" -o -name "*-${type}-*.md" 2>/dev/null); do
        num=$(basename "$f" .md | grep -oE "(feat|bug|chore|refactor|fix|docs)-[0-9]+" | grep -oE "[0-9]+$")
        [[ -z "$num" ]] && continue
        num=$((10#$num))
        if [[ "$num" -gt "$max" ]]; then
            max=$num
        fi
    done

    printf "%04d" $((max + 1))

# Get next milestone number
_next-milestone:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    max=0
    for d in milestones/[0-9]*/; do
        [[ -d "$d" ]] || continue
        num=$(basename "$d" | sed -n 's/^\([0-9]*\).*/\1/p')
        if [[ -n "$num" ]] && [[ "$num" -gt "$max" ]]; then
            max=$num
        fi
    done
    printf "%02d" $((max + 1))

# Internal: create a new story
# Usage: just board _new-story "title" [type] [epics] [milestone]
# Examples:
#   just board _new-story "Add login" feat "web-ui, cli"
#   just board _new-story "Fix crash" bug "" "27-crt-design-system"
_new-story TITLE TYPE="" EPICS="" MILESTONE="":
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    title="{{TITLE}}"
    type="{{TYPE}}"
    selected_epics="{{EPICS}}"
    milestone="{{MILESTONE}}"

    # If type not provided, prompt for it
    if [[ -z "$type" ]]; then
        echo "Select type:"
        echo "  1) feat     - New functionality"
        echo "  2) bug      - Bug fix"
        echo "  3) chore    - Maintenance"
        echo "  4) refactor - Code restructuring"
        read -p "Type [1-4]: " type_choice

        case "$type_choice" in
            1) type="feat" ;;
            2) type="bug" ;;
            3) type="chore" ;;
            4) type="refactor" ;;
            *) type="feat" ;;
        esac
    fi

    # Validate type
    case "$type" in
        feat|bug|chore|refactor|fix|docs) ;;
        *)
            echo "Error: Invalid type '$type'. Must be: feat, bug, chore, refactor, fix, docs"
            exit 1
            ;;
    esac

    # Generate ID
    id=$(just board _next-id "$type")

    # If epics not provided, prompt for them
    if [[ -z "$selected_epics" ]] && [[ -t 0 ]]; then
        echo ""
        echo "Available epics:"
        i=1
        declare -a epic_names=()
        for epic_dir in epics/*/; do
            [[ -d "$epic_dir" ]] || continue
            name=$(basename "$epic_dir")
            epic_names+=("$name")
            echo "  $i) $name"
            i=$((i + 1))
        done

        echo ""
        read -p "Select epics (comma-separated numbers, or Enter for none): " epic_choice

        if [[ -n "$epic_choice" ]]; then
            IFS=',' read -ra choices <<< "$epic_choice"
            for choice in "${choices[@]}"; do
                choice=$(echo "$choice" | tr -d ' ')
                if [[ "$choice" =~ ^[0-9]+$ ]] && [[ $choice -ge 1 ]] && [[ $choice -le ${#epic_names[@]} ]]; then
                    [[ -n "$selected_epics" ]] && selected_epics="$selected_epics, "
                    selected_epics="${selected_epics}${epic_names[$((choice-1))]}"
                fi
            done
        fi
    fi

    # Create slug from title
    slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')

    # Build filename with new [TYPE][NNNN] format
    # Format: [TYPE][NNNN]-verb-phrase.md
    type_upper="${type^^}"
    filename="stages/backlog/stories/[${type_upper}][${id}]-${slug}.md"

    # Create the story file
    date=$(date +%Y-%m-%d)
    {
        echo '---'
        echo "id: ${type_upper}${id}"
        echo "title: ${title}"
        echo "type: ${type}"
        echo "status: backlog"
        echo "priority: medium"
        echo "epics: [${selected_epics}]"
        echo "depends: []"
        echo "estimate:"
        echo "created: ${date}"
        echo "updated: ${date}"
        if [[ -n "$milestone" ]]; then
            echo "milestone: ${milestone}"
        fi
        echo '---'
        echo ""
        echo "# ${title}"
        echo ""
        echo "## Summary"
        echo ""
        echo "[Description of the work]"
        echo ""
        echo "## Acceptance Criteria"
        echo ""
        echo "- [ ] Criterion 1"
        echo "- [ ] Criterion 2"
        echo ""
        echo "## Implementation Notes"
        echo ""
        echo "[Technical approach]"
    } > "$filename"

    echo ""
    echo "Created: $filename"

    # Create symlinks in selected epics
    if [[ -n "$selected_epics" ]]; then
        story_name=$(basename "$filename")
        IFS=', ' read -ra epic_list <<< "$selected_epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            if [[ -d "epics/$epic" ]]; then
                rel_path="../../stages/backlog/stories/$story_name"
                ln -sf "$rel_path" "epics/$epic/$story_name"
                echo "Linked to epic: $epic"
            fi
        done
    fi

# Internal: create a new epic
# Usage: just board _new-epic "name" [description]
_new-epic NAME DESCRIPTION="":
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"
    description="{{DESCRIPTION}}"
    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    dir="epics/$slug"

    if [[ -d "$dir" ]]; then
        echo "Error: Epic '$slug' already exists"
        exit 1
    fi

    mkdir -p "$dir"

    # If description not provided and we have a terminal, prompt for it
    if [[ -z "$description" ]] && [[ -t 0 ]]; then
        read -p "Description (one line): " description
    fi

    {
        echo '---'
        echo "id: ${slug}"
        echo "title: ${name}"
        echo "status: active"
        echo "description: ${description}"
        echo '---'
        echo ""
        echo "# ${name}"
        echo ""
        echo "${description}"
        echo ""
        echo "## Stories"
        echo ""
        echo '> Stories are linked via `just board link <story> <epic>`.'
    } > "$dir/README.md"

    echo "Created: $dir/README.md"

# Internal: create a new milestone
_new-milestone NAME:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"
    id=$(just board _next-milestone)
    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    dir="milestones/${id}-${slug}"

    if [[ -d "$dir" ]]; then
        echo "Error: Milestone directory already exists"
        exit 1
    fi

    mkdir -p "$dir"

    # Create README.md
    {
        echo '---'
        echo "id: ${id}-${slug}"
        echo "title: ${name}"
        echo "status: planned"
        echo "epics: []"
        echo '---'
        echo ""
        echo "# ${name}"
        echo ""
        echo "## Overview"
        echo ""
        echo "[Description of the milestone]"
        echo ""
        echo "## Epics"
        echo ""
        echo '> Epics linked to this milestone via `just board link-epic <epic> <milestone>`.'
        echo ""
        echo "## Design"
        echo ""
        echo "See [design.md](design.md) for architecture decisions."
    } > "$dir/README.md"

    # Create design.md template
    date=$(date +%Y-%m-%d)
    {
        echo '---'
        echo "created: ${date}"
        echo '---'
        echo ""
        echo "# Milestone ${id}: ${name} - Design"
        echo ""
        echo "> [One-line summary]"
        echo ""
        echo "## Overview"
        echo ""
        echo "[Description of the milestone]"
        echo ""
        echo "## Key Decisions"
        echo ""
        echo "| Decision | Choice | Rationale |"
        echo "|----------|--------|-----------|"
        echo "| | | |"
        echo ""
        echo "## Architecture"
        echo ""
        echo "[Technical approach]"
        echo ""
        echo "## Deliverables"
        echo ""
        echo "- [ ] Item 1"
        echo "- [ ] Item 2"
    } > "$dir/design.md"

    echo "Created: $dir/README.md"
    echo "Created: $dir/design.md"

# Router for new command
# Usage:
#   just board new story "title" [type] [epics] [milestone]
#   just board new epic "name" [description]
#   just board new milestone "name"
new TYPE NAME *ARGS:
    #!/usr/bin/env bash
    type="{{TYPE}}"
    name="{{NAME}}"

    case "$type" in
        story)
            just board _new-story "$name" {{ARGS}}
            ;;
        epic)
            just board _new-epic "$name" {{ARGS}}
            ;;
        milestone)
            just board _new-milestone "$name"
            ;;
        *)
            echo "Error: Unknown type '$type'"
            echo "Usage:"
            echo "  just board new story \"title\" [type] [epics] [milestone]"
            echo "  just board new epic \"name\" [description]"
            echo "  just board new milestone \"name\""
            exit 1
            ;;
    esac

    # Regenerate README
    just board generate
    echo ""
    echo "Board README.md updated"

# Find story by ID or pattern
_find-story PATTERN:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    pattern="{{PATTERN}}"

    # Search in stages (icebox, backlog, in-progress, done)
    for stage in icebox backlog in-progress done; do
        dir="stages/$stage/stories"
        [[ -d "$dir" ]] || continue

        # Try exact match first
        if [[ -f "$dir/${pattern}.md" ]]; then
            echo "$dir/${pattern}.md"
            exit 0
        fi

        # Try pattern match
        match=$(find "$dir" -maxdepth 1 -name "*${pattern}*.md" -type f 2>/dev/null | head -1)
        if [[ -n "$match" ]]; then
            echo "$match"
            exit 0
        fi
    done

    echo ""

# Start work on a story (move to in-progress)
start ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path=$(just board _find-story "{{ID}}")

    if [[ -z "$path" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    # Check if already in progress
    if [[ "$path" == *"in-progress"* ]]; then
        echo "Error: Story is already in progress"
        exit 1
    fi

    # Check if already done
    if [[ "$path" == *"done"* ]]; then
        echo "Error: Story is already done"
        exit 1
    fi

    name=$(basename "$path")
    dest="stages/in-progress/stories/$name"

    # Get epics from frontmatter before moving
    epics=$(grep -m1 "^epics:" "$path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")

    # Move the file
    mv "$path" "$dest"

    # Update status in frontmatter
    sed -i 's/^status: .*/status: in-progress/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    echo "Started: $name"

    # Update symlinks in epics
    if [[ -n "$epics" ]]; then
        IFS=', ' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            link="epics/$epic/$name"
            if [[ -L "$link" ]]; then
                rm "$link"
                ln -sf "../../stages/in-progress/stories/$name" "$link"
                echo "Updated symlink in epic: $epic"
            fi
        done
    fi

    # Regenerate README
    just board generate
    echo "Board updated"

# Complete a story (move to done)
# Usage: just board done <id> [summary]
# Examples:
#   just board done feat-01                    # Prompts for changelog summary
#   just board done feat-01 "Added login"      # Uses provided summary
#   just board done feat-01 ""                 # Skips changelog (empty string)
done ID SUMMARY="__PROMPT__":
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path=$(just board _find-story "{{ID}}")

    if [[ -z "$path" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    # Check if already done
    if [[ "$path" == *"done"* ]]; then
        echo "Error: Story is already done"
        exit 1
    fi

    name=$(basename "$path")
    dest="stages/done/stories/$name"

    # Get epics from frontmatter before moving
    epics=$(grep -m1 "^epics:" "$path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")

    # Get title for changelog
    title=$(grep -m1 "^title:" "$path" 2>/dev/null | sed 's/^title: *//' || basename "$path" .md)

    # Handle summary - prompt only if set to __PROMPT__ and we have a terminal
    summary="{{SUMMARY}}"
    if [[ "$summary" == "__PROMPT__" ]]; then
        if [[ -t 0 ]]; then
            echo "Enter changelog summary for '$title' (or Enter to skip):"
            read -r summary
        else
            summary=""
        fi
    fi

    # Move the file
    mv "$path" "$dest"

    # Update status in frontmatter
    sed -i 's/^status: .*/status: done/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    echo "Completed: $name"

    # Update symlinks in epics
    if [[ -n "$epics" ]]; then
        IFS=', ' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            link="epics/$epic/$name"
            if [[ -L "$link" ]]; then
                rm "$link"
                ln -sf "../../stages/done/stories/$name" "$link"
                echo "Updated symlink in epic: $epic"
            fi
        done
    fi

    # Sync milestone implementation.md if story has milestone field
    milestone=$(grep -m1 "^milestone:" "$dest" 2>/dev/null | sed 's/^milestone: *//' || echo "")
    if [[ -n "$milestone" ]]; then
        # Get story ID from frontmatter
        story_id=$(grep -m1 "^id:" "$dest" 2>/dev/null | sed 's/^id: *//' || echo "")
        if [[ -n "$story_id" ]]; then
            # Find the milestone implementation.md
            impl_file=""
            for epic_dir in epics/*/; do
                [[ -d "${epic_dir}milestones" ]] || continue
                for ms_dir in "${epic_dir}milestones"/*/; do
                    [[ -d "$ms_dir" ]] || continue
                    ms_name=$(basename "$ms_dir")
                    # Match milestone by ID or partial match
                    if [[ "$ms_name" == "$milestone"* ]] || [[ "$ms_name" == *"$milestone"* ]]; then
                        if [[ -f "${ms_dir}implementation.md" ]]; then
                            impl_file="${ms_dir}implementation.md"
                            break 2
                        fi
                    fi
                done
            done

            if [[ -n "$impl_file" ]]; then
                # Update checkbox for this story (handle both [TYPE][NNNN] and TYPENNNN formats)
                # Match patterns like: - [ ] [FEAT][0110] or - [ ] FEAT0110
                if grep -qE "^\s*- \[ \] .*${story_id}" "$impl_file" 2>/dev/null; then
                    sed -i -E "s/^(\s*- )\[ \]( .*${story_id})/\1[x]\2/" "$impl_file"
                    echo "Updated milestone implementation.md: checked $story_id"
                fi

                # Check if all stories in the Stories section are complete
                # Count unchecked boxes in Stories section
                stories_section=$(sed -n '/^## Stories/,/^## /p' "$impl_file" | head -n -1)
                unchecked=$(echo "$stories_section" | grep -cE "^\s*- \[ \]" 2>/dev/null || echo "0")
                if [[ "$unchecked" == "0" ]]; then
                    # Get milestone ID from path
                    ms_id=$(basename "$(dirname "$impl_file")")
                    echo ""
                    echo "All stories complete. Run \`just board done-milestone $ms_id\` to complete the milestone."
                fi
            fi
        fi
    fi

    # Add to changelog if summary provided
    if [[ -n "$summary" ]]; then
        date=$(date +%Y-%m-%d)
        # Insert after header row in changelog table
        if [[ -f CHANGELOG.md ]]; then
            sed -i "4a\\| $date | $title | $summary |" CHANGELOG.md
            echo "Added to CHANGELOG.md"
        fi
    fi

    # Regenerate README
    just board generate
    echo "Board updated"

# Move story to icebox (blocked or deferred)
ice ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path=$(just board _find-story "{{ID}}")

    if [[ -z "$path" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    # Check if already in icebox
    if [[ "$path" == *"icebox"* ]]; then
        echo "Error: Story is already in icebox"
        exit 1
    fi

    # Check if already done
    if [[ "$path" == *"done"* ]]; then
        echo "Error: Cannot ice a completed story"
        exit 1
    fi

    name=$(basename "$path")
    dest="stages/icebox/stories/$name"

    # Get epics from frontmatter before moving
    epics=$(grep -m1 "^epics:" "$path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")

    # Move the file
    mv "$path" "$dest"

    # Update status in frontmatter
    sed -i 's/^status: .*/status: icebox/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    echo "Iced: $name"

    # Update symlinks in epics
    if [[ -n "$epics" ]]; then
        IFS=', ' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            link="epics/$epic/$name"
            if [[ -L "$link" ]]; then
                rm "$link"
                ln -sf "../../stages/icebox/stories/$name" "$link"
                echo "Updated symlink in epic: $epic"
            fi
        done
    fi

    # Regenerate README
    just board generate
    echo "Board updated"

# Move story from icebox to backlog (unblock)
thaw ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    path=$(just board _find-story "{{ID}}")

    if [[ -z "$path" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    # Check if in icebox
    if [[ "$path" != *"icebox"* ]]; then
        echo "Error: Story is not in icebox (current location: $path)"
        exit 1
    fi

    name=$(basename "$path")
    dest="stages/backlog/stories/$name"

    # Get epics from frontmatter before moving
    epics=$(grep -m1 "^epics:" "$path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")

    # Move the file
    mv "$path" "$dest"

    # Update status in frontmatter
    sed -i 's/^status: .*/status: backlog/' "$dest"
    sed -i "s/^updated: .*/updated: $(date +%Y-%m-%d)/" "$dest"

    echo "Thawed: $name"

    # Update symlinks in epics
    if [[ -n "$epics" ]]; then
        IFS=', ' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            link="epics/$epic/$name"
            if [[ -L "$link" ]]; then
                rm "$link"
                ln -sf "../../stages/backlog/stories/$name" "$link"
                echo "Updated symlink in epic: $epic"
            fi
        done
    fi

    # Regenerate README
    just board generate
    echo "Board updated"

# Start a milestone (set status to in-progress)
start-milestone ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    milestone="{{ID}}"

    # Find milestone directory (allow partial match)
    ms_dir=""
    for d in milestones/*/; do
        if [[ "$(basename "$d")" == *"$milestone"* ]]; then
            ms_dir="$d"
            break
        fi
    done

    if [[ -z "$ms_dir" ]] || [[ ! -d "$ms_dir" ]]; then
        echo "Error: Milestone '$milestone' not found"
        echo "Available milestones:"
        for m in milestones/*/; do
            [[ -d "$m" ]] && echo "  - $(basename "$m")"
        done
        exit 1
    fi

    ms_name=$(basename "$ms_dir")
    ms_readme="${ms_dir}README.md"

    if [[ ! -f "$ms_readme" ]]; then
        echo "Error: Milestone README not found at $ms_readme"
        exit 1
    fi

    # Check current status
    current_status=$(grep -m1 "^status:" "$ms_readme" 2>/dev/null | sed 's/^status: *//' || echo "")
    if [[ "$current_status" == "in-progress" ]]; then
        echo "Milestone '$ms_name' is already in-progress"
        exit 0
    fi
    if [[ "$current_status" == "done" ]]; then
        echo "Error: Milestone '$ms_name' is already done"
        exit 1
    fi

    # Update status
    sed -i 's/^status: .*/status: in-progress/' "$ms_readme"
    echo "Started milestone: $ms_name"

    # Regenerate README
    just board generate
    echo "Board updated"

# Complete a milestone (set status to done)
done-milestone ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    milestone="{{ID}}"

    # Find milestone directory in epics (allow partial match)
    ms_dir=""
    ms_epic=""
    for epic_dir in epics/*/; do
        [[ -d "${epic_dir}milestones" ]] || continue
        for d in "${epic_dir}milestones"/*/; do
            [[ -d "$d" ]] || continue
            if [[ "$(basename "$d")" == *"$milestone"* ]]; then
                ms_dir="$d"
                ms_epic=$(basename "$epic_dir")
                break 2
            fi
        done
    done

    # Also check top-level milestones for backwards compatibility
    if [[ -z "$ms_dir" ]] && [[ -d milestones ]]; then
        for d in milestones/*/; do
            if [[ "$(basename "$d")" == *"$milestone"* ]]; then
                ms_dir="$d"
                break
            fi
        done
    fi

    if [[ -z "$ms_dir" ]] || [[ ! -d "$ms_dir" ]]; then
        echo "Error: Milestone '$milestone' not found"
        echo "Available milestones:"
        for epic_dir in epics/*/; do
            [[ -d "${epic_dir}milestones" ]] || continue
            for m in "${epic_dir}milestones"/*/; do
                [[ -d "$m" ]] && echo "  - $(basename "$m") ($(basename "$epic_dir"))"
            done
        done
        exit 1
    fi

    ms_name=$(basename "$ms_dir")
    ms_readme="${ms_dir}README.md"

    if [[ ! -f "$ms_readme" ]]; then
        echo "Error: Milestone README not found at $ms_readme"
        exit 1
    fi

    # Check current status
    current_status=$(grep -m1 "^status:" "$ms_readme" 2>/dev/null | sed 's/^status: *//' || echo "")
    if [[ "$current_status" == "done" ]]; then
        echo "Milestone '$ms_name' is already done"
        exit 0
    fi

    # Update status
    sed -i 's/^status: .*/status: done/' "$ms_readme"

    # Add completed date if not already present
    date=$(date +%Y-%m-%d)
    if ! grep -q "^completed:" "$ms_readme"; then
        # Insert after status line
        sed -i "/^status: done$/a completed: $date" "$ms_readme"
    fi

    echo "Completed milestone: $ms_name"

    # Check if all milestones in epic are done
    if [[ -n "$ms_epic" ]]; then
        epic_dir="epics/$ms_epic"
        if [[ -d "${epic_dir}/milestones" ]]; then
            all_done=true
            total_ms=0
            for m in "${epic_dir}/milestones"/*/; do
                [[ -d "$m" ]] || continue
                [[ -f "${m}README.md" ]] || continue
                total_ms=$((total_ms + 1))
                m_status=$(grep -m1 "^status:" "${m}README.md" 2>/dev/null | sed 's/^status: *//' || echo "")
                if [[ "$m_status" != "done" ]]; then
                    all_done=false
                fi
            done
            if [[ "$all_done" == "true" ]] && [[ $total_ms -gt 0 ]]; then
                echo ""
                echo "All milestones complete. Run \`just board done-epic $ms_epic\` to complete the epic."
            fi
        fi
    fi

    # Regenerate README
    just board generate
    echo "Board updated"

# Complete an epic (set status to done)
done-epic ID:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    epic="{{ID}}"

    # Find epic directory (allow partial match)
    epic_dir=""
    for d in epics/*/; do
        [[ -d "$d" ]] || continue
        if [[ "$(basename "$d")" == *"$epic"* ]]; then
            epic_dir="$d"
            break
        fi
    done

    if [[ -z "$epic_dir" ]] || [[ ! -d "$epic_dir" ]]; then
        echo "Error: Epic '$epic' not found"
        echo "Available epics:"
        for e in epics/*/; do
            [[ -d "$e" ]] && echo "  - $(basename "$e")"
        done
        exit 1
    fi

    epic_name=$(basename "$epic_dir")
    epic_readme="${epic_dir}README.md"

    if [[ ! -f "$epic_readme" ]]; then
        echo "Error: Epic README not found at $epic_readme"
        exit 1
    fi

    # Check current status
    current_status=$(grep -m1 "^status:" "$epic_readme" 2>/dev/null | sed 's/^status: *//' || echo "")
    if [[ "$current_status" == "done" ]]; then
        echo "Epic '$epic_name' is already done"
        exit 0
    fi

    # Update status
    sed -i 's/^status: .*/status: done/' "$epic_readme"

    # Add completed date if not already present
    date=$(date +%Y-%m-%d)
    if ! grep -q "^completed:" "$epic_readme"; then
        # Insert after status line
        sed -i "/^status: done$/a completed: $date" "$epic_readme"
    fi

    echo "Completed epic: $epic_name"

    # Regenerate README
    just board generate
    echo "Board updated"

# Link a story to an epic
link STORY EPIC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    story_pattern="{{STORY}}"
    epic="{{EPIC}}"

    # Find the story
    story_path=$(just board _find-story "$story_pattern")
    if [[ -z "$story_path" ]]; then
        echo "Error: Story '$story_pattern' not found"
        exit 1
    fi

    # Check epic exists
    if [[ ! -d "epics/$epic" ]]; then
        echo "Error: Epic '$epic' not found"
        echo "Available epics:"
        for e in epics/*/; do
            [[ -d "$e" ]] && echo "  - $(basename "$e")"
        done
        exit 1
    fi

    story_name=$(basename "$story_path")

    # Determine relative path based on story location
    if [[ "$story_path" == *"backlog"* ]]; then
        rel_path="../../stages/backlog/stories/$story_name"
    elif [[ "$story_path" == *"in-progress"* ]]; then
        rel_path="../../stages/in-progress/stories/$story_name"
    else
        rel_path="../../stages/done/stories/$story_name"
    fi

    # Create symlink
    link_path="epics/$epic/$story_name"
    if [[ -L "$link_path" ]]; then
        echo "Story already linked to epic"
    else
        ln -sf "$rel_path" "$link_path"
        echo "Linked $story_name to epic $epic"
    fi

    # Update frontmatter epics field
    current_epics=$(grep -m1 "^epics:" "$story_path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")
    if [[ -z "$current_epics" ]]; then
        new_epics="[$epic]"
    # Use word boundary matching to avoid substring matches (e.g., 'cli' matching 'cli-tools')
    elif ! echo ", $current_epics," | grep -q ", *$epic *,"; then
        new_epics="[$current_epics, $epic]"
    else
        new_epics="[$current_epics]"
    fi
    sed -i "s/^epics: .*/epics: $new_epics/" "$story_path"
    echo "Updated story frontmatter"

# Unlink a story from an epic
unlink STORY EPIC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    story_pattern="{{STORY}}"
    epic="{{EPIC}}"

    # Find the story
    story_path=$(just board _find-story "$story_pattern")
    if [[ -z "$story_path" ]]; then
        echo "Error: Story '$story_pattern' not found"
        exit 1
    fi

    story_name=$(basename "$story_path")
    link_path="epics/$epic/$story_name"

    # Remove symlink
    if [[ -L "$link_path" ]]; then
        rm "$link_path"
        echo "Unlinked $story_name from epic $epic"
    else
        echo "Story was not linked to epic $epic"
    fi

    # Update frontmatter epics field
    current_epics=$(grep -m1 "^epics:" "$story_path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")
    # Remove the epic from the list using word boundary matching
    # Convert to comma-separated with padding, remove exact match, clean up
    new_epics=$(echo ", $current_epics," | sed "s/, *$epic *,/,/g" | sed 's/^, *//;s/, *$//' | sed 's/  */ /g')
    sed -i "s/^epics: .*/epics: [$new_epics]/" "$story_path"
    echo "Updated story frontmatter"

# Link an epic to a milestone
link-epic EPIC MILESTONE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    epic="{{EPIC}}"
    milestone="{{MILESTONE}}"

    # Check epic exists
    if [[ ! -d "epics/$epic" ]]; then
        echo "Error: Epic '$epic' not found"
        exit 1
    fi

    # Find milestone directory (allow partial match)
    ms_dir=""
    for d in milestones/*/; do
        if [[ "$(basename "$d")" == *"$milestone"* ]]; then
            ms_dir="$d"
            break
        fi
    done

    if [[ -z "$ms_dir" ]] || [[ ! -d "$ms_dir" ]]; then
        echo "Error: Milestone '$milestone' not found"
        echo "Available milestones:"
        for m in milestones/*/; do
            [[ -d "$m" ]] && echo "  - $(basename "$m")"
        done
        exit 1
    fi

    ms_name=$(basename "$ms_dir")

    # Create symlink to epic in milestone directory
    link_path="${ms_dir}${epic}"
    if [[ -L "$link_path" ]]; then
        echo "Epic already linked to milestone"
    else
        ln -sf "../../epics/$epic" "$link_path"
        echo "Linked epic $epic to milestone $ms_name"
    fi

    # Update milestone README frontmatter
    ms_readme="${ms_dir}README.md"
    if [[ -f "$ms_readme" ]]; then
        current_epics=$(grep -m1 "^epics:" "$ms_readme" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")
        if [[ -z "$current_epics" ]]; then
            new_epics="[$epic]"
        elif [[ "$current_epics" != *"$epic"* ]]; then
            new_epics="[$current_epics, $epic]"
        else
            new_epics="[$current_epics]"
        fi
        sed -i "s/^epics: .*/epics: $new_epics/" "$ms_readme"
        echo "Updated milestone frontmatter"
    fi

# Internal: list epics
_list-epics:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    echo "Epics"
    echo "====="
    echo ""

    if [[ ! -d epics ]]; then
        echo "No epics directory found"
        exit 0
    fi

    for epic_dir in epics/*/; do
        [[ -d "$epic_dir" ]] || continue
        name=$(basename "$epic_dir")

        # Get status from README
        status=""
        [[ -f "${epic_dir}README.md" ]] && status=$(grep -m1 "^status:" "${epic_dir}README.md" 2>/dev/null | sed 's/^status: *//' || echo "")

        # Count symlinks
        count=$(find "$epic_dir" -maxdepth 1 -type l -name "*.md" 2>/dev/null | wc -l | tr -d ' ')

        # Count by status
        backlog=0
        in_prog=0
        done_cnt=0
        for link in "$epic_dir"*.md; do
            [[ -L "$link" ]] || continue
            target=$(readlink "$link")
            if [[ "$target" == *"backlog"* ]]; then
                backlog=$((backlog + 1))
            elif [[ "$target" == *"in-progress"* ]]; then
                in_prog=$((in_prog + 1))
            elif [[ "$target" == *"done"* ]]; then
                done_cnt=$((done_cnt + 1))
            fi
        done

        echo "$name ($status)"
        echo "  Stories: $count (backlog: $backlog, in-progress: $in_prog, done: $done_cnt)"
    done

# Internal: list milestones
_list-milestones:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    echo "Milestones"
    echo "=========="
    echo ""

    if [[ ! -d milestones ]]; then
        echo "No milestones directory found"
        exit 0
    fi

    for ms_dir in milestones/*/; do
        [[ -d "$ms_dir" ]] || continue
        name=$(basename "$ms_dir")

        # Get status from README
        status=""
        [[ -f "${ms_dir}README.md" ]] && status=$(grep -m1 "^status:" "${ms_dir}README.md" 2>/dev/null | sed 's/^status: *//' || echo "")

        # Get linked epics
        epics=""
        [[ -f "${ms_dir}README.md" ]] && epics=$(grep -m1 "^epics:" "${ms_dir}README.md" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")

        # Count epic symlinks
        epic_count=$(find "$ms_dir" -maxdepth 1 -type l -not -name "*.md" 2>/dev/null | wc -l | tr -d ' ')

        echo "$name ($status)"
        [[ -n "$epics" ]] && echo "  Epics: $epics"
        echo "  Epic links: $epic_count"
    done

# Router for list command
list TYPE:
    #!/usr/bin/env bash
    type="{{TYPE}}"

    case "$type" in
        epics)
            just board _list-epics
            ;;
        milestones)
            just board _list-milestones
            ;;
        *)
            echo "Error: Unknown type '$type'"
            echo "Usage: just board list <epics|milestones>"
            exit 1
            ;;
    esac

# Show story details
show ID:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    path=$(just board _find-story "{{ID}}")

    if [[ -z "$path" ]]; then
        echo "Error: Story '{{ID}}' not found"
        exit 1
    fi

    name=$(basename "$path" .md)
    echo "Story: $name"
    echo "Path: $path"
    echo ""

    # Extract frontmatter fields
    echo "Frontmatter:"
    sed -n '/^---$/,/^---$/p' "$path" | grep -v "^---$" | while read -r line; do
        echo "  $line"
    done
    echo ""

    # Show linked epics
    epics=$(grep -m1 "^epics:" "$path" 2>/dev/null | sed 's/^epics: *//' | tr -d '[]' || echo "")
    if [[ -n "$epics" ]]; then
        echo "Linked Epics:"
        IFS=', ' read -ra epic_list <<< "$epics"
        for epic in "${epic_list[@]}"; do
            epic=$(echo "$epic" | tr -d ' ')
            if [[ -d "epics/$epic" ]]; then
                echo "  - $epic"
            fi
        done
        echo ""
    fi

    # Show dependencies
    depends=$(grep -m1 "^depends:" "$path" 2>/dev/null | sed 's/^depends: *//' | tr -d '[]' || echo "")
    if [[ -n "$depends" ]] && [[ "$depends" != "" ]]; then
        echo "Dependencies:"
        IFS=', ' read -ra dep_list <<< "$depends"
        for dep in "${dep_list[@]}"; do
            dep=$(echo "$dep" | tr -d ' ')
            dep_path=$(just board _find-story "$dep")
            if [[ -n "$dep_path" ]]; then
                dep_status=$(grep -m1 "^status:" "$dep_path" 2>/dev/null | sed 's/^status: *//' || echo "unknown")
                echo "  - $dep ($dep_status)"
            else
                echo "  - $dep (not found)"
            fi
        done
    fi

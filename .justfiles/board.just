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
    echo "  just board new story \"title\"  Create story in backlog"
    echo "  just board start <id>         Move story to in-progress"
    echo "  just board done <id>          Move story to done + changelog"
    echo ""
    echo "Epic/Milestone management:"
    echo "  just board new epic \"name\"      Create epic directory"
    echo "  just board new milestone \"name\" Create milestone directory"
    echo "  just board link <story> <epic>   Add story to epic"
    echo "  just board unlink <story> <epic> Remove story from epic"
    echo "  just board link-epic <epic> <milestone>  Add epic to milestone"
    echo ""
    echo "Utility:"
    echo "  just board list epics      Show all epics with story counts"
    echo "  just board list milestones Show all milestones with progress"
    echo "  just board show <id>       Display story details"

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

    # Helper: render checkbox based on status
    story_checkbox() {
        local status=$(get_status "$1")
        case "$status" in
            done) echo "[x]" ;;
            in-progress) echo "[~]" ;;
            *) echo "[ ]" ;;
        esac
    }

    echo "# Planning Board"
    echo ""
    echo "> Auto-generated from directory structure. Run \`just board generate\` to update."
    echo ""

    # In Progress stories
    echo "## In Progress"
    echo ""
    if [[ -d stages/in-progress/stories ]] && ls stages/in-progress/stories/*.md &>/dev/null; then
        echo "| Story | Type | Priority | Epics |"
        echo "|-------|------|----------|-------|"
        for story in stages/in-progress/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            title=$(get_field "$story" "title")
            type=$(get_field "$story" "type")
            priority=$(get_field "$story" "priority")
            epics=$(get_field "$story" "epics" | tr -d '[]')
            [[ -z "$title" ]] && title="$name"
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
        echo "| Story | Type | Priority | Epics |"
        echo "|-------|------|----------|-------|"
        for story in stages/backlog/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            title=$(get_field "$story" "title")
            type=$(get_field "$story" "type")
            priority=$(get_field "$story" "priority")
            epics=$(get_field "$story" "epics" | tr -d '[]')
            [[ -z "$title" ]] && title="$name"
            echo "| [$name](stages/backlog/stories/$name.md) | $type | $priority | $epics |"
        done
    else
        echo "*Backlog empty*"
    fi
    echo ""

    # Epics
    echo "## Epics"
    echo ""
    if [[ -d epics ]]; then
        echo "| Epic | Status | Stories |"
        echo "|------|--------|---------|"
        for epic_dir in epics/*/; do
            [[ -d "$epic_dir" ]] || continue
            name=$(basename "$epic_dir")
            [[ -f "${epic_dir}README.md" ]] || continue
            status=$(get_field "${epic_dir}README.md" "status")
            # Count symlinks (stories) in epic dir
            count=$(find "$epic_dir" -maxdepth 1 -type l -name "*.md" 2>/dev/null | wc -l | tr -d ' ')
            echo "| [$name](epics/$name/) | $status | $count |"
        done
    fi
    echo ""

    # Milestones
    echo "## Milestones"
    echo ""
    if [[ -d milestones ]]; then
        echo "| Milestone | Status | Epics |"
        echo "|-----------|--------|-------|"
        for ms_dir in milestones/*/; do
            [[ -d "$ms_dir" ]] || continue
            name=$(basename "$ms_dir")
            [[ -f "${ms_dir}README.md" ]] || continue
            status=$(get_field "${ms_dir}README.md" "status")
            epics=$(get_field "${ms_dir}README.md" "epics" | tr -d '[]')
            echo "| [$name](milestones/$name/) | $status | $epics |"
        done
    fi
    echo ""

    # Done stories (collapsed)
    echo "## Done"
    echo ""
    echo "<details>"
    echo "<summary>View completed stories</summary>"
    echo ""
    if [[ -d stages/done/stories ]] && ls stages/done/stories/*.md &>/dev/null; then
        for story in stages/done/stories/*.md; do
            [[ -f "$story" ]] || continue
            name=$(basename "$story" .md)
            echo "- [$name](stages/done/stories/$name.md)"
        done
    else
        echo "*No completed stories*"
    fi
    echo ""
    echo "</details>"

# Show board status summary
status:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    # Count stories in each stage
    in_progress=$(find stages/in-progress/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')
    backlog=$(find stages/backlog/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')
    done_count=$(find stages/done/stories -name "*.md" ! -name ".gitkeep" 2>/dev/null | wc -l | tr -d ' ')

    echo "Board Status"
    echo "============"
    echo ""
    echo "Stories by Stage:"
    echo "  In Progress: $in_progress"
    echo "  Backlog:     $backlog"
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
    max=0

    # Search in all stages for stories with this type prefix
    for f in $(find stages -name "${type}-*.md" -o -name "*-${type}-*.md" 2>/dev/null); do
        # Extract sequence number after type prefix (e.g., feat-01, m37-feat-01)
        # Pattern: TYPE-NN where NN is the sequence number we want
        num=$(basename "$f" .md | grep -oE "(feat|bug|chore|refactor|fix|docs)-[0-9]+" | grep -oE "[0-9]+$")
        # Skip if no match found
        [[ -z "$num" ]] && continue
        # Strip leading zeros to avoid octal interpretation (e.g., 08 -> 8)
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
_new-story TITLE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    title="{{TITLE}}"

    # Prompt for type
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

    # Generate ID
    id=$(just board _next-id "$type")

    # Prompt for epics
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

    selected_epics=""
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

    # Create slug
    slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    filename="stages/backlog/stories/${type}-${id}-${slug}.md"

    # Create the story file
    date=$(date +%Y-%m-%d)
    {
        echo '---'
        echo "id: ${type^^}${id}"
        echo "title: ${title}"
        echo "type: ${type}"
        echo "status: backlog"
        echo "priority: medium"
        echo "epics: [${selected_epics}]"
        echo "depends: []"
        echo "estimate:"
        echo "created: ${date}"
        echo "updated: ${date}"
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
_new-epic NAME:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"
    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    dir="epics/$slug"

    if [[ -d "$dir" ]]; then
        echo "Error: Epic '$slug' already exists"
        exit 1
    fi

    mkdir -p "$dir"

    read -p "Description (one line): " description

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
new TYPE NAME:
    #!/usr/bin/env bash
    type="{{TYPE}}"
    name="{{NAME}}"

    case "$type" in
        story)
            just board _new-story "$name"
            ;;
        epic)
            just board _new-epic "$name"
            ;;
        milestone)
            just board _new-milestone "$name"
            ;;
        *)
            echo "Error: Unknown type '$type'"
            echo "Usage: just board new <story|epic|milestone> \"name\""
            exit 1
            ;;
    esac

    # Regenerate README
    just board generate > "{{board_dir}}/README.md"
    echo ""
    echo "Board README.md updated"

# Find story by ID or pattern
_find-story PATTERN:
    #!/usr/bin/env bash
    cd "{{board_dir}}"
    pattern="{{PATTERN}}"

    # Search in stages (backlog, in-progress, done)
    for stage in backlog in-progress done; do
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
    just board generate > README.md
    echo "Board updated"

# Complete a story (move to done)
done ID:
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

    # Prompt for changelog entry
    title=$(grep -m1 "^title:" "$path" 2>/dev/null | sed 's/^title: *//' || basename "$path" .md)
    echo "Enter changelog summary for '$title' (or Enter to skip):"
    read -r summary

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
    just board generate > README.md
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

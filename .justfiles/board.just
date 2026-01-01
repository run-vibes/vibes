# Board management commands
# Usage: just board, just board new, just board start, etc.

# Use parent of .justfiles directory (project root) + docs/board
board_dir := source_directory() / ".." / "docs" / "board"

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
            {
                echo "---"
                echo "created: $(date +%Y-%m-%d)"
                echo "---"
                echo ""
                echo "# ${type}-${id}: ${desc}"
                echo ""
                echo "## Summary"
                echo ""
                echo "[Description of the ${type}]"
                echo ""
                echo "## Implementation"
                echo ""
                echo "[Steps to implement]"
            } > "$filename"
            echo "✓ Created: $filename"
            ;;
        milestone)
            id=$(just board _next-milestone)
            dirname="backlog/milestone-${id}-${slug}"
            mkdir -p "$dirname"
            {
                echo "---"
                echo "created: $(date +%Y-%m-%d)"
                echo "---"
                echo ""
                echo "# Milestone ${id}: ${desc} - Design"
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
            } > "${dirname}/design.md"
            echo "✓ Created: ${dirname}/design.md"
            ;;
        *)
            echo "Error: Unknown type '$type'. Use: feat, bug, chore, milestone"
            exit 1
            ;;
    esac

    just board generate > README.md
    echo "✓ Board updated"

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

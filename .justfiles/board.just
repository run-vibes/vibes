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

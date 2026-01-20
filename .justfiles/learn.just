# Learning capture commands
# Captures and manages learnings from development work

# Run commands from project root
set working-directory := '..'

# Learnings output directory
learnings_dir := "docs/learnings"

# Default: show available learn commands
default:
    @echo "Learning Commands"
    @echo ""
    @echo "  just learn reflect    Interactive ad-hoc learning capture"
    @echo "  just learn list       Show all learnings with status"
    @echo "  just learn apply      Suggest and apply pending learnings"

# Interactive ad-hoc learning capture
reflect:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "=== Ad-hoc Learning Capture ==="
    echo ""
    echo "Topic (short slug, e.g., 'test-patterns', 'api-design'):"
    read -r topic

    if [[ -z "$topic" ]]; then
        echo "Topic is required. Aborting."
        exit 1
    fi

    # Sanitize topic to slug format
    slug=$(echo "$topic" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | sed 's/[^a-z0-9-]//g')

    # Generate filename with date
    date_prefix=$(date +%Y-%m-%d)
    base_file="{{learnings_dir}}/${date_prefix}-${slug}.md"

    # Handle multiple reflections on same day
    file="$base_file"
    counter=2
    while [[ -f "$file" ]]; do
        file="{{learnings_dir}}/${date_prefix}-${slug}-${counter}.md"
        ((counter++))
    done

    # Prompt for reflection (same questions as story completion)
    echo ""
    echo "What triggered this reflection? (context)"
    read -r context

    echo ""
    echo "What did you learn? (insight)"
    read -r insight

    echo ""
    echo "What action should be taken? (Enter to skip)"
    read -r action

    echo ""
    echo "What files/conventions should this apply to? (Enter to skip)"
    read -r applies_to

    # Default values
    [[ -z "$action" ]] && action="None specified"
    [[ -z "$applies_to" ]] && applies_to="To be determined"

    # Generate a title from the topic
    title=$(echo "$slug" | sed 's/-/ /g' | sed 's/\b\(.\)/\u\1/g')

    # Create the learning file
    {
        echo "# Learning: ${title}"
        echo ""
        echo "> Ad-hoc learning captured on ${date_prefix}"
        echo ""
        echo "### L001: ${title}"
        echo ""
        echo "| Field | Value |"
        echo "|-------|-------|"
        echo "| **Category** | (to be determined) |"
        echo "| **Context** | ${context} |"
        echo "| **Insight** | ${insight} |"
        echo "| **Suggested Action** | ${action} |"
        echo "| **Applies To** | ${applies_to} |"
        echo "| **Applied** | |"
    } > "$file"

    echo ""
    echo "Created: $file"

# Show all learnings with status
list:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Learnings"
    echo "========="
    echo ""

    if [[ ! -d "{{learnings_dir}}" ]]; then
        echo "No learnings directory found"
        exit 0
    fi

    files=$(find "{{learnings_dir}}" -name "*.md" ! -name ".gitkeep" 2>/dev/null | sort -r)

    if [[ -z "$files" ]]; then
        echo "No learnings captured yet."
        echo ""
        echo "Use 'just learn reflect' to capture a learning."
        exit 0
    fi

    echo "| Date | Topic | Applied |"
    echo "|------|-------|---------|"

    for file in $files; do
        name=$(basename "$file" .md)
        # Extract date (first 10 chars YYYY-MM-DD)
        file_date="${name:0:10}"
        # Extract topic (rest after date and hyphen)
        topic="${name:11}"

        # Count total learnings (L### headers)
        total_count=$(grep -cE "^### L[0-9]+" "$file" 2>/dev/null || echo "0")
        total_count=$(echo "$total_count" | tr -d '[:space:]')

        # Count applied learnings (Applied field with content after the empty space)
        # Lines look like: | **Applied** | | (empty) or | **Applied** | some text |
        applied_count=0
        while IFS= read -r line; do
            # Check if the Applied value (between last two |) has content
            value=$(echo "$line" | sed 's/.*\*\*Applied\*\* | *//' | sed 's/ *|$//')
            if [[ -n "$value" ]]; then
                applied_count=$((applied_count + 1))
            fi
        done < <(grep -F "| **Applied**" "$file" 2>/dev/null || true)

        if [[ "$total_count" == "0" ]]; then
            applied_status="-"
        elif [[ "$applied_count" -gt 0 ]]; then
            applied_status="${applied_count}/${total_count}"
        else
            applied_status="0/${total_count}"
        fi

        echo "| $file_date | $topic | $applied_status |"
    done

# Suggest and apply pending learnings using AI
apply *ARGS:
    npx tsx verification/ai/learn-apply.ts {{ARGS}}

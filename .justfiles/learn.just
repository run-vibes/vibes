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
    @echo "  just learn reflect       Interactive ad-hoc learning capture"
    @echo "  just learn list          Show all learnings with status"
    @echo "  just learn show <path>   Show full details of a specific learning"
    @echo "  just learn apply         Suggest and apply pending learnings"
    @echo ""
    @echo "Path format for 'show':"
    @echo "  scope/story/id  - e.g., coherence-verification/05-learnings-capture/CHORE0202/L001"
    @echo "  story/id        - e.g., FEAT0199/L001"
    @echo "  id              - e.g., L001 (shows all with that ID)"

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
list *ARGS:
    npx tsx verification/ai/learn-list.ts {{ARGS}}

# Show full details of a specific learning
show PATH:
    npx tsx verification/ai/learn-show.ts {{PATH}}

# Suggest and apply pending learnings using AI
apply *ARGS:
    npx tsx verification/ai/learn-apply.ts {{ARGS}}

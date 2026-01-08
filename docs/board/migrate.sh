#!/usr/bin/env bash
#
# Migration script for board restructure
# Moves stories from milestone directories to stages/, adds frontmatter, creates symlinks
#
set -euo pipefail

BOARD_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BOARD_DIR"

# Epic mapping from milestone keywords to epic names
# Maps partial milestone names to comma-separated epic lists
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

# Counters for generating unique story IDs (separate counter per type)
FEAT_COUNTER=1
BUG_COUNTER=1
CHORE_COUNTER=1
REFACTOR_COUNTER=1
DOCS_COUNTER=1

# Dry run mode (set to "false" to actually migrate)
DRY_RUN="${DRY_RUN:-true}"

log() {
    echo "[migrate] $*"
}

debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo "[debug] $*"
    fi
}

# Get epics for a milestone name
# Args: milestone_name (e.g., "milestone-26-assessment-framework")
# Returns: comma-separated epic list (e.g., "core,cli,plugin-system")
get_epics() {
    local milestone_name="$1"
    local epics="core"  # Default

    for key in "${!EPIC_MAP[@]}"; do
        if [[ "$milestone_name" == *"$key"* ]]; then
            epics="${EPIC_MAP[$key]}"
            break
        fi
    done

    echo "$epics"
}

# Extract story type from filename
# Args: filename (e.g., "feat-01-design-tokens.md")
# Returns: type (feat, bug, chore, refactor, fix, docs)
get_story_type() {
    local filename="$1"
    local basename
    basename=$(basename "$filename" .md)

    if [[ "$basename" =~ ^(feat|bug|chore|refactor|fix|docs)- ]]; then
        echo "${BASH_REMATCH[1]}"
    else
        echo "feat"  # Default
    fi
}

# Generate a unique story ID and increment counter
# Args: type (feat, bug, etc.), optional existing_id
# Sets: LAST_STORY_ID variable with generated ID
# Note: Uses global counters to avoid subshell issues
generate_story_id() {
    local type="$1"
    local existing_id="${2:-}"

    # If we have an existing ID in the right format, use it
    if [[ -n "$existing_id" && "$existing_id" =~ ^[FBCRD][0-9]{3}$ ]]; then
        LAST_STORY_ID="$existing_id"
        return
    fi

    case "$type" in
        feat|feature)
            LAST_STORY_ID=$(printf "F%03d" "$FEAT_COUNTER")
            ((FEAT_COUNTER++))
            ;;
        bug)
            LAST_STORY_ID=$(printf "B%03d" "$BUG_COUNTER")
            ((BUG_COUNTER++))
            ;;
        fix)
            # fix maps to bug
            LAST_STORY_ID=$(printf "B%03d" "$BUG_COUNTER")
            ((BUG_COUNTER++))
            ;;
        chore)
            LAST_STORY_ID=$(printf "C%03d" "$CHORE_COUNTER")
            ((CHORE_COUNTER++))
            ;;
        refactor)
            LAST_STORY_ID=$(printf "R%03d" "$REFACTOR_COUNTER")
            ((REFACTOR_COUNTER++))
            ;;
        docs)
            LAST_STORY_ID=$(printf "D%03d" "$DOCS_COUNTER")
            ((DOCS_COUNTER++))
            ;;
        *)
            LAST_STORY_ID=$(printf "F%03d" "$FEAT_COUNTER")
            ((FEAT_COUNTER++))
            ;;
    esac
}

# Extract title from markdown file
# Args: filepath
# Returns: title string
get_title_from_file() {
    local filepath="$1"

    # Try to get title from first H1
    local title
    title=$(grep -m1 '^# ' "$filepath" 2>/dev/null | sed 's/^# //' || true)

    if [[ -z "$title" ]]; then
        # Fall back to filename
        title=$(basename "$filepath" .md | sed 's/^[a-z]*-[0-9]*-//' | tr '-' ' ')
    fi

    echo "$title"
}

# Check if file has YAML frontmatter
# Args: filepath
# Returns: 0 if has frontmatter, 1 if not
has_frontmatter() {
    local filepath="$1"
    head -1 "$filepath" | grep -q '^---$'
}

# Extract value from frontmatter
# Args: filepath, key
# Returns: value or empty string
get_frontmatter_value() {
    local filepath="$1"
    local key="$2"

    if ! has_frontmatter "$filepath"; then
        return
    fi

    # Extract frontmatter block and find key
    # Use || true to handle missing keys gracefully with pipefail enabled
    sed -n '1,/^---$/p' "$filepath" | tail -n +2 | head -n -1 | \
        grep "^${key}:" | sed "s/^${key}: *//" | tr -d '"' | tr -d "'" || true
}

# Create or update frontmatter for a story
# Args: filepath, story_id, type, status, epics_csv, milestone_name
# Outputs: new file content to stdout
create_story_frontmatter() {
    local filepath="$1"
    local story_id="$2"
    local type="$3"
    local status="$4"
    local epics_csv="$5"
    local milestone_name="$6"

    local title
    title=$(get_title_from_file "$filepath")

    local priority="medium"
    local created
    local updated
    created=$(date +%Y-%m-%d)
    updated="$created"

    # Try to preserve existing values
    if has_frontmatter "$filepath"; then
        local existing_status existing_created existing_priority
        existing_status=$(get_frontmatter_value "$filepath" "status")
        existing_created=$(get_frontmatter_value "$filepath" "created")
        existing_priority=$(get_frontmatter_value "$filepath" "priority")

        [[ -n "$existing_status" ]] && status="$existing_status"
        [[ -n "$existing_created" ]] && created="$existing_created"
        [[ -n "$existing_priority" ]] && priority="$existing_priority"
    fi

    # Convert status values
    case "$status" in
        done|completed) status="done" ;;
        in-progress|in_progress|active) status="in-progress" ;;
        *) status="backlog" ;;
    esac

    # Format epics as YAML array
    local epics_yaml
    epics_yaml="[$(echo "$epics_csv" | sed 's/,/, /g')]"

    # Build frontmatter
    cat <<EOF
---
id: $story_id
title: $title
type: $type
status: $status
priority: $priority
epics: $epics_yaml
depends: []
estimate:
created: $created
updated: $updated
milestone: $milestone_name
---
EOF

    # Append rest of file (skip old frontmatter if present)
    if has_frontmatter "$filepath"; then
        # Skip frontmatter (everything from line 1 to the closing ---)
        sed '1,/^---$/d' "$filepath"
    else
        cat "$filepath"
    fi
}

# Create symlink in epic directory
# Args: story_file (relative to stages/), epic_name
create_epic_symlink() {
    local story_file="$1"
    local epic_name="$2"
    local story_basename
    story_basename=$(basename "$story_file")

    local epic_dir="epics/$epic_name"
    local link_path="$epic_dir/$story_basename"
    local target="../../$story_file"

    if [[ ! -d "$epic_dir" ]]; then
        log "Warning: Epic directory $epic_dir does not exist, skipping symlink"
        return
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log "[dry-run] Would create symlink: $link_path -> $target"
    else
        mkdir -p "$epic_dir"
        ln -sf "$target" "$link_path"
        log "Created symlink: $link_path -> $target"
    fi
}

# Process a single story file
# Args: src_path, stage (done|in-progress|backlog), milestone_name
process_story() {
    local src_path="$1"
    local stage="$2"
    local milestone_name="${3:-}"

    local filename
    filename=$(basename "$src_path")
    local type
    type=$(get_story_type "$filename")

    # Generate story ID (sets LAST_STORY_ID)
    generate_story_id "$type"
    local story_id="$LAST_STORY_ID"

    # Determine epics from milestone or use default
    local epics_csv
    if [[ -n "$milestone_name" ]]; then
        epics_csv=$(get_epics "$milestone_name")
    else
        # Standalone items - try to infer from content or use default
        epics_csv="core"
    fi

    # Extract milestone number and prefix filename to avoid collisions
    # e.g., "milestone-36-firehose" -> "m36-feat-01-backend.md"
    local dest_filename="$filename"
    if [[ -n "$milestone_name" ]]; then
        local milestone_num
        milestone_num=$(echo "$milestone_name" | grep -oE 'milestone-([0-9]+)' | grep -oE '[0-9]+' || true)
        if [[ -n "$milestone_num" ]]; then
            dest_filename="m${milestone_num}-${filename}"
        fi
    fi

    # Target path
    local dest_dir="stages/$stage/stories"
    local dest_path="$dest_dir/$dest_filename"

    debug "Processing: $src_path -> $dest_path (epics: $epics_csv)"

    if [[ "$DRY_RUN" == "true" ]]; then
        log "[dry-run] Would migrate: $src_path -> $dest_path"
        log "[dry-run]   Story ID: $story_id, Type: $type, Epics: $epics_csv"
    else
        mkdir -p "$dest_dir"

        # Create new file with frontmatter
        local temp_file
        temp_file=$(mktemp)
        create_story_frontmatter "$src_path" "$story_id" "$type" "$stage" "$epics_csv" "$milestone_name" > "$temp_file"
        mv "$temp_file" "$dest_path"

        log "Migrated: $src_path -> $dest_path"

        # Create symlinks in epic directories
        IFS=',' read -ra epic_array <<< "$epics_csv"
        for epic in "${epic_array[@]}"; do
            create_epic_symlink "stages/$stage/stories/$dest_filename" "$epic"
        done
    fi
}

# Process a milestone directory
# Args: milestone_dir, stage
process_milestone() {
    local milestone_dir="$1"
    local stage="$2"

    local milestone_name
    milestone_name=$(basename "$milestone_dir")
    log "Processing milestone: $milestone_name (stage: $stage)"

    # Check for stories directory
    local stories_dir="$milestone_dir/stories"
    if [[ -d "$stories_dir" ]]; then
        for story_file in "$stories_dir"/*.md; do
            [[ -f "$story_file" ]] || continue
            process_story "$story_file" "$stage" "$milestone_name"
        done
    else
        debug "No stories directory in $milestone_name"
    fi

    # Create milestone entry in milestones/ directory
    local epics_csv
    epics_csv=$(get_epics "$milestone_name")

    # Extract milestone number and short name
    local milestone_short_name
    milestone_short_name=$(echo "$milestone_name" | sed 's/^milestone-[0-9]*-//')
    local milestone_id
    milestone_id=$(echo "$milestone_name" | grep -oE '[0-9]+' | head -1)

    local milestone_dest="milestones/$milestone_name"

    if [[ "$DRY_RUN" == "true" ]]; then
        log "[dry-run] Would create milestone: $milestone_dest"
    else
        mkdir -p "$milestone_dest"

        # Copy design docs if they exist
        for doc in design.md implementation.md decisions.md; do
            if [[ -f "$milestone_dir/$doc" ]]; then
                cp "$milestone_dir/$doc" "$milestone_dest/"
                log "Copied: $milestone_dir/$doc -> $milestone_dest/$doc"
            fi
        done

        # Copy reference directory if it exists
        if [[ -d "$milestone_dir/reference" ]]; then
            cp -r "$milestone_dir/reference" "$milestone_dest/"
            log "Copied: $milestone_dir/reference -> $milestone_dest/reference"
        fi

        # Determine milestone status from stage
        local milestone_status
        case "$stage" in
            done) milestone_status="done" ;;
            in-progress) milestone_status="in-progress" ;;
            *) milestone_status="planned" ;;
        esac

        # Create README.md with frontmatter
        local title
        title=$(echo "$milestone_short_name" | tr '-' ' ' | sed 's/\b\(.\)/\u\1/g')

        cat > "$milestone_dest/README.md" <<EOF
---
id: $milestone_id-$milestone_short_name
title: $title
status: $milestone_status
epics: [$(echo "$epics_csv" | sed 's/,/, /g')]
---

# $title

## Overview

Milestone $milestone_id: $title

## Epics

EOF

        # Create symlinks to epic directories
        IFS=',' read -ra epic_array <<< "$epics_csv"
        for epic in "${epic_array[@]}"; do
            if [[ -d "epics/$epic" ]]; then
                ln -sf "../../epics/$epic" "$milestone_dest/$epic"
                echo "- [$epic](epics/$epic)" >> "$milestone_dest/README.md"
            fi
        done

        # Add design doc links
        cat >> "$milestone_dest/README.md" <<EOF

## Design

EOF
        if [[ -f "$milestone_dest/design.md" ]]; then
            echo "See [design.md](design.md) for architecture decisions." >> "$milestone_dest/README.md"
        else
            echo "_No design document._" >> "$milestone_dest/README.md"
        fi

        if [[ -f "$milestone_dest/implementation.md" ]]; then
            echo "" >> "$milestone_dest/README.md"
            echo "See [implementation.md](implementation.md) for implementation plan." >> "$milestone_dest/README.md"
        fi

        log "Created milestone: $milestone_dest"
    fi
}

# Process standalone items (bug-*.md, feat-*.md, etc.) in a stage directory
# Args: stage_dir, stage
process_standalone_items() {
    local stage_dir="$1"
    local stage="$2"

    # Find standalone markdown files (not directories)
    for item in "$stage_dir"/*.md; do
        [[ -f "$item" ]] || continue
        local basename
        basename=$(basename "$item")

        # Skip files that don't look like work items
        if [[ ! "$basename" =~ ^(bug|feat|chore|fix|refactor|docs)- ]]; then
            debug "Skipping non-work-item: $basename"
            continue
        fi

        log "Processing standalone item: $item"
        process_story "$item" "$stage" ""
    done
}

# Process a stage directory (done, in-progress, backlog)
# Args: stage, src_dir
process_stage() {
    local stage="$1"
    local src_dir="$2"

    log "=== Processing stage: $stage ($src_dir) ==="

    if [[ ! -d "$src_dir" ]]; then
        log "Stage directory $src_dir does not exist, skipping"
        return
    fi

    # Process milestone directories
    for milestone_dir in "$src_dir"/milestone-*/; do
        [[ -d "$milestone_dir" ]] || continue
        process_milestone "${milestone_dir%/}" "$stage"
    done

    # Process standalone items
    process_standalone_items "$src_dir" "$stage"
}

# Main
main() {
    log "Board Migration Script"
    log "======================"
    log "Board directory: $BOARD_DIR"
    log "Dry run: $DRY_RUN"
    log ""

    if [[ "$DRY_RUN" == "true" ]]; then
        log "Running in DRY RUN mode. Set DRY_RUN=false to actually migrate."
        log ""
    fi

    # Ensure target directories exist
    if [[ "$DRY_RUN" != "true" ]]; then
        mkdir -p stages/done/stories
        mkdir -p stages/in-progress/stories
        mkdir -p stages/backlog/stories
        mkdir -p milestones
    fi

    # Process each stage
    process_stage "done" "done"
    process_stage "in-progress" "in-progress"
    process_stage "backlog" "backlog"

    log ""
    log "=== Migration Summary ==="
    local total_stories=$((FEAT_COUNTER + BUG_COUNTER + CHORE_COUNTER + REFACTOR_COUNTER + DOCS_COUNTER - 5))
    log "Stories processed: $total_stories"
    log "  Features: $((FEAT_COUNTER - 1))"
    log "  Bugs/Fixes: $((BUG_COUNTER - 1))"
    log "  Chores: $((CHORE_COUNTER - 1))"
    log "  Refactors: $((REFACTOR_COUNTER - 1))"
    log "  Docs: $((DOCS_COUNTER - 1))"

    if [[ "$DRY_RUN" == "true" ]]; then
        log ""
        log "This was a dry run. To actually migrate, run:"
        log "  DRY_RUN=false ./migrate.sh"
    else
        log "Migration complete!"
        log ""
        log "Next steps:"
        log "  1. Review migrated files in stages/"
        log "  2. Check symlinks in epics/"
        log "  3. Verify milestone directories in milestones/"
        log "  4. Run 'just board' to regenerate README.md"
    fi
}

main "$@"

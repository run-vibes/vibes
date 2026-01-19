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
    echo "  just board new milestone \"name\" [epic]"
    echo "                              Create milestone with README, SRS, DESIGN"
    echo "  just board start-milestone <id>"
    echo "                              Set milestone to in-progress"
    echo "  just board done-milestone <id>"
    echo "                              Set milestone to done"
    echo "  just board done-epic <id>       Set epic to done"
    echo "  just board link <story> <milestone>"
    echo "                              Link story to milestone"
    echo "  just board unlink <story> <milestone>"
    echo "                              Remove story from milestone"
    echo ""
    echo "Utility:"
    echo "  just board list epics      Show all epics with milestone counts"
    echo "  just board list milestones Show all milestones with story counts"
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

                echo "### [$epic_name](epics/$epic_name/) ($epic_status) - $total_ms milestones, $done_ms done"
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

    # Update milestone READMEs with story progress
    for epic_dir in epics/*/; do
        [[ -d "${epic_dir}milestones" ]] || continue
        epic_id=$(basename "$epic_dir")

        for ms_dir in "${epic_dir}milestones"/*/; do
            [[ -d "$ms_dir" ]] || continue
            ms_id=$(basename "$ms_dir")
            ms_readme="${ms_dir}README.md"
            [[ -f "$ms_readme" ]] || continue

            # Build the scope path to match stories
            scope_path="${epic_id}/${ms_id}"

            # Collect stories matching this scope from all stages
            declare -a story_files=()
            for stage in backlog in-progress done icebox; do
                stage_dir="stages/${stage}/stories"
                [[ -d "$stage_dir" ]] || continue
                while IFS= read -r -d '' story_file; do
                    story_scope=$(get_field "$story_file" "scope")
                    if [[ "$story_scope" == "$scope_path" ]]; then
                        story_files+=("$story_file")
                    fi
                done < <(find "$stage_dir" -maxdepth 1 -name "*.md" -print0 2>/dev/null)
            done

            # Skip if no stories found
            [[ ${#story_files[@]} -eq 0 ]] && continue

            # Sort stories by ID number
            IFS=$'\n' sorted_stories=($(for f in "${story_files[@]}"; do
                story_id=$(get_field "$f" "id")
                echo "$story_id|$f"
            done | sort -t'|' -k1 -V | cut -d'|' -f2))
            unset IFS

            # Count stats
            total_stories=${#sorted_stories[@]}
            done_stories=0
            for story_file in "${sorted_stories[@]}"; do
                story_status=$(get_field "$story_file" "status")
                if [[ "$story_status" == "done" ]]; then
                    done_stories=$((done_stories + 1))
                fi
            done

            # Count requirements from SRS.md if it exists
            srs_file="${ms_dir}SRS.md"
            total_reqs=0
            verified_reqs=0
            if [[ -f "$srs_file" ]]; then
                # Count lines matching | SRS-* | pattern (requirement rows)
                total_reqs=$(grep -cE '^\| *SRS-' "$srs_file" 2>/dev/null || echo "0")
            fi

            # Generate new Stories section content
            stories_content=""
            stories_content+="## Stories"$'\n'
            stories_content+=""$'\n'
            stories_content+="| # | Story | Description | Status |"$'\n'
            stories_content+="|---|-------|-------------|--------|"$'\n'

            story_num=1
            for story_file in "${sorted_stories[@]}"; do
                story_id=$(get_field "$story_file" "id")
                story_title=$(grep -m1 "^title:" "$story_file" 2>/dev/null | sed 's/^title: *//' || echo "")
                story_status=$(get_field "$story_file" "status")

                # Build relative path from milestone dir to story
                story_basename=$(basename "$story_file")
                # Determine stage from path
                if [[ "$story_file" == *"/backlog/"* ]]; then
                    rel_path="../../../../stages/backlog/stories/${story_basename}"
                elif [[ "$story_file" == *"/in-progress/"* ]]; then
                    rel_path="../../../../stages/in-progress/stories/${story_basename}"
                elif [[ "$story_file" == *"/done/"* ]]; then
                    rel_path="../../../../stages/done/stories/${story_basename}"
                elif [[ "$story_file" == *"/icebox/"* ]]; then
                    rel_path="../../../../stages/icebox/stories/${story_basename}"
                else
                    rel_path="$story_basename"
                fi

                stories_content+="| ${story_num} | [${story_id}](${rel_path}) | ${story_title} | ${story_status} |"$'\n'
                story_num=$((story_num + 1))
            done

            stories_content+=""$'\n'
            stories_content+="## Progress"$'\n'
            stories_content+=""$'\n'
            stories_content+="**Requirements:** ${verified_reqs}/${total_reqs} verified"$'\n'
            stories_content+="**Stories:** ${done_stories}/${total_stories} complete"$'\n'

            # Update milestone README: replace from ## Stories to end of file
            # First check if ## Stories section exists
            if grep -q "^## Stories" "$ms_readme"; then
                # Extract content before ## Stories, trim trailing blank lines
                head_content=$(sed -n '1,/^## Stories/{ /^## Stories/!p }' "$ms_readme" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}')
                # Write new content with proper spacing
                {
                    echo "$head_content"
                    echo ""
                    echo "$stories_content"
                } > "$ms_readme"
            else
                # Append Stories section if it doesn't exist
                echo "" >> "$ms_readme"
                echo "$stories_content" >> "$ms_readme"
            fi

            echo "Updated ${ms_dir}README.md (${done_stories}/${total_stories} stories complete)"
        done
    done

    # Generate epic READMEs with milestone progress
    for epic_dir in epics/*/; do
        [[ -d "$epic_dir" ]] || continue
        just board _generate-epic-readme "$(basename "$epic_dir")"
    done

# Generate a single epic README with milestone progress
_generate-epic-readme EPIC:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    epic="{{EPIC}}"
    epic_dir="epics/$epic"
    epic_readme="${epic_dir}/README.md"

    if [[ ! -f "$epic_readme" ]]; then
        # No README, skip
        exit 0
    fi

    # Check if epic has milestones directory
    if [[ ! -d "${epic_dir}/milestones" ]]; then
        # No milestones, nothing to generate
        exit 0
    fi

    # Count milestones
    ms_count=$(find "${epic_dir}/milestones" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$ms_count" == "0" ]]; then
        exit 0
    fi

    # Helper to get frontmatter field
    get_field() {
        local file="$1"
        local field="$2"
        grep -m1 "^${field}:" "$file" 2>/dev/null | sed "s/^${field}: *//" || echo ""
    }

    # Collect milestone data
    declare -A ms_status
    declare -A ms_title
    declare -a ms_list=()

    for ms_dir in $(ls -d "${epic_dir}/milestones"/*/ 2>/dev/null | sort -t'/' -k5 -V); do
        [[ -d "$ms_dir" ]] || continue
        ms_name=$(basename "$ms_dir")
        ms_readme="${ms_dir}README.md"
        [[ -f "$ms_readme" ]] || continue

        ms_list+=("$ms_name")
        ms_status["$ms_name"]=$(get_field "$ms_readme" "status" | tr -d ' ')
        ms_title["$ms_name"]=$(get_field "$ms_readme" "title")
    done

    # No milestones with READMEs found
    [[ ${#ms_list[@]} -eq 0 ]] && exit 0

    # Count stories per milestone and totals
    total_stories=0
    done_stories=0
    in_progress_stories=0
    active_milestone=""

    declare -A ms_story_total
    declare -A ms_story_done

    for ms_name in "${ms_list[@]}"; do
        ms_story_total["$ms_name"]=0
        ms_story_done["$ms_name"]=0

        # Track active milestone (first in-progress one)
        if [[ "${ms_status[$ms_name]}" == "in-progress" ]] && [[ -z "$active_milestone" ]]; then
            active_milestone="${ms_title[$ms_name]:-$ms_name}"
        fi

        # Count stories with scope matching this milestone
        scope_pattern="^scope: *${epic}/${ms_name}$"
        for stage in backlog in-progress done icebox; do
            stage_dir="stages/${stage}/stories"
            [[ -d "$stage_dir" ]] || continue

            for story in "$stage_dir"/*.md; do
                [[ -f "$story" ]] || continue
                if grep -qE "$scope_pattern" "$story" 2>/dev/null; then
                    ms_story_total["$ms_name"]=$((ms_story_total["$ms_name"] + 1))
                    total_stories=$((total_stories + 1))

                    if [[ "$stage" == "done" ]]; then
                        ms_story_done["$ms_name"]=$((ms_story_done["$ms_name"] + 1))
                        done_stories=$((done_stories + 1))
                    elif [[ "$stage" == "in-progress" ]]; then
                        in_progress_stories=$((in_progress_stories + 1))
                    fi
                fi
            done
        done
    done

    # Count milestone progress
    done_milestones=0
    total_milestones=${#ms_list[@]}
    for ms_name in "${ms_list[@]}"; do
        if [[ "${ms_status[$ms_name]}" == "done" ]]; then
            done_milestones=$((done_milestones + 1))
        fi
    done

    # Read existing README content
    existing_content=$(cat "$epic_readme")

    # Find line numbers of frontmatter delimiters
    first_delim=$(echo "$existing_content" | grep -n "^---$" | head -1 | cut -d: -f1)
    second_delim=$(echo "$existing_content" | grep -n "^---$" | head -2 | tail -1 | cut -d: -f1)

    # Extract frontmatter
    frontmatter=$(echo "$existing_content" | head -n "$second_delim")

    # Check for existing generated markers
    if echo "$existing_content" | grep -q "<!-- BEGIN GENERATED -->"; then
        # Has markers - extract content between frontmatter end and BEGIN marker
        begin_line=$(echo "$existing_content" | grep -n "<!-- BEGIN GENERATED -->" | head -1 | cut -d: -f1)
        preserved_content=$(echo "$existing_content" | sed -n "$((second_delim + 1)),$((begin_line - 1))p")
    else
        # No markers - preserve everything except ## Milestones section
        after_fm=$(echo "$existing_content" | tail -n +$((second_delim + 1)))

        # Check if there's a ## Milestones section to remove
        if echo "$after_fm" | grep -q "^## Milestones"; then
            # Get line number of ## Milestones in after_fm
            ms_line=$(echo "$after_fm" | grep -n "^## Milestones" | head -1 | cut -d: -f1)
            # Preserve content before Milestones
            preserved_content=$(echo "$after_fm" | head -n $((ms_line - 1)))
        else
            # No Milestones section, preserve everything
            preserved_content="$after_fm"
        fi
    fi

    # Clean up preserved content (remove leading and trailing empty lines)
    preserved_content=$(echo "$preserved_content" | sed '/./,$!d' | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}' 2>/dev/null || echo "$preserved_content")

    # Build new README
    {
        echo "$frontmatter"
        echo ""

        # Preserved content (Vision, Goals, etc.)
        if [[ -n "$preserved_content" ]]; then
            echo "$preserved_content"
            echo ""
        fi

        # Generated section
        echo "<!-- BEGIN GENERATED -->"
        echo "## Milestones"
        echo ""

        # Progress summary
        echo "**Progress:** ${done_milestones}/${total_milestones} milestones complete, ${done_stories}/${total_stories} stories done"
        if [[ -n "$active_milestone" ]]; then
            echo "**Active:** ${active_milestone}"
        fi
        if [[ $in_progress_stories -gt 0 ]]; then
            echo "**In Progress:** ${in_progress_stories} stories"
        fi
        echo ""

        # Milestone table
        echo "| ID | Milestone | Stories | Status |"
        echo "|----|-----------|---------|--------|"

        for ms_name in "${ms_list[@]}"; do
            ms_id=$(echo "$ms_name" | grep -oE '^[0-9]+' || echo "")
            title="${ms_title[$ms_name]:-$ms_name}"
            status="${ms_status[$ms_name]:-planned}"
            story_count="${ms_story_done[$ms_name]}/${ms_story_total[$ms_name]}"

            echo "| ${ms_id} | [${title}](milestones/${ms_name}/) | ${story_count} | ${status} |"
        done

        echo "<!-- END GENERATED -->"
    } > "$epic_readme"

    echo "Updated ${epic_readme}"

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

    mkdir -p "$dir/milestones"

    # If description not provided and we have a terminal, prompt for it
    if [[ -z "$description" ]] && [[ -t 0 ]]; then
        read -p "Description (one line): " description
    fi

    # Use templates if available, otherwise use inline defaults
    readme_template="templates/epic-README.md"
    prd_template="templates/PRD.md"

    if [[ -f "$readme_template" ]]; then
        # Read template and substitute variables
        sed -e "s/\${EPIC_ID}/$slug/g" \
            -e "s/\${EPIC_TITLE}/$name/g" \
            -e "s/\${EPIC_SUMMARY}/$description/g" \
            -e "s/\${EPIC_NAME}/$slug/g" \
            "$readme_template" > "$dir/README.md"
    else
        # Fallback inline template
        {
            echo '---'
            echo "id: ${slug}"
            echo "title: ${name}"
            echo "status: backlog"
            echo '---'
            echo ""
            echo "# ${name}"
            echo ""
            echo "> ${description}"
            echo ""
            echo "## Documents"
            echo ""
            echo "| Document | Description |"
            echo "|----------|-------------|"
            echo "| [PRD.md](PRD.md) | Product requirements and success criteria |"
            echo ""
            echo "## Milestones"
            echo ""
            echo "| ID | Milestone | Status | Progress |"
            echo "|----|-----------|--------|----------|"
            echo ""
            echo "## Status"
            echo ""
            echo "**Active Milestone:** None"
            echo "**Stories In Progress:** 0"
            echo "**Completion:** 0/0 milestones done"
        } > "$dir/README.md"
    fi

    if [[ -f "$prd_template" ]]; then
        # Read template and substitute variables
        sed -e "s/\${EPIC_ID}/$slug/g" \
            -e "s/\${EPIC_TITLE}/$name/g" \
            -e "s/\${VALUE_PROPOSITION}/$description/g" \
            -e "s/\${EPIC_NAME}/$slug/g" \
            "$prd_template" > "$dir/PRD.md"
    else
        # Fallback inline template
        {
            echo "# ${name} — Product Requirements"
            echo ""
            echo "> ${description}"
            echo ""
            echo "## Problem Statement"
            echo ""
            echo "<!-- What user problem does this solve? -->"
            echo ""
            echo "## Users"
            echo ""
            echo "<!-- Who benefits from this? Primary and secondary users -->"
            echo ""
            echo "## Requirements"
            echo ""
            echo "### Functional Requirements"
            echo ""
            echo "| ID | Requirement | Priority |"
            echo "|----|-------------|----------|"
            echo ""
            echo "### Non-Functional Requirements"
            echo ""
            echo "| ID | Requirement | Priority |"
            echo "|----|-------------|----------|"
            echo ""
            echo "## Success Criteria"
            echo ""
            echo "- [ ] TBD"
            echo ""
            echo "## Milestones"
            echo ""
            echo "| ID | Milestone | Status |"
            echo "|----|-----------|--------|"
        } > "$dir/PRD.md"
    fi

    echo "Created: $dir/README.md"
    echo "Created: $dir/PRD.md"

# Internal: create a new milestone
# Usage: just board _new-milestone "name" [epic]
_new-milestone NAME EPIC="":
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    name="{{NAME}}"
    epic="{{EPIC}}"
    templates_dir="templates"
    id=$(just board _next-milestone)
    slug=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd 'a-z0-9-')
    milestone_id="${id}-${slug}"
    date=$(date +%Y-%m-%d)

    # Determine milestone location
    if [[ -n "$epic" ]]; then
        # Find epic directory (allow partial match)
        epic_dir=""
        for d in epics/*/; do
            [[ -d "$d" ]] || continue
            if [[ "$(basename "$d")" == *"$epic"* ]]; then
                epic_dir="$d"
                break
            fi
        done

        if [[ -z "$epic_dir" ]]; then
            echo "Error: Epic '$epic' not found"
            echo "Available epics:"
            for e in epics/*/; do
                [[ -d "$e" ]] && echo "  - $(basename "$e")"
            done
            exit 1
        fi

        epic_name=$(basename "$epic_dir")
        dir="${epic_dir}milestones/${milestone_id}"
        mkdir -p "$dir"
    else
        # No epic specified - create in top-level milestones (backwards compat)
        dir="milestones/${milestone_id}"
        epic_name=""
        mkdir -p "$dir"
    fi

    if [[ -d "$dir" ]] && [[ -f "$dir/README.md" ]]; then
        echo "Error: Milestone already exists"
        exit 1
    fi

    # Helper: substitute template variables in content
    substitute_vars() {
        local content="$1"
        # Use sed for substitution to handle multi-line content
        echo "$content" | sed \
            -e "s|\\\${MILESTONE_ID}|${milestone_id}|g" \
            -e "s|\\\${MILESTONE_TITLE}|${name}|g" \
            -e "s|\\\${MILESTONE_SUMMARY}|[One-line summary of the milestone]|g" \
            -e "s|\\\${MILESTONE_GOAL}|[Goal of the milestone]|g" \
            -e "s|\\\${EPIC_ID}|${epic_name:-none}|g" \
            -e "s|\\\${EPIC_NAME}|${epic_name:-none}|g" \
            -e "s|\\\${EPIC_TITLE}|${epic_name:-none}|g" \
            -e "s|\\\${DATE}|${date}|g" \
            -e "s|\\\${DESIGN_SUMMARY}|[Design approach summary]|g" \
            -e "s|\\\${STORY_ID}|STORY0001|g" \
            -e "s|\\\${STORY_LINK}|../../stages/backlog/stories/STORY0001.md|g" \
            -e "s|\\\${STORY_DESCRIPTION}|[Story description]|g" \
            -e "s|\\\${REQUIREMENT_1}|[Requirement]|g" \
            -e "s|\\\${VERIFICATION_1}|[Verification method]|g" \
            -e "s|\\\${REQUIREMENT_2}|[Requirement]|g" \
            -e "s|\\\${VERIFICATION_2}|[Verification method]|g" \
            -e "s|\\\${NFR_REQUIREMENT_1}|[Non-functional requirement]|g" \
            -e "s|\\\${NFR_VERIFICATION_1}|[Verification method]|g" \
            -e "s|\\\${DECISION_1}|[Decision]|g" \
            -e "s|\\\${CHOICE_1}|[Choice]|g" \
            -e "s|\\\${RATIONALE_1}|[Rationale]|g" \
            -e "s|\\\${COMPONENT_1}|Component|g"
    }

    # Create README.md from template
    if [[ -f "$templates_dir/milestone-README.md" ]]; then
        template_content=$(cat "$templates_dir/milestone-README.md")
        # Remove template comment block (lines between <!-- and -->)
        template_content=$(echo "$template_content" | sed '/^<!--$/,/^-->$/d')
        substitute_vars "$template_content" > "$dir/README.md"
    else
        # Fallback if template doesn't exist
        {
            echo '---'
            echo "id: ${milestone_id}"
            echo "title: ${name}"
            echo "status: backlog"
            echo "epic: ${epic_name:-none}"
            echo '---'
            echo ""
            echo "# ${name}"
            echo ""
            echo "> [One-line summary of the milestone]"
            echo ""
            echo "## Documents"
            echo ""
            echo "| Document | Description |"
            echo "|----------|-------------|"
            echo "| [SRS.md](SRS.md) | Requirements and verification criteria |"
            echo "| [DESIGN.md](DESIGN.md) | Architecture and implementation details |"
            echo ""
            echo "## Stories"
            echo ""
            echo '> Stories linked via `just board link <story> <milestone>`.'
            echo ""
            echo "## Progress"
            echo ""
            echo "**Requirements:** 0/0 verified"
            echo "**Stories:** 0/0 complete"
        } > "$dir/README.md"
    fi

    # Create SRS.md from template
    if [[ -f "$templates_dir/SRS.md" ]]; then
        template_content=$(cat "$templates_dir/SRS.md")
        # Remove template comment block
        template_content=$(echo "$template_content" | sed '/^<!--$/,/^-->$/d')
        substitute_vars "$template_content" > "$dir/SRS.md"
    else
        # Fallback if template doesn't exist
        {
            echo "# ${name} — Software Requirements Specification"
            echo ""
            echo "> [Goal of the milestone]"
            echo ""
            echo "**Epic:** ${epic_name:-none}"
            echo "**Status:** backlog"
            echo ""
            echo "## Scope"
            echo ""
            echo "## Requirements"
            echo ""
            echo "### Functional Requirements"
            echo ""
            echo "| ID | Requirement | Source | Verification |"
            echo "|----|-------------|--------|--------------|"
            echo "| SRS-01 | [Requirement] | FR-01 | [Verification method] |"
            echo ""
            echo "## Stories"
            echo ""
            echo "| Story | Requirements | Status |"
            echo "|-------|--------------|--------|"
        } > "$dir/SRS.md"
    fi

    # Create DESIGN.md from template
    if [[ -f "$templates_dir/DESIGN.md" ]]; then
        template_content=$(cat "$templates_dir/DESIGN.md")
        # Remove template comment block
        template_content=$(echo "$template_content" | sed '/^<!--$/,/^-->$/d')
        substitute_vars "$template_content" > "$dir/DESIGN.md"
    else
        # Fallback if template doesn't exist
        {
            echo "# ${name} — Design Document"
            echo ""
            echo "> [Design approach summary]"
            echo ""
            echo "**SRS:** [SRS.md](SRS.md)"
            echo ""
            echo "## Overview"
            echo ""
            echo "## Architecture"
            echo ""
            echo "## Key Decisions"
            echo ""
            echo "| Decision | Choice | Rationale |"
            echo "|----------|--------|-----------|"
        } > "$dir/DESIGN.md"
    fi

    echo ""
    echo "Created milestone: ${milestone_id}"
    echo ""
    echo "Files created:"
    echo "  $dir/README.md"
    echo "  $dir/SRS.md"
    echo "  $dir/DESIGN.md"

# Router for new command
# Usage:
#   just board new story "title" [type] [epics] [milestone]
#   just board new epic "name" [description]
#   just board new milestone "name" [epic]
new TYPE NAME *ARGS:
    #!/usr/bin/env bash
    type="{{TYPE}}"
    name="{{NAME}}"
    args="{{ARGS}}"

    case "$type" in
        story)
            just board _new-story "$name" $args
            ;;
        epic)
            just board _new-epic "$name" "$args"
            ;;
        milestone)
            just board _new-milestone "$name" "$args"
            ;;
        *)
            echo "Error: Unknown type '$type'"
            echo "Usage:"
            echo "  just board new story \"title\" [type] [epics] [milestone]"
            echo "  just board new epic \"name\" [description]"
            echo "  just board new milestone \"name\" [epic]"
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

    # Regenerate README
    just board generate
    echo "Board updated"

# Start a milestone (set status to in-progress)
start-milestone ID:
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

    # Update epic README milestone table
    if [[ -n "$ms_epic" ]]; then
        epic_readme="epics/$ms_epic/README.md"
        if [[ -f "$epic_readme" ]]; then
            # Extract milestone ID (e.g., "01" from "01-verification-artifact-pipeline")
            ms_id=$(echo "$ms_name" | grep -oE '^[0-9]+')
            if [[ -n "$ms_id" ]]; then
                # Update the milestone row status to in-progress
                sed -i -E "s/^(\| *${ms_id} *\|[^|]+\|)[^|]+\|$/\1 in-progress |/" "$epic_readme"
                echo "Updated $epic_readme milestone table"
            fi
        fi
    fi

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
    already_done=false
    if [[ "$current_status" == "done" ]]; then
        already_done=true
    else
        # Update status
        sed -i 's/^status: .*/status: done/' "$ms_readme"

        # Add completed date if not already present
        date=$(date +%Y-%m-%d)
        if ! grep -q "^completed:" "$ms_readme"; then
            # Insert after status line
            sed -i "/^status: done$/a completed: $date" "$ms_readme"
        fi
    fi

    echo "Completed milestone: $ms_name"

    # Update epic README milestone table (always, in case it's out of sync)
    if [[ -n "$ms_epic" ]]; then
        epic_readme="epics/$ms_epic/README.md"
        if [[ -f "$epic_readme" ]]; then
            # Extract milestone ID (e.g., "01" from "01-verification-artifact-pipeline")
            ms_id=$(echo "$ms_name" | grep -oE '^[0-9]+')
            if [[ -n "$ms_id" ]]; then
                # Update the milestone row: | 01 | ... | in-progress | -> | 01 | ... | done |
                sed -i -E "s/^(\| *${ms_id} *\|[^|]+\|)[^|]+\|$/\1 done |/" "$epic_readme"
                echo "Updated $epic_readme milestone table"
            fi
        fi
    fi

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

# Link a story to a milestone
link STORY MILESTONE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    story_pattern="{{STORY}}"
    milestone="{{MILESTONE}}"

    # Find the story
    story_path=$(just board _find-story "$story_pattern")
    if [[ -z "$story_path" ]]; then
        echo "Error: Story '$story_pattern' not found"
        exit 1
    fi

    # Find milestone directory (allow partial match, search in epics)
    ms_dir=""
    for epic_dir in epics/*/; do
        [[ -d "${epic_dir}milestones" ]] || continue
        for d in "${epic_dir}milestones"/*/; do
            [[ -d "$d" ]] || continue
            if [[ "$(basename "$d")" == *"$milestone"* ]]; then
                ms_dir="$d"
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

    story_name=$(basename "$story_path")
    ms_name=$(basename "$ms_dir")

    # Calculate relative path depth based on milestone location
    # If ms_dir is like epics/foo/milestones/01-bar/, we need ../../../../stages/...
    depth=$(echo "$ms_dir" | tr -cd '/' | wc -c)
    rel_prefix=""
    for ((i=0; i<depth; i++)); do
        rel_prefix="../$rel_prefix"
    done

    # Determine relative path based on story location
    if [[ "$story_path" == *"backlog"* ]]; then
        rel_path="${rel_prefix}stages/backlog/stories/$story_name"
    elif [[ "$story_path" == *"in-progress"* ]]; then
        rel_path="${rel_prefix}stages/in-progress/stories/$story_name"
    elif [[ "$story_path" == *"icebox"* ]]; then
        rel_path="${rel_prefix}stages/icebox/stories/$story_name"
    else
        rel_path="${rel_prefix}stages/done/stories/$story_name"
    fi

    # Create symlink in milestone directory
    link_path="${ms_dir}${story_name}"
    if [[ -L "$link_path" ]]; then
        echo "Story already linked to milestone"
    else
        ln -sf "$rel_path" "$link_path"
        echo "Linked $story_name to milestone $ms_name"
    fi

    # Update frontmatter milestone field
    current_milestone=$(grep -m1 "^milestone:" "$story_path" 2>/dev/null | sed 's/^milestone: *//' || echo "")
    if [[ -z "$current_milestone" ]]; then
        # Add milestone field after epics field
        sed -i "/^epics:/a milestone: $ms_name" "$story_path"
    else
        sed -i "s/^milestone: .*/milestone: $ms_name/" "$story_path"
    fi
    echo "Updated story frontmatter"

# Unlink a story from a milestone
unlink STORY MILESTONE:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "{{board_dir}}"

    story_pattern="{{STORY}}"
    milestone="{{MILESTONE}}"

    # Find the story
    story_path=$(just board _find-story "$story_pattern")
    if [[ -z "$story_path" ]]; then
        echo "Error: Story '$story_pattern' not found"
        exit 1
    fi

    # Find milestone directory
    ms_dir=""
    for epic_dir in epics/*/; do
        [[ -d "${epic_dir}milestones" ]] || continue
        for d in "${epic_dir}milestones"/*/; do
            [[ -d "$d" ]] || continue
            if [[ "$(basename "$d")" == *"$milestone"* ]]; then
                ms_dir="$d"
                break 2
            fi
        done
    done

    if [[ -z "$ms_dir" ]]; then
        echo "Error: Milestone '$milestone' not found"
        exit 1
    fi

    story_name=$(basename "$story_path")
    link_path="${ms_dir}${story_name}"

    # Remove symlink
    if [[ -L "$link_path" ]]; then
        rm "$link_path"
        echo "Unlinked $story_name from milestone $(basename "$ms_dir")"
    else
        echo "Story was not linked to milestone $milestone"
    fi

    # Clear frontmatter milestone field
    sed -i '/^milestone: .*/d' "$story_path"
    echo "Updated story frontmatter"

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

        # Count milestones and their status
        total_ms=0
        done_ms=0
        in_progress_ms=0
        if [[ -d "${epic_dir}milestones" ]]; then
            for ms_dir in "${epic_dir}milestones"/*/; do
                [[ -d "$ms_dir" ]] || continue
                [[ -f "${ms_dir}README.md" ]] || continue
                total_ms=$((total_ms + 1))
                ms_status=$(grep -m1 "^status:" "${ms_dir}README.md" 2>/dev/null | sed 's/^status: *//' || echo "")
                if [[ "$ms_status" == "done" ]]; then
                    done_ms=$((done_ms + 1))
                elif [[ "$ms_status" == "in-progress" ]]; then
                    in_progress_ms=$((in_progress_ms + 1))
                fi
            done
        fi

        echo "$name ($status)"
        echo "  Milestones: $total_ms (in-progress: $in_progress_ms, done: $done_ms)"
    done

# Internal: list milestones
_list-milestones:
    #!/usr/bin/env bash
    cd "{{board_dir}}"

    echo "Milestones"
    echo "=========="
    echo ""

    found=false
    for epic_dir in epics/*/; do
        [[ -d "${epic_dir}milestones" ]] || continue
        epic_name=$(basename "$epic_dir")

        for ms_dir in "${epic_dir}milestones"/*/; do
            [[ -d "$ms_dir" ]] || continue
            found=true
            name=$(basename "$ms_dir")

            # Get status from README
            status=""
            [[ -f "${ms_dir}README.md" ]] && status=$(grep -m1 "^status:" "${ms_dir}README.md" 2>/dev/null | sed 's/^status: *//' || echo "")

            # Count story symlinks
            story_count=$(find "$ms_dir" -maxdepth 1 -type l -name "*.md" 2>/dev/null | wc -l | tr -d ' ')

            echo "$name ($status) - epic: $epic_name"
            echo "  Stories: $story_count"
        done
    done

    if [[ "$found" == "false" ]]; then
        echo "No milestones found"
    fi

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

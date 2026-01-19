# Verification commands for coherence testing
# Captures visual artifacts to verify implementation matches specifications

# Run commands from project root
set working-directory := '..'

# Verification output directory
verify_dir := "verification"

# Default: show available verify commands
default:
    @echo "Verification Commands"
    @echo ""
    @echo "Artifact capture:"
    @echo "  just verify snapshots    Tier 1: Capture key screen snapshots (~30s)"
    @echo "  just verify checkpoints  Tier 2: Capture interaction sequences (~2min)"
    @echo "  just verify videos       Tier 3: Record CLI + Web videos (~5min)"
    @echo "  just verify all          Run all tiers"
    @echo ""
    @echo "Story verification:"
    @echo "  just verify story <ID>        Capture artifacts for story, generate report"
    @echo "  just verify story-report <ID> Generate report only (artifacts exist)"
    @echo ""
    @echo "Reporting:"
    @echo "  just verify report       Generate verification/report.md (global)"
    @echo "  just verify summary      Show artifact counts"
    @echo ""
    @echo "Utility:"
    @echo "  just verify clean        Remove all artifacts"
    @echo "  just verify init         Create verification directory structure"

# Initialize verification directory structure
init:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p {{verify_dir}}/snapshots
    mkdir -p {{verify_dir}}/checkpoints
    mkdir -p {{verify_dir}}/videos/cli
    mkdir -p {{verify_dir}}/videos/web
    mkdir -p {{verify_dir}}/videos/stitched

    # Create .gitignore to only track report.md
    echo '# Ignore all artifacts except the report' > {{verify_dir}}/.gitignore
    echo '*' >> {{verify_dir}}/.gitignore
    echo '!.gitignore' >> {{verify_dir}}/.gitignore
    echo '!report.md' >> {{verify_dir}}/.gitignore
    echo '!snapshots.json' >> {{verify_dir}}/.gitignore
    echo '!checkpoints.json' >> {{verify_dir}}/.gitignore

    echo "✓ Verification directory initialized"

# Tier 1: Capture key screen snapshots
snapshots: init
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Capturing snapshots..."

    # Check if snapshots.json exists
    if [[ ! -f {{verify_dir}}/snapshots.json ]]; then
        echo "No snapshots.json found. Creating default..."
        echo '{' > {{verify_dir}}/snapshots.json
        echo '  "baseUrl": "http://localhost:3000",' >> {{verify_dir}}/snapshots.json
        echo '  "snapshots": [' >> {{verify_dir}}/snapshots.json
        echo '    {"name": "dashboard", "url": "/groove/status", "waitFor": "[data-testid=status-indicator]"},' >> {{verify_dir}}/snapshots.json
        echo '    {"name": "sessions", "url": "/sessions", "waitFor": ".session-row"},' >> {{verify_dir}}/snapshots.json
        echo '    {"name": "stream", "url": "/groove/stream", "waitFor": "[data-testid=stream-view]"}' >> {{verify_dir}}/snapshots.json
        echo '  ]' >> {{verify_dir}}/snapshots.json
        echo '}' >> {{verify_dir}}/snapshots.json
    fi

    # Run Playwright to capture snapshots
    npx playwright test e2e-tests/tests/snapshots.spec.ts --reporter=list 2>/dev/null || {
        echo "Note: Snapshot spec not found or server not running."
        echo "To capture snapshots, ensure vibes server is running."
    }

    # Count captured snapshots
    count=$(find {{verify_dir}}/snapshots -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    echo "✓ Captured $count snapshots"

# Tier 2: Capture interaction checkpoint sequences
checkpoints: init
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Capturing interaction checkpoints..."

    # Check if checkpoints.json exists
    if [[ ! -f {{verify_dir}}/checkpoints.json ]]; then
        echo "No checkpoints.json found. Creating default..."
        echo '{' > {{verify_dir}}/checkpoints.json
        echo '  "baseUrl": "http://localhost:3000",' >> {{verify_dir}}/checkpoints.json
        echo '  "checkpoints": [' >> {{verify_dir}}/checkpoints.json
        echo '    {"name": "navigate-groove", "steps": [' >> {{verify_dir}}/checkpoints.json
        echo '      {"action": "goto", "url": "/", "screenshot": "01-home"}' >> {{verify_dir}}/checkpoints.json
        echo '    ]}' >> {{verify_dir}}/checkpoints.json
        echo '  ]' >> {{verify_dir}}/checkpoints.json
        echo '}' >> {{verify_dir}}/checkpoints.json
    fi

    # Run Playwright checkpoint tests
    npx playwright test e2e-tests/tests/checkpoints.spec.ts --reporter=list 2>/dev/null || {
        echo "Note: Checkpoint spec not found or server not running."
    }

    # Count captured checkpoints
    count=$(find {{verify_dir}}/checkpoints -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    echo "✓ Captured $count checkpoint images"

# Tier 3: Record CLI + Web videos and stitch together
videos: init
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Recording videos..."

    # Record CLI via VHS (if tapes exist)
    if [[ -d {{verify_dir}}/tapes ]]; then
        echo "Recording CLI videos..."
        for tape in {{verify_dir}}/tapes/*.tape; do
            [[ -f "$tape" ]] || continue
            name=$(basename "$tape" .tape)
            echo "  Recording: $name"
            # VHS outputs webm directly to verification/videos/cli/
            vhs "$tape" 2>/dev/null && echo "    Created: ${name}.webm" || true
        done
    fi

    # Record Web via Playwright
    echo "Recording Web videos..."
    npx --prefix e2e-tests playwright test e2e-tests/tests/videos.spec.ts --reporter=list 2>/dev/null || {
        echo "Note: Video spec not found or tests failed."
    }

    # Copy Playwright video recordings with unique names based on parent directory
    for results_dir in test-results e2e-tests/test-results; do
        if [[ -d "$results_dir" ]]; then
            find "$results_dir" -name "*.webm" 2>/dev/null | while read -r video; do
                # Extract test name from parent directory
                parent_dir=$(basename "$(dirname "$video")")
                # Try to extract "walkthrough" style name, else use full parent dir
                if echo "$parent_dir" | grep -q "walkthrough"; then
                    test_name=$(echo "$parent_dir" | grep -oE '[a-z]+-walkthrough')
                else
                    test_name="$parent_dir"
                fi
                [[ -z "$test_name" ]] && test_name="$parent_dir"
                dest="{{verify_dir}}/videos/web/${test_name}.webm"
                cp "$video" "$dest" 2>/dev/null && echo "    Copied: ${test_name}.webm"
            done
        fi
    done

    # Stitch CLI + Web videos together if both exist
    echo "Stitching videos..."
    cli_count=$(find {{verify_dir}}/videos/cli -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')
    web_count=$(find {{verify_dir}}/videos/web -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')

    if [[ "$cli_count" -gt 0 ]] && [[ "$web_count" -gt 0 ]]; then
        cli_video=$(find {{verify_dir}}/videos/cli -name "*.webm" | head -1)
        web_video=$(find {{verify_dir}}/videos/web -name "*.webm" | head -1)

        if [[ -n "$cli_video" ]] && [[ -n "$web_video" ]]; then
            # Both videos are 1280x720, stitch side by side at full resolution
            ffmpeg -i "$cli_video" -i "$web_video" \
                -filter_complex "[0:v][1:v]hstack=inputs=2" \
                -c:v libvpx-vp9 {{verify_dir}}/videos/stitched/combined.webm -y 2>/dev/null && \
                echo "✓ Created stitched video: verification/videos/stitched/combined.webm (2560x720)"
        fi
    fi

    echo "✓ Video recording complete"
    echo "  CLI videos: $cli_count"
    echo "  Web videos: $web_count"

# Run all verification tiers
all: snapshots checkpoints videos report
    @echo ""
    @echo "✓ All verification tiers complete"

# Generate verification report
report: init
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Generating verification report..."

    branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    report_date=$(date -Iseconds)

    snapshot_count=$(find {{verify_dir}}/snapshots -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    checkpoint_count=$(find {{verify_dir}}/checkpoints -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    cli_video_count=$(find {{verify_dir}}/videos/cli -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')
    web_video_count=$(find {{verify_dir}}/videos/web -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')
    stitched_count=$(find {{verify_dir}}/videos/stitched -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')

    # Find related stories
    stories=""
    if [[ "$branch" =~ ^(feat|fix|chore|refactor)/([0-9]+) ]]; then
        story_id="${BASH_REMATCH[2]}"
        stories=$(find docs/board/stages -name "*${story_id}*" -type f 2>/dev/null | head -3 | xargs -I{} basename {} .md | tr '\n' ', ' | sed 's/,$//')
    fi
    [[ -z "$stories" ]] && stories="(none detected)"

    # Status icons
    snap_status="⚠️"; [[ "$snapshot_count" -gt 0 ]] && snap_status="✅"
    check_status="⚠️"; [[ "$checkpoint_count" -gt 0 ]] && check_status="✅"
    cli_status="⚠️"; [[ "$cli_video_count" -gt 0 ]] && cli_status="✅"
    web_status="⚠️"; [[ "$web_video_count" -gt 0 ]] && web_status="✅"
    stitch_status="⚠️"; [[ "$stitched_count" -gt 0 ]] && stitch_status="✅"

    # Write report header
    echo "# Verification Report" > {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md
    echo "Generated: ${report_date}" >> {{verify_dir}}/report.md
    echo "Branch: ${branch}" >> {{verify_dir}}/report.md
    echo "Stories: ${stories}" >> {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md
    echo "## Summary" >> {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md
    echo "| Tier | Artifact Type | Count | Status |" >> {{verify_dir}}/report.md
    echo "|------|---------------|-------|--------|" >> {{verify_dir}}/report.md
    echo "| 1 | Snapshots | ${snapshot_count} | ${snap_status} |" >> {{verify_dir}}/report.md
    echo "| 2 | Checkpoints | ${checkpoint_count} | ${check_status} |" >> {{verify_dir}}/report.md
    echo "| 3 | CLI Videos | ${cli_video_count} | ${cli_status} |" >> {{verify_dir}}/report.md
    echo "| 3 | Web Videos | ${web_video_count} | ${web_status} |" >> {{verify_dir}}/report.md
    echo "| 3 | Stitched Videos | ${stitched_count} | ${stitch_status} |" >> {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md

    # Artifacts section
    echo "## Artifacts" >> {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md
    echo "### Snapshots (Tier 1)" >> {{verify_dir}}/report.md
    if [[ "$snapshot_count" -gt 0 ]]; then
        find {{verify_dir}}/snapshots -name "*.png" | while read -r f; do
            echo "- $(basename "$f")" >> {{verify_dir}}/report.md
        done
    else
        echo "*No snapshots captured*" >> {{verify_dir}}/report.md
    fi
    echo "" >> {{verify_dir}}/report.md

    echo "### Checkpoints (Tier 2)" >> {{verify_dir}}/report.md
    if [[ "$checkpoint_count" -gt 0 ]]; then
        find {{verify_dir}}/checkpoints -name "*.png" | sort | while read -r f; do
            echo "- $(basename "$f")" >> {{verify_dir}}/report.md
        done
    else
        echo "*No checkpoints captured*" >> {{verify_dir}}/report.md
    fi
    echo "" >> {{verify_dir}}/report.md

    echo "### Videos (Tier 3)" >> {{verify_dir}}/report.md
    total_videos=$((cli_video_count + web_video_count + stitched_count))
    if [[ "$total_videos" -gt 0 ]]; then
        if [[ "$cli_video_count" -gt 0 ]]; then
            echo "**CLI:**" >> {{verify_dir}}/report.md
            find {{verify_dir}}/videos/cli -name "*.webm" 2>/dev/null | while read -r f; do
                echo "- $(basename "$f")" >> {{verify_dir}}/report.md
            done
        fi
        if [[ "$web_video_count" -gt 0 ]]; then
            echo "" >> {{verify_dir}}/report.md
            echo "**Web:**" >> {{verify_dir}}/report.md
            find {{verify_dir}}/videos/web -name "*.webm" 2>/dev/null | while read -r f; do
                echo "- $(basename "$f")" >> {{verify_dir}}/report.md
            done
        fi
        if [[ "$stitched_count" -gt 0 ]]; then
            echo "" >> {{verify_dir}}/report.md
            echo "**Stitched:**" >> {{verify_dir}}/report.md
            find {{verify_dir}}/videos/stitched -name "*.webm" 2>/dev/null | while read -r f; do
                echo "- $(basename "$f")" >> {{verify_dir}}/report.md
            done
        fi
    else
        echo "*No videos recorded*" >> {{verify_dir}}/report.md
    fi

    echo "" >> {{verify_dir}}/report.md
    echo "## Notes" >> {{verify_dir}}/report.md
    echo "" >> {{verify_dir}}/report.md
    echo "- Artifacts are stored locally (gitignored)" >> {{verify_dir}}/report.md
    echo "- Only this report is committed" >> {{verify_dir}}/report.md
    echo "- Review artifacts locally before creating PR" >> {{verify_dir}}/report.md

    echo "✓ Generated {{verify_dir}}/report.md"

# Show artifact summary
summary:
    #!/usr/bin/env bash
    echo "Verification Artifacts"
    echo "======================"
    echo ""

    snapshot_count=$(find {{verify_dir}}/snapshots -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    checkpoint_count=$(find {{verify_dir}}/checkpoints -name "*.png" 2>/dev/null | wc -l | tr -d ' ')
    cli_video_count=$(find {{verify_dir}}/videos/cli -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')
    web_video_count=$(find {{verify_dir}}/videos/web -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')
    stitched_count=$(find {{verify_dir}}/videos/stitched -name "*.webm" 2>/dev/null | wc -l | tr -d ' ')

    echo "Tier 1 - Snapshots:     $snapshot_count"
    echo "Tier 2 - Checkpoints:   $checkpoint_count"
    echo "Tier 3 - CLI Videos:    $cli_video_count"
    echo "Tier 3 - Web Videos:    $web_video_count"
    echo "Tier 3 - Stitched:      $stitched_count"
    echo ""

    if [[ -f {{verify_dir}}/report.md ]]; then
        echo "Report: {{verify_dir}}/report.md"
    else
        echo "Report: not generated (run 'just verify report')"
    fi

# Remove all verification artifacts
clean:
    #!/usr/bin/env bash
    echo "Cleaning verification artifacts..."
    rm -rf {{verify_dir}}/snapshots/*
    rm -rf {{verify_dir}}/checkpoints/*
    rm -rf {{verify_dir}}/videos/cli/*
    rm -rf {{verify_dir}}/videos/web/*
    rm -rf {{verify_dir}}/videos/stitched/*
    echo "✓ Artifacts cleaned (report.md preserved if present)"

# ============================================================================
# Story-Scoped Verification
# ============================================================================

# Generate verification report for a specific story (artifacts must exist)
story-report STORY_ID:
    #!/usr/bin/env bash
    set -euo pipefail

    story_id="{{STORY_ID}}"
    echo "Generating verification report for story: $story_id"

    # Find the story file by ID
    # Story IDs are like FEAT0109, which appear in filenames as [FEAT][0109]
    # Extract type and number from ID (e.g., FEAT0109 -> FEAT, 0109)
    if [[ "$story_id" =~ ^([A-Z]+)([0-9]+)$ ]]; then
        id_type="${BASH_REMATCH[1]}"
        id_num="${BASH_REMATCH[2]}"
        # Search for [TYPE][NUM] pattern in filename
        story_file=$(find docs/board/stages -type f -name "*.md" | grep -E "\[${id_type}\]\[${id_num}\]" | head -1)
    else
        # Fallback: search for ID literally
        story_file=$(find docs/board/stages -type f -name "*.md" | grep -i "${story_id}" | head -1)
    fi

    if [[ -z "$story_file" ]]; then
        echo "Error: Story file not found for ID: $story_id"
        echo "Hint: Use format like FEAT0109 or BUG0001"
        exit 1
    fi
    echo "Found story: $story_file"

    # Extract frontmatter fields
    title=$(grep "^title:" "$story_file" | sed 's/title: //' | head -1)
    scope=$(grep "^scope:" "$story_file" | sed 's/scope: //' | head -1)
    id=$(grep "^id:" "$story_file" | sed 's/id: //' | head -1)

    # Determine report directory based on scope
    if [[ -n "$scope" ]]; then
        report_dir="{{verify_dir}}/reports/${scope}"
    else
        report_dir="{{verify_dir}}/reports"
    fi
    mkdir -p "$report_dir"
    report_file="${report_dir}/${id}.md"

    # Extract acceptance criteria with verify annotations
    # Format: - [ ] Criterion text <!-- verify: type:name -->
    echo "Extracting acceptance criteria..."

    # Parse criteria and annotations
    criteria=()
    annotations=()
    in_criteria=false

    while IFS= read -r line; do
        # Check if we're in the Acceptance Criteria section
        if [[ "$line" =~ ^##.*[Aa]cceptance.*[Cc]riteria ]]; then
            in_criteria=true
            continue
        fi
        # Exit criteria section on next header
        if [[ "$in_criteria" == true ]] && [[ "$line" =~ ^## ]]; then
            break
        fi
        # Parse criterion lines
        if [[ "$in_criteria" == true ]] && [[ "$line" =~ ^-\ \[.\]\ (.*)$ ]]; then
            criterion="${BASH_REMATCH[1]}"
            # Extract verify annotation if present
            if [[ "$criterion" =~ \<!--\ verify:\ ([^-]+)\ --\> ]]; then
                annotation="${BASH_REMATCH[1]}"
                # Remove annotation from criterion text
                criterion=$(echo "$criterion" | sed 's/ *<!-- verify:[^>]*-->//')
            else
                annotation=""
            fi
            criteria+=("$criterion")
            annotations+=("$annotation")
        fi
    done < "$story_file"

    # Generate report
    branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    report_date=$(date -Iseconds)

    echo "# Verification: ${id}" > "$report_file"
    echo "" >> "$report_file"
    echo "**Story:** ${title}" >> "$report_file"
    echo "**Scope:** ${scope:-_(none)_}" >> "$report_file"
    echo "**Generated:** ${report_date}" >> "$report_file"
    echo "**Branch:** ${branch}" >> "$report_file"
    echo "" >> "$report_file"

    if [[ ${#criteria[@]} -eq 0 ]]; then
        echo "## Coverage" >> "$report_file"
        echo "" >> "$report_file"
        echo "_No acceptance criteria found in story file._" >> "$report_file"
        echo "" >> "$report_file"
        echo "✓ Generated $report_file (no criteria)"
        exit 0
    fi

    # Check artifact existence and build coverage table
    echo "## Coverage" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Criterion | Artifact | Status |" >> "$report_file"
    echo "|-----------|----------|--------|" >> "$report_file"

    covered=0
    uncovered=()

    for i in "${!criteria[@]}"; do
        criterion="${criteria[$i]}"
        annotation="${annotations[$i]}"

        if [[ -z "$annotation" ]]; then
            echo "| ${criterion} | _(no annotation)_ | ⚠️ |" >> "$report_file"
            uncovered+=("$criterion (no verify annotation)")
        else
            # Parse annotation: type:name
            ann_type=$(echo "$annotation" | cut -d: -f1 | tr -d ' ')
            ann_name=$(echo "$annotation" | cut -d: -f2 | tr -d ' ')

            # Check if artifact exists based on type
            artifact_exists=false
            artifact_path=""

            case "$ann_type" in
                snapshot)
                    artifact_path="{{verify_dir}}/snapshots/${ann_name}.png"
                    [[ -f "$artifact_path" ]] && artifact_exists=true
                    ;;
                checkpoint)
                    # Checkpoints are directories with multiple PNGs
                    if find "{{verify_dir}}/checkpoints" -name "${ann_name}*.png" 2>/dev/null | grep -q .; then
                        artifact_exists=true
                        artifact_path="{{verify_dir}}/checkpoints/${ann_name}*"
                    fi
                    ;;
                video)
                    # Videos can be in cli/, web/, or direct path
                    for vdir in "{{verify_dir}}/videos/cli" "{{verify_dir}}/videos/web" "{{verify_dir}}/videos"; do
                        if [[ -f "${vdir}/${ann_name}.webm" ]] || [[ -f "${vdir}/${ann_name}.mp4" ]]; then
                            artifact_exists=true
                            artifact_path="${vdir}/${ann_name}.*"
                            break
                        fi
                    done
                    ;;
                *)
                    echo "| ${criterion} | ${annotation} | ❌ (unknown type) |" >> "$report_file"
                    uncovered+=("$criterion (unknown type: $ann_type)")
                    continue
                    ;;
            esac

            if [[ "$artifact_exists" == true ]]; then
                echo "| ${criterion} | ${ann_type}:${ann_name} | ✅ |" >> "$report_file"
                ((covered++))
            else
                echo "| ${criterion} | ${ann_type}:${ann_name} | ❌ (missing) |" >> "$report_file"
                uncovered+=("$criterion (artifact not found: ${ann_type}:${ann_name})")
            fi
        fi
    done

    total=${#criteria[@]}
    pct=$((covered * 100 / total))

    echo "" >> "$report_file"
    echo "**Coverage: ${covered}/${total} (${pct}%)**" >> "$report_file"

    # Uncovered section
    if [[ ${#uncovered[@]} -gt 0 ]]; then
        echo "" >> "$report_file"
        echo "## Uncovered Criteria" >> "$report_file"
        echo "" >> "$report_file"
        for item in "${uncovered[@]}"; do
            echo "- ${item}" >> "$report_file"
        done
    fi

    echo ""
    echo "✓ Generated $report_file"
    echo "  Coverage: ${covered}/${total} (${pct}%)"

# Capture artifacts for a story and generate report
story STORY_ID: (story-report STORY_ID)
    @echo ""
    @echo "Note: 'just verify story' currently generates the report only."
    @echo "To capture artifacts, run 'just verify all' first, then 'just verify story-report <ID>'."
    @echo ""
    @echo "Future: Will selectively capture only artifacts referenced in story."

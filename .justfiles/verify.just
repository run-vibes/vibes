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
    @echo "Reporting:"
    @echo "  just verify report       Generate verification/report.md"
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

            # Run VHS - outputs GIF to verification/output/
            vhs "$tape" 2>/dev/null || true

            # Convert GIF to MP4
            gif_path="{{verify_dir}}/output/${name}.gif"
            if [[ -f "$gif_path" ]]; then
                ffmpeg -i "$gif_path" -movflags faststart -pix_fmt yuv420p -vf "fps=30" \
                    "{{verify_dir}}/videos/cli/${name}.mp4" -y 2>/dev/null && \
                    echo "    Converted: ${name}.mp4"
            fi
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
    cli_count=$(find {{verify_dir}}/videos/cli -name "*.mp4" 2>/dev/null | wc -l | tr -d ' ')
    web_count=$(find {{verify_dir}}/videos/web \( -name "*.webm" -o -name "*.mp4" \) 2>/dev/null | wc -l | tr -d ' ')

    if [[ "$cli_count" -gt 0 ]] && [[ "$web_count" -gt 0 ]]; then
        cli_video=$(find {{verify_dir}}/videos/cli -name "*.mp4" | head -1)
        web_video=$(find {{verify_dir}}/videos/web \( -name "*.webm" -o -name "*.mp4" \) | head -1)

        if [[ -n "$cli_video" ]] && [[ -n "$web_video" ]]; then
            ffmpeg -i "$cli_video" -i "$web_video" \
                -filter_complex "[0:v]scale=640:-1[left];[1:v]scale=640:-1[right];[left][right]hstack=inputs=2" \
                {{verify_dir}}/videos/stitched/combined.mp4 -y 2>/dev/null && \
                echo "✓ Created stitched video: verification/videos/stitched/combined.mp4"
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
    cli_video_count=$(find {{verify_dir}}/videos/cli -name "*.mp4" 2>/dev/null | wc -l | tr -d ' ')
    web_video_count=$(find {{verify_dir}}/videos/web \( -name "*.webm" -o -name "*.mp4" \) 2>/dev/null | wc -l | tr -d ' ')
    stitched_count=$(find {{verify_dir}}/videos/stitched -name "*.mp4" 2>/dev/null | wc -l | tr -d ' ')

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
            find {{verify_dir}}/videos/cli -name "*.mp4" 2>/dev/null | while read -r f; do
                echo "- $(basename "$f")" >> {{verify_dir}}/report.md
            done
        fi
        if [[ "$web_video_count" -gt 0 ]]; then
            echo "" >> {{verify_dir}}/report.md
            echo "**Web:**" >> {{verify_dir}}/report.md
            find {{verify_dir}}/videos/web \( -name "*.webm" -o -name "*.mp4" \) 2>/dev/null | while read -r f; do
                echo "- $(basename "$f")" >> {{verify_dir}}/report.md
            done
        fi
        if [[ "$stitched_count" -gt 0 ]]; then
            echo "" >> {{verify_dir}}/report.md
            echo "**Stitched:**" >> {{verify_dir}}/report.md
            find {{verify_dir}}/videos/stitched -name "*.mp4" 2>/dev/null | while read -r f; do
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
    cli_video_count=$(find {{verify_dir}}/videos/cli -name "*.mp4" 2>/dev/null | wc -l | tr -d ' ')
    web_video_count=$(find {{verify_dir}}/videos/web \( -name "*.webm" -o -name "*.mp4" \) 2>/dev/null | wc -l | tr -d ' ')
    stitched_count=$(find {{verify_dir}}/videos/stitched -name "*.mp4" 2>/dev/null | wc -l | tr -d ' ')

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

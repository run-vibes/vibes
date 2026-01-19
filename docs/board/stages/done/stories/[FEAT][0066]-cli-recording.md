---
id: FEAT0066
title: CLI recording with VHS
type: feat
status: done
priority: medium
scope: verification
depends: []
estimate: 3h
created: 2026-01-09
---

# CLI recording with VHS

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Use VHS (charmbracelet/vhs) to generate GIF recordings of CLI commands for documentation and verify CLI output consistency.

## Context

VHS is a tool that generates terminal GIFs from a simple tape file DSL. This provides:
1. Visual documentation showing how CLI commands work
2. Regression testing by comparing expected output

## Tasks

### Task 1: Add VHS to Nix flake

**Files:**
- Modify: `flake.nix`

**Steps:**
1. Add VHS to devShell packages:
   ```nix
   devShells.default = pkgs.mkShell {
     packages = with pkgs; [
       # ... existing packages
       vhs
     ];
   };
   ```
2. Run: `direnv reload`
3. Verify: `vhs --version`
4. Commit: `chore: add VHS to nix flake`

### Task 2: Create recording structure

**Files:**
- Create: `cli/recordings/tapes/.gitkeep`
- Create: `cli/recordings/output/.gitkeep`
- Create: `cli/recordings/expected/.gitkeep`
- Modify: `.gitignore`

**Steps:**
1. Create directory structure:
   ```
   cli/recordings/
   ├── tapes/       # VHS tape files
   ├── output/      # Generated GIFs (gitignored)
   └── expected/    # Expected text output
   ```
2. Add `cli/recordings/output/` to `.gitignore`
3. Commit: `chore: add CLI recording directory structure`

### Task 3: Create help tape

**Files:**
- Create: `cli/recordings/tapes/help.tape`
- Create: `cli/recordings/expected/help.txt`

**Steps:**
1. Create VHS tape file:
   ```
   Output help.gif

   Set FontSize 14
   Set Width 800
   Set Height 400

   Type "vibes --help"
   Enter
   Sleep 2s
   ```
2. Run: `vhs cli/recordings/tapes/help.tape`
3. Capture expected output to `expected/help.txt`
4. Commit: `feat(cli): add help command recording`

### Task 4: Create additional tapes

**Files:**
- Create: `cli/recordings/tapes/sessions.tape`
- Create: `cli/recordings/tapes/version.tape`

**Steps:**
1. Create tapes for:
   - `vibes sessions` (list sessions)
   - `vibes --version` (version info)
2. Generate GIFs and capture expected output
3. Commit: `feat(cli): add sessions and version recordings`

### Task 5: Add just commands

**Files:**
- Modify: `justfile`

**Steps:**
1. Add commands:
   ```just
   # Generate all CLI recordings
   cli-record:
       for tape in cli/recordings/tapes/*.tape; do \
           vhs "$tape" -o "cli/recordings/output/$(basename $tape .tape).gif"; \
       done

   # Verify CLI output matches expected
   cli-verify:
       @echo "Verifying CLI output..."
       vibes --help > /tmp/help-actual.txt
       diff cli/recordings/expected/help.txt /tmp/help-actual.txt
   ```
2. Run: `just cli-record`
3. Run: `just cli-verify`
4. Commit: `chore: add CLI recording just commands`

## Acceptance Criteria

- [ ] VHS available in Nix shell
- [ ] Directory structure created
- [ ] Help tape generates valid GIF
- [ ] Sessions tape generates valid GIF
- [ ] `just cli-record` generates all GIFs
- [ ] `just cli-verify` passes when output matches expected
- [ ] Expected output files committed

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0066`
3. Commit, push, and create PR

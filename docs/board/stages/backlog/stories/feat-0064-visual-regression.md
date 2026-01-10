---
id: FEAT0064
title: Visual regression testing
type: feat
status: pending
priority: medium
epics: [verification]
depends: []
estimate: 4h
created: 2026-01-09
---

# Visual regression testing

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add Playwright-based screenshot testing for design-system components and web-ui pages to catch unintended UI changes.

## Context

Visual regression testing compares screenshots against baselines to detect UI drift. This catches styling changes, layout shifts, and rendering issues that unit tests miss.

## Tasks

### Task 1: Setup Playwright for design-system

**Files:**
- Create: `design-system/playwright.config.ts`
- Create: `design-system/e2e/visual.spec.ts`
- Modify: `design-system/package.json`

**Steps:**
1. Add Playwright dev dependency to design-system
2. Create Playwright config with snapshot settings:
   - Viewport: 1280x720
   - Threshold: 0.1% pixel difference
   - Snapshot path: `e2e/snapshots/`
3. Create visual test file for key components:
   - Button (all variants)
   - Badge (all variants)
   - Panel
   - Header
   - SessionCard
   - TerminalPanel
4. Run: `npm test --workspace=@vibes/design-system`
5. Commit: `feat(design-system): add visual regression testing setup`

### Task 2: Create component snapshots

**Files:**
- Create: `design-system/e2e/snapshots/*.png` (generated)
- Modify: `design-system/e2e/visual.spec.ts`

**Steps:**
1. Write snapshot tests for each component:
   ```typescript
   test('Button primary renders correctly', async ({ page }) => {
     await page.goto('/iframe.html?id=button--primary');
     await expect(page).toHaveScreenshot('button-primary.png');
   });
   ```
2. Generate initial baselines with `--update-snapshots`
3. Verify snapshots look correct
4. Commit baselines: `feat(design-system): add component snapshot baselines`

### Task 3: Setup page-level screenshots for web-ui

**Files:**
- Modify: `web-ui/playwright.config.ts`
- Create: `web-ui/e2e/visual.spec.ts`
- Create: `web-ui/e2e/snapshots/*.png` (generated)

**Steps:**
1. Update Playwright config with snapshot settings
2. Create visual test file for key pages:
   - Dashboard Overview
   - Dashboard Strategy
   - Dashboard Health
   - Settings
   - Assessment Status
3. Mock API responses for consistent data
4. Generate initial baselines
5. Run: `just web e2e`
6. Commit: `feat(web-ui): add page-level visual regression tests`

### Task 4: Add just commands

**Files:**
- Modify: `justfile`

**Steps:**
1. Add `just web visual` - Run visual regression tests
2. Add `just web visual-update` - Update snapshot baselines
3. Document commands in CLAUDE.md
4. Run: `just web visual`
5. Commit: `chore: add visual regression just commands`

## Acceptance Criteria

- [ ] Design-system has Playwright visual tests for 6+ components
- [ ] Web-ui has visual tests for 5+ pages
- [ ] Baselines committed to git
- [ ] CI fails when screenshots differ beyond threshold
- [ ] `just web visual` runs all visual tests
- [ ] `just web visual-update` regenerates baselines

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0064`
3. Commit, push, and create PR

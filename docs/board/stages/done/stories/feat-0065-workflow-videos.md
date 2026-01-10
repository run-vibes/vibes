---
id: FEAT0065
title: Workflow video recording
type: feat
status: done
priority: medium
epics: [verification]
depends: [FEAT0064]
estimate: 3h
created: 2026-01-09
---

# Workflow video recording

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Record video walkthroughs of key user journeys during E2E tests to verify flows stay functional and provide debugging artifacts.

## Context

Video recordings capture the full user experience during E2E tests. When tests fail, videos show exactly what happened. They also serve as living documentation of how the UI should behave.

## Tasks

### Task 1: Enable video recording in Playwright

**Files:**
- Modify: `web-ui/playwright.config.ts`

**Steps:**
1. Update Playwright config:
   ```typescript
   use: {
     video: 'on-first-retry', // Or 'on' for always
     trace: 'on-first-retry',
   },
   outputDir: 'e2e/results/',
   ```
2. Add `e2e/results/` and `e2e/videos/` to `.gitignore`
3. Run: `just web e2e`
4. Commit: `feat(web-ui): enable video recording for E2E tests`

### Task 2: Create navigation workflow test

**Files:**
- Create: `web-ui/e2e/workflows/navigation.spec.ts`

**Steps:**
1. Create test that navigates through main sections:
   ```typescript
   test('core navigation flow', async ({ page }) => {
     await page.goto('/');
     await page.click('text=SESSIONS');
     await expect(page).toHaveURL('/sessions');
     await page.click('text=FIREHOSE');
     await expect(page).toHaveURL('/firehose');
     await page.click('text=GROOVE');
     await expect(page).toHaveURL('/groove');
   });
   ```
2. Run test and verify video is generated
3. Commit: `test(web-ui): add navigation workflow test`

### Task 3: Create dashboard workflow test

**Files:**
- Create: `web-ui/e2e/workflows/dashboard.spec.ts`

**Steps:**
1. Create test for dashboard journey:
   - Navigate to Dashboard Overview
   - Click through to Learnings
   - Navigate to Attribution
   - Navigate to Strategy
   - Navigate to Health
2. Verify each page loads correctly
3. Run test and verify video
4. Commit: `test(web-ui): add dashboard workflow test`

### Task 4: Create settings workflow test

**Files:**
- Create: `web-ui/e2e/workflows/settings.spec.ts`

**Steps:**
1. Create test for settings interactions:
   - Navigate to Settings
   - Toggle theme (dark/light)
   - Toggle CRT effects
   - Toggle learning indicator
   - Verify changes persist
2. Run test and verify video
3. Commit: `test(web-ui): add settings workflow test`

### Task 5: Add CI artifact upload

**Files:**
- Modify: `.github/workflows/ci.yml`

**Steps:**
1. Add step to upload videos as artifacts on failure:
   ```yaml
   - name: Upload test artifacts
     if: failure()
     uses: actions/upload-artifact@v4
     with:
       name: playwright-results
       path: web-ui/e2e/results/
       retention-days: 7
   ```
2. Test by triggering CI
3. Commit: `ci: upload E2E video artifacts on failure`

## Acceptance Criteria

- [ ] Playwright configured to record videos
- [ ] Navigation workflow test exists and passes
- [ ] Dashboard workflow test exists and passes
- [ ] Settings workflow test exists and passes
- [ ] Videos generated in `e2e/results/`
- [ ] CI uploads artifacts on test failure

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0065`
3. Commit, push, and create PR

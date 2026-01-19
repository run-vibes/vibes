---
id: CHORE0067
title: Test organization cleanup
type: chore
status: done
priority: low
scope: verification
depends: []
estimate: 3h
created: 2026-01-09
---

# Test organization cleanup

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Audit and standardize test file organization across web-ui and design-system packages, adopting co-located tests as the convention.

## Context

Tests should be co-located with their source files (e.g., `Button.test.tsx` next to `Button.tsx`) unless they test across boundaries (integration/E2E tests). This makes tests discoverable and keeps related code together.

## Tasks

### Task 1: Audit current test locations

**Steps:**
1. List all test files in web-ui:
   ```bash
   find web-ui/src -name "*.test.*" -type f
   ```
2. List all test files in design-system:
   ```bash
   find design-system/src -name "*.test.*" -type f
   ```
3. Identify files in `__tests__/` folders that should be co-located
4. Document findings in scratch notes
5. No commit (research task)

### Task 2: Move tests to co-located positions

**Files:**
- Move any `__tests__/*.test.ts` to sibling of source file

**Steps:**
1. For each test in `__tests__/` folder:
   - Move to same directory as source file
   - Update any relative imports
2. Remove empty `__tests__/` directories
3. Run: `just web test`
4. Commit: `refactor(web-ui): co-locate test files with source`

### Task 3: Standardize test file naming

**Steps:**
1. Ensure pattern: `<SourceFile>.test.tsx` or `<SourceFile>.test.ts`
2. Check for mismatches (e.g., `componentTests.tsx` instead of `Component.test.tsx`)
3. Rename any non-conforming files
4. Run: `just web test`
5. Commit: `refactor: standardize test file naming`

### Task 4: Consolidate test utilities

**Files:**
- Create: `web-ui/src/test-utils/index.ts`
- Modify: Various test files

**Steps:**
1. Identify duplicate test utilities:
   - Render wrappers with providers
   - Mock factories
   - Common assertions
2. Create `test-utils/` directory with shared utilities
3. Update tests to import from shared location
4. Run: `just web test`
5. Commit: `refactor(web-ui): consolidate test utilities`

### Task 5: Create TESTING.md

**Files:**
- Create: `docs/TESTING.md`

**Steps:**
1. Document testing conventions:
   - Co-located tests (with source file)
   - E2E tests in `e2e/` directory
   - Integration tests in `tests/` (Rust) or `e2e/` (TS)
   - Test file naming: `<Source>.test.{ts,tsx}`
   - Test utilities location
   - How to run tests (`just tests run`, `just web test`)
2. Add coverage gap list (components/hooks without tests)
3. Commit: `docs: add TESTING.md conventions`

## Acceptance Criteria

- [ ] All web-ui tests are co-located with source files
- [ ] All design-system tests are co-located with source files
- [ ] No empty `__tests__/` directories remain
- [ ] Test file naming is consistent
- [ ] Test utilities consolidated in `test-utils/`
- [ ] `docs/TESTING.md` documents conventions
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done CHORE0067`
3. Commit, push, and create PR

# Testing Conventions

This document describes testing conventions for the vibes project.

## Test Organization

### Co-located Tests

Tests are co-located with their source files:

```
src/
├── components/
│   ├── Button.tsx
│   └── Button.test.tsx      # Co-located with source
├── hooks/
│   ├── useTheme.ts
│   └── useTheme.test.ts     # Co-located with source
```

**Why:** Keeps related code together, makes tests discoverable, easier to maintain.

### Integration & E2E Tests

Tests that span multiple modules live in dedicated directories:

- `e2e-tests/` - Playwright E2E tests (browser automation)
- `<crate>/tests/` - Rust integration tests (cross-module)

## File Naming

| Type | Pattern | Example |
|------|---------|---------|
| Unit test | `<Source>.test.{ts,tsx}` | `Button.test.tsx` |
| Integration test | `<feature>.spec.ts` | `navigation.spec.ts` |
| Visual test | `visual.spec.ts` | `visual.spec.ts` |

## Running Tests

### All Tests

```bash
just pre-commit          # Full test suite (Rust + TypeScript)
```

### By Package

```bash
just tests run           # Rust tests (all crates)
just web test            # Web UI unit tests
npm test --workspace=@vibes/design-system  # Design system tests
```

### E2E Tests

```bash
just web e2e             # Run all E2E tests
just web e2e-headed      # Run with visible browser
just web visual          # Run visual regression only
just web visual-update   # Update visual baselines
just web workflows       # Run workflow tests (generates videos)
```

### Specific Tests

```bash
# Rust: run tests matching pattern
cargo nextest run <pattern>

# TypeScript: run tests matching pattern
npm test --workspace=vibes-web-ui -- --run -t "<pattern>"
```

## Visual Regression

Visual regression tests capture screenshots and compare against baselines.

**Baselines:** Stored in `e2e-tests/snapshots/`

**Update baselines:**
```bash
just web visual-update
```

**CI behavior:** Fails if screenshots differ by more than 1% pixel ratio.

## Workflow Videos

E2E tests record videos for debugging and documentation.

**Location:** `e2e-tests/test-results/` (gitignored)

**CI:** Uploaded as artifacts on test failure.

## CLI Recordings

CLI documentation uses VHS for terminal recordings.

```bash
just cli record          # Generate all GIFs
just cli verify          # Verify output matches expected
```

**Tapes:** `cli/recordings/tapes/*.tape`
**Output:** `cli/recordings/output/*.gif` (gitignored)
**Expected:** `cli/recordings/expected/*.txt`

## Test Tools

| Tool | Purpose |
|------|---------|
| `vitest` | TypeScript unit tests |
| `@testing-library/react` | React component testing |
| `@playwright/test` | E2E and visual regression |
| `cargo-nextest` | Rust test runner |
| `VHS` | CLI recording |

## Best Practices

1. **Test behavior, not implementation** - Focus on what the code does, not how
2. **One assertion per concept** - Keep tests focused
3. **Descriptive names** - Test names should describe the scenario
4. **Avoid mocking internals** - Mock at boundaries (APIs, file system)
5. **Keep tests fast** - Slow tests don't get run

## Coverage

Run coverage report:

```bash
just coverage summary    # Text summary
just coverage html       # HTML report
```

---
id: C005
title: Chore: Remove Old Styles and Document Tokens
type: chore
status: done
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-04
updated: 2026-01-07
milestone: 37-crt-design-system
---

# Chore: Remove Old Styles and Document Tokens

## Problem

After migrating to the CRT design system, old ad-hoc styles remain in the codebase. The design token system needs documentation for future development.

## Goal

Remove unused styles, ensure all components use design tokens, and document the token system.

## Tasks

### Task 1: Audit Hardcoded Colors

Search for hardcoded color values:
- Hex colors (#xxx, #xxxxxx)
- RGB/HSL values
- Tailwind color classes (bg-gray-800, etc.)

Replace with design token variables.

### Task 2: Audit Hardcoded Typography

Search for hardcoded font values:
- Font-family declarations
- Font-size values
- Tailwind text classes

Replace with typography tokens.

### Task 3: Remove Unused CSS

Identify and remove:
- Unused component styles
- Dead CSS selectors
- Redundant style files

### Task 4: Consolidate Tailwind Config

Update tailwind.config.js to extend with design tokens:
```js
theme: {
  extend: {
    colors: {
      phosphor: 'var(--phosphor)',
      screen: 'var(--screen)',
      // ...
    },
    fontFamily: {
      display: 'var(--font-display)',
      mono: 'var(--font-mono)',
    }
  }
}
```

**Note:** This project uses pure CSS with design tokens, not Tailwind CSS. Task skipped as not applicable.

### Task 5: Document Design Tokens

Create `design-system/DESIGN_TOKENS.md`:
- List all available tokens
- Usage examples
- Theme customization guide
- Component styling patterns

### Task 6: Final Verification

Run visual regression:
- Compare against prototypes
- Check all pages/routes
- Verify theme toggle
- Test responsive breakpoints

## Acceptance Criteria

- [x] No hardcoded colors in component files
- [x] No hardcoded font values in component files
- [x] Unused CSS removed
- [x] Tailwind config extended with tokens (N/A - project uses pure CSS tokens)
- [x] DESIGN_TOKENS.md documentation complete
- [x] All pages match CRT aesthetic
- [x] `just pre-commit` passes

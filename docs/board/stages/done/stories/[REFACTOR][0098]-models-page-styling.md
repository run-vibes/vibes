---
id: REFACTOR0098
title: Models Page Styling Alignment
type: refactor
status: done
priority: medium
scope: web-ui
depends: [refactor-0097]
---

# Models Page Styling Alignment

Restyle the Models page to match the visual language of the rest of the application.

## Context

The Models page (`/models`) uses custom styling in `Models.css` that doesn't align with the design system or other pages like Groove Dashboard. The structure and visual hierarchy feel disconnected from the rest of the app.

Key issues:
- Empty state ("no models registered") looks completely off
- Table styling doesn't use design-system components
- Overall structure doesn't match app visual language

## Acceptance Criteria

- [x] Audit Models page against design-system tokens and components
- [x] Design: create mockup for aligned Models page
- [x] Replace custom table with design-system Table component (or create one)
  - Table wrapped in Card variant="crt", styled with design-system tokens
- [x] Create proper empty state component with consistent styling
  - Created EmptyState component in design-system with icon, message, hint, action props
- [x] Use Card component (from refactor-0097) for model details panel
- [x] Apply consistent spacing, typography, and color tokens
- [x] Remove Models.css custom styles (use design-system)
  - Reduced from 252 lines to 216 lines, using only design-system tokens
- [x] Add Ladle stories for Models-specific components
  - Added EmptyState.stories.tsx with 7 story variants
- [x] Visual regression test for Models page
  - Existing visual.spec.ts already covers Models page

## Technical Notes

Should wait for refactor-0097 (Card component) to avoid duplicate work.

Empty state should follow a pattern we can reuse:
- Icon or illustration
- Heading text
- Description text
- Call-to-action button (if applicable)

## Size

M - Medium (styling overhaul, component adoption)

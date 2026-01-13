---
id: refactor-0097
title: Unified Card Component with Visual Regression Testing
type: refactor
status: pending
priority: high
epics: [web-ui, design-system]
---

# Unified Card Component with Visual Regression Testing

Create a base Card component in the design system and add visual regression testing to catch UI inconsistencies.

## Context

Currently the web-ui has multiple inconsistent card patterns:
- `TrendCard`, `LearningsCard`, `AttributionCard`, `StrategyCard` (custom `.dashboard-card` class)
- `HealthCard` (uses `Panel variant="crt"`)
- Settings uses yet another pattern with sections/panels

Cards should share a common base component with consistent styling, then extend only when necessary.

Additionally, Ladle styleguide doesn't look 1:1 with the application, making it unreliable for design verification.

## Acceptance Criteria

### Base Card Component
- [ ] Create `Card` component in design-system with variants (default, crt, elevated)
- [ ] Card supports: header, body, footer slots
- [ ] Card supports: padding sizes (compact, default, spacious)
- [ ] Document Card API in Ladle stories
- [ ] Migrate dashboard cards to use base Card
- [ ] Migrate Settings panels to use base Card
- [ ] Remove duplicate card CSS

### Visual Regression Testing
- [ ] Add Playwright or similar for visual snapshot testing
- [ ] Create baseline snapshots for Ladle stories
- [ ] Create baseline snapshots for key app pages (Dashboard, Settings, Models)
- [ ] Configure CI to run visual regression on PRs
- [ ] Generate visual diff report in PR comments
- [ ] Document how to update baselines when intentional changes are made

## Technical Notes

Visual regression options to evaluate:
- **Playwright** - built-in screenshot comparison
- **Chromatic** - Storybook/Ladle integration, cloud-based
- **Percy** - CI integration, visual review workflow
- **reg-suit** - open source, self-hosted

Recommend starting with Playwright since we already use it for e2e.

## Size

L - Large (new component, migration, CI integration)

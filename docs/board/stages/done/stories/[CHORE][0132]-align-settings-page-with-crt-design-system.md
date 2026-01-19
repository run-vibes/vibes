---
id: CHORE0132
title: Align Settings page with CRT design system
type: chore
status: done
priority: medium
scope: web-ui/27-crt-visual-design
depends: []
estimate:
created: 2026-01-08
---

# Align Settings page with CRT design system

## Summary

The `/settings` page has inconsistent styling compared to other pages in the app:

1. **No page padding** - Content is flush against edges instead of using `padding: var(--space-4)`
2. **Constrained width** - `max-width: 600px` doesn't utilize horizontal space on large screens
3. **Inconsistent fonts** - Uses mix of font families instead of consistent `--font-mono`
4. **"Local" badge in navbar** - Should be removed from the Header component

## Acceptance Criteria

- [ ] Settings page has same padding as Firehose/Sessions pages
- [ ] Settings panels use full available width with responsive grid
- [ ] Font family is consistent with other pages (mono for content)
- [ ] "Local" badge removed from Header component in design-system
- [ ] Header styling matches other pages
- [ ] Page looks good at both narrow and wide viewports

## Implementation Notes

### Settings page changes

Update `Settings.css`:
- Add `padding: var(--space-4)` to `.settings-page`
- Remove `max-width: 600px` from `.settings-content`
- Use responsive grid similar to AssessmentStatus for panels
- Ensure consistent font families

### Header component changes

Update `design-system/src/compositions/Header/Header.tsx`:
- Remove line 62: `{isLocal && <Badge status="idle">Local</Badge>}`
- Remove `isLocal` prop from HeaderProps interface
- Update Header tests if any reference isLocal

Update `web-ui/src/App.tsx`:
- Remove `isLocal` prop from Header usage

## Files to modify

- `web-ui/src/pages/Settings.css`
- `web-ui/src/pages/Settings.tsx` (if structure changes needed)
- `design-system/src/compositions/Header/Header.tsx`
- `design-system/src/compositions/Header/Header.test.tsx`
- `web-ui/src/App.tsx`

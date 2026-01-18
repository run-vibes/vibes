# Dashboard Trends Route Returns Not Found

The `/groove/dashboard/trends` route shows "Not Found" with no styling or spacing.

## Problem

When navigating to `/groove/dashboard/trends`:
1. The page displays raw "Not Found" text
2. There's no padding or spacing around the message
3. The route may not be properly defined or the component is missing

## Expected Behavior

Either:
- The trends page should render with proper content and CRT styling
- Or if the route is intentionally unavailable, show a styled 404 page with proper spacing

## Acceptance Criteria

- [ ] `/groove/dashboard/trends` route renders properly
- [ ] If route exists: shows trends content with CRT styling
- [ ] If route removed: update navigation to not link to it
- [ ] Error states have proper padding (`var(--space-4)` minimum)

## Technical Notes

Check:
- `web-ui/src/pages/dashboard/` for trends route/component
- `web-ui/src/routeTree.gen.ts` for route registration
- Dashboard navigation tabs for stale links

---
id: chore-0001-nav-cleanup
title: "Chore: Clean Up Navigation Items"
type: chore
status: backlog
priority: medium
epics: [web-ui]
depends: []
estimate:
created: 2026-01-07
updated: 2026-01-07
---

# Chore: Clean Up Navigation Items

## Summary

Clean up the main navigation by removing redundant items, using full names, and standardizing the header actions.

## Changes

### Navigation Items

1. **Remove "Dash"** - It's redundant with clicking the logo (both go to homepage)
2. **Rename "Sess" → "Sessions"** - Use full name for clarity
3. **Rename "Fire" → "Firehose"** - Use full name for clarity

### Header Actions

Apply the design from `docs/design/prototypes/22-vibes-home-dashboard.html`:

```html
<span class="header-action">◐ THEME</span>
<span class="header-action">⚙ SETTINGS</span>
```

- Icon + text format
- `header-action` class styling (display font, dim color, hover brightens)
- Consistent spacing

### Before

```
VIBES  [DASH] [SESS] [FIRE] [GROOVE]     [◐] [THEME] [⚙] [SETTINGS]
```

### After

```
VIBES  [SESSIONS] [FIREHOSE] [GROOVE]     [◐ THEME] [⚙ SETTINGS]
```

## Acceptance Criteria

- [ ] "Dash" nav item removed
- [ ] "Sess" renamed to "Sessions"
- [ ] "Fire" renamed to "Firehose"
- [ ] Theme action shows "◐ THEME"
- [ ] Settings action shows "⚙ SETTINGS"
- [ ] Clicking logo returns to homepage
- [ ] All routes still work correctly

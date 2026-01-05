# Milestone 37: CRT Design System - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** Implement the CRT Essence design system across the vibes web UI.

**Design:** See [design.md](design.md) for design tokens and architecture.

---

## Current State

The web UI uses ad-hoc Tailwind styles with no consistent design system. Components have inconsistent colors, spacing, and typography.

**Reference prototypes:**
- `docs/design/prototypes/19-crt-essence-v4.html` (dark theme)
- `docs/design/prototypes/15-crt-daylight-v2.html` (light theme)

---

## Stories

| # | Story | Description |
|---|-------|-------------|
| 1 | [feat-01-design-tokens](stories/feat-01-design-tokens.md) | Create CSS custom properties for all design tokens |
| 2 | [feat-02-theme-toggle](stories/feat-02-theme-toggle.md) | Add theme switching with localStorage persistence |
| 3 | [feat-03-crt-effects](stories/feat-03-crt-effects.md) | Implement optional scanlines and vignette effects |
| 4 | [feat-04-typography](stories/feat-04-typography.md) | Apply VT323/IBM Plex Mono fonts across UI |
| 5 | [feat-05-core-components](stories/feat-05-core-components.md) | Restyle buttons, cards, inputs with CRT tokens |
| 6 | [feat-06-navigation](stories/feat-06-navigation.md) | Restyle sidebar and navigation with phosphor glow |
| 7 | [feat-07-firehose](stories/feat-07-firehose.md) | Apply CRT styling to event stream |
| 8 | [feat-08-session-cards](stories/feat-08-session-cards.md) | Restyle session cards with glow effects |
| 9 | [chore-09-cleanup](stories/chore-09-cleanup.md) | Remove old ad-hoc styles, document tokens |

> **Status:** Check story frontmatter or run `just board` for current status.

## Dependencies

```
feat-01-design-tokens (foundation)
       │
       ├── feat-02-theme-toggle
       │       │
       │       └── feat-03-crt-effects (requires theme context)
       │
       └── feat-04-typography
               │
               ├── feat-05-core-components
               │       │
               │       ├── feat-06-navigation
               │       ├── feat-07-firehose
               │       └── feat-08-session-cards
               │
               └── chore-09-cleanup (runs last)
```

Story 1 must complete first. Stories 2-4 can run in parallel after Story 1. Stories 5-8 depend on tokens and typography. Story 9 runs last.

## Completion Criteria

- [ ] All design tokens defined as CSS custom properties
- [ ] Theme toggle works with localStorage persistence
- [ ] Dark theme matches prototype 19 aesthetic
- [ ] Light theme matches prototype 15 aesthetic
- [ ] Scanlines/vignette effects toggleable
- [ ] All core components use design tokens
- [ ] No hardcoded colors remain
- [ ] `just pre-commit` passes

## Testing Strategy

Each story should include:
1. Visual comparison against reference prototype
2. Cross-browser check (Chrome, Firefox, Safari)
3. Theme toggle verification
4. Mobile responsiveness check

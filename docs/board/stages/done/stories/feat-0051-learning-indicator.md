---
id: FEAT0051
title: Learning indicator (Settings + Header)
type: feat
status: done
priority: low
epics: [plugin-system]
depends: [FEAT0043]
estimate: 2h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# Learning indicator (Settings + Header)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add opt-in learning indicator in header with Settings toggle for power users.

## Context

Power users can opt-in to see a learning indicator in the header that shows when the groove system is active. This is hidden by default to keep the UI clean for users who don't need visibility. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Add groove settings

**Files:**
- Modify: `web-ui/src/pages/Settings.tsx`
- Create: `web-ui/src/hooks/useGrooveSettings.ts`

**Steps:**
1. Update `Settings.tsx`:
   - Add GROOVE section
   - Learning Indicator toggle
   - Dashboard Auto-Refresh toggle
2. Create `useGrooveSettings.ts`:
   ```typescript
   interface GrooveSettings {
     showLearningIndicator: boolean;
     dashboardAutoRefresh: boolean;
   }

   function useGrooveSettings(): {
     settings: GrooveSettings;
     updateSetting: (key: keyof GrooveSettings, value: boolean) => void;
   }
   ```
3. Use localStorage for persistence
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add groove settings`

### Task 2: Create learning indicator

**Files:**
- Create: `web-ui/src/components/LearningIndicator.tsx`
- Create: `web-ui/src/components/LearningIndicator.css`

**Steps:**
1. Create `LearningIndicator` component:
   - 🧠 icon with states
   - Tooltip showing current status
   - Click to expand detailed status
2. Implement states:
   - Hidden (not rendered when disabled)
   - Idle (static 🧠)
   - Active (pulsing 🧠)
   - Error (red 🧠)
3. Style with CRT theme:
   - Subtle glow when active
   - Pulse animation
4. Run: `npm run typecheck --workspace=web-ui`
5. Commit: `feat(web-ui): add LearningIndicator component`

### Task 3: Integrate with header

**Files:**
- Modify: `web-ui/src/components/Header.tsx`
- Modify: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Update `Header.tsx`:
   - Conditionally render LearningIndicator
   - Position in header toolbar
   - Check groove settings for visibility
2. Update `useDashboard.ts`:
   - Add activity event subscription
   - Export indicator state hook
3. Run: `npm run typecheck --workspace=web-ui`
4. Commit: `feat(web-ui): integrate learning indicator in header`

### Task 4: Add tests

**Files:**
- Create: `web-ui/src/hooks/__tests__/useGrooveSettings.test.ts`
- Create: `web-ui/src/components/__tests__/LearningIndicator.test.tsx`

**Steps:**
1. Write settings hook tests:
   - Test default values
   - Test persistence
   - Test updates
2. Write indicator tests:
   - Test state rendering
   - Test visibility toggle
   - Test click interaction
3. Run: `npm test --workspace=web-ui -- --run`
4. Commit: `test(web-ui): add learning indicator tests`

## Acceptance Criteria

- [ ] Settings page has GROOVE section
- [ ] Learning Indicator toggle in settings
- [ ] Settings persist in localStorage
- [ ] Indicator hidden by default
- [ ] Indicator shows in header when enabled
- [ ] Indicator states: idle, active, error
- [ ] Pulsing animation when active
- [ ] Click shows status tooltip
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0051`
3. Commit, push, and create PR

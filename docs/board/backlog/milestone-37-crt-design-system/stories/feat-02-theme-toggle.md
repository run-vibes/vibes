---
created: 2026-01-04
status: pending
---

# Feature: Add Theme Toggle with Persistence

## Problem

Users cannot switch between dark and light themes. The UI is fixed to one appearance regardless of user preference or environment.

## Goal

Add a theme toggle that switches between CRT Essence (dark) and CRT Daylight (light) themes, with localStorage persistence.

## Tasks

### Task 1: Create Theme Context

Create a React context to manage theme state:
- `ThemeProvider` component wrapping the app
- `useTheme()` hook returning `{ theme, toggleTheme }`
- Initialize from localStorage or system preference

### Task 2: Apply Theme to Document

Set `data-theme` attribute on `<html>` element:
```typescript
useEffect(() => {
  document.documentElement.dataset.theme = theme;
}, [theme]);
```

### Task 3: Create Theme Toggle Button

Add toggle button to header/settings:
- Sun/moon icons (or CRT-themed alternatives)
- Keyboard shortcut (Ctrl/Cmd + Shift + T)
- Accessible label

### Task 4: Persist Theme Selection

Save to localStorage on change:
```typescript
localStorage.setItem('vibes-theme', theme);
```

Load on app start, falling back to system preference.

### Task 5: Respect System Preference

Use `prefers-color-scheme` media query as fallback:
```typescript
const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches
  ? 'dark'
  : 'light';
```

## Acceptance Criteria

- [ ] Theme toggle button visible in UI
- [ ] Click toggles between dark and light themes
- [ ] Theme persists across page reloads
- [ ] System preference used as default (no stored pref)
- [ ] Keyboard shortcut works
- [ ] No flash of wrong theme on page load

---
id: FEAT0146
title: "Feature: Apply CRT Typography System"
type: feat
status: done
priority: medium
scope: models/01-model-management
depends: []
estimate:
created: 2026-01-04
---

# Feature: Apply CRT Typography System

## Problem

The UI uses default system fonts. The CRT aesthetic requires VT323 for display text and IBM Plex Mono for code/body text.

## Goal

Integrate VT323 and IBM Plex Mono fonts, applying them consistently across all UI elements.

## Tasks

### Task 1: Add Font Imports

Add Google Fonts imports to `index.html`:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=VT323&display=swap" rel="stylesheet">
```

### Task 2: Define Font Utility Classes

Create utility classes in CSS:
```css
.font-display { font-family: var(--font-display); }
.font-mono { font-family: var(--font-mono); }
```

### Task 3: Apply VT323 to Headers

Update header elements to use display font:
- Navigation items
- Page titles
- Section headers
- Button text
- Labels and badges

### Task 4: Apply IBM Plex Mono to Body

Update body text to use mono font:
- Paragraphs
- Code blocks
- Input fields
- Data displays
- Event content

### Task 5: Set Font Sizes

Apply font size tokens consistently:
- `--font-size-2xl` for hero/page titles
- `--font-size-xl` for section headers
- `--font-size-lg` for component headers
- `--font-size-base` for body text
- `--font-size-sm` for secondary text
- `--font-size-xs` for captions/badges

## Acceptance Criteria

- [x] VT323 renders for display text
- [x] IBM Plex Mono renders for body/code
- [x] Fonts load without FOUT (flash of unstyled text)
- [x] Font sizes match design spec
- [x] Text remains readable at all sizes
- [x] Consistent font usage across all components

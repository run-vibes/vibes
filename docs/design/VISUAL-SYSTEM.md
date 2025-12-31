# vibes Visual System

> **The Warm Terminal** — In a world of cold, clinical developer tools, vibes feels like a well-worn leather chair in a server room.

This document defines the complete visual design system for vibes and its plugins (including groove). It serves as the source of truth for all UI implementation.

## Table of Contents

1. [Brand Philosophy](#brand-philosophy)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Layout & Panel Structure](#layout--panel-structure)
5. [Visualizations](#visualizations)
6. [Component Library](#component-library)
7. [Keyboard & Command System](#keyboard--command-system)
8. [Iggy Stream Views](#iggy-stream-views)
9. [groove Plugin Views](#groove-plugin-views)
10. [Accessibility](#accessibility)
11. [User Preferences](#user-preferences)
12. [Light Theme](#light-theme)

---

## Brand Philosophy

### Core Identity

**vibes** is the warm terminal. Professional and serious, but with warmth that says *"we've got your back."*

| Principle | What it means |
|-----------|---------------|
| **Warm, not cold** | Dark backgrounds with amber warmth, not cyan sterility |
| **Dense, not sparse** | Information-rich screens that respect your expertise |
| **Keyboard-first** | Everything accessible without a mouse, but mouse works too |
| **Semantic color** | Colors mean things. Never decorative. |
| **Mainframe soul** | Panel structure, line commands, status awareness |
| **Plugin as application** | Plugins feel like apps within the shell, with their own identity |

### The Phosphor Metaphor

The warm glow of CRT phosphor screens is our visual anchor. Not retro-kitsch or nostalgia—but the *feeling* of reliability, the comfort of a tool that's been running since before you were born and will run after you're gone.

### Brand Hierarchy

- **vibes** = The shell/platform with its own identity (amber accent)
- **Plugins** (groove, future plugins) = Sub-brands that live within vibes but are visually distinguishable (each gets a unique accent color)

---

## Color System

### The Warm Terminal Palette

#### Background Scale (warm charcoal → elevated surfaces)

| Token | Hex | Swatch | Usage |
|-------|-----|--------|-------|
| `bg-base` | `#1a1816` | ![#1a1816](https://via.placeholder.com/20/1a1816/1a1816) | The deepest background |
| `bg-surface` | `#242220` | ![#242220](https://via.placeholder.com/20/242220/242220) | Cards, panels, elevated areas |
| `bg-elevated` | `#2e2c29` | ![#2e2c29](https://via.placeholder.com/20/2e2c29/2e2c29) | Hover states, active panels |
| `bg-overlay` | `#383532` | ![#383532](https://via.placeholder.com/20/383532/383532) | Modals, dropdowns |

> **Note:** These are NOT pure black. They have warm undertones. Compare to pure `#000000` — ours feel like worn leather.

#### Text Scale (high to low intensity)

| Token | Hex | Swatch | Usage |
|-------|-----|--------|-------|
| `text-primary` | `#f0ebe3` | ![#f0ebe3](https://via.placeholder.com/20/f0ebe3/f0ebe3) | Main content, headings |
| `text-secondary` | `#b8b2a8` | ![#b8b2a8](https://via.placeholder.com/20/b8b2a8/b8b2a8) | Descriptions, metadata |
| `text-muted` | `#6b665c` | ![#6b665c](https://via.placeholder.com/20/6b665c/6b665c) | Timestamps, hints, disabled |
| `text-faint` | `#4a4640` | ![#4a4640](https://via.placeholder.com/20/4a4640/4a4640) | Borders, subtle separators |

> **Note:** Primary is cream, not pure white. Easier on eyes.

#### Semantic Colors (functional, never decorative)

| Token | Hex | Swatch | Meaning | Usage |
|-------|-----|--------|---------|-------|
| `amber` | `#e6b450` | ![#e6b450](https://via.placeholder.com/20/e6b450/e6b450) | Action, focus | Links, vibes core accent |
| `amber-dim` | `#a68332` | ![#a68332](https://via.placeholder.com/20/a68332/a68332) | Amber at low intensity | Inactive states |
| `green` | `#7ec699` | ![#7ec699](https://via.placeholder.com/20/7ec699/7ec699) | Success, ready | Connected, healthy |
| `green-dim` | `#4a7a5c` | ![#4a7a5c](https://via.placeholder.com/20/4a7a5c/4a7a5c) | Green at low intensity | Background indicators |
| `red` | `#e05252` | ![#e05252](https://via.placeholder.com/20/e05252/e05252) | Error, critical | Errors, ABEND, destructive |
| `red-dim` | `#8c3a3a` | ![#8c3a3a](https://via.placeholder.com/20/8c3a3a/8c3a3a) | Red at low intensity | Error backgrounds |
| `blue` | `#6ba3d6` | ![#6ba3d6](https://via.placeholder.com/20/6ba3d6/6ba3d6) | Info, labels | Secondary actions, info |
| `blue-dim` | `#4a6d8c` | ![#4a6d8c](https://via.placeholder.com/20/4a6d8c/4a6d8c) | Blue at low intensity | Info backgrounds |

> **Rule:** Never use these for decoration. Only for meaning.

#### Plugin Accent Colors

Each plugin gets its own accent color for visual distinction:

| Plugin | Hex | Swatch | Description |
|--------|-----|--------|-------------|
| vibes core | `#e6b450` | ![#e6b450](https://via.placeholder.com/20/e6b450/e6b450) | The shell, sessions, config |
| groove | `#c9a227` | ![#c9a227](https://via.placeholder.com/20/c9a227/c9a227) | Learning system (gold/vinyl) |
| *reserved* | `#5fb3a1` | ![#5fb3a1](https://via.placeholder.com/20/5fb3a1/5fb3a1) | Future plugin (teal) |
| *reserved* | `#b07cc6` | ![#b07cc6](https://via.placeholder.com/20/b07cc6/b07cc6) | Future plugin (violet) |

Plugin accent appears in: header tint, active nav, focus rings.

#### The Phosphor Glow

CSS for that CRT warmth (applied sparingly to key elements):

```css
/* Subtle glow for focused/active elements */
.phosphor-glow {
  text-shadow:
    0 0 1px currentColor,
    0 0 4px rgba(230, 180, 80, 0.2);
}

/* Stronger glow for emphasis */
.phosphor-glow-strong {
  text-shadow:
    0 0 2px currentColor,
    0 0 8px rgba(230, 180, 80, 0.3),
    0 0 16px rgba(230, 180, 80, 0.1);
}
```

---

## Typography

### Monospace Everything

vibes is a terminal. Everything is monospace. This isn't a limitation—it's a feature.

#### Font Stack

```css
font-family:
  "Berkeley Mono",      /* Ideal - warmth and personality */
  "JetBrains Mono",     /* Bundled default (open source) */
  "Fira Code",
  "SF Mono",
  "Consolas",
  monospace;
```

> **Note:** We ship JetBrains Mono as the bundled default. Users can override with their preferred font.

#### Type Scale

Sized for information density:

| Token | Size | rem | Usage |
|-------|------|-----|-------|
| `text-xs` | 11px | 0.6875rem | Timestamps, line numbers |
| `text-sm` | 12px | 0.75rem | Secondary info, metadata |
| `text-base` | 13px | 0.8125rem | Primary content, default |
| `text-lg` | 14px | 0.875rem | Emphasized content |
| `text-xl` | 16px | 1rem | Panel headers |
| `text-2xl` | 18px | 1.125rem | Page titles |

> **Note:** Smaller than typical web. Density matters. These are optimized for developers who want more on screen.

**Line height:** 1.5 for readability (20px at text-base)

#### Intensity Levels (The 3270 Way)

Instead of font-weight for emphasis:

| Level | Style | Usage |
|-------|-------|-------|
| High | `text-primary` + `font-medium` | Focus, headings |
| Normal | `text-primary` + `font-normal` | Default content |
| Low | `text-secondary` + `font-normal` | Secondary info |
| Dim | `text-muted` + `font-normal` | Hints, disabled |

Example:
```
SESSION-001  auth-refactor   ACTIVE   2m ago
↑ high       ↑ normal        ↑ high   ↑ dim
```

#### Grid Discipline

All layouts align to character grid where possible.

- Character width: ~7.8px at text-base (varies by font)
- Column widths: Multiples of 8 characters when practical

```
ID          NAME              STATUS    DURATION
────────    ────────────────  ────────  ────────
sess-001    auth-refactor     ACTIVE    2m 34s
sess-002    fix-tests         IDLE      12m 01s
```

This creates the "everything lines up" feel of mainframe panels.

---

## Layout & Panel Structure

### The Mainframe Panel Model

Every screen is a **panel**. Panels have structure:

```
┌─ PANEL HEADER ──────────────────────────────────── STATUS ── TIME ─┐
│                                                                     │
│  BODY AREA                                                          │
│  (content, data, visualizations)                                    │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  COMMAND/STATUS LINE                                   FUNCTION KEYS │
└─────────────────────────────────────────────────────────────────────┘
```

#### Panel Header Anatomy

```
┌─ vibes › groove › Dashboard ────────── ● connected ── 14:32:01 ───┐
   │       │         │                   │              │
   │       │         │                   │              └─ Clock
   │       │         │                   └─ System status
   │       │         └─ Current panel/page
   │       └─ Plugin namespace (if in plugin)
   └─ Product name
```

When in core vibes (not a plugin):
```
┌─ vibes › Sessions ──────────────────── ● connected ── 14:32:01 ───┐
```

### Responsive Philosophy

**Principle:** "Works at 80, shines at 160"

| Width | Experience |
|-------|------------|
| 80 chars | Baseline. Everything functional. Stack if needed. |
| 120 chars | Comfortable. Side nav + content. |
| 160 chars | Luxurious. Multi-pane layouts, detail panels. |
| 200+ chars | Ultrawide. Dashboard grids, side-by-side compare. |

The grid discipline still applies—align to character widths. But panels GROW to use available space.

**Example - Firehose at different widths:**

80 chars:
```
┌─ FIREHOSE ─────────────────────────────────────────────────────┐
│ 14:32:01 SESSION sess-abc "auth refactor"                      │
│ 14:32:02 TOOL    Read src/lib.rs                               │
└────────────────────────────────────────────────────────────────┘
```

160 chars:
```
┌─ FIREHOSE ──────────────────────────┬─ EVENT DETAIL ───────────┐
│ 14:32:01.234 SESSION sess-abc...    │ Type: SessionCreated     │
│ 14:32:02.789 TOOL    Read src/lib.. │ Session: sess-abc        │
│ ▶ selected                          │ Name: "auth refactor"    │
└─────────────────────────────────────┴──────────────────────────┘
```

---

## Visualizations

All visualizations should feel like they belong in a terminal. No glossy charts.

### Sparklines (Unicode block characters)

```
Using Unicode blocks: ▁ ▂ ▃ ▄ ▅ ▆ ▇ █

Events:  ▁▂▃▅▇█▇▅▃▂▁▁▂▄▆█▇▅▃▁▁▂▃▄▅▆▇█▇▆▅▄  (last 30m)
Tokens:  ▂▂▃▃▄▅▆▇████▇▆▅▄▃▂▂▁▁▁▂▃▄▅▆▇███  (last 30m)
```

Color using semantic colors:
- Green sparkline = healthy metrics
- Amber sparkline = attention needed
- Red spike = error event

### Progress/Gauge Bars

```
Confidence:  ████████░░  82%     (filled █, empty ░)
Tokens:      ██████████  12.4k   (full = at limit)
Progress:    ████░░░░░░  40%
```

Use 10 characters for standard gauges (each block = 10%).

### Timeline/Scrubber

```
├────────────────●───────────────────────────────────────────────┤
12:00           ▲                                           14:32
             12:47:23

Dense regions show activity clusters:
├──░░░░░████████░░░░░██████░░░░░░░░░████████████░░░░░░░░░░░░░░─┤
   quiet  busy   quiet busy  quiet        very busy       quiet
```

### ASCII Charts

For larger visualizations:

```
Events per hour (last 24h)
┌────────────────────────────────────────────────────────────────┐
│                                    ██                          │
│                                    ██ ██                       │
│                          ██        ██ ██                       │
│                    ██    ██ ██     ██ ██ ██                    │
│              ██    ██ ██ ██ ██ ██  ██ ██ ██ ██                 │
│  ██    ██ ██ ██ ██ ██ ██ ██ ██ ██ ██ ██ ██ ██ ██ ██    ██ ██  │
└────────────────────────────────────────────────────────────────┘
 00   03   06   09   12   15   18   21   00
```

Uses half-block characters for 2x vertical resolution: `▄ █`

### Box Drawing Characters

Panels use box-drawing characters for authentic terminal feel:

```
Single line:  ┌ ─ ┐ │ └ ┘ ├ ┤ ┬ ┴ ┼
Double line:  ╔ ═ ╗ ║ ╚ ╝ ╠ ╣ ╦ ╩ ╬  (for modals/emphasis)
```

---

## Component Library

### Status Indicators

**Connection states (system-level):**
- `● connected` (green, filled)
- `○ connecting` (amber, hollow, animated pulse)
- `● disconnected` (red, filled)

**Session states (mainframe job status style):**
- `ACTIVE` (green, high intensity)
- `IDLE` (amber, normal intensity)
- `WAITING` (amber, pulsing - needs input)
- `COMPLETE` (dim, normal intensity)
- `ABEND` (red, high intensity - abnormal end)

**groove states:**
- `◉ learning` (gold, the groove icon)
- `◉ ready` (green)
- `◉ paused` (dim)

### Buttons

```
Primary action:    [ Start Session ]   amber bg, dark text
Secondary action:  [ Cancel ]          border only, text color
Destructive:       [ Kill Session ]    red border, red text
Disabled:          [ Waiting... ]      dim, no interaction
```

- Style: Square brackets evoke terminal. No rounded corners.
- Hover: Subtle phosphor glow effect.
- Focus: Strong amber outline (accessibility).

Keyboard shortcut hints:
```
[ Start Session ]  ⌘S
└─ shortcut shown dimmed to the right
```

### Input Fields

```
Standard input:
  Session name: [auth-refactor________]
                 └─ block cursor, underscores show width

Command input (bottom of screen):
  Command: =sessions___________________________________
           └─ full width, command prefix style

Search/filter:
  /auth____________  (prefix / indicates search mode)
```

Focus state: amber underline or border, phosphor glow.

### Tables (with line commands)

```
Cmd  ID          Name              Status    Duration    Events
───  ──────────  ────────────────  ────────  ──────────  ────────
 _   sess-001    auth-refactor     ACTIVE    2m 34s      ████░░ 47
 s   sess-002    fix-tests         IDLE      12m 01s     ██░░░░ 12
 _   sess-003    docs-update       COMPLETE  1h 02m      █████░ 89
```

Line command column:
- `_` = empty, ready for input
- `s` = select (highlight row, show detail)
- `a` = attach (connect to session)
- `k` = kill (terminate session)
- `d` = delete (remove from history)

Press Enter to execute. Mouse click also works.

### Navigation

**Top-level nav (always visible):**
```
┌─────────────────────────────────────────────────────────────┐
│  Sessions   Firehose   History   Config   groove ▾          │
│  ════════                                                   │
└─────────────────────────────────────────────────────────────┘
```
Active tab: underline (════) + high intensity

**Plugin sub-nav (when in plugin):**
```
┌─────────────────────────────────────────────────────────────┐
│  groove › Dashboard   Learnings   Assess   Settings         │
│           ═════════                                         │
└─────────────────────────────────────────────────────────────┘
```

Keyboard: Tab numbers (1-9) or `=name` to jump directly.

### Modals

```
╔═══════════════════════════════════════════════════════════════╗
║  Kill Session                                                 ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Are you sure you want to kill session "auth-refactor"?       ║
║                                                               ║
║  This will terminate the Claude process immediately.          ║
║  Unsaved work may be lost.                                    ║
║                                                               ║
║                          [ Cancel ]  [ Kill Session ]         ║
╚═══════════════════════════════════════════════════════════════╝
```

- Double-line border for modals = elevated importance
- Background: overlay at 80% opacity
- Focus trap: Tab cycles within modal
- Escape: closes modal

### Toast Notifications

Appears at bottom of screen, above command line:

```
┌─────────────────────────────────────────────────────────────────┐
│  ● Session attached: auth-refactor                              │
└─────────────────────────────────────────────────────────────────┘
```

Types:
- `● info` (blue dot) — "Session attached"
- `● success` (green dot) — "Learning captured"
- `● warning` (amber dot) — "Connection unstable"
- `● error` (red dot) — "Permission denied"

groove-specific toasts use the ◉ icon:
```
◉ groove: Picked up your preference for explicit error handling
```

Auto-dismiss after 4s. Stack up to 3.

---

## Keyboard & Command System

### The Command Line

A persistent command input at the bottom of the screen:

```
┌─────────────────────────────────────────────────────────────────┐
│ Command: _                                    F1=Help  F3=Back  │
└─────────────────────────────────────────────────────────────────┘
```

Press `/` or `:` to focus from anywhere. Escape returns focus.

### Command Syntax

**Navigation (= prefix):**
```
=sessions          Jump to Sessions panel
=firehose          Jump to Firehose
=groove            Jump to groove dashboard
=groove.learn      Jump to groove learnings
=1, =2, =3         Jump to tab by number
```

**Search (/ prefix):**
```
/auth              Filter current view for "auth"
/error             Show only errors
/sess-001          Find specific session
```

**Actions (no prefix):**
```
attach sess-001    Attach to session
kill sess-001      Kill session
new "my session"   Create new session
pause              Pause firehose
export json        Export current view
```

**groove commands (groove. prefix):**
```
groove.status      Show groove status
groove.forget 47   Forget learning #47
groove.pause       Pause learning
```

### Global Keyboard Shortcuts

**Navigation:**
| Key | Action |
|-----|--------|
| `/` or `:` | Focus command line |
| `Escape` | Return to content / close modal |
| `1-9` | Jump to tab N |
| `[` | Previous tab |
| `]` | Next tab |
| `g g` | Go to top (vim-style) |
| `G` | Go to bottom |
| `?` | Show keyboard help overlay |

**Firehose-specific:**
| Key | Action |
|-----|--------|
| `Space` | Pause/resume stream |
| `f` | Toggle filter panel |
| `r` | Toggle replay mode |
| `j` / `k` | Navigate events (vim-style) |
| `Enter` | Expand selected event |
| `y` | Yank (copy) event to clipboard |

**Session list:**
| Key | Action |
|-----|--------|
| `n` | New session |
| `a` | Attach to selected |
| `k` | Kill selected |
| `Enter` | View detail |

**Line commands (when row focused):**
| Key | Action |
|-----|--------|
| `s` | Select |
| `a` | Attach |
| `k` | Kill |
| `d` | Delete |
| `e` | Edit/expand |

### Function Key Bar

Displayed at bottom, context-sensitive:

Sessions view:
```
F1=Help  F2=New  F3=Back  F5=Refresh  F7=Up  F8=Down  F10=Actions
```

Firehose view:
```
F1=Help  F3=Back  F4=Filter  F5=Pause  F6=Replay  F9=Export
```

Clickable AND keyboard-accessible. Hidden on narrow screens.

### Command Autocomplete

```
Command: =gro_
         ┌──────────────────┐
         │ =groove          │ ← highlighted
         │ =groove.dash     │
         │ =groove.learn    │
         │ =groove.assess   │
         └──────────────────┘
```

Tab or ↓ to select, Enter to execute. Fuzzy matching: "gd" matches "groove.dash".

---

## Iggy Stream Views

### Firehose (Live Stream)

```
┌─ vibes › Firehose ────────────────────── ● connected ── 14:32 ──┐
│                                                                  │
│ ┌─ Controls ───────────────────────────────────────────────────┐ │
│ │ 🔴 LIVE  ▁▂▃▅▇█▇▅▃▂  1.2k/hr   [Filter ▾]  [⏸ Pause]  [⟳]  │ │
│ └──────────────────────────────────────────────────────────────┘ │
│                                                                  │
│ ┌─ Stream ─────────────────────────────────────────────────────┐ │
│ │ TIME         TYPE      SESSION       SUMMARY                 │ │
│ │ ───────────  ────────  ────────────  ─────────────────────── │ │
│ │ 14:32:01.23  SESSION   sess-abc      Created "auth-refactor" │ │
│ │ 14:32:01.45  CLAUDE    sess-abc      TextDelta: "Let me..."  │ │
│ │ 14:32:02.78  TOOL      sess-abc      Read src/lib.rs (2.1kb) │ │
│ │ 14:32:03.01  ◉ ASSESS  sess-abc      Lightweight: OK         │ │
│ │ 14:32:03.23  CLAUDE    sess-abc      TextDelta: "I see..."   │ │
│ │ 14:32:03.45  TOOL      sess-abc      Edit src/auth.rs:47-52  │ │
│ │ 14:32:04.00  HOOK      sess-abc      ToolResult: success     │ │
│ │ 14:32:04.12  ERROR     sess-abc      Permission denied       │ │
│ │ ▼ streaming...                                                │ │
│ └──────────────────────────────────────────────────────────────┘ │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│ Command: _                                F5=Pause  F6=Replay    │
└──────────────────────────────────────────────────────────────────┘
```

**Event type colors:**
- `SESSION` = blue (lifecycle events)
- `CLAUDE` = primary (AI responses)
- `TOOL` = amber (tool calls)
- `HOOK` = dim (hook events, often noisy)
- `◉ ASSESS` = gold (groove assessments)
- `ERROR` = red (errors and failures)

### Replay Mode

```
┌─ vibes › Firehose › Replay ─────────────────────────── 14:32 ───┐
│                                                                  │
│ ┌─ Timeline ───────────────────────────────────────────────────┐ │
│ │                                                               │ │
│ │  Dec 30                                              Dec 31   │ │
│ │  ├──░░░░████████░░░░░██████░░░░░░░░░████████████░░░░░░░░░●──┤ │ │
│ │     12:00    14:00    16:00    18:00    20:00    22:00  now   │ │
│ │                                                      ▲        │ │
│ │                                                   cursor      │ │
│ │                                                               │ │
│ │  [⏮ Start]  [◀◀ -1h]  [◀ -5m]  [▶ Play]  [▶▶ +5m]  [⏭ Now]   │ │
│ │                                                               │ │
│ │  Jump to: [2024-12-30 14:__:__]  Speed: [1x ▾]                │ │
│ └───────────────────────────────────────────────────────────────┘ │
│                                                                  │
│ ┌─ Events at cursor ───────────────────────────────────────────┐ │
│ │ 14:32:01.23  SESSION   sess-abc      Created "auth-refactor" │ │
│ │ ...                                                           │ │
│ └───────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### Session Timeline

```
┌─ vibes › Sessions › auth-refactor ──────────────────── 14:32 ───┐
│                                                                  │
│  Status: ACTIVE    Duration: 47m 23s    Events: 1,247            │
│                                                                  │
│ ┌─ Session Timeline ───────────────────────────────────────────┐ │
│ │  ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐    │ │
│ │  │████│░░░░│████│████│░░░░│████│░░░░│░░░░│████│████│▓▓▓▓│    │ │
│ │  └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘    │ │
│ │   0m   5m   10m  15m  20m  25m  30m  35m  40m  45m  now      │ │
│ │                                                               │ │
│ │  Markers:                                                     │ │
│ │    ▼ Error at 12m (Permission denied)                         │ │
│ │    ◉ Learning at 23m (Captured error handling preference)     │ │
│ │    ★ Checkpoint at 30m (Medium assessment triggered)          │ │
│ └───────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

### Dashboard

```
┌─ vibes › Dashboard ─────────────────────── ● connected ── 14:32 ─┐
│                                                                   │
│ ┌─ Sessions ────┐ ┌─ Events ────────┐ ┌─ groove ────────────────┐ │
│ │               │ │                 │ │                         │ │
│ │      3        │ │     1.2k        │ │  ◉ learning             │ │
│ │    active     │ │    per hour     │ │                         │ │
│ │               │ │  ▁▂▃▅▇█▇▅▃▂▁▁▂  │ │  Learnings:  47         │ │
│ │  12 today     │ │                 │ │  Confidence: ████████░░ │ │
│ │  89 this week │ │  ▲ +23% vs avg  │ │  Circuit:    ● CLOSED   │ │
│ └───────────────┘ └─────────────────┘ └─────────────────────────┘ │
│                                                                   │
│ ┌─ Active Sessions ───────────────────────────────────────────┐  │
│ │ _  auth-refactor     ACTIVE    47m    ████████░░  1.2k evts │  │
│ │ _  fix-tests         IDLE      12m    ██░░░░░░░░  89 events │  │
│ │ _  docs-update       WAITING   2m     ███░░░░░░░  234 evts  │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ ┌─ Recent Errors ─────────────────┐ ┌─ System Health ───────────┐│
│ │ 14:12  sess-abc  Permission...  │ │ EventLog:  ● connected    ││
│ │ 13:45  sess-def  Tool timeout   │ │ Iggy:      ● running      ││
│ │ 12:30  sess-abc  Read failed    │ │ Consumers: 3/3 healthy    ││
│ └─────────────────────────────────┘ └───────────────────────────┘│
└───────────────────────────────────────────────────────────────────┘
```

### Debug View (Forensics)

```
┌─ vibes › Debug ─────────────────────────────────────── 14:32 ───┐
│                                                                  │
│ ┌─ Event Inspector ────────────────────────────────────────────┐ │
│ │                                                               │ │
│ │  Event ID:    evt-7f3a8b2c-1234-5678-9abc-def012345678       │ │
│ │  Timestamp:   2024-12-31T14:32:03.012Z                       │ │
│ │  Type:        ERROR                                          │ │
│ │  Session:     sess-abc (auth-refactor)                       │ │
│ │  Offset:      1,247                                          │ │
│ │                                                               │ │
│ │  ┌─ Payload ───────────────────────────────────────────────┐ │ │
│ │  │ {                                                        │ │ │
│ │  │   "error": "PermissionDenied",                           │ │ │
│ │  │   "path": "/etc/passwd",                                 │ │ │
│ │  │   "operation": "read"                                    │ │ │
│ │  │ }                                                        │ │ │
│ │  └──────────────────────────────────────────────────────────┘ │ │
│ │                                                               │ │
│ │  ┌─ Context (events before/after) ─────────────────────────┐ │ │
│ │  │ -2  14:32:02.901  TOOL     Read src/auth.rs             │ │ │
│ │  │ -1  14:32:02.998  CLAUDE   "Now let me check..."        │ │ │
│ │  │ ►0  14:32:03.012  ERROR    Permission denied ◄──────────│ │ │
│ │  │ +1  14:32:03.234  CLAUDE   "I see there was an error"   │ │ │
│ │  └──────────────────────────────────────────────────────────┘ │ │
│ └───────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  [ ◀ Prev Error ]  [ Next Error ▶ ]  [ Copy JSON ]  [ Export ]   │
└──────────────────────────────────────────────────────────────────┘
```

---

## groove Plugin Views

### groove Dashboard

```
┌─ vibes › groove ─────────────────────────── ◉ learning ── 14:32 ─┐
│                                                                   │
│  ◉ groove: You're in the groove. 47 learnings applied today.     │
│                                                                   │
│ ┌─ Status ──────────────────────────────────────────────────────┐│
│ │                                                                ││
│ │  Scope       Learnings   Confidence    Activity                ││
│ │  ─────────   ─────────   ────────────  ────────────────────    ││
│ │  Project     12          ████████░░    ▁▂▃▅▇█  2h ago          ││
│ │  User        47          █████████░    ▂▃▄▅▆▇  2h ago          ││
│ │  System      3           ██████░░░░    ▁▁▁▂▂▃  1w ago          ││
│ │                                                                ││
│ │  Total: 62 learnings across 3 scopes                           ││
│ └────────────────────────────────────────────────────────────────┘│
│                                                                   │
│ ┌─ Circuit Breaker ─────────┐ ┌─ Assessment Activity ───────────┐│
│ │                           │ │                                  ││
│ │  Status:  ● CLOSED        │ │  Lightweight   ████████ 127 OK  ││
│ │           (healthy)       │ │  Medium        ██░░░░░░   8     ││
│ │                           │ │  Heavy         ░░░░░░░░   0     ││
│ │  Last trip: 3 days ago    │ │                                  ││
│ │  Reason: High error rate  │ │  Last 24h   ▁▂▃▂▃▅▇█▇▅▃▂▁▂▃▄   ││
│ └───────────────────────────┘ └──────────────────────────────────┘│
│                                                                   │
│ ┌─ Recent Insights ─────────────────────────────────────────────┐│
│ │                                                                ││
│ │  ◉ Prefers explicit error types over anyhow in library code   ││
│ │    Confidence: ████████░░ 87%   Scope: Project   2h ago        ││
│ │                                                                ││
│ │  ◉ Uses cargo-nextest for testing                              ││
│ │    Confidence: █████████░ 94%   Scope: User      1d ago        ││
│ │                                                                ││
│ │                                    [ View All Learnings → ]    ││
│ └────────────────────────────────────────────────────────────────┘│
└───────────────────────────────────────────────────────────────────┘
```

### Learnings Browser

```
┌─ vibes › groove › Learnings ─────────────────────────── 14:32 ───┐
│                                                                   │
│ ┌─ Filter ────────────────────────────────────────────────────┐  │
│ │ Scope: [All ▾]  Confidence: [All ▾]  Search: [___________]  │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ ┌─ Learnings ─────────────────────────────────────────────────┐  │
│ │ Cmd  ID   Learning                          Conf    Scope    │  │
│ │ ───  ───  ────────────────────────────────  ──────  ──────── │  │
│ │  _   47   Prefers explicit error types...   ████░░  Project  │  │
│ │  _   46   Uses cargo-nextest for testing    █████░  User     │  │
│ │  _   45   Avoids unwrap() in production     ████░░  User     │  │
│ │  _   44   Prefers match over if-let chains  ███░░░  Project  │  │
│ │  ...                                                          │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ Line commands: s=select  f=forget  e=edit  v=view detail          │
├───────────────────────────────────────────────────────────────────┤
│ Command: _                                          47 learnings  │
└───────────────────────────────────────────────────────────────────┘
```

### Learning Detail

```
┌─ vibes › groove › Learning #47 ──────────────────────── 14:32 ───┐
│                                                                   │
│  ◉ Prefers explicit error types over anyhow in library code      │
│                                                                   │
│ ┌─ Details ───────────────────────────────────────────────────┐  │
│ │                                                              │  │
│ │  ID:          47                                             │  │
│ │  Scope:       Project (vibes)                                │  │
│ │  Confidence:  ████████░░  87%                                │  │
│ │  Created:     2024-12-30 14:23:01                            │  │
│ │  Last used:   2024-12-31 12:45:00 (2h ago)                   │  │
│ │  Times used:  12                                             │  │
│ │  Trust:       ● Verified (Local)                             │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ ┌─ Evidence ──────────────────────────────────────────────────┐  │
│ │                                                              │  │
│ │  Session: auth-refactor (2024-12-30)                         │  │
│ │  You corrected: "use anyhow::Result" → "use thiserror"       │  │
│ │  Pattern observed 3 times in this session.                   │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ ┌─ Attribution ───────────────────────────────────────────────┐  │
│ │                                                              │  │
│ │  This learning has contributed to:                           │  │
│ │    • 8 sessions with improved error handling                 │  │
│ │    • Estimated 12 corrections avoided                        │  │
│ │    • Attribution score: ████████░░ 0.84                      │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  [ Edit ]  [ Forget ]  [ Export ]                    [ ◀ Back ]   │
└───────────────────────────────────────────────────────────────────┘
```

### Assessment Stream

```
┌─ vibes › groove › Assess ────────────────── ◉ monitoring ─ 14:32 ┐
│                                                                   │
│ ┌─ Assessment Feed ───────────────────────────────────────────┐  │
│ │                                                              │  │
│ │  TIME         LEVEL        SESSION       RESULT              │  │
│ │  ───────────  ───────────  ────────────  ─────────────────── │  │
│ │  14:32:03.01  Lightweight  sess-abc      ● Patterns OK       │  │
│ │  14:32:02.45  Lightweight  sess-abc      ● No issues         │  │
│ │  14:31:58.12  Medium       sess-abc      ◉ Learning captured │  │
│ │  14:31:45.00  Lightweight  sess-abc      ● Patterns OK       │  │
│ │  14:30:12.34  Lightweight  sess-def      ⚠ Correction noted  │  │
│ │  14:28:00.00  Heavy        sess-def      ★ Checkpoint saved  │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                   │
│ ┌─ Legend ────────────────────────────────────────────────────┐  │
│ │  Lightweight  = Pattern check (every few seconds)            │  │
│ │  Medium       = Deeper analysis (on notable events)          │  │
│ │  Heavy        = Full checkpoint (session milestones)         │  │
│ │                                                              │  │
│ │  ● OK         ⚠ Noted         ◉ Learning         ★ Saved    │  │
│ └──────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
```

---

## Accessibility

### Color

- All semantic colors meet WCAG AA contrast (4.5:1 minimum)
- Never rely on color alone—always pair with icon/text/pattern
- High contrast mode available (boosts to WCAG AAA)

### Keyboard

- 100% keyboard navigable—no mouse required
- Visible focus indicators (amber outline + glow)
- Skip links for main content areas
- Focus trap in modals

### Screen Readers

- Semantic HTML (proper headings, landmarks, roles)
- ARIA labels for icons and status indicators
- Live regions for streaming content (polite announcements)
- Status changes announced (session attached, error occurred)

### Motion

- Respects `prefers-reduced-motion`
- Glow/pulse effects disabled when reduced motion preferred
- Essential animations only (no decorative motion)

### Text

- Scales with browser zoom (rem-based sizing)
- User can override font family in settings
- Minimum touch target: 44x44px for interactive elements

---

## User Preferences

### Appearance Settings

```
Theme
───────────────────────────────────────────────────────────────
( ) System (follow OS preference)
(●) Dark (the warm terminal - default)
( ) Light (warm cream background)
( ) High contrast (accessibility mode)

Font
───────────────────────────────────────────────────────────────
Family:  [JetBrains Mono ▾]  (or system monospace)
Size:    [13px ▾]  (11-18px range)

Density
───────────────────────────────────────────────────────────────
( ) Compact (more info, less spacing)
(●) Comfortable (default)
( ) Spacious (more breathing room)

Effects
───────────────────────────────────────────────────────────────
[x] Phosphor glow on focus
[x] Subtle scanline texture
[ ] CRT flicker effect (nostalgia mode)
[x] Animate streaming events

Keyboard
───────────────────────────────────────────────────────────────
[x] Vim-style navigation (j/k/g/G)
[x] Show function key bar
[x] Enable line commands
```

---

## Light Theme

For those who prefer light mode:

### Light Theme Palette

| Token | Hex | Description |
|-------|-----|-------------|
| `bg-base` | `#faf7f2` | Warm cream base |
| `bg-surface` | `#f0ebe3` | Slightly darker for cards |
| `bg-elevated` | `#e6e0d6` | Hover states |
| `bg-overlay` | `#dcd5c9` | Modals |
| `text-primary` | `#1a1816` | Near-black, warm |
| `text-secondary` | `#4a4640` | Medium gray |
| `text-muted` | `#8a8478` | Light gray |

Semantic colors adjusted for light background:
| Token | Hex |
|-------|-----|
| `amber` | `#b8860b` |
| `green` | `#2d7a4a` |
| `red` | `#c23030` |
| `blue` | `#3a6fa5` |

Same warm feeling, just inverted. Cream instead of charcoal.

---

## Related Documents

- [groove Branding Guide](../groove/BRANDING.md) - Voice, personality, messaging
- [Continual Learning Design](../plans/14-continual-learning/design.md) - Technical architecture
- [PRD](../PRD.md) - Overall product requirements

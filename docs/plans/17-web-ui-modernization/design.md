# Web UI Modernization Design

> **Phase 4.5** - Full redesign of vibes web UI using the visual system with Iggy stream views.

## Overview

Transform the vibes web UI from the current GitHub-inspired cold theme to the warm terminal aesthetic defined in [VISUAL-SYSTEM.md](../../design/VISUAL-SYSTEM.md). Add real-time Iggy event stream views (Firehose, Debug) and establish a reusable component library with Ladle as the styleguide.

## Goals

1. Apply warm terminal aesthetic across all pages
2. Create `@vibes/design-system` package for reusable components
3. Document components in Ladle styleguide
4. Add Firehose and Debug views for Iggy event streams
5. Enable theme switching (dark/light)

## Package Structure

```
vibes/
├── design-system/              # New: shared component library
│   ├── package.json            # @vibes/design-system
│   ├── src/
│   │   ├── tokens/
│   │   │   ├── colors.css      # Color variables (dark + light)
│   │   │   ├── typography.css  # Font stacks, sizes, weights
│   │   │   ├── spacing.css     # Spacing scale
│   │   │   └── index.css       # Aggregates all tokens
│   │   ├── primitives/
│   │   │   ├── Button/
│   │   │   │   ├── Button.tsx
│   │   │   │   ├── Button.module.css
│   │   │   │   └── Button.stories.tsx
│   │   │   ├── Badge/
│   │   │   ├── Panel/
│   │   │   └── ...
│   │   ├── compositions/
│   │   │   ├── SessionCard/
│   │   │   ├── StreamView/
│   │   │   └── ...
│   │   └── index.ts            # Public exports
│   ├── .ladle/
│   │   └── config.mjs          # Ladle configuration
│   └── vite.config.ts
│
├── web-ui/                     # Existing: consumes design-system
│   ├── package.json            # depends on @vibes/design-system
│   └── src/
│       ├── pages/              # Page components (use design-system)
│       └── ...
```

The design-system is a workspace package that web-ui imports. Ladle runs from design-system for isolated component development.

## Design Tokens

Tokens translate VISUAL-SYSTEM.md into CSS variables:

```css
/* tokens/colors.css */
:root {
  /* Base palette */
  --color-bg-base: #1a1816;
  --color-bg-surface: #242220;
  --color-bg-elevated: #2e2c29;

  /* Text */
  --color-text-primary: #f0ebe3;
  --color-text-secondary: #a39e93;
  --color-text-dim: #6b665c;

  /* Semantic */
  --color-accent: #e6b450;        /* vibes amber */
  --color-success: #7ec699;
  --color-warning: #e6b450;
  --color-error: #e05252;
  --color-info: #7eb8c9;

  /* Plugin accents */
  --color-groove: #c9a227;        /* groove gold */

  /* Borders */
  --color-border: #3d3a36;
  --color-border-subtle: #2e2c29;

  /* Glow effects */
  --glow-amber: 0 0 20px rgba(230, 180, 80, 0.15);
}

/* Light theme override */
[data-theme="light"] {
  --color-bg-base: #f5f2ed;
  --color-bg-surface: #ffffff;
  --color-bg-elevated: #f9f7f4;
  --color-text-primary: #1a1816;
  --color-text-secondary: #4a4640;
  --color-text-dim: #6b665c;
  --color-border: #d4d0c8;
  --color-border-subtle: #e8e4dc;
}
```

Theme switching via `data-theme` attribute on `<html>`.

## Primitive Components

Foundational building blocks with Ladle stories:

### Button
```tsx
<Button variant="primary">Action</Button>    // Amber fill
<Button variant="secondary">Cancel</Button>  // Border only
<Button variant="ghost">Menu</Button>        // Text only
```

### Badge
```tsx
<Badge status="success">Connected</Badge>
<Badge status="warning">Processing</Badge>
<Badge status="error">Failed</Badge>
<Badge status="idle">Idle</Badge>
```

### Panel
```tsx
<Panel title="Session" variant="default">...</Panel>
<Panel title="Events" variant="elevated">...</Panel>
<Panel variant="inset">...</Panel>
```

### StatusIndicator
```tsx
<StatusIndicator state="live" />      // Pulsing green
<StatusIndicator state="paused" />    // Static amber
<StatusIndicator state="offline" />   // Dim gray
```

### Text
```tsx
<Text intensity="high">Important</Text>
<Text intensity="normal">Body text</Text>
<Text intensity="dim">Metadata</Text>
<Text mono>code_value</Text>
```

## Composition Components

Built from primitives:

### SessionCard
```tsx
<SessionCard
  id="sess-abc123"
  name="auth-refactor"
  status="processing"
  subscribers={2}
  updatedAt={timestamp}
  onClick={...}
/>
```

### Terminal
xterm.js wrapper with vibes styling (warm charcoal, amber cursor, phosphor glow).

### StreamView
Real-time event display (core of Firehose/Debug):
```tsx
<StreamView
  events={events}
  filter={filterFn}
  onEventClick={inspectEvent}
  paused={isPaused}
/>
```

### EventInspector
```tsx
<EventInspector event={selectedEvent} onClose={...} />
```

### Header
```tsx
<Header
  identity={user}
  tunnelStatus="connected"
  theme={theme}
  onThemeToggle={...}
/>
```

### CommandPalette
```tsx
<CommandPalette
  commands={availableCommands}
  onSelect={executeCommand}
/>
```

## Iggy Stream Views

### Firehose View (`/streams/firehose`)

Full-screen real-time event stream:

```
┌─ Firehose ──────────────────────────────────────────────┐
│ ● LIVE  │ 127 events/sec │ ▶ Pause │ 🔍 Filter │ ⚙     │
├─────────────────────────────────────────────────────────┤
│ 14:23:01.234  SessionCreated     sess-abc   auth-work  │
│ 14:23:01.456  Hook               sess-abc   SessionSt… │
│ 14:23:01.789  PtyOutput          sess-abc   [42 bytes] │
│ 14:23:02.012  Claude.TextDelta   sess-abc   "I'll hel… │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└─────────────────────────────────────────────────────────┘
```

Features:
- Virtualized scrolling (handles thousands of events)
- Color-coded by event type
- Click to inspect in side panel
- Pause/resume without losing events (buffered)
- Filter by event type, session, text search

### Debug View (`/streams/debug`)

Split-pane inspector for troubleshooting:

```
┌─ Debug ─────────────────────────┬─ Inspector ───────────┐
│ Filter: [session:abc] [type:*] │ Event: Claude.TextDe… │
├─────────────────────────────────┤                       │
│ ▸ 14:23:01 SessionCreated      │ {                     │
│ ▸ 14:23:01 Hook (SessionStart) │   "session_id": "…",  │
│ ▾ 14:23:02 Claude.TextDelta ◀──│   "event": {          │
│ ▸ 14:23:02 Claude.ToolUse      │     "text": "I'll…"   │
│ ▸ 14:23:03 PtyOutput           │   }                   │
│                                 │ }                     │
│                                 │                       │
│                                 │ [Copy] [Related]      │
└─────────────────────────────────┴───────────────────────┘
```

Features:
- Persistent filters (saved to localStorage)
- Collapsible event groups
- JSON syntax highlighting
- "Related events" links
- Export filtered events to JSON

## Page Structure

```
/                       # Home - warm welcome, quick actions
/sessions               # Session list (renamed from /claude)
/sessions/:id           # Session detail with Terminal
/streams                # Stream dashboard (links to views)
/streams/firehose       # Real-time firehose
/streams/debug          # Debug inspector
/groove                 # groove dashboard
/groove/learnings       # Learning browser
/groove/quarantine      # Quarantine management
/settings               # App settings, theme toggle, notifications
```

Navigation:
```
┌─────────────────────────────────────────────────────────┐
│ ◈ vibes    Sessions  Streams  ◉ groove    [🔔] [☀/🌙] │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Session Terminal (existing, unchanged)
```
/sessions/:id page
    └── Terminal component
          └── /ws/pty/:sessionId
                ├── PTY I/O (stdin/stdout)
                └── Session-scoped events
```

### Firehose (new)
```
/streams/firehose page
    └── StreamView component
          └── /ws/firehose
                └── All events across all sessions
```

### Server Endpoint

New endpoint: `GET /ws/firehose`
- Broadcasts all `VibesEvent` from the EventLog
- Optional query params: `?types=Claude,Hook&session=abc`
- Server-side filtering reduces bandwidth

### Hook

```tsx
const { events, isConnected, isPaused, pause, resume, clear } = useFirehose({
  filter: { types: ['Claude', 'Hook'], session: 'abc' },
  bufferSize: 1000,
});
```

Performance:
- Ring buffer for fixed memory
- Virtualized rendering
- Pause stops view updates but keeps buffering
- Debounced filter updates

## Implementation Phases

### Phase 1: Foundation
1. Set up `design-system/` workspace package with Vite + Ladle
2. Create `tokens/` - colors, typography, spacing
3. Build primitives: Button, Badge, Panel, Text, StatusIndicator
4. Ladle stories for each primitive

### Phase 2: Compositions
1. Build SessionCard, Header compositions
2. Build StreamView (virtualized event list)
3. Build EventInspector (JSON tree view)
4. Ladle stories for compositions

### Phase 3: Server Integration
1. Add `/ws/firehose` endpoint to vibes-server
2. Implement `useFirehose()` hook in web-ui
3. Wire up filtering and buffering

### Phase 4: Migrate web-ui
1. Update web-ui to import from `@vibes/design-system`
2. Replace `index.css` with token imports
3. Refactor existing pages to use new components
4. Update routes (`/claude` → `/sessions`)

### Phase 5: New Pages
1. Build `/streams` dashboard
2. Build `/streams/firehose` page
3. Build `/streams/debug` page
4. Add theme toggle to settings

### Phase 6: Polish
1. CommandPalette (Cmd+K)
2. Keyboard navigation
3. Responsive breakpoints
4. Light theme refinement

## Testing Strategy

| Layer | Approach |
|-------|----------|
| Primitives | Ladle visual stories + snapshot tests |
| Compositions | Ladle stories + interaction tests |
| Hooks | Unit tests with mock WebSocket |
| Pages | Playwright E2E |
| Performance | Lighthouse audit, 1000+ events test |

## Success Criteria

1. All primitives/compositions documented in Ladle
2. Every VISUAL-SYSTEM.md token has a CSS variable
3. No cold blue (#58a6ff) remains in UI
4. Theme switching works via toggle
5. Firehose: real-time events, pause, filter, inspect
6. Debug: filter events, view JSON, copy
7. Performance: 1000 events at 60fps
8. Responsive: 768px through ultrawide

## Out of Scope

- Session timeline visualization
- Replay functionality
- Full groove dashboard (beyond quarantine)
- CommandPalette (Phase 6 nice-to-have)

## Related Documents

- [VISUAL-SYSTEM.md](../../design/VISUAL-SYSTEM.md) - Design tokens and visual language
- [groove BRANDING.md](../../groove/BRANDING.md) - groove plugin branding
- [PRD.md](../../PRD.md) - Product requirements

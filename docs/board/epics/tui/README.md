---
id: tui
title: Terminal User Interface
status: planned
description: Interactive TUI for controlling agents, approving permissions, viewing output - embeddable in web via PTY
---

# Terminal User Interface

Interactive terminal interface (lazygit-style) for controlling agents, approving permissions, and viewing output. Embeddable in Web UI via PTY.

## Overview

A full TUI application using ratatui that provides:

- Real-time agent control and monitoring
- Permission approval interface
- Session management
- Swarm visualization
- Observability dashboard

Key feature: PTY server allows embedding the TUI in the web UI via xterm.js.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    vibes-tui                         │
├─────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────┐   │
│  │                   App                         │   │
│  │  ┌────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │ State  │  │  Views  │  │  KeyBindings │   │   │
│  │  └────────┘  └─────────┘  └──────────────┘   │   │
│  └──────────────────────────────────────────────┘   │
│                        │                             │
│  ┌─────────────────────▼─────────────────────────┐  │
│  │              VibesClient (WebSocket)           │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │   vibes-server   │
              └──────────────────┘
```

## App Structure

```rust
pub struct App {
    pub state: AppState,
    pub views: ViewStack,
    pub keybindings: KeyBindings,
    pub theme: Theme,
    pub client: VibesClient,
}

pub struct AppState {
    pub session: Option<SessionId>,
    pub agents: HashMap<AgentId, AgentState>,
    pub swarms: HashMap<SwarmId, SwarmState>,
    pub selected: Selection,
    pub mode: Mode,
}

pub enum Mode {
    Normal,
    Command,
    Search,
    Help,
}
```

## Views

```rust
pub enum View {
    Dashboard,              // Overview of all activity
    Session(SessionId),     // Single session detail
    Agent(AgentId),         // Single agent detail
    Swarm(SwarmId),         // Swarm visualization
    Models,                 // Model registry
    Observe,                // Observability dashboard
    Evals,                  // Evaluation results
    Settings,               // Configuration
}

pub struct ViewStack {
    pub current: View,
    pub history: Vec<View>,
}
```

### Dashboard View

```
┌─ vibes ─────────────────────────────────────────────┐
│ Sessions: 3 active   Agents: 7 running   Cost: $12  │
├─────────────────────────────────────────────────────┤
│ ● session-abc   2 agents   feature/auth   Running   │
│   session-def   1 agent    bugfix/leak    Paused    │
│   session-ghi   4 agents   swarm          Active    │
├─────────────────────────────────────────────────────┤
│ Recent Activity                                      │
│ 14:32 agent-1 completed task "implement login"      │
│ 14:31 agent-2 waiting for permission                │
│ 14:30 swarm-1 started parallel execution            │
├─────────────────────────────────────────────────────┤
│ [j/k] Navigate  [Enter] Select  [n] New  [?] Help   │
└─────────────────────────────────────────────────────┘
```

### Agent View

```
┌─ Agent: agent-1 ────────────────────────────────────┐
│ Session: session-abc   Model: claude-sonnet-4-20250514       │
│ Status: Running   Task: implement login flow        │
├──────────────────────────┬──────────────────────────┤
│ Output                   │ Context                  │
│                          │                          │
│ > Analyzing codebase...  │ Files: 12                │
│ > Found auth module      │ Tokens: 45,231           │
│ > Implementing login     │ Tools: 8 calls           │
│   handler...             │ Duration: 4m 32s         │
│                          │                          │
├──────────────────────────┴──────────────────────────┤
│ ⚠ Permission Request: Write to src/auth/login.rs    │
│ [y] Approve  [n] Deny  [v] View diff  [e] Edit      │
├─────────────────────────────────────────────────────┤
│ [p] Pause  [c] Cancel  [r] Restart  [Esc] Back      │
└─────────────────────────────────────────────────────┘
```

### Swarm View

```
┌─ Swarm: code-review ────────────────────────────────┐
│ Strategy: Parallel   Status: Running                │
│ Task: Review PR #123                                │
├─────────────────────────────────────────────────────┤
│                                                     │
│     ┌──────────┐                                    │
│     │ agent-1  │ ──── Security review (45%)        │
│     │ ████░░░░ │                                    │
│     └──────────┘                                    │
│                                                     │
│     ┌──────────┐                                    │
│     │ agent-2  │ ──── Performance review (72%)     │
│     │ ██████░░ │                                    │
│     └──────────┘                                    │
│                                                     │
│     ┌──────────┐                                    │
│     │ agent-3  │ ──── Code style review (100%)     │
│     │ ████████ │ ✓                                  │
│     └──────────┘                                    │
│                                                     │
├─────────────────────────────────────────────────────┤
│ [Enter] Agent detail  [m] Merge results  [Esc] Back│
└─────────────────────────────────────────────────────┘
```

## Keybindings

```rust
pub struct KeyBindings {
    pub global: HashMap<KeyEvent, Action>,
    pub view_specific: HashMap<View, HashMap<KeyEvent, Action>>,
}

// Global
pub const GLOBAL_KEYS: &[(&str, &str)] = &[
    ("j/k", "Navigate down/up"),
    ("h/l", "Navigate left/right"),
    ("Enter", "Select/Confirm"),
    ("Esc", "Back/Cancel"),
    (":", "Command mode"),
    ("/", "Search"),
    ("?", "Help"),
    ("q", "Quit"),
    ("1-9", "Jump to view"),
];

// Context-specific
pub const AGENT_KEYS: &[(&str, &str)] = &[
    ("y", "Approve permission"),
    ("n", "Deny permission"),
    ("p", "Pause agent"),
    ("r", "Resume agent"),
    ("c", "Cancel task"),
    ("v", "View diff"),
];
```

## Theme System

```rust
pub struct Theme {
    pub name: String,

    // Colors
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Status colors
    pub running: Color,
    pub paused: Color,
    pub completed: Color,
    pub failed: Color,

    // UI elements
    pub border: Color,
    pub selection: Color,
    pub highlight: Color,

    // Text styles
    pub bold: Style,
    pub dim: Style,
    pub italic: Style,
}

// CRT-inspired default theme
pub fn vibes_default() -> Theme {
    Theme {
        name: "vibes".into(),
        bg: Color::Rgb(18, 18, 18),
        fg: Color::Rgb(0, 255, 136),        // Phosphor green
        accent: Color::Rgb(0, 200, 255),    // Cyan accent
        success: Color::Rgb(0, 255, 136),
        warning: Color::Rgb(255, 200, 0),
        error: Color::Rgb(255, 85, 85),
        running: Color::Rgb(0, 255, 136),
        paused: Color::Rgb(255, 200, 0),
        completed: Color::Rgb(100, 100, 100),
        failed: Color::Rgb(255, 85, 85),
        border: Color::Rgb(60, 60, 60),
        selection: Color::Rgb(40, 80, 40),
        highlight: Color::Rgb(0, 150, 100),
        // ...
    }
}
```

## PTY Server for Web Embedding

```rust
pub struct PtyServer {
    pub socket_path: PathBuf,
    pub sessions: HashMap<ConnectionId, PtySession>,
}

pub struct PtySession {
    pub pty: Pty,
    pub app: App,
    pub size: (u16, u16),
}

impl PtyServer {
    pub async fn accept(&mut self, conn: TcpStream) -> Result<ConnectionId>;
    pub async fn resize(&mut self, conn: ConnectionId, cols: u16, rows: u16) -> Result<()>;
    pub async fn input(&mut self, conn: ConnectionId, data: &[u8]) -> Result<()>;
    pub async fn output(&mut self, conn: ConnectionId) -> Result<Vec<u8>>;
}
```

### Web Integration

```typescript
// In web-ui using xterm.js
const terminal = new Terminal();
const socket = new WebSocket('/api/tui/pty');

socket.onmessage = (event) => {
    terminal.write(event.data);
};

terminal.onData((data) => {
    socket.send(data);
});
```

## CLI Commands

```
vibes tui                             # Launch TUI
vibes tui --theme <name>              # Use specific theme
vibes tui --session <id>              # Start in session view
vibes tui --agent <id>                # Start in agent view

# PTY server (for web embedding)
vibes tui serve                       # Start PTY server
vibes tui serve --port 8081           # Custom port
```

<!-- BEGIN GENERATED -->
## Milestones

**Progress:** 3/6 milestones complete, 21/25 stories done
**Active:** Customizable Themes

| ID | Milestone | Stories | Status |
|----|-----------|---------|--------|
| 41 | [Terminal UI Framework](milestones/41-terminal-ui-framework/) | 5/5 | done |
| 42 | [Terminal Dashboard](milestones/42-terminal-dashboard/) | 4/4 | done |
| 43 | [Terminal Agent Control](milestones/43-terminal-agent-control/) | 4/4 | done |
| 44 | [Swarm Monitoring](milestones/44-swarm-monitoring/) | 4/4 | planned |
| 45 | [Customizable Themes](milestones/45-customizable-themes/) | 4/4 | in-progress |
| 46 | [Terminal Server](milestones/46-terminal-server/) | 0/4 | planned |
<!-- END GENERATED -->

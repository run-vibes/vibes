---
id: FEAT0170
title: App struct and event loop
type: feat
status: done
priority: high
scope: tui/41-terminal-ui-framework
depends: [m41-feat-01]
estimate: 4h
---

# App Struct and Event Loop

## Summary

Establish the core application structure and main loop that drives the TUI. This is the heartbeat of the application - all subsequent features build on this foundation.

## Features

### App Struct

```rust
pub struct App {
    pub state: AppState,
    pub views: ViewStack,        // Placeholder until Story 3
    pub keybindings: KeyBindings, // Placeholder until Story 4
    pub theme: Theme,
    pub client: Option<VibesClient>, // Placeholder until Story 5
    pub running: bool,
}
```

### AppState

```rust
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

pub enum Selection {
    None,
    Session(usize),
    Agent(usize),
    Swarm(usize),
}
```

### Event Loop

```rust
pub async fn run(&mut self) -> Result<()> {
    let mut terminal = setup_terminal()?;

    loop {
        // Render
        terminal.draw(|f| self.render(f))?;

        // Handle input with timeout for tick
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm::event::read()? {
                self.handle_key(key);
            }
        }

        // Check exit condition
        if !self.running {
            break;
        }

        // Tick: process async updates (WebSocket messages, etc.)
        self.tick().await;
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
```

### Terminal Setup/Teardown

```rust
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<...>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
```

## Implementation

1. Create `src/app.rs` with App struct
2. Create `src/state.rs` with AppState, Mode, Selection
3. Create `src/terminal.rs` with setup/teardown functions
4. Implement basic event loop with tick + input
5. Add placeholder render that just shows "vibes TUI" text
6. Hardcode `q` to quit (proper keybindings in Story 4)
7. Add panic hook to restore terminal on crash

## Acceptance Criteria

- [ ] App struct created with all fields
- [ ] AppState with Mode and Selection enums
- [ ] Event loop runs and handles keyboard input
- [ ] Terminal enters alternate screen on start
- [ ] Terminal restored cleanly on exit (no garbage)
- [ ] `q` key quits the application
- [ ] Panic hook restores terminal on crash
- [ ] Basic "vibes TUI" placeholder renders

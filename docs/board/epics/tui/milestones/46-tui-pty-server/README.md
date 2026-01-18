---
id: 46-tui-pty-server
title: Terminal Server
status: planned
epics: [tui]
---

# Terminal Server

## Overview

Sixth milestone of the TUI epic. Implements PTY server for embedding the TUI in the web UI via xterm.js.

## Goals

- PTY server with socket connections
- Session management for multiple connections
- Resize handling
- Web UI integration via xterm.js
- CLI command for starting PTY server

## Key Deliverables

- `PtyServer` implementation
- `PtySession` management
- WebSocket PTY endpoint
- xterm.js integration in web-ui
- `vibes tui serve` command

## Architecture

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

## Web Integration

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

## Epics

- [tui](../../epics/tui)

## Stories

| ID | Title | Status |
|----|-------|--------|
| m46-feat-01 | PTY server core | backlog |
| m46-feat-02 | Session management | backlog |
| m46-feat-03 | WebSocket PTY endpoint | backlog |
| m46-feat-04 | xterm.js web integration | backlog |

## Design

See [../../epics/tui/README.md](../../epics/tui/README.md) for architecture.

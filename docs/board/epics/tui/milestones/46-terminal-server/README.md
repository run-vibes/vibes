---
id: 46-terminal-server
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

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0116](../../../../stages/backlog/stories/[FEAT][0116]-pty-server-core.md) | PTY server core | backlog |
| 2 | [FEAT0117](../../../../stages/backlog/stories/[FEAT][0117]-session-management.md) | Session management | backlog |
| 3 | [FEAT0118](../../../../stages/backlog/stories/[FEAT][0118]-websocket-pty-endpoint.md) | WebSocket PTY endpoint | backlog |
| 4 | [FEAT0119](../../../../stages/backlog/stories/[FEAT][0119]-xtermjs-web-integration.md) | xterm.js web integration | backlog |

## Progress

**Requirements:** 0/0 verified
**Stories:** 0/4 complete


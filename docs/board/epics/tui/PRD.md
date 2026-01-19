# Terminal User Interface - Product Requirements

> Interactive TUI for controlling agents and approving permissions

## Problem Statement

While the CLI handles commands and the web UI provides visual monitoring, users need a full-screen terminal interface for real-time agent control. This TUI should feel like lazygit - immediate, keyboard-driven, and information-dense. A key innovation is PTY embedding, allowing the TUI to run inside the web UI via xterm.js.

## Users

- **Primary**: Developers who prefer terminal interfaces over browsers
- **Secondary**: Remote users accessing vibes through SSH
- **Tertiary**: Web UI users who want embedded terminal experience

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | Dashboard view showing active sessions and agents | must |
| FR-02 | Real-time agent output display | must |
| FR-03 | Permission approval interface (approve/deny/edit) | must |
| FR-04 | Session and agent navigation | must |
| FR-05 | Swarm visualization with agent coordination display | should |
| FR-06 | PTY server for web embedding via xterm.js | should |
| FR-07 | Customizable color themes | could |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | Vim-style keybindings (j/k navigation, etc.) | must |
| NFR-02 | CRT-inspired theme matching web UI aesthetic | should |
| NFR-03 | Responsive layout adapting to terminal size | should |

## Success Criteria

- [ ] Users can control agents entirely from TUI without switching to CLI
- [ ] Permission requests can be reviewed and approved in under 2 seconds
- [ ] TUI works embedded in web UI via PTY server
- [ ] Keyboard-only operation for all features

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 41 | [Terminal UI Framework](milestones/41-terminal-ui-framework/) | done |
| 42 | [Terminal Dashboard](milestones/42-terminal-dashboard/) | done |
| 43 | [Terminal Agent Control](milestones/43-terminal-agent-control/) | done |
| 44 | [Swarm Monitoring](milestones/44-swarm-monitoring/) | planned |
| 45 | [Customizable Themes](milestones/45-customizable-themes/) | in-progress |
| 46 | [Terminal Server](milestones/46-terminal-server/) | planned |

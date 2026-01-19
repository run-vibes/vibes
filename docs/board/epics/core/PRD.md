# Core Infrastructure - Product Requirements

> Foundation systems that power all vibes capabilities

## Problem Statement

vibes requires a robust foundation layer that handles the fundamentals: proxying requests, managing terminal sessions, storing events, and orchestrating sessions. Without a solid core, all other features become unreliable and difficult to build upon.

## Users

- **Primary**: All vibes components that depend on core infrastructure
- **Secondary**: Plugin developers who extend vibes functionality
- **Tertiary**: Operators who deploy and maintain vibes instances

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | Proxy server for routing requests to AI providers | must |
| FR-02 | PTY backend for terminal session management | must |
| FR-03 | Event bus for inter-component communication | must |
| FR-04 | Event-sourced storage layer using Apache Iggy | must |
| FR-05 | Session management with state tracking | must |
| FR-06 | Reliable test infrastructure | must |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | High availability for core services | must |
| NFR-02 | Sub-second latency for proxy operations | should |
| NFR-03 | Horizontal scalability for session management | could |

## Success Criteria

- [ ] Core services maintain 99.9% uptime
- [ ] Event storage provides durable, replayable history
- [ ] Test suite runs reliably without flaky failures
- [ ] All state changes captured as events

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 01 | [Remote Session Access](milestones/01-remote-session-access/) | done |
| 08 | [Persistent Conversations](milestones/08-persistent-conversations/) | done |
| 09 | [Parallel Workspaces](milestones/09-parallel-workspaces/) | done |
| 11 | [Reliable Test Suite](milestones/11-reliable-test-suite/) | done |
| 12 | [Full Terminal Emulation](milestones/12-full-terminal-emulation/) | done |
| 13 | [Efficient Scrollback](milestones/13-efficient-scrollback/) | done |
| 14 | [Visual Project Planning](milestones/14-visual-project-planning/) | done |
| 16 | [Bundled Event Store](milestones/16-bundled-event-store/) | done |
| 18 | [Native Event Storage](milestones/18-native-event-storage/) | done |
| 19 | [Event-Driven Architecture](milestones/19-event-driven-architecture/) | done |
| 28 | [Organized Project Tracking](milestones/28-organized-project-tracking/) | done |

---
id: m43-feat-02
title: Output stream panel
type: feat
status: backlog
priority: high
epics: [tui]
depends: [m43-feat-01]
estimate: 4h
milestone: 43-tui-agent-control
---

# Output Stream Panel

## Summary

Implement the real-time output streaming panel for the agent detail view. This panel displays the agent's output as it works, with proper scrolling, line wrapping, and visual formatting for different output types.

## Features

### Output Types

Different visual treatment for output categories:
- **Text output**: Standard agent messages
- **Tool calls**: Highlighted with tool name
- **Thinking**: Dimmed/italic style
- **Errors**: Red-tinted for visibility

### Scrolling Behavior

- Auto-scroll to bottom when new output arrives (if already at bottom)
- Manual scroll with j/k or arrow keys when in output focus
- Scroll position indicator
- Jump to bottom with `G`, jump to top with `g`

### Line Wrapping

- Wrap long lines to panel width
- Preserve indentation for wrapped continuation lines
- Handle ANSI escape codes (strip or interpret)

### Output Buffer

```rust
pub struct OutputBuffer {
    lines: Vec<OutputLine>,
    max_lines: usize,  // Ring buffer to prevent unbounded growth
    scroll_offset: usize,
    auto_scroll: bool,
}

pub struct OutputLine {
    timestamp: DateTime<Utc>,
    content: String,
    line_type: OutputLineType,
}

pub enum OutputLineType {
    Text,
    ToolCall { tool_name: String },
    Thinking,
    Error,
}
```

## Implementation

1. Create `src/widgets/output_panel.rs` with OutputBuffer
2. Implement ring buffer logic for bounded memory
3. Add scroll state management
4. Implement rendering with line type styling
5. Add WebSocket subscription for agent output events
6. Wire output panel into AgentView left column
7. Add keyboard handlers for scroll navigation
8. Handle auto-scroll toggle (user scroll disables, new output re-enables if at bottom)

## Acceptance Criteria

- [ ] Output panel renders in left column of AgentView
- [ ] New output lines appear in real-time via WebSocket
- [ ] j/k keys scroll output when panel focused
- [ ] Auto-scroll enabled when at bottom
- [ ] Manual scroll disables auto-scroll
- [ ] Different output types have distinct styling
- [ ] Ring buffer prevents unbounded memory growth
- [ ] Long lines wrap correctly within panel width
- [ ] Scroll position indicator shows location in buffer

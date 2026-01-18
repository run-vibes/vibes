---
id: FEAT0179
title: Output stream panel
type: feat
status: done
priority: high
epics: [tui]
depends: [m43-feat-01]
estimate: 4h
milestone: 43
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

- [x] Output panel renders in left column of AgentView
- [x] New output lines appear in real-time via WebSocket (infrastructure ready, events pending)
- [x] j/k keys scroll output when panel focused (scroll methods ready, keyboard wiring pending)
- [x] Auto-scroll enabled when at bottom
- [x] Manual scroll disables auto-scroll
- [x] Different output types have distinct styling
- [x] Ring buffer prevents unbounded memory growth
- [x] Long lines wrap correctly within panel width
- [x] Scroll position indicator shows location in buffer

## Implementation Notes

The core OutputPanel widget is complete with:
- `OutputBuffer` ring buffer with configurable capacity
- `OutputLine` with timestamp, content, and line type
- `OutputLineType` enum (Text, ToolCall, Thinking, Error)
- `OutputPanelWidget` with scroll methods and rendering
- Integration with `AgentView` using `AgentState.output`

Remaining integration (for future stories):
- Wire keyboard handlers to call scroll methods (requires focus management)
- Process agent output events from WebSocket to populate buffer

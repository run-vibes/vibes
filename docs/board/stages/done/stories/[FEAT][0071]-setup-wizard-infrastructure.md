---
id: FEAT0071
title: Setup Wizard Infrastructure
type: feat
status: done
priority: medium
scope: networking
---

# Setup Wizard Infrastructure

Add dialoguer dependency and create setup module with reusable prompt helpers.

## Context

Milestone 35 requires interactive CLI wizards for tunnel and auth setup. This story establishes the foundation: the dialoguer library for interactive prompts and helper functions for consistent output formatting.

## Acceptance Criteria

- [x] Add `dialoguer = "0.11"` to vibes-cli/Cargo.toml
- [x] Create `vibes-cli/src/commands/setup/mod.rs` with module structure
- [x] Create `vibes-cli/src/commands/setup/prompts.rs` with:
  - `print_header(title)` - Draws boxed header
  - `print_step(message)` - Prints step with trailing space (no newline)
  - `print_success(message)` - Green checkmark + message
  - `print_error(message)` - Red X + message
- [x] Tests verify prompt output formatting
- [x] Export setup module from commands/mod.rs

## Technical Notes

```rust
// prompts.rs
use console::style;

pub fn print_header(title: &str) {
    let width = 60;
    let border = "─".repeat(width);
    println!("┌{}┐", border);
    println!("│ {:<width$} │", title, width = width - 2);
    println!("└{}┘\n", border);
}

pub fn print_success(message: &str) {
    println!("\n{} {}", style("✓").green().bold(), style(message).green());
}
```

## Size

S - Small (add dependency, create module structure)

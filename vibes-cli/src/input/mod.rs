//! Input handling for CLI
//!
//! Provides readline-like input with history support using crossterm.
//!
//! Note: With PTY mode, terminal-native input is used instead.
//! These modules are retained for potential non-PTY use cases.

mod history;
mod readline;

#[allow(unused_imports)]
pub use history::InputHistory;
#[allow(unused_imports)]
pub use readline::{Readline, ReadlineResult};

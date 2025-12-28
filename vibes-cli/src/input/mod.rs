//! Input handling for CLI
//!
//! Provides readline-like input with history support using crossterm.

mod history;
mod readline;

pub use history::InputHistory;
pub use readline::{Readline, ReadlineResult};

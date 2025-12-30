//! Capture pipeline for session events
//!
//! This module handles collecting, parsing, and extracting learnings
//! from Claude Code sessions.

mod collector;

pub use collector::{SessionBuffer, SessionCollector, ToolEvent};

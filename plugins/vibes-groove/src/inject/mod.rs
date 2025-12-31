//! Injection pipeline for learnings
//!
//! This module handles formatting learnings and injecting them
//! into Claude Code sessions via CLAUDE.md or hook responses.

mod formatter;
mod injector;

pub use formatter::{FormattedSection, LearningFormatter};
pub use injector::{ClaudeCodeInjector, InjectionMethod, InjectionResult};

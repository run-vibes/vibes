//! Capture pipeline for session events
//!
//! This module handles collecting, parsing, and extracting learnings
//! from Claude Code sessions.

mod collector;
mod extractor;
mod parser;

pub use collector::{SessionBuffer, SessionCollector, ToolEvent};
pub use extractor::{ExtractedLearning, LearningCategory, LearningExtractor};
pub use parser::{
    ParsedTranscript, TranscriptMessage, TranscriptMetadata, TranscriptParser, TranscriptToolUse,
};

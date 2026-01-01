//! Claude Code hooks integration
//!
//! This module provides structured data capture from Claude Code sessions
//! via the hooks system. Hook scripts send events directly to Iggy via CLI.
//!
//! ## Hook Types
//!
//! - **PreToolUse** - Called before a tool is executed
//! - **PostToolUse** - Called after a tool completes
//! - **Stop** - Called when Claude stops (provides transcript path)
//! - **SessionStart** - Called when a session starts
//! - **UserPromptSubmit** - Called when user submits a prompt
//!
//! ## Architecture
//!
//! ```text
//! Claude Code ---> Hook Script ---> vibes event send ---> Iggy HTTP API
//! ```

mod installer;
pub mod scripts;
mod types;

pub use installer::{HookInstaller, HookInstallerConfig, InstallError};
pub use types::{
    HookEvent, HookResponse, HookType, PostToolUseData, PreToolUseData, SessionStartData, StopData,
    UserPromptSubmitData,
};

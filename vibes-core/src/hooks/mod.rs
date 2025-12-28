//! Claude Code hooks integration
//!
//! This module provides structured data capture from Claude Code sessions
//! via the hooks system. Hook scripts send data to vibes via a Unix socket
//! (on Linux/macOS) or TCP (on Windows).
//!
//! ## Hook Types
//!
//! - **PreToolUse** - Called before a tool is executed
//! - **PostToolUse** - Called after a tool completes
//! - **Stop** - Called when Claude stops (provides transcript path)
//!
//! ## Architecture
//!
//! ```text
//! Claude Code ---> Hook Script ---> vibes-hook-send ---> Unix Socket ---> HookReceiver
//! ```

mod installer;
mod receiver;
pub mod scripts;
mod types;

pub use installer::{HookInstaller, HookInstallerConfig, InstallError};
pub use receiver::{HookReceiver, HookReceiverConfig};
pub use types::{HookEvent, HookType, PostToolUseData, PreToolUseData, StopData};

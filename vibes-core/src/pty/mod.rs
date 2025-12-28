//! PTY-based Claude backend
//!
//! Spawns Claude Code in a pseudo-terminal for full interactive support.

mod config;
mod error;
mod manager;
mod scrollback;
mod session;

pub use config::PtyConfig;
pub use error::PtyError;
pub use manager::{PtyManager, PtySessionInfo};
pub use scrollback::{ScrollbackBuffer, DEFAULT_CAPACITY};
pub use session::{PtySession, PtySessionHandle, PtyState};

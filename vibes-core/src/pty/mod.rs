//! PTY-based Claude backend
//!
//! Spawns Claude Code in a pseudo-terminal for full interactive support.

mod backend;
mod config;
mod error;
mod manager;
mod scrollback;
mod session;

pub use backend::{MockPtyBackend, PtyBackend, RealPtyBackend, create_backend};
pub use config::PtyConfig;
pub use error::PtyError;
pub use manager::{PtyManager, PtySessionInfo};
pub use scrollback::{DEFAULT_CAPACITY, ScrollbackBuffer};
pub use session::{PtySession, PtySessionHandle, PtyState};

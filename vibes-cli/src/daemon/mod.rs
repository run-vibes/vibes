//! Daemon management module
//!
//! Handles daemon lifecycle including:
//! - State file persistence
//! - Process management
//! - Auto-start functionality

mod autostart;
mod state;

pub use autostart::ensure_daemon_running;
// ensure_daemon_running_default will be used by claude command in Task 4.2
pub use state::{
    clear_daemon_state, is_process_alive, read_daemon_state, terminate_process, write_daemon_state,
    DaemonState,
};

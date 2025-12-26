//! Daemon management module
//!
//! Handles daemon lifecycle including:
//! - State file persistence
//! - Process management
//! - Auto-start functionality

mod state;

pub use state::{
    clear_daemon_state, is_process_alive, read_daemon_state, terminate_process, write_daemon_state,
    DaemonState,
};

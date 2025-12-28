//! PTY session manager

use std::collections::HashMap;

use super::{PtyConfig, PtySessionHandle};

/// Manages multiple PTY sessions
pub struct PtyManager {
    sessions: HashMap<String, PtySessionHandle>,
    config: PtyConfig,
}

impl PtyManager {
    /// Create a new PTY manager
    pub fn new(config: PtyConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }
}

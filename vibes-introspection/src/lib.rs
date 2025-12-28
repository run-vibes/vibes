//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{harness_for_command, Harness};
pub use paths::ConfigPaths;

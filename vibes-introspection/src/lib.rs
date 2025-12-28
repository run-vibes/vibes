//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod paths;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use paths::ConfigPaths;

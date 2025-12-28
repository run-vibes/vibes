//! vibes-introspection - Harness capability discovery
//!
//! This crate provides traits and implementations for discovering
//! what capabilities an AI coding assistant provides.

pub mod error;
pub mod paths;

pub use error::{IntrospectionError, Result};
pub use paths::ConfigPaths;

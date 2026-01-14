//! Ollama subprocess lifecycle management.
//!
//! Manages starting, stopping, and detecting the Ollama server process
//! with graceful handling of missing installation and already-running instances.

mod manager;

pub use manager::OllamaManager;

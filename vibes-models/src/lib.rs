//! Model management for vibes.
//!
//! This crate provides:
//! - Model registry for discovering available models
//! - Credential management for API keys
//! - Provider trait for unified inference interface
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   ModelRegistry                      │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
//! │  │  Anthropic  │  │   OpenAI    │  │   Ollama    │  │
//! │  │  Provider   │  │  Provider   │  │  Provider   │  │
//! │  └─────────────┘  └─────────────┘  └─────────────┘  │
//! └─────────────────────────────────────────────────────┘
//!                          │
//!                          ▼
//! ┌─────────────────────────────────────────────────────┐
//! │                  CredentialStore                     │
//! │         (System Keyring + Env Fallback)             │
//! └─────────────────────────────────────────────────────┘
//! ```

mod error;
mod types;

pub mod auth;
pub mod providers;
pub mod registry;

pub use error::{Error, Result};
pub use registry::ModelRegistry;
pub use types::{Capabilities, ModelId, ModelInfo, Pricing};

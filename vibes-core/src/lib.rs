//! vibes-core: Core library for the vibes Claude Code proxy
//!
//! This crate provides the foundational components for vibes:
//!
//! - **Session management** - [`Session`] and [`SessionManager`] for managing Claude Code interactions
//! - **Event system** - [`EventBus`] trait and [`MemoryEventBus`] for real-time event distribution
//! - **Backend abstraction** - [`ClaudeBackend`] trait with [`PrintModeBackend`] and [`MockBackend`] implementations
//! - **Event types** - [`ClaudeEvent`] and [`VibesEvent`] for typed event handling
//!
//! # Quick Start
//!
//! ```no_run
//! use std::sync::Arc;
//! use vibes_core::{
//!     SessionManager, MemoryEventBus,
//!     PrintModeBackendFactory, PrintModeConfig, BackendFactory,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create event bus and backend factory
//! let event_bus = Arc::new(MemoryEventBus::new(1000));
//! let factory: Arc<dyn BackendFactory> = Arc::new(PrintModeBackendFactory::new(
//!     PrintModeConfig::default()
//! ));
//!
//! // Create session manager
//! let manager = SessionManager::new(factory, event_bus);
//!
//! // Create a session
//! let session_id = manager.create_session(Some("My Session".to_string())).await;
//!
//! // Get session state
//! let state = manager.get_session_state(&session_id).await?;
//! println!("Session state: {:?}", state);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                 SessionManager                   │
//! │  ┌─────────────────────────────────────────────┐│
//! │  │                  Session                    ││
//! │  │  ┌───────────────┐  ┌───────────────────┐  ││
//! │  │  │ ClaudeBackend │  │     EventBus      │  ││
//! │  │  │ (PrintMode)   │  │ (MemoryEventBus)  │  ││
//! │  │  └───────────────┘  └───────────────────┘  ││
//! │  └─────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────┘
//! ```

pub mod auth;
pub mod backend;
pub mod error;
pub mod events;
pub mod history;
pub mod notifications;
pub mod parser;
pub mod plugins;
pub mod session;
pub mod tunnel;

// Re-export key types for convenience
pub use auth::{AccessConfig, AccessIdentity, AuthContext, AuthError, JwtValidator};
pub use backend::{
    BackendFactory, BackendState, ClaudeBackend, MockBackend, PrintModeBackend,
    PrintModeBackendFactory, PrintModeConfig,
};
pub use error::{BackendError, EventBusError, NotificationError, SessionError, VibesError};
pub use events::{ClaudeEvent, EventBus, MemoryEventBus, Usage, VibesEvent};
pub use history::HistoryError;
pub use notifications::{
    NotificationConfig, NotificationData, NotificationEvent, NotificationService, PushNotification,
    PushSubscription, SubscriptionKeys, SubscriptionStore, VapidKeyManager, VapidKeys,
};
pub use plugins::{PluginHost, PluginHostConfig, PluginHostError, PluginInfo, PluginState};
pub use session::{Session, SessionManager, SessionState};
pub use tunnel::{
    CloudflaredInfo, LogLevel, RestartPolicy, TunnelConfig, TunnelError, TunnelEvent,
    TunnelManager, TunnelMode, TunnelState, check_installation,
};

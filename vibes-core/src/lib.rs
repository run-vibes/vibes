//! vibes-core: Core library for the vibes Claude Code proxy
//!
//! This crate provides the foundational components for vibes:
//!
//! - **PTY management** - [`pty::PtyManager`] for spawning and managing Claude PTY sessions
//! - **Session management** - [`Session`] and [`SessionManager`] for session metadata and state
//! - **Event system** - [`EventBus`] trait and [`MemoryEventBus`] for real-time event distribution
//! - **Hooks integration** - [`HookReceiver`] for structured data capture from Claude Code
//! - **Event types** - [`ClaudeEvent`] and [`VibesEvent`] for typed event handling
//!
//! # Quick Start
//!
//! ```no_run
//! use vibes_core::pty::{PtyManager, PtyConfig};
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create PTY manager
//!     let mut pty_manager = PtyManager::new(PtyConfig::default());
//!
//!     // Create a Claude PTY session
//!     let session_id = pty_manager.create_session(Some("My Session".to_string()))?;
//!
//!     // Get the session
//!     if let Some(session) = pty_manager.get_session(&session_id) {
//!         println!("Session started: {}", session.id);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                  PtyManager                      │
//! │  ┌─────────────────────────────────────────────┐│
//! │  │               PtySession                    ││
//! │  │  ┌───────────────┐  ┌───────────────────┐  ││
//! │  │  │   PTY Master  │  │   Claude Process  │  ││
//! │  │  │  (portable)   │  │                   │  ││
//! │  │  └───────────────┘  └───────────────────┘  ││
//! │  └─────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────┘
//! ```

pub mod auth;
pub mod backend;
pub mod error;
pub mod events;
pub mod history;
pub mod hooks;
pub mod notifications;
pub mod plugins;
pub mod pty;
pub mod session;
pub mod tunnel;

// Re-export key types for convenience
pub use auth::{AccessConfig, AccessIdentity, AuthContext, AuthError, JwtValidator};
pub use backend::{BackendFactory, BackendState, ClaudeBackend, MockBackend, MockBackendFactory};
pub use error::{BackendError, EventBusError, NotificationError, SessionError, VibesError};
pub use events::{ClaudeEvent, EventBus, InputSource, MemoryEventBus, Usage, VibesEvent};
pub use history::HistoryError;
pub use notifications::{
    NotificationConfig, NotificationData, NotificationEvent, NotificationService, PushNotification,
    PushSubscription, SubscriptionKeys, SubscriptionStore, VapidKeyManager, VapidKeys,
};
pub use plugins::{PluginHost, PluginHostConfig, PluginHostError, PluginInfo, PluginState};
pub use session::{
    ClientId, DisconnectResult, Session, SessionLifecycleManager, SessionManager, SessionOwnership,
    SessionState,
};
pub use tunnel::{
    CloudflaredInfo, LogLevel, RestartPolicy, TunnelConfig, TunnelError, TunnelEvent,
    TunnelManager, TunnelMode, TunnelState, check_installation,
};
pub use hooks::{
    HookEvent, HookInstaller, HookInstallerConfig, HookReceiver, HookReceiverConfig, HookType,
    InstallError, PostToolUseData, PreToolUseData, StopData,
};

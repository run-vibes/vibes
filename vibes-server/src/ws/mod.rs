//! WebSocket module for real-time communication

mod connection;
mod protocol;

pub use connection::ws_handler;
pub use protocol::{ClientMessage, ServerMessage, vibes_event_to_server_message};

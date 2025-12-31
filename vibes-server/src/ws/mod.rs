//! WebSocket module for real-time communication

mod connection;
mod firehose;
mod protocol;

pub use connection::ws_handler;
pub use firehose::firehose_ws;
pub use protocol::{ClientMessage, ServerMessage, vibes_event_to_server_message};

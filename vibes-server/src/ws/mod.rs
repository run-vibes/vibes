//! WebSocket module for real-time communication

mod assessment;
mod connection;
mod firehose;
mod protocol;

pub use assessment::assessment_ws;
pub use connection::ws_handler;
pub use firehose::firehose_ws;
pub use protocol::{ClientMessage, ServerMessage, vibes_event_to_server_message};

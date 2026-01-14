//! WebSocket module for real-time communication

mod assessment;
mod connection;
mod firehose;
pub mod protocol;

pub use assessment::assessment_ws;
pub use connection::ws_handler;
pub use firehose::firehose_ws;
pub use protocol::{AgentInfo, ClientMessage, ServerMessage, vibes_event_to_server_message};

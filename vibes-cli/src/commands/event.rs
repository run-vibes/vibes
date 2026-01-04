//! Event command for sending events to the EventLog via Iggy HTTP API.

use std::io::Read;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::Deserialize;
use vibes_core::hooks::HookEvent;
use vibes_core::{StoredEvent, VibesEvent};

use crate::config::IggyClientConfig;
use crate::iggy_client::IggyHttpClient;

/// Event management commands
#[derive(Debug, Args)]
pub struct EventArgs {
    #[command(subcommand)]
    pub command: EventCommand,
}

/// Event subcommands
#[derive(Debug, Subcommand)]
pub enum EventCommand {
    /// Send an event to the EventLog
    Send(SendArgs),
}

/// Arguments for the `event send` command
#[derive(Debug, Args)]
pub struct SendArgs {
    /// Event type: hook, session-state
    #[arg(short = 't', long = "type")]
    pub event_type: String,

    /// Session ID for event attribution
    #[arg(short, long)]
    pub session: Option<String>,

    /// Event payload as JSON (reads from stdin if omitted)
    #[arg(short, long)]
    pub data: Option<String>,

    /// Iggy topic name
    #[arg(long, default_value = "events")]
    pub topic: String,

    /// Iggy stream name
    #[arg(long, default_value = "vibes")]
    pub stream: String,
}

/// Run the event command
pub async fn run(args: EventArgs) -> Result<()> {
    match args.command {
        EventCommand::Send(send_args) => execute_send(send_args).await,
    }
}

/// Payload for session-state events
#[derive(Debug, Deserialize)]
struct SessionStatePayload {
    state: String,
}

/// Execute the send subcommand
async fn execute_send(args: SendArgs) -> Result<()> {
    // 1. Read payload from --data or stdin
    let payload = match args.data {
        Some(data) => data,
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    // 2. Parse and wrap in VibesEvent based on event_type
    let event = match args.event_type.as_str() {
        "hook" => {
            let hook: HookEvent =
                serde_json::from_str(&payload).context("Failed to parse hook event JSON")?;
            VibesEvent::Hook {
                session_id: args.session,
                event: hook,
            }
        }
        "session-state" => {
            let session_id = args
                .session
                .ok_or_else(|| anyhow::anyhow!("--session is required for session-state events"))?;
            let state_payload: SessionStatePayload = serde_json::from_str(&payload)
                .context("Failed to parse session-state JSON (expected {\"state\": \"...\"})")?;
            VibesEvent::SessionStateChanged {
                session_id,
                state: state_payload.state,
            }
        }
        other => anyhow::bail!(
            "Unknown event type: {}. Valid types: hook, session-state",
            other
        ),
    };

    // 3. Connect to Iggy and authenticate
    let config = IggyClientConfig::from_env();
    let mut client = IggyHttpClient::from_config(&config);
    client
        .authenticate(&config.username, &config.password)
        .await
        .context("Failed to authenticate with Iggy")?;

    // 4. Wrap in StoredEvent (adds event_id) and serialize
    let stored = StoredEvent::new(event);
    let serialized = serde_json::to_vec(&stored).context("Failed to serialize event")?;
    client
        .send_message(&args.stream, &args.topic, &serialized)
        .await
        .context("Failed to send message to Iggy")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        event: EventArgs,
    }

    // ==================== Event Parsing Tests ====================

    #[test]
    fn parse_hook_event_json() {
        let json =
            r#"{"type":"stop","transcript_path":null,"reason":"user","session_id":"sess-123"}"#;
        let hook: HookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(hook.session_id(), Some("sess-123"));
    }

    #[test]
    fn parse_session_state_payload() {
        let json = r#"{"state":"Processing"}"#;
        let payload: SessionStatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.state, "Processing");
    }

    #[test]
    fn invalid_hook_json_returns_error() {
        let json = r#"{"not_valid": true}"#;
        let result: Result<HookEvent, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_session_state_json_returns_error() {
        let json = r#"{"wrong_field": "value"}"#;
        let result: Result<SessionStatePayload, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ==================== CLI Argument Tests ====================

    #[test]
    fn parse_send_with_all_options() {
        let cli = TestCli::try_parse_from([
            "test",
            "send",
            "--type",
            "hook",
            "--session",
            "sess-123",
            "--data",
            r#"{"type":"stop"}"#,
        ])
        .unwrap();

        match cli.event.command {
            EventCommand::Send(args) => {
                assert_eq!(args.event_type, "hook");
                assert_eq!(args.session, Some("sess-123".to_string()));
                assert_eq!(args.data, Some(r#"{"type":"stop"}"#.to_string()));
                assert_eq!(args.topic, "events");
                assert_eq!(args.stream, "vibes");
            }
        }
    }

    #[test]
    fn parse_send_minimal() {
        let cli =
            TestCli::try_parse_from(["test", "send", "--type", "hook", "--data", "{}"]).unwrap();

        match cli.event.command {
            EventCommand::Send(args) => {
                assert_eq!(args.event_type, "hook");
                assert!(args.session.is_none());
                assert_eq!(args.data, Some("{}".to_string()));
            }
        }
    }

    #[test]
    fn parse_send_without_data_for_stdin() {
        let cli = TestCli::try_parse_from(["test", "send", "--type", "session-state"]).unwrap();

        match cli.event.command {
            EventCommand::Send(args) => {
                assert_eq!(args.event_type, "session-state");
                assert!(args.data.is_none()); // Will read from stdin
            }
        }
    }

    #[test]
    fn parse_send_with_custom_stream_topic() {
        let cli = TestCli::try_parse_from([
            "test",
            "send",
            "--type",
            "hook",
            "--data",
            "{}",
            "--stream",
            "custom-stream",
            "--topic",
            "custom-topic",
        ])
        .unwrap();

        match cli.event.command {
            EventCommand::Send(args) => {
                assert_eq!(args.stream, "custom-stream");
                assert_eq!(args.topic, "custom-topic");
            }
        }
    }
}

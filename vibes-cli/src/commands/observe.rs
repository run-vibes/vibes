//! Observe command for viewing and managing traces
//!
//! Provides `vibes observe traces` for tailing live trace events and
//! `vibes observe config` for managing observe settings.

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::debug;
use vibes_observe::TraceEvent;
use vibes_server::ws::ServerMessage;

use crate::config::{DEFAULT_HOST, DEFAULT_PORT};

/// Observe management commands
#[derive(Debug, Args)]
pub struct ObserveArgs {
    #[command(subcommand)]
    pub command: ObserveCommands,
}

/// Observe subcommands
#[derive(Debug, Subcommand)]
pub enum ObserveCommands {
    /// Tail live trace events
    Traces(TracesArgs),
    /// Show observe configuration
    Config,
}

/// Arguments for the `observe traces` command
#[derive(Debug, Args)]
pub struct TracesArgs {
    /// Filter by session ID (prefix match)
    #[arg(short, long)]
    pub session: Option<String>,

    /// Filter by agent ID (exact match)
    #[arg(short, long)]
    pub agent: Option<String>,

    /// Minimum log level to display
    #[arg(short, long, value_enum, default_value = "info")]
    pub level: LogLevel,

    /// Output format
    #[arg(short, long, value_enum, default_value = "tree")]
    pub format: OutputFormat,
}

/// Log level filter
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// Output format for traces
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Tree-structured output
    #[default]
    Tree,
    /// JSON output (one event per line)
    Json,
    /// Compact single-line output
    Compact,
}

/// Run the observe command
pub async fn run(args: ObserveArgs) -> Result<()> {
    match args.command {
        ObserveCommands::Traces(traces_args) => run_traces(traces_args).await,
        ObserveCommands::Config => run_config(),
    }
}

/// Connect to the traces WebSocket endpoint and stream events
async fn run_traces(args: TracesArgs) -> Result<()> {
    // Build URL with query parameters for filtering
    let mut url = format!("ws://{}:{}/ws/traces", DEFAULT_HOST, DEFAULT_PORT);
    let mut params = Vec::new();

    if let Some(ref session) = args.session {
        params.push(format!("session={}", urlencoding::encode(session)));
    }
    if let Some(ref agent) = args.agent {
        params.push(format!("agent={}", urlencoding::encode(agent)));
    }
    params.push(format!("level={}", args.level.as_str()));

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    debug!("Connecting to traces WebSocket: {}", url);

    let (ws_stream, _response) = connect_async(&url)
        .await
        .context("Failed to connect to vibes server. Is it running?")?;

    let (_sender, mut receiver) = ws_stream.split();

    // Wait for subscription confirmation
    let first_msg = receiver
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("Connection closed before receiving confirmation"))??;

    if let Message::Text(text) = first_msg {
        let msg: ServerMessage = serde_json::from_str(&text)?;
        match msg {
            ServerMessage::TraceSubscribed => {
                eprintln!("Subscribed to traces (level: {})", args.level.as_str());
                if let Some(ref session) = args.session {
                    eprintln!("  session filter: {}*", session);
                }
                if let Some(ref agent) = args.agent {
                    eprintln!("  agent filter: {}", agent);
                }
                eprintln!();
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Server error: {}", message);
            }
            _ => {
                debug!("Unexpected first message: {:?}", msg);
            }
        }
    }

    // Stream trace events
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                let msg: ServerMessage = serde_json::from_str(&text)?;
                if let ServerMessage::TraceEvent(event) = msg {
                    print_trace_event(&event, args.format);
                }
            }
            Ok(Message::Close(_)) => {
                eprintln!("Connection closed by server");
                break;
            }
            Err(e) => {
                anyhow::bail!("WebSocket error: {}", e);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Print a trace event in the specified format
fn print_trace_event(event: &TraceEvent, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string(event) {
                println!("{}", json);
            }
        }
        OutputFormat::Compact => {
            print_compact(event);
        }
        OutputFormat::Tree => {
            print_tree(event);
        }
    }
}

/// Print event in compact single-line format
fn print_compact(event: &TraceEvent) {
    let level_char = match event.level.as_str() {
        "trace" => 'T',
        "debug" => 'D',
        "info" => 'I',
        "warn" => 'W',
        "error" => 'E',
        _ => '?',
    };

    let duration = event
        .duration_ms
        .map(|d| format!(" ({:.1}ms)", d))
        .unwrap_or_default();

    let session = event
        .session_id
        .as_ref()
        .map(|s| format!(" [{}]", truncate_id(s, 8)))
        .unwrap_or_default();

    println!(
        "{} {}{}{} {}",
        level_char,
        event.timestamp.format("%H:%M:%S%.3f"),
        session,
        duration,
        event.name
    );
}

/// Print event in tree-structured format
fn print_tree(event: &TraceEvent) {
    let level_style = match event.level.as_str() {
        "error" => "\x1b[31m", // red
        "warn" => "\x1b[33m",  // yellow
        "info" => "\x1b[32m",  // green
        "debug" => "\x1b[36m", // cyan
        "trace" => "\x1b[90m", // gray
        _ => "\x1b[0m",
    };
    let reset = "\x1b[0m";

    // Header line with level and name
    println!(
        "{}[{}]{} {}",
        level_style,
        event.level.to_uppercase(),
        reset,
        event.name
    );

    // Details indented
    let indent = "  ";
    println!(
        "{}time: {}",
        indent,
        event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
    );

    if let Some(duration) = event.duration_ms {
        println!("{}duration: {:.2}ms", indent, duration);
    }

    if let Some(ref session) = event.session_id {
        println!("{}session: {}", indent, session);
    }

    if let Some(ref agent) = event.agent_id {
        println!("{}agent: {}", indent, agent);
    }

    // Show span IDs for tracing correlation
    println!(
        "{}trace: {} / span: {}",
        indent, event.trace_id, event.span_id
    );

    if let Some(ref parent) = event.parent_span_id {
        println!("{}parent: {}", indent, parent);
    }

    // Show attributes if any
    if !event.attributes.is_empty() {
        println!("{}attributes:", indent);
        for (key, value) in &event.attributes {
            println!("{}  {}: {}", indent, key, value);
        }
    }

    // Status indicator for errors
    if event.status == vibes_observe::SpanStatus::Error {
        println!("{}\x1b[31mstatus: ERROR{}", indent, reset);
    }

    println!(); // Blank line between events
}

/// Truncate an ID string to show first N characters with ellipsis
fn truncate_id(s: &str, len: usize) -> String {
    if s.len() <= len {
        s.to_string()
    } else {
        format!("{}...", &s[..len])
    }
}

/// Show observe configuration
fn run_config() -> Result<()> {
    println!("Observe configuration:");
    println!();
    println!(
        "  Server endpoint: ws://{}:{}/ws/traces",
        DEFAULT_HOST, DEFAULT_PORT
    );
    println!();
    println!("Note: Observe settings are currently not persisted.");
    println!("Use command-line flags to filter traces:");
    println!("  --session, -s   Filter by session ID prefix");
    println!("  --agent, -a     Filter by agent ID");
    println!("  --level, -l     Minimum log level (trace, debug, info, warn, error)");
    println!("  --format, -f    Output format (tree, json, compact)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use clap::Parser;
    use std::collections::HashMap;
    use vibes_observe::SpanStatus;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        observe: ObserveArgs,
    }

    fn make_test_event() -> TraceEvent {
        TraceEvent {
            trace_id: "trace-123".to_string(),
            span_id: "span-456".to_string(),
            parent_span_id: None,
            name: "test::operation".to_string(),
            level: "info".to_string(),
            timestamp: Utc::now(),
            duration_ms: Some(42.5),
            session_id: Some("sess-abc123".to_string()),
            agent_id: None,
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
        }
    }

    #[test]
    fn parse_traces_minimal() {
        let cli = TestCli::try_parse_from(["test", "traces"]).unwrap();
        match cli.observe.command {
            ObserveCommands::Traces(args) => {
                assert!(args.session.is_none());
                assert!(args.agent.is_none());
                assert!(matches!(args.level, LogLevel::Info));
                assert!(matches!(args.format, OutputFormat::Tree));
            }
            _ => panic!("Expected Traces command"),
        }
    }

    #[test]
    fn parse_traces_with_filters() {
        let cli = TestCli::try_parse_from([
            "test",
            "traces",
            "--session",
            "sess-123",
            "--agent",
            "agent-1",
            "--level",
            "debug",
            "--format",
            "json",
        ])
        .unwrap();

        match cli.observe.command {
            ObserveCommands::Traces(args) => {
                assert_eq!(args.session, Some("sess-123".to_string()));
                assert_eq!(args.agent, Some("agent-1".to_string()));
                assert!(matches!(args.level, LogLevel::Debug));
                assert!(matches!(args.format, OutputFormat::Json));
            }
            _ => panic!("Expected Traces command"),
        }
    }

    #[test]
    fn parse_traces_short_flags() {
        let cli = TestCli::try_parse_from([
            "test", "traces", "-s", "sess-1", "-l", "warn", "-f", "compact",
        ])
        .unwrap();

        match cli.observe.command {
            ObserveCommands::Traces(args) => {
                assert_eq!(args.session, Some("sess-1".to_string()));
                assert!(matches!(args.level, LogLevel::Warn));
                assert!(matches!(args.format, OutputFormat::Compact));
            }
            _ => panic!("Expected Traces command"),
        }
    }

    #[test]
    fn parse_config_command() {
        let cli = TestCli::try_parse_from(["test", "config"]).unwrap();
        assert!(matches!(cli.observe.command, ObserveCommands::Config));
    }

    #[test]
    fn truncate_id_short() {
        assert_eq!(truncate_id("abc", 8), "abc");
    }

    #[test]
    fn truncate_id_long() {
        assert_eq!(truncate_id("abcdefghijk", 8), "abcdefgh...");
    }

    #[test]
    fn level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "trace");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn print_compact_format() {
        // Just ensure it doesn't panic
        let event = make_test_event();
        print_compact(&event);
    }

    #[test]
    fn print_tree_format() {
        // Just ensure it doesn't panic
        let event = make_test_event();
        print_tree(&event);
    }
}

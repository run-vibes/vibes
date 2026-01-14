//! Eval commands for managing longitudinal studies.

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::VibesClient;
use vibes_server::ws::{ServerMessage, StudyInfo};

/// Eval management arguments.
#[derive(Args, Debug)]
pub struct EvalArgs {
    #[command(subcommand)]
    pub command: EvalCommands,
}

/// Eval subcommands.
#[derive(Subcommand, Debug)]
pub enum EvalCommands {
    /// Manage longitudinal studies
    Study(StudyArgs),
}

/// Study management arguments.
#[derive(Args, Debug)]
pub struct StudyArgs {
    #[command(subcommand)]
    pub command: StudyCommands,
}

/// Study subcommands.
#[derive(Subcommand, Debug)]
pub enum StudyCommands {
    /// Start a new longitudinal study
    Start {
        /// Human-readable name for the study
        name: String,

        /// Period specification (e.g., "weekly:2" for 2 weeks)
        #[arg(short, long, default_value = "daily")]
        period: String,

        /// Optional description for the study
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Stop a running study
    Stop {
        /// Study ID to stop
        id: String,
    },

    /// Show current study status
    Status,

    /// List all studies
    List,

    /// Force a checkpoint recording
    Checkpoint {
        /// Study ID to checkpoint
        id: String,
    },

    /// Generate a summary report
    Report {
        /// Study ID to report on
        id: String,
    },
}

/// Run eval command.
pub async fn run(args: EvalArgs) -> Result<()> {
    match args.command {
        EvalCommands::Study(study_args) => run_study(study_args).await,
    }
}

/// Run study command.
async fn run_study(args: StudyArgs) -> Result<()> {
    match args.command {
        StudyCommands::Start {
            name,
            period,
            description,
        } => start_study(&name, &period, description).await,
        StudyCommands::Stop { id } => stop_study(&id).await,
        StudyCommands::Status => study_status().await,
        StudyCommands::List => list_studies().await,
        StudyCommands::Checkpoint { id } => record_checkpoint(&id).await,
        StudyCommands::Report { id } => study_report(&id).await,
    }
}

/// Parse period string like "weekly:2" into (period_type, period_value).
fn parse_period(period: &str) -> (String, Option<u32>) {
    if let Some((period_type, value)) = period.split_once(':') {
        let period_value = value.parse().ok();
        (period_type.to_string(), period_value)
    } else {
        (period.to_string(), None)
    }
}

/// Start a new longitudinal study.
async fn start_study(name: &str, period: &str, description: Option<String>) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let (period_type, period_value) = parse_period(period);

    client
        .send_create_study(&request_id, name, &period_type, period_value, description)
        .await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::StudyCreated { study, .. } => {
                print_study_created(&study);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error creating study: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Stop a running study.
async fn stop_study(study_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    client.send_stop_study(&request_id, study_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::StudyStopped { study_id, .. } => {
                println!("Stopped study: {}", study_id);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error stopping study: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Show status of active studies.
async fn study_status() -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    client.send_list_studies(&request_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::StudyList { studies, .. } => {
                let active: Vec<_> = studies
                    .iter()
                    .filter(|s| s.status == "running" || s.status == "paused")
                    .collect();

                if active.is_empty() {
                    println!("No active studies");
                } else {
                    println!("Active Studies");
                    println!("{}", "─".repeat(66));
                    println!(
                        "{:<20} {:<24} {:<12} {:>10}",
                        "ID", "NAME", "STARTED", "CHECKPOINTS"
                    );

                    for study in active {
                        let started = study
                            .started_at
                            .map(format_relative_time)
                            .unwrap_or_else(|| "-".to_string());

                        println!(
                            "{:<20} {:<24} {:<12} {:>10}",
                            truncate_id(&study.id),
                            truncate_str(&study.name, 24),
                            started,
                            study.checkpoint_count
                        );
                    }
                }
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error listing studies: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// List all studies.
async fn list_studies() -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    client.send_list_studies(&request_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::StudyList { studies, .. } => {
                if studies.is_empty() {
                    println!("No studies found");
                } else {
                    println!("Studies");
                    println!("{}", "─".repeat(78));
                    println!(
                        "{:<20} {:<24} {:<10} {:<12} {:>10}",
                        "ID", "NAME", "STATUS", "PERIOD", "CHECKPOINTS"
                    );

                    for study in &studies {
                        let period = format_period(&study.period_type, study.period_value);
                        println!(
                            "{:<20} {:<24} {:<10} {:<12} {:>10}",
                            truncate_id(&study.id),
                            truncate_str(&study.name, 24),
                            &study.status,
                            period,
                            study.checkpoint_count
                        );
                    }
                }
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error listing studies: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Record a checkpoint for a study.
async fn record_checkpoint(study_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    client.send_record_checkpoint(&request_id, study_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::CheckpointRecorded { checkpoint, .. } => {
                println!("Recorded checkpoint: {}", checkpoint.id);
                println!("  Events analyzed: {}", checkpoint.events_analyzed);
                println!("  Sessions completed: {}", checkpoint.sessions_completed);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error recording checkpoint: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Generate a report for a study.
async fn study_report(study_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    client.send_get_study(&request_id, study_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::StudyDetails {
                study, checkpoints, ..
            } => {
                print_study_report(&study, &checkpoints);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error getting study: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

// === Output Formatting ===

fn print_study_created(study: &StudyInfo) {
    println!("Started longitudinal study: {}", study.name);
    println!("ID: {}", study.id);
    println!(
        "Period: {}",
        format_period(&study.period_type, study.period_value)
    );
    println!();
    println!("Tracking metrics:");
    println!("  - Session success rate");
    println!("  - First attempt success rate");
    println!("  - Cost per successful task");
    println!("  - Learning effectiveness");
}

fn print_study_report(study: &StudyInfo, checkpoints: &[vibes_server::ws::CheckpointInfo]) {
    println!("Study Report: {}", study.name);

    if let (Some(start), Some(end)) = (study.started_at, study.stopped_at) {
        println!(
            "Period: {} - {}",
            format_timestamp(start),
            format_timestamp(end)
        );
    } else if let Some(start) = study.started_at {
        println!("Period: {} - present", format_timestamp(start));
    }

    println!("{}", "─".repeat(68));
    println!();

    // Summary
    let total_sessions: u32 = checkpoints.iter().map(|c| c.sessions_completed).sum();
    let avg_success_rate = if !checkpoints.is_empty() {
        checkpoints
            .iter()
            .filter_map(|c| c.success_rate)
            .sum::<f64>()
            / checkpoints.len() as f64
    } else {
        0.0
    };

    println!("Summary:");
    println!("  Total sessions: {}", total_sessions);
    println!("  Success rate: {:.1}%", avg_success_rate * 100.0);
    println!("  Checkpoints: {}", checkpoints.len());
    println!();

    // Trends (if we have multiple checkpoints)
    if checkpoints.len() >= 2 {
        println!("Trends:");
        if let (Some(first), Some(last)) = (checkpoints.first(), checkpoints.last()) {
            if let (Some(first_rate), Some(last_rate)) =
                (first.first_attempt_rate, last.first_attempt_rate)
            {
                let trend = if last_rate > first_rate {
                    "↑"
                } else if last_rate < first_rate {
                    "↓"
                } else {
                    "→"
                };
                println!(
                    "  {} First attempt success: {:.0}% → {:.0}%",
                    trend,
                    first_rate * 100.0,
                    last_rate * 100.0
                );
            }

            if let (Some(first_iter), Some(last_iter)) = (first.avg_iterations, last.avg_iterations)
            {
                let trend = if last_iter < first_iter {
                    "↑"
                } else if last_iter > first_iter {
                    "↓"
                } else {
                    "→"
                };
                println!(
                    "  {} Avg iterations: {:.1} → {:.1}",
                    trend, first_iter, last_iter
                );
            }
        }
    }
}

fn format_period(period_type: &str, period_value: Option<u32>) -> String {
    match period_value {
        Some(v) => format!("{} {}", v, period_type),
        None => period_type.to_string(),
    }
}

fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{} min ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| dt.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn truncate_id(id: &str) -> String {
    if id.len() > 18 {
        format!("{}...", &id[..15])
    } else {
        id.to_string()
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

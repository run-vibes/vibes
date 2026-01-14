//! Agent management commands

use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use vibes_core::agent::AgentType;
use vibes_server::ws::{AgentInfo, ServerMessage};

use crate::client::VibesClient;

/// Agent management arguments
#[derive(Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

/// CLI-friendly agent type that maps to AgentType
#[derive(Debug, Clone, ValueEnum)]
pub enum CliAgentType {
    /// Ad-hoc agent for one-off tasks
    Adhoc,
    /// Background agent for ongoing work
    Background,
    /// Subagent spawned by other agents
    Subagent,
    /// Interactive agent for user sessions
    Interactive,
}

impl From<CliAgentType> for AgentType {
    fn from(cli_type: CliAgentType) -> Self {
        match cli_type {
            CliAgentType::Adhoc => AgentType::AdHoc,
            CliAgentType::Background => AgentType::Background,
            CliAgentType::Subagent => AgentType::Subagent,
            CliAgentType::Interactive => AgentType::Interactive,
        }
    }
}

/// Agent subcommands
#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// List all agents
    List,
    /// Spawn a new agent
    Spawn {
        /// Type of agent to spawn
        #[arg(value_enum)]
        agent_type: CliAgentType,
        /// Optional name for the agent
        #[arg(short, long)]
        name: Option<String>,
        /// Optional task to start immediately
        #[arg(short, long)]
        task: Option<String>,
    },
    /// Get detailed status of an agent
    Status {
        /// Agent ID or prefix
        agent_id: String,
    },
    /// Pause an agent
    Pause {
        /// Agent ID or prefix
        agent_id: String,
    },
    /// Resume a paused agent
    Resume {
        /// Agent ID or prefix
        agent_id: String,
    },
    /// Cancel current task on an agent
    Cancel {
        /// Agent ID or prefix
        agent_id: String,
    },
    /// Stop and remove an agent
    Stop {
        /// Agent ID or prefix
        agent_id: String,
    },
}

/// Run agent command
pub async fn run(args: AgentArgs) -> Result<()> {
    match args.command {
        AgentCommands::List => list_agents().await,
        AgentCommands::Spawn {
            agent_type,
            name,
            task,
        } => spawn_agent(agent_type.into(), name, task).await,
        AgentCommands::Status { agent_id } => agent_status(&agent_id).await,
        AgentCommands::Pause { agent_id } => pause_agent(&agent_id).await,
        AgentCommands::Resume { agent_id } => resume_agent(&agent_id).await,
        AgentCommands::Cancel { agent_id } => cancel_agent(&agent_id).await,
        AgentCommands::Stop { agent_id } => stop_agent(&agent_id).await,
    }
}

/// Generate a unique request ID
fn request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// List all agents
async fn list_agents() -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_list_agents(&req_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentList {
                request_id: rid,
                agents,
            } if rid == req_id => {
                if agents.is_empty() {
                    println!("No active agents");
                } else {
                    print_agent_list(&agents);
                }
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error listing agents: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Print agent list in the specified format
fn print_agent_list(agents: &[AgentInfo]) {
    println!("Active agents:");
    println!();

    for agent in agents {
        let short_id = &agent.id[..8];
        let status_str = format_status(&agent.status);
        let duration = agent
            .current_task_metrics
            .as_ref()
            .map(|m| format_duration(m.duration))
            .unwrap_or_default();

        println!(
            "  {} {} [{:?}] {} {}",
            short_id, agent.name, agent.agent_type, status_str, duration
        );
    }
}

/// Format agent status for display
fn format_status(status: &vibes_core::agent::AgentStatus) -> String {
    match status {
        vibes_core::agent::AgentStatus::Idle => "idle".to_string(),
        vibes_core::agent::AgentStatus::Running { task, .. } => {
            format!("running (task {})", &task.0.to_string()[..8])
        }
        vibes_core::agent::AgentStatus::Paused { reason, .. } => {
            format!("paused: {}", truncate(reason, 30))
        }
        vibes_core::agent::AgentStatus::WaitingForInput { prompt, .. } => {
            format!("waiting: {}", truncate(prompt, 30))
        }
        vibes_core::agent::AgentStatus::Failed { error, .. } => {
            format!("failed: {}", truncate(error, 30))
        }
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format duration for display
fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Spawn a new agent
async fn spawn_agent(
    agent_type: AgentType,
    name: Option<String>,
    task: Option<String>,
) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client
        .send_spawn_agent(&req_id, agent_type, name, task)
        .await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentSpawned {
                request_id: rid,
                agent,
            } if rid == req_id => {
                println!("Spawned agent: {} ({})", agent.name, &agent.id[..8]);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error spawning agent: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Get detailed status of an agent
async fn agent_status(agent_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_agent_status(&req_id, agent_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentStatusResponse {
                request_id: rid,
                agent,
            } if rid == req_id => {
                print_agent_details(&agent);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Agent not found or error: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Print detailed agent information
fn print_agent_details(agent: &AgentInfo) {
    println!("Agent: {}", agent.name);
    println!("  ID:     {}", agent.id);
    println!("  Type:   {:?}", agent.agent_type);
    println!("  Status: {}", format_status(&agent.status));

    // Print context
    println!("  Location: {:?}", agent.context.location);
    println!("  Model:    {}", agent.context.model.0);

    if !agent.context.tools.is_empty() {
        let tools: Vec<&str> = agent.context.tools.iter().map(|t| t.0.as_str()).collect();
        println!("  Tools:    {}", tools.join(", "));
    }

    // Print metrics if running
    if let Some(metrics) = &agent.current_task_metrics {
        println!("  Duration: {}", format_duration(metrics.duration));
        if metrics.tokens_used > 0 {
            println!("  Tokens:   {}", metrics.tokens_used);
        }
        if metrics.tool_calls > 0 {
            println!("  Tool calls: {}", metrics.tool_calls);
        }
        if metrics.iterations > 0 {
            println!("  Iterations: {}", metrics.iterations);
        }
    }
}

/// Pause an agent
async fn pause_agent(agent_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_pause_agent(&req_id, agent_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentAck {
                request_id: rid,
                agent_id: aid,
                operation,
            } if rid == req_id => {
                println!("Agent {} {}", &aid[..8.min(aid.len())], operation);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error pausing agent: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Resume a paused agent
async fn resume_agent(agent_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_resume_agent(&req_id, agent_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentAck {
                request_id: rid,
                agent_id: aid,
                operation,
            } if rid == req_id => {
                println!("Agent {} {}", &aid[..8.min(aid.len())], operation);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error resuming agent: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Cancel current task on an agent
async fn cancel_agent(agent_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_cancel_agent(&req_id, agent_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentAck {
                request_id: rid,
                agent_id: aid,
                operation,
            } if rid == req_id => {
                println!("Agent {} {}", &aid[..8.min(aid.len())], operation);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error cancelling agent task: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Stop and remove an agent
async fn stop_agent(agent_id: &str) -> Result<()> {
    let mut client = VibesClient::connect().await?;
    let req_id = request_id();

    client.send_stop_agent(&req_id, agent_id).await?;

    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AgentAck {
                request_id: rid,
                agent_id: aid,
                operation,
            } if rid == req_id => {
                println!("Agent {} {}", &aid[..8.min(aid.len())], operation);
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error stopping agent: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

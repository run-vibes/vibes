//! AI models management commands.
//!
//! Provides commands for listing models, viewing details, and managing
//! API credentials.

use std::sync::Arc;

use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use dialoguer::{Password, theme::ColorfulTheme};
use tracing::debug;
use vibes_models::auth::{CredentialSource, CredentialStore};
use vibes_models::providers::OllamaProvider;
use vibes_models::registry::ModelRegistry;
use vibes_models::{Capabilities, ModelId};

/// Models management arguments.
#[derive(Args, Debug)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommands,
}

/// Models subcommands.
#[derive(Subcommand, Debug)]
pub enum ModelsCommands {
    /// List available models
    List {
        /// Filter by provider name
        #[arg(long)]
        provider: Option<String>,

        /// Filter by capability (chat, vision, tools, embeddings)
        #[arg(long)]
        capability: Option<String>,
    },
    /// Show detailed model information
    Info {
        /// Model name (e.g., claude-sonnet-4, gpt-4o)
        model: String,
    },
    /// Manage API credentials
    Auth {
        /// Provider to configure (e.g., anthropic, openai)
        provider: Option<String>,

        /// List configured providers
        #[arg(long)]
        list: bool,

        /// Delete stored credentials
        #[arg(long)]
        delete: bool,
    },
}

/// Run models command.
pub async fn run(args: ModelsArgs) -> Result<()> {
    match args.command {
        ModelsCommands::List {
            provider,
            capability,
        } => list_models(provider, capability).await,
        ModelsCommands::Info { model } => show_model_info(&model).await,
        ModelsCommands::Auth {
            provider,
            list,
            delete,
        } => manage_auth(provider, list, delete).await,
    }
}

/// Build a model registry with all available providers.
///
/// Registers local providers (Ollama) and cloud providers with configured credentials.
async fn build_registry() -> ModelRegistry {
    let mut registry = ModelRegistry::new();

    // Register Ollama provider if available
    let ollama_url =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama = OllamaProvider::with_base_url(&ollama_url);

    // Try to refresh models - silently skip if Ollama isn't running
    if let Err(e) = ollama.refresh_models().await {
        debug!("Ollama not available: {}", e);
    } else {
        let model_count = ollama.models().len();
        debug!("Registered Ollama provider with {} models", model_count);
        registry.register_provider(Arc::new(ollama));
    }

    // TODO: Register cloud providers (Anthropic, OpenAI) when credentials are available

    registry
}

/// List available models with optional filtering.
async fn list_models(
    provider_filter: Option<String>,
    capability_filter: Option<String>,
) -> Result<()> {
    let registry = build_registry().await;

    // Get models based on filters
    let models = match (&provider_filter, &capability_filter) {
        (Some(provider), Some(cap)) => {
            let cap = parse_capability(cap)?;
            registry
                .find_by_provider(provider)
                .into_iter()
                .filter(|m| m.capabilities.matches(&cap))
                .collect()
        }
        (Some(provider), None) => registry.find_by_provider(provider),
        (None, Some(cap)) => {
            let cap = parse_capability(cap)?;
            registry.find_by_capability(cap)
        }
        (None, None) => registry.list_models(),
    };

    if models.is_empty() {
        if provider_filter.is_some() || capability_filter.is_some() {
            println!("No models match the specified filters.");
        } else {
            println!("No models registered.");
            println!();
            println!("Register a provider to see available models.");
        }
        return Ok(());
    }

    // Build table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Provider").fg(Color::Cyan),
        Cell::new("Model").fg(Color::Cyan),
        Cell::new("Context").fg(Color::Cyan),
        Cell::new("Capabilities").fg(Color::Cyan),
    ]);

    for model in models {
        let caps = format_capabilities(&model.capabilities);
        let context = format_context(model.context_window as usize);

        table.add_row(vec![
            Cell::new(&model.provider),
            Cell::new(&model.name),
            Cell::new(context),
            Cell::new(caps),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// Show detailed information about a specific model.
async fn show_model_info(model_name: &str) -> Result<()> {
    let registry = build_registry().await;

    // Try to find the model - it might be specified as "provider/model" or just "model"
    let model = if model_name.contains('/') {
        let parts: Vec<&str> = model_name.splitn(2, '/').collect();
        let id = ModelId::new(parts[0], parts[1]);
        registry.get_model(&id)
    } else {
        // Search across all providers
        registry
            .list_models()
            .into_iter()
            .find(|m| m.id.model() == model_name || m.name == model_name)
    };

    let Some(model) = model else {
        bail!("Model '{}' not found", model_name);
    };

    println!("Model: {}", model.name);
    println!("ID: {}", model.id);
    println!("Provider: {}", model.provider);
    println!();

    println!("Capabilities:");
    println!("  Context window: {} tokens", model.context_window);
    println!(
        "  Chat: {}",
        if model.capabilities.chat { "yes" } else { "no" }
    );
    println!(
        "  Vision: {}",
        if model.capabilities.vision {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Tool use: {}",
        if model.capabilities.tools {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Embeddings: {}",
        if model.capabilities.embeddings {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Streaming: {}",
        if model.capabilities.streaming {
            "yes"
        } else {
            "no"
        }
    );

    if let Some(pricing) = &model.pricing {
        println!();
        println!("Pricing (per 1M tokens):");
        println!("  Input: ${:.2}", pricing.input_per_million);
        println!("  Output: ${:.2}", pricing.output_per_million);
    }

    Ok(())
}

/// Manage API credentials for providers.
async fn manage_auth(provider: Option<String>, list: bool, delete: bool) -> Result<()> {
    let store = CredentialStore::new("vibes").with_env_fallback();

    // List configured providers
    if list {
        let providers = store.list_providers();
        if providers.is_empty() {
            println!("No API credentials configured.");
            println!();
            println!("Configure credentials with: vibes models auth <provider>");
        } else {
            println!("Configured providers:");
            println!();
            for provider in providers {
                let source = store.credential_source(&provider);
                let source_str = match source {
                    Some(CredentialSource::Keyring) => "(keyring)",
                    Some(CredentialSource::Environment) => "(environment)",
                    None => "",
                };
                println!("  {} {}", provider, source_str);
            }
        }
        return Ok(());
    }

    // Require provider for other operations
    let Some(provider) = provider else {
        bail!("Provider required. Use --list to see configured providers.");
    };

    // Delete credentials
    if delete {
        match store.delete(&provider) {
            Ok(()) => {
                println!("Credentials for '{}' deleted.", provider);
            }
            Err(vibes_models::Error::CredentialsNotFound(_)) => {
                println!("No credentials found for '{}'.", provider);
            }
            Err(e) => {
                bail!("Failed to delete credentials: {}", e);
            }
        }
        return Ok(());
    }

    // Store new credentials (interactive)
    let env_hint = get_env_var_name(&provider)
        .map(|v| format!(" (or set {})", v))
        .unwrap_or_default();

    println!("Enter API key for {}{}", provider, env_hint);

    let key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("API key")
        .interact()?;

    if key.is_empty() {
        bail!("API key cannot be empty");
    }

    store.set(&provider, &key)?;
    println!("Credentials for '{}' saved to keyring.", provider);

    Ok(())
}

/// Parse a capability string into a Capabilities filter.
fn parse_capability(cap: &str) -> Result<Capabilities> {
    let mut caps = Capabilities::default();

    match cap.to_lowercase().as_str() {
        "chat" => caps.chat = true,
        "vision" => caps.vision = true,
        "tools" | "tool_use" => caps.tools = true,
        "embeddings" | "embedding" => caps.embeddings = true,
        "streaming" | "stream" => caps.streaming = true,
        _ => bail!(
            "Unknown capability '{}'. Valid: chat, vision, tools, embeddings, streaming",
            cap
        ),
    }

    Ok(caps)
}

/// Format capabilities as a comma-separated string.
fn format_capabilities(caps: &Capabilities) -> String {
    let mut parts = Vec::new();

    if caps.chat {
        parts.push("chat");
    }
    if caps.vision {
        parts.push("vision");
    }
    if caps.tools {
        parts.push("tools");
    }
    if caps.embeddings {
        parts.push("embeddings");
    }

    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(", ")
    }
}

/// Format context window size.
fn format_context(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{}M", tokens / 1_000_000)
    } else if tokens >= 1_000 {
        format!("{}K", tokens / 1_000)
    } else {
        format!("{}", tokens)
    }
}

/// Get environment variable name for a provider.
fn get_env_var_name(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "openai" => Some("OPENAI_API_KEY"),
        "google" => Some("GOOGLE_API_KEY"),
        "groq" => Some("GROQ_API_KEY"),
        "mistral" => Some("MISTRAL_API_KEY"),
        "cohere" => Some("COHERE_API_KEY"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_capability_chat() {
        let caps = parse_capability("chat").unwrap();
        assert!(caps.chat);
        assert!(!caps.vision);
    }

    #[test]
    fn parse_capability_vision() {
        let caps = parse_capability("vision").unwrap();
        assert!(caps.vision);
        assert!(!caps.chat);
    }

    #[test]
    fn parse_capability_tools() {
        let caps = parse_capability("tools").unwrap();
        assert!(caps.tools);

        let caps = parse_capability("tool_use").unwrap();
        assert!(caps.tools);
    }

    #[test]
    fn parse_capability_embeddings() {
        let caps = parse_capability("embeddings").unwrap();
        assert!(caps.embeddings);

        let caps = parse_capability("embedding").unwrap();
        assert!(caps.embeddings);
    }

    #[test]
    fn parse_capability_unknown() {
        let result = parse_capability("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn format_capabilities_multiple() {
        let caps = Capabilities {
            chat: true,
            vision: true,
            tools: false,
            embeddings: false,
            streaming: true,
        };
        assert_eq!(format_capabilities(&caps), "chat, vision");
    }

    #[test]
    fn format_capabilities_empty() {
        let caps = Capabilities::default();
        assert_eq!(format_capabilities(&caps), "-");
    }

    #[test]
    fn format_context_millions() {
        assert_eq!(format_context(1_000_000), "1M");
        assert_eq!(format_context(2_000_000), "2M");
    }

    #[test]
    fn format_context_thousands() {
        assert_eq!(format_context(128_000), "128K");
        assert_eq!(format_context(8_000), "8K");
    }

    #[test]
    fn format_context_small() {
        assert_eq!(format_context(512), "512");
    }

    #[test]
    fn env_var_names() {
        assert_eq!(get_env_var_name("anthropic"), Some("ANTHROPIC_API_KEY"));
        assert_eq!(get_env_var_name("openai"), Some("OPENAI_API_KEY"));
        assert_eq!(get_env_var_name("unknown"), None);
    }
}

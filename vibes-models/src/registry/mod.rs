//! Model registry for discovering and cataloging available models.
//!
//! The [`ModelRegistry`] maintains a catalog of all available models across providers
//! and provides methods for querying by ID, provider, or capabilities.
//!
//! # Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use vibes_models::registry::ModelRegistry;
//! use vibes_models::Capabilities;
//!
//! let mut registry = ModelRegistry::new();
//!
//! // Register providers
//! registry.register_provider(Arc::new(anthropic_provider));
//! registry.register_provider(Arc::new(openai_provider));
//!
//! // Query models
//! let all_models = registry.list_models();
//! let vision_models = registry.find_by_capability(Capabilities { vision: true, ..Default::default() });
//! let anthropic_models = registry.find_by_provider("anthropic");
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::providers::ModelProvider;
use crate::{Capabilities, ModelId, ModelInfo, Result};

/// Registry for managing model providers and discovering available models.
///
/// The registry maintains both a collection of providers and a cached catalog
/// of all models. Call [`refresh`](ModelRegistry::refresh) to update the catalog
/// from registered providers.
#[derive(Default)]
pub struct ModelRegistry {
    providers: HashMap<String, Arc<dyn ModelProvider>>,
    models: HashMap<ModelId, ModelInfo>,
}

impl ModelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a model provider.
    ///
    /// The provider's models will be added to the catalog automatically.
    /// If a provider with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider to register
    pub fn register_provider(&mut self, provider: Arc<dyn ModelProvider>) {
        let name = provider.name().to_string();

        // Remove old models from this provider
        self.models.retain(|_, info| info.provider != name);

        // Add models from the new provider
        for model in provider.models() {
            self.models.insert(model.id.clone(), model);
        }

        self.providers.insert(name, provider);
    }

    /// Get a provider by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The provider name (e.g., "anthropic", "openai")
    ///
    /// # Returns
    ///
    /// The provider if found, None otherwise.
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn ModelProvider>> {
        self.providers.get(name).cloned()
    }

    /// List all registered provider names.
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// List all available models.
    pub fn list_models(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// Get the number of registered models.
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Get a model by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The model ID (e.g., "anthropic:claude-sonnet-4")
    ///
    /// # Returns
    ///
    /// The model info if found, None otherwise.
    pub fn get_model(&self, id: &ModelId) -> Option<&ModelInfo> {
        self.models.get(id)
    }

    /// Find models that match the given capabilities.
    ///
    /// A model matches if it has all the capabilities that are set to `true`
    /// in the filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - Capabilities filter (models must have all true capabilities)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Find models with vision support
    /// let vision_models = registry.find_by_capability(Capabilities {
    ///     vision: true,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn find_by_capability(&self, filter: Capabilities) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| m.capabilities.matches(&filter))
            .collect()
    }

    /// Find all models from a specific provider.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name
    pub fn find_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// Find models that are local (not cloud-hosted).
    pub fn find_local(&self) -> Vec<&ModelInfo> {
        self.models.values().filter(|m| m.local).collect()
    }

    /// Find models that are cloud-hosted.
    pub fn find_cloud(&self) -> Vec<&ModelInfo> {
        self.models.values().filter(|m| !m.local).collect()
    }

    /// Refresh the model catalog from all registered providers.
    ///
    /// This re-queries all providers for their available models and updates
    /// the catalog. Useful when providers may have added new models.
    pub fn refresh(&mut self) -> Result<()> {
        self.models.clear();

        for provider in self.providers.values() {
            for model in provider.models() {
                self.models.insert(model.id.clone(), model);
            }
        }

        Ok(())
    }

    /// Remove a provider and its models from the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The provider name to remove
    ///
    /// # Returns
    ///
    /// True if the provider was removed, false if it wasn't found.
    pub fn remove_provider(&mut self, name: &str) -> bool {
        if self.providers.remove(name).is_some() {
            self.models.retain(|_, info| info.provider != name);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{ChatRequest, ChatResponse, ChatStream, Content, StopReason, Usage};
    use async_trait::async_trait;

    /// A mock provider for testing.
    struct MockProvider {
        name: String,
        models: Vec<ModelInfo>,
    }

    impl MockProvider {
        fn new(name: &str, models: Vec<ModelInfo>) -> Self {
            Self {
                name: name.to_string(),
                models,
            }
        }

        fn anthropic() -> Self {
            Self::new(
                "anthropic",
                vec![
                    ModelInfo::builder("anthropic", "claude-sonnet-4")
                        .context_window(200_000)
                        .capabilities(Capabilities::full())
                        .build(),
                    ModelInfo::builder("anthropic", "claude-haiku-3")
                        .context_window(200_000)
                        .capabilities(Capabilities::chat())
                        .build(),
                ],
            )
        }

        fn openai() -> Self {
            Self::new(
                "openai",
                vec![
                    ModelInfo::builder("openai", "gpt-4o")
                        .context_window(128_000)
                        .capabilities(Capabilities::full())
                        .build(),
                    ModelInfo::builder("openai", "text-embedding-3-large")
                        .context_window(8192)
                        .capabilities(Capabilities::embeddings())
                        .build(),
                ],
            )
        }

        fn ollama() -> Self {
            Self::new(
                "ollama",
                vec![
                    ModelInfo::builder("ollama", "llama3")
                        .context_window(8192)
                        .capabilities(Capabilities::chat())
                        .local()
                        .build(),
                ],
            )
        }
    }

    #[async_trait]
    impl ModelProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn models(&self) -> Vec<ModelInfo> {
            self.models.clone()
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
            Ok(ChatResponse {
                content: Content::text("mock response"),
                stop_reason: StopReason::EndTurn,
                tool_calls: vec![],
                usage: Usage::new(10, 5),
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream> {
            use tokio_stream::iter;
            Ok(Box::pin(iter(vec![])))
        }

        fn supports_tools(&self) -> bool {
            self.models.iter().any(|m| m.capabilities.tools)
        }

        fn supports_vision(&self) -> bool {
            self.models.iter().any(|m| m.capabilities.vision)
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.model_count(), 0);
        assert!(registry.list_providers().is_empty());
    }

    #[test]
    fn register_provider_adds_models() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));

        assert_eq!(registry.model_count(), 2);
        assert!(registry.list_providers().contains(&"anthropic"));
    }

    #[test]
    fn get_provider_returns_registered_provider() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));

        let provider = registry.get_provider("anthropic");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "anthropic");

        assert!(registry.get_provider("unknown").is_none());
    }

    #[test]
    fn get_model_by_id() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));

        let id = ModelId::new("anthropic", "claude-sonnet-4");
        let model = registry.get_model(&id);

        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "claude-sonnet-4");
        assert_eq!(model.unwrap().context_window, 200_000);
    }

    #[test]
    fn find_by_capability_filters_correctly() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        registry.register_provider(Arc::new(MockProvider::openai()));

        // Find vision models
        let vision_filter = Capabilities {
            vision: true,
            ..Default::default()
        };
        let vision_models = registry.find_by_capability(vision_filter);
        assert_eq!(vision_models.len(), 2); // claude-sonnet-4 and gpt-4o

        // Find embedding models
        let embed_filter = Capabilities {
            embeddings: true,
            ..Default::default()
        };
        let embed_models = registry.find_by_capability(embed_filter);
        assert_eq!(embed_models.len(), 1);
        assert_eq!(embed_models[0].name, "text-embedding-3-large");
    }

    #[test]
    fn find_by_provider_returns_provider_models() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        registry.register_provider(Arc::new(MockProvider::openai()));

        let anthropic_models = registry.find_by_provider("anthropic");
        assert_eq!(anthropic_models.len(), 2);
        assert!(anthropic_models.iter().all(|m| m.provider == "anthropic"));

        let openai_models = registry.find_by_provider("openai");
        assert_eq!(openai_models.len(), 2);
    }

    #[test]
    fn find_local_and_cloud() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        registry.register_provider(Arc::new(MockProvider::ollama()));

        let local = registry.find_local();
        assert_eq!(local.len(), 1);
        assert_eq!(local[0].provider, "ollama");

        let cloud = registry.find_cloud();
        assert_eq!(cloud.len(), 2);
        assert!(cloud.iter().all(|m| !m.local));
    }

    #[test]
    fn refresh_updates_catalog() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));

        assert_eq!(registry.model_count(), 2);

        // Refresh should maintain the same models
        registry.refresh().unwrap();
        assert_eq!(registry.model_count(), 2);
    }

    #[test]
    fn remove_provider_removes_models() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        registry.register_provider(Arc::new(MockProvider::openai()));

        assert_eq!(registry.model_count(), 4);

        let removed = registry.remove_provider("anthropic");
        assert!(removed);
        assert_eq!(registry.model_count(), 2);
        assert!(registry.get_provider("anthropic").is_none());

        // Removing non-existent provider returns false
        assert!(!registry.remove_provider("anthropic"));
    }

    #[test]
    fn registering_same_provider_replaces_models() {
        let mut registry = ModelRegistry::new();

        // Register with 2 models
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        assert_eq!(registry.model_count(), 2);

        // Register same provider with different models
        let updated = MockProvider::new(
            "anthropic",
            vec![
                ModelInfo::builder("anthropic", "claude-opus-4")
                    .context_window(200_000)
                    .capabilities(Capabilities::full())
                    .build(),
            ],
        );
        registry.register_provider(Arc::new(updated));

        // Should only have the new model
        assert_eq!(registry.model_count(), 1);
        assert!(
            registry
                .get_model(&ModelId::new("anthropic", "claude-opus-4"))
                .is_some()
        );
        assert!(
            registry
                .get_model(&ModelId::new("anthropic", "claude-sonnet-4"))
                .is_none()
        );
    }

    #[test]
    fn list_models_returns_all() {
        let mut registry = ModelRegistry::new();
        registry.register_provider(Arc::new(MockProvider::anthropic()));
        registry.register_provider(Arc::new(MockProvider::openai()));

        let models = registry.list_models();
        assert_eq!(models.len(), 4);
    }
}

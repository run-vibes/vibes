---
id: models
title: Model Management Platform
status: planned
description: Unified model management - cloud providers, local models, registry, caching, routing
---




# Model Management Platform

Full platform for model management: cloud providers (Anthropic, OpenAI, Google, Groq), local models (Ollama, llama.cpp), with registry, auth, downloads, caching, and smart routing.

## Overview

vibes needs to orchestrate LLM inference across multiple providers and local models. Instead of building this into each component, create a unified model management layer that handles:

- **Registry**: Discover and catalog available models
- **Auth**: Secure API key and credential management
- **Downloads**: Local model weight management
- **Cache**: Response caching for cost/latency optimization
- **Routing**: Smart model selection based on task requirements
- **API**: Unified inference interface

## Module Structure

```
vibes-models/
├── registry/           # Model catalog and discovery
├── auth/               # API key and credential management
├── downloads/          # Local model weight management
├── cache/              # Response caching
├── routing/            # Smart model selection
└── api/                # Unified inference API
```

## Key Types

### Provider Trait

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream>;
    async fn embed(&self, request: EmbedRequest) -> Result<EmbedResponse>;

    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn pricing(&self, model: &str) -> Option<Pricing>;
}
```

### Model Registry

```rust
pub struct ModelRegistry {
    providers: HashMap<String, Arc<dyn ModelProvider>>,
    models: HashMap<ModelId, ModelInfo>,
}

pub struct ModelInfo {
    pub id: ModelId,
    pub provider: String,
    pub name: String,
    pub context_window: u32,
    pub max_output: Option<u32>,
    pub capabilities: Capabilities,
    pub pricing: Option<Pricing>,
    pub local: bool,
}

pub struct Capabilities {
    pub chat: bool,
    pub vision: bool,
    pub tools: bool,
    pub embeddings: bool,
    pub streaming: bool,
}
```

### Credential Management

```rust
pub struct CredentialStore {
    keyring: SystemKeyring,
    env_fallback: bool,
}

impl CredentialStore {
    pub fn get(&self, provider: &str) -> Result<ApiKey>;
    pub fn set(&self, provider: &str, key: ApiKey) -> Result<()>;
    pub fn delete(&self, provider: &str) -> Result<()>;
    pub fn list_providers(&self) -> Vec<String>;
}
```

### Smart Routing

```rust
pub struct Router {
    registry: Arc<ModelRegistry>,
    rules: Vec<RoutingRule>,
    fallbacks: HashMap<String, Vec<String>>,
}

pub enum RoutingRule {
    PreferLocal { max_context: u32 },
    CostOptimize { budget: f64 },
    LatencyOptimize { max_ms: u32 },
    CapabilityMatch { required: Capabilities },
    LoadBalance { weights: HashMap<String, f64> },
}
```

## Providers

### Cloud Providers

| Provider | Features | Priority |
|----------|----------|----------|
| Anthropic | Chat, Vision, Tools, Streaming | High |
| OpenAI | Chat, Vision, Tools, Embeddings, Streaming | High |
| Google (Gemini) | Chat, Vision, Tools, Streaming | Medium |
| Groq | Chat, Streaming (fast) | Medium |

### Local Providers

| Provider | Features | Priority |
|----------|----------|----------|
| Ollama | Chat, Embeddings, Pull models | High |
| llama.cpp | Chat, GGUF models | Medium |

## Provider Configuration

### Ollama

Ollama runs locally and requires no API key. The provider connects to Ollama's REST API.

**Default URL:** `http://localhost:11434`

**Environment Variables:**

| Variable | Description | Default |
|----------|-------------|---------|
| `OLLAMA_HOST` | Base URL for Ollama API | `http://localhost:11434` |

**Usage:**

```rust
use vibes_models::providers::OllamaProvider;

// Use default URL (localhost:11434)
let provider = OllamaProvider::new();

// Use custom URL (e.g., remote server)
let provider = OllamaProvider::with_base_url("http://192.168.1.100:11434");

// Discover installed models
provider.refresh_models().await?;
let models = provider.models();

// Chat with a model
let request = ChatRequest::new("llama3:latest", vec![
    Message::user("Hello!")
]);
let response = provider.chat(request).await?;
```

**Requirements:**

1. Install Ollama: https://ollama.ai
2. Start the server: `ollama serve`
3. Pull a model: `ollama pull llama3`

**Model Discovery:**

Models are discovered dynamically via the `/api/tags` endpoint. Call `refresh_models()` to update the cached list. All Ollama models are marked with `local: true` in `ModelInfo`.

**Error Handling:**

| Error | Cause | Solution |
|-------|-------|----------|
| Connection refused | Ollama not running | Start with `ollama serve` |
| Model not found | Model not installed | Run `ollama pull <model>` |
| Timeout | Slow inference or large model | Increase timeout or use smaller model |

## Response Caching

```rust
pub struct ResponseCache {
    store: CacheStore,
    policy: CachePolicy,
}

pub struct CachePolicy {
    pub ttl: Duration,
    pub max_entries: usize,
    pub max_size_bytes: u64,
    pub cache_embeddings: bool,
    pub cache_completions: bool,
}

pub enum CacheStore {
    InMemory(LruCache),
    File(PathBuf),
    Sqlite(SqlitePool),
}
```

## Plugin API Integration

```rust
// In vibes-plugin-api
impl PluginContext {
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream>;
    pub async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    pub fn list_models(&self) -> Vec<ModelInfo>;
}
```

## CLI Commands

```
vibes models list                    # List available models
vibes models info <model>            # Show model details
vibes models auth <provider>         # Configure API key
vibes models pull <model>            # Download local model
vibes models cache status            # Show cache stats
vibes models cache clear             # Clear response cache
```

<!-- BEGIN GENERATED -->
## Milestones

**Progress:** 1/1 milestones complete, 9/9 stories done

| ID | Milestone | Stories | Status |
|----|-----------|---------|--------|
| 37 | [Model Management](milestones/37-model-management/) | 9/9 | done |
<!-- END GENERATED -->

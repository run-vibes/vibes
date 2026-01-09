//! Embedding generation for semantic similarity search
//!
//! Provides the `Embedder` trait for generating text embeddings and
//! `LocalEmbedder` for local inference using gte-small model.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ort::session::Session;
use ort::value::Tensor;
use thiserror::Error;
use tokenizers::Tokenizer;
use tracing::{debug, instrument};

/// Errors that can occur during embedding operations
#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("Model not found at {0}")]
    ModelNotFound(String),

    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    #[error("Tokenization failed: {0}")]
    TokenizationError(String),

    #[error("Inference failed: {0}")]
    InferenceError(String),

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for embedder operations
pub type EmbedderResult<T> = Result<T, EmbedderError>;

/// Trait for generating text embeddings
///
/// Embedders convert text into fixed-dimensional vectors suitable for
/// semantic similarity search. The trait is async to support both local
/// inference and potential remote API calls.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> EmbedderResult<Vec<f32>>;

    /// Generate embeddings for multiple texts in a batch
    ///
    /// Default implementation calls `embed` sequentially; implementations
    /// should override for efficient batching.
    async fn embed_batch(&self, texts: &[&str]) -> EmbedderResult<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Returns the dimensionality of generated embeddings
    fn dimensions(&self) -> usize;
}

/// Compute cosine similarity between two embedding vectors
///
/// Returns a value in [-1, 1] where 1 means identical direction.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Embedding dimensions must match");

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Embedding dimensions for gte-small model
pub const GTE_SMALL_DIMENSIONS: usize = 384;

/// Default model cache directory
pub fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("vibes")
        .join("models")
        .join("gte-small")
}

/// Local embedder using gte-small ONNX model
///
/// Uses ONNX Runtime for inference and HuggingFace tokenizers
/// for text tokenization. Model files are expected at:
/// - `{cache_dir}/model.onnx`
/// - `{cache_dir}/tokenizer.json`
pub struct LocalEmbedder {
    session: Arc<Mutex<Session>>,
    tokenizer: Arc<Tokenizer>,
    model_dir: PathBuf,
}

impl LocalEmbedder {
    /// Create a new LocalEmbedder loading model from the specified directory
    ///
    /// The directory must contain:
    /// - `model.onnx` - ONNX model file
    /// - `tokenizer.json` - HuggingFace tokenizer config
    #[instrument(skip_all, fields(model_dir = %model_dir.as_ref().display()))]
    pub fn from_dir<P: AsRef<Path>>(model_dir: P) -> EmbedderResult<Self> {
        let model_dir = model_dir.as_ref().to_path_buf();
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            return Err(EmbedderError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }
        if !tokenizer_path.exists() {
            return Err(EmbedderError::ModelNotFound(
                tokenizer_path.display().to_string(),
            ));
        }

        debug!("Loading ONNX model from {}", model_path.display());
        let session = Session::builder()
            .map_err(|e| EmbedderError::ModelLoadError(e.to_string()))?
            .commit_from_file(&model_path)
            .map_err(|e| EmbedderError::ModelLoadError(e.to_string()))?;

        debug!("Loading tokenizer from {}", tokenizer_path.display());
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| EmbedderError::ModelLoadError(e.to_string()))?;

        debug!("LocalEmbedder initialized successfully");

        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            tokenizer: Arc::new(tokenizer),
            model_dir,
        })
    }

    /// Create a new LocalEmbedder using the default cache directory
    pub fn new() -> EmbedderResult<Self> {
        Self::from_dir(default_cache_dir())
    }

    /// Get the model directory path
    pub fn model_dir(&self) -> &Path {
        &self.model_dir
    }

    /// Check if the model files exist at the given directory
    pub fn model_exists<P: AsRef<Path>>(model_dir: P) -> bool {
        let model_dir = model_dir.as_ref();
        model_dir.join("model.onnx").exists() && model_dir.join("tokenizer.json").exists()
    }

    /// Check if the model files exist at the default cache directory
    pub fn default_model_exists() -> bool {
        Self::model_exists(default_cache_dir())
    }

    /// Download the gte-small model files to the specified directory
    ///
    /// Downloads from HuggingFace:
    /// - model.onnx (~67MB)
    /// - tokenizer.json (~700KB)
    #[instrument(skip_all, fields(model_dir = %model_dir.as_ref().display()))]
    pub async fn download_model<P: AsRef<Path>>(model_dir: P) -> EmbedderResult<()> {
        let model_dir = model_dir.as_ref();
        std::fs::create_dir_all(model_dir)?;

        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        // HuggingFace URLs for Supabase/gte-small ONNX model
        const MODEL_URL: &str =
            "https://huggingface.co/Supabase/gte-small/resolve/main/onnx/model.onnx";
        const TOKENIZER_URL: &str =
            "https://huggingface.co/Supabase/gte-small/resolve/main/tokenizer.json";

        // Download model if not present
        if !model_path.exists() {
            tracing::info!("Downloading gte-small model (~67MB)...");
            download_file(MODEL_URL, &model_path).await?;
            tracing::info!("Model downloaded to {}", model_path.display());
        } else {
            debug!("Model already exists at {}", model_path.display());
        }

        // Download tokenizer if not present
        if !tokenizer_path.exists() {
            tracing::info!("Downloading tokenizer...");
            download_file(TOKENIZER_URL, &tokenizer_path).await?;
            tracing::info!("Tokenizer downloaded to {}", tokenizer_path.display());
        } else {
            debug!("Tokenizer already exists at {}", tokenizer_path.display());
        }

        Ok(())
    }

    /// Download model to default cache directory
    pub async fn download_default_model() -> EmbedderResult<()> {
        Self::download_model(default_cache_dir()).await
    }

    /// Ensure model exists, downloading if necessary, then load it
    #[instrument(skip_all)]
    pub async fn ensure_model() -> EmbedderResult<Self> {
        let model_dir = default_cache_dir();
        if !Self::model_exists(&model_dir) {
            Self::download_model(&model_dir).await?;
        }
        // Load synchronously (model loading is fast, download is slow)
        Self::from_dir(model_dir)
    }

    /// Run a health check on the embedder
    ///
    /// Verifies the model is working by generating a test embedding.
    /// Returns model information on success.
    pub async fn health_check(&self) -> EmbedderResult<ModelInfo> {
        // Generate a test embedding to verify model works
        let test_text = "health check";
        let embedding = self.embed(test_text).await?;

        // Verify dimensions
        if embedding.len() != GTE_SMALL_DIMENSIONS {
            return Err(EmbedderError::InferenceError(format!(
                "Expected {} dimensions, got {}",
                GTE_SMALL_DIMENSIONS,
                embedding.len()
            )));
        }

        // Verify embedding is not all zeros
        let sum: f32 = embedding.iter().map(|x| x.abs()).sum();
        if sum < f32::EPSILON {
            return Err(EmbedderError::InferenceError(
                "Health check embedding is all zeros".to_string(),
            ));
        }

        Ok(ModelInfo {
            model_name: "gte-small".to_string(),
            dimensions: GTE_SMALL_DIMENSIONS,
            model_dir: self.model_dir.clone(),
        })
    }
}

/// Information about a loaded model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model name (e.g., "gte-small")
    pub model_name: String,
    /// Embedding dimensions
    pub dimensions: usize,
    /// Directory containing model files
    pub model_dir: PathBuf,
}

/// Download a file from URL to disk using atomic writes
///
/// Uses the temp-file-then-rename pattern for crash safety:
/// 1. Download to `{path}.tmp`
/// 2. `sync_all()` to flush to disk
/// 3. Atomic `rename()` to final path
///
/// This prevents partial/corrupted files if the download is interrupted
/// (network failure, process crash, power loss). The file either exists
/// completely or not at all - never in a half-written state.
async fn download_file(url: &str, path: &Path) -> EmbedderResult<()> {
    use std::io::Write;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| EmbedderError::DownloadError(format!("Failed to fetch {}: {}", url, e)))?;

    if !response.status().is_success() {
        return Err(EmbedderError::DownloadError(format!(
            "Failed to download {}: HTTP {}",
            url,
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| EmbedderError::DownloadError(format!("Failed to read response: {}", e)))?;

    // Atomic write: temp file → sync → rename
    // rename() is atomic on POSIX systems when source and dest are on same filesystem
    let temp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&temp_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    std::fs::rename(&temp_path, path)?;

    Ok(())
}

#[async_trait]
impl Embedder for LocalEmbedder {
    async fn embed(&self, text: &str) -> EmbedderResult<Vec<f32>> {
        // Run sync inference in blocking task to not block async runtime
        let text = text.to_string();
        let session = self.session.clone();
        let tokenizer = self.tokenizer.clone();

        tokio::task::spawn_blocking(move || embed_sync_impl(&session, &tokenizer, &text))
            .await
            .map_err(|e| EmbedderError::InferenceError(format!("Task join failed: {}", e)))?
    }

    async fn embed_batch(&self, texts: &[&str]) -> EmbedderResult<Vec<Vec<f32>>> {
        let texts: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let session = self.session.clone();
        let tokenizer = self.tokenizer.clone();

        tokio::task::spawn_blocking(move || {
            texts
                .iter()
                .map(|text| embed_sync_impl(&session, &tokenizer, text))
                .collect()
        })
        .await
        .map_err(|e| EmbedderError::InferenceError(format!("Task join failed: {}", e)))?
    }

    fn dimensions(&self) -> usize {
        GTE_SMALL_DIMENSIONS
    }
}

/// Sync embedding implementation used by spawn_blocking
fn embed_sync_impl(
    session: &Arc<Mutex<Session>>,
    tokenizer: &Arc<Tokenizer>,
    text: &str,
) -> EmbedderResult<Vec<f32>> {
    // Tokenize
    let encoding = tokenizer
        .encode(text, true)
        .map_err(|e| EmbedderError::TokenizationError(e.to_string()))?;

    let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
    let attention_mask: Vec<i64> = encoding
        .get_attention_mask()
        .iter()
        .map(|&m| m as i64)
        .collect();
    let token_type_ids: Vec<i64> = encoding
        .get_type_ids()
        .iter()
        .map(|&id| id as i64)
        .collect();

    let seq_len = input_ids.len();

    // Create tensors with shape [1, seq_len]
    let input_ids_tensor =
        Tensor::from_array(([1usize, seq_len], input_ids.clone())).map_err(|e| {
            EmbedderError::InferenceError(format!("Failed to create input_ids tensor: {}", e))
        })?;

    let attention_mask_tensor = Tensor::from_array(([1usize, seq_len], attention_mask.clone()))
        .map_err(|e| {
            EmbedderError::InferenceError(format!("Failed to create attention_mask tensor: {}", e))
        })?;

    let token_type_ids_tensor =
        Tensor::from_array(([1usize, seq_len], token_type_ids)).map_err(|e| {
            EmbedderError::InferenceError(format!("Failed to create token_type_ids tensor: {}", e))
        })?;

    // Run inference (lock session for mutable access)
    let mut session_guard = session
        .lock()
        .map_err(|e| EmbedderError::InferenceError(format!("Failed to lock session: {}", e)))?;
    let outputs = session_guard
        .run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
            "token_type_ids" => token_type_ids_tensor
        ])
        .map_err(|e| EmbedderError::InferenceError(format!("ONNX inference failed: {}", e)))?;

    // Extract output - gte-small outputs last_hidden_state with shape [1, seq_len, 384]
    let output = outputs.get("last_hidden_state").ok_or_else(|| {
        EmbedderError::InferenceError("No output 'last_hidden_state' found".to_string())
    })?;

    let (shape, data) = output.try_extract_tensor::<f32>().map_err(|e| {
        EmbedderError::InferenceError(format!("Failed to extract output tensor: {}", e))
    })?;

    let shape_vec: Vec<usize> = shape.iter().map(|&d| d as usize).collect();

    // Mean pooling with attention mask
    mean_pooling_impl(&shape_vec, data, &attention_mask)
}

/// Apply mean pooling to hidden states using attention mask
fn mean_pooling_impl(
    shape: &[usize],
    hidden_states: &[f32],
    attention_mask: &[i64],
) -> EmbedderResult<Vec<f32>> {
    if shape.len() != 3 {
        return Err(EmbedderError::InferenceError(format!(
            "Expected 3D tensor, got shape {:?}",
            shape
        )));
    }

    let seq_len = shape[1];
    let hidden_dim = shape[2];

    let mask_sum: f32 = attention_mask.iter().map(|&m| m as f32).sum();
    if mask_sum == 0.0 {
        return Err(EmbedderError::InferenceError(
            "Empty attention mask".to_string(),
        ));
    }

    // Data is row-major: [batch][seq][hidden] laid out contiguously
    let mut sum = vec![0.0f32; hidden_dim];
    for (i, &mask) in attention_mask.iter().enumerate() {
        if mask > 0 && i < seq_len {
            let row_start = i * hidden_dim;
            for (j, s) in sum.iter_mut().enumerate() {
                *s += hidden_states[row_start + j];
            }
        }
    }

    for s in &mut sum {
        *s /= mask_sum;
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Cosine similarity tests ---

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_range() {
        // Similarity should always be in [-1, 1]
        let a = vec![0.5, -0.3, 0.8];
        let b = vec![-0.2, 0.7, 0.1];
        let sim = cosine_similarity(&a, &b);
        assert!((-1.0..=1.0).contains(&sim));
    }

    // --- Mean pooling tests ---

    #[test]
    fn test_mean_pooling_single_token() {
        let shape = [1, 1, 3]; // batch=1, seq=1, hidden=3
        let hidden_states = [1.0, 2.0, 3.0];
        let attention_mask = [1];

        let result = mean_pooling_impl(&shape, &hidden_states, &attention_mask).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_mean_pooling_two_tokens() {
        let shape = [1, 2, 3]; // batch=1, seq=2, hidden=3
        let hidden_states = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let attention_mask = [1, 1];

        let result = mean_pooling_impl(&shape, &hidden_states, &attention_mask).unwrap();
        // Mean of [1,2,3] and [4,5,6] = [2.5, 3.5, 4.5]
        assert_eq!(result, vec![2.5, 3.5, 4.5]);
    }

    #[test]
    fn test_mean_pooling_with_padding() {
        let shape = [1, 3, 2]; // batch=1, seq=3, hidden=2
        let hidden_states = [1.0, 2.0, 3.0, 4.0, 0.0, 0.0]; // Last token is padding
        let attention_mask = [1, 1, 0]; // Only first two tokens count

        let result = mean_pooling_impl(&shape, &hidden_states, &attention_mask).unwrap();
        // Mean of [1,2] and [3,4] = [2.0, 3.0]
        assert_eq!(result, vec![2.0, 3.0]);
    }

    #[test]
    fn test_mean_pooling_empty_mask_error() {
        let shape = [1, 2, 2];
        let hidden_states = [1.0, 2.0, 3.0, 4.0];
        let attention_mask = [0, 0]; // All masked out

        let result = mean_pooling_impl(&shape, &hidden_states, &attention_mask);
        assert!(result.is_err());
    }

    #[test]
    fn test_mean_pooling_invalid_shape() {
        let shape = [1, 2]; // 2D instead of 3D
        let hidden_states = [1.0, 2.0];
        let attention_mask = [1, 1];

        let result = mean_pooling_impl(&shape, &hidden_states, &attention_mask);
        assert!(result.is_err());
    }

    // --- Utility function tests ---

    #[test]
    fn test_default_cache_dir() {
        let dir = default_cache_dir();
        assert!(dir.ends_with("gte-small"));
    }

    #[test]
    fn test_model_exists_false_for_nonexistent() {
        let dir = PathBuf::from("/nonexistent/path/to/model");
        assert!(!LocalEmbedder::model_exists(&dir));
    }

    #[test]
    fn test_model_info_debug() {
        let info = ModelInfo {
            model_name: "test".to_string(),
            dimensions: 384,
            model_dir: PathBuf::from("/test"),
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("384"));
    }

    #[test]
    fn test_embedder_error_display() {
        let err = EmbedderError::ModelNotFound("/path".to_string());
        assert!(err.to_string().contains("/path"));

        let err = EmbedderError::DownloadError("network error".to_string());
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn test_gte_small_dimensions_constant() {
        assert_eq!(GTE_SMALL_DIMENSIONS, 384);
    }

    // --- Integration tests (require model to be present) ---

    /// Helper to check if model is available for integration tests
    fn model_available() -> bool {
        LocalEmbedder::default_model_exists()
    }

    #[tokio::test]
    async fn test_embedding_dimensions() {
        if !model_available() {
            eprintln!("Skipping test_embedding_dimensions: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");
        let embedding = embedder.embed("test text").await.expect("Embedding failed");

        assert_eq!(embedding.len(), GTE_SMALL_DIMENSIONS);
    }

    #[tokio::test]
    async fn test_embedding_determinism() {
        if !model_available() {
            eprintln!("Skipping test_embedding_determinism: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");
        let text = "This is a test sentence for embedding.";

        let embedding1 = embedder.embed(text).await.expect("First embedding failed");
        let embedding2 = embedder.embed(text).await.expect("Second embedding failed");

        // Same input should produce same output
        assert_eq!(embedding1.len(), embedding2.len());
        for (a, b) in embedding1.iter().zip(embedding2.iter()) {
            assert!((a - b).abs() < 1e-6, "Embeddings differ: {} vs {}", a, b);
        }
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        if !model_available() {
            eprintln!("Skipping test_batch_embedding: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");
        let texts = ["Hello world", "Goodbye moon", "Testing embeddings"];

        let embeddings = embedder
            .embed_batch(&texts)
            .await
            .expect("Batch embedding failed");

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), GTE_SMALL_DIMENSIONS);
        }
    }

    #[tokio::test]
    async fn test_similar_texts_have_high_similarity() {
        if !model_available() {
            eprintln!("Skipping test_similar_texts_have_high_similarity: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");

        let emb1 = embedder
            .embed("The cat sat on the mat")
            .await
            .expect("Embed failed");
        let emb2 = embedder
            .embed("A cat was sitting on a mat")
            .await
            .expect("Embed failed");
        let emb3 = embedder
            .embed("Quantum mechanics describes particle behavior")
            .await
            .expect("Embed failed");

        let sim_similar = cosine_similarity(&emb1, &emb2);
        let sim_different = cosine_similarity(&emb1, &emb3);

        // Similar sentences should have higher similarity than different ones
        assert!(
            sim_similar > sim_different,
            "Expected similar texts to have higher similarity: {} vs {}",
            sim_similar,
            sim_different
        );
    }

    #[tokio::test]
    async fn test_health_check() {
        if !model_available() {
            eprintln!("Skipping test_health_check: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");
        let info = embedder.health_check().await.expect("Health check failed");

        assert_eq!(info.model_name, "gte-small");
        assert_eq!(info.dimensions, GTE_SMALL_DIMENSIONS);
    }

    #[tokio::test]
    async fn test_embedder_dimensions_method() {
        if !model_available() {
            eprintln!("Skipping test_embedder_dimensions_method: model not available");
            return;
        }

        let embedder = LocalEmbedder::new().expect("Failed to load model");
        assert_eq!(embedder.dimensions(), GTE_SMALL_DIMENSIONS);
    }
}

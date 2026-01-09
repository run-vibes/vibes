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
}

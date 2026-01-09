//! Embedding generation for semantic similarity search
//!
//! Provides the `Embedder` trait for generating text embeddings and
//! `LocalEmbedder` for local inference using gte-small model.

use async_trait::async_trait;
use thiserror::Error;

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

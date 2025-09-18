//! Unified Input Layer for Zeta Reticula
//! 
//! This module provides a unified interface for handling both safetensors and JSON models
//! from Hugging Face, eliminating duplication across the codebase.

pub mod embedding;
pub mod tokenizer;
pub mod model_loader;
pub mod input_processor;
pub mod error;

pub use embedding::{EmbeddingLayer, EmbeddingConfig};
pub use tokenizer::{UnifiedTokenizer, TokenizerConfig};
pub use model_loader::{ModelLoader, ModelFormat, ModelConfig};
pub use input_processor::{InputProcessor, ProcessedInput};
pub use error::{UnifiedInputError, Result};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Main unified input layer that coordinates all input processing
#[derive(Debug)]
pub struct UnifiedInputLayer {
    tokenizer: Arc<RwLock<UnifiedTokenizer>>,
    embedding: Arc<RwLock<EmbeddingLayer>>,
    model_loader: Arc<ModelLoader>,
    processor: Arc<InputProcessor>,
}

impl UnifiedInputLayer {
    /// Create a new unified input layer
    pub async fn new(config: UnifiedInputConfig) -> Result<Self> {
        let tokenizer = Arc::new(RwLock::new(UnifiedTokenizer::new(config.tokenizer_config).await?));
        let embedding = Arc::new(RwLock::new(EmbeddingLayer::new(config.embedding_config).await?));
        let model_loader = Arc::new(ModelLoader::new(config.model_config).await?);
        let processor = Arc::new(InputProcessor::new(
            Arc::clone(&tokenizer),
            Arc::clone(&embedding),
            Arc::clone(&model_loader),
        ).await?);

        Ok(Self {
            tokenizer,
            embedding,
            model_loader,
            processor,
        })
    }

    /// Process input text through the unified pipeline
    pub async fn process_input(&self, text: &str, model_path: &str) -> Result<ProcessedInput> {
        self.processor.process(text, model_path).await
    }

    /// Load a model from either safetensors or JSON format
    pub async fn load_model(&self, path: &str) -> Result<()> {
        self.model_loader.load_model(path).await
    }

    /// Get tokenizer reference
    pub fn tokenizer(&self) -> Arc<RwLock<UnifiedTokenizer>> {
        Arc::clone(&self.tokenizer)
    }

    /// Get embedding layer reference
    pub fn embedding(&self) -> Arc<RwLock<EmbeddingLayer>> {
        Arc::clone(&self.embedding)
    }
}

/// Configuration for the unified input layer
#[derive(Debug, Clone)]
pub struct UnifiedInputConfig {
    pub tokenizer_config: TokenizerConfig,
    pub embedding_config: EmbeddingConfig,
    pub model_config: ModelConfig,
}

impl Default for UnifiedInputConfig {
    fn default() -> Self {
        Self {
            tokenizer_config: TokenizerConfig::default(),
            embedding_config: EmbeddingConfig::default(),
            model_config: ModelConfig::default(),
        }
    }
}

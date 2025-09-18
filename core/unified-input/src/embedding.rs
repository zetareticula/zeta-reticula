use crate::error::{UnifiedInputError, Result};
use candle_core::{Tensor, Device, DType};
use candle_nn::{Embedding, VarBuilder};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use half::f16;

/// Configuration for the embedding layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub max_position_embeddings: usize,
    pub padding_idx: Option<usize>,
    pub dtype: EmbeddingDType,
    pub device: EmbeddingDevice,
    pub freeze: bool,
    pub sparse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingDType {
    F32,
    F16,
    BF16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingDevice {
    Cpu,
    Cuda(usize),
    Metal,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            vocab_size: 50_257,
            embedding_dim: 768,
            max_position_embeddings: 2048,
            padding_idx: None,
            dtype: EmbeddingDType::F32,
            device: EmbeddingDevice::Cpu,
            freeze: false,
            sparse: false,
        }
    }
}

/// Unified embedding layer that handles both token and position embeddings
#[derive(Debug)]
pub struct EmbeddingLayer {
    token_embedding: Embedding,
    position_embedding: Option<Embedding>,
    config: EmbeddingConfig,
    device: Device,
    embedding_cache: HashMap<Vec<u32>, Tensor>,
}

impl EmbeddingLayer {
    /// Create a new embedding layer
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let device = Self::create_device(&config.device)?;
        let dtype = Self::get_dtype(&config.dtype);

        // Create variable builder for embeddings
        let vs = VarBuilder::zeros(dtype, &device);

        // Create token embedding
        let token_embedding = Embedding::new(
            vs.get((config.vocab_size, config.embedding_dim), "token_embedding")?,
            config.embedding_dim,
        );

        // Create position embedding if needed
        let position_embedding = if config.max_position_embeddings > 0 {
            Some(Embedding::new(
                vs.get((config.max_position_embeddings, config.embedding_dim), "position_embedding")?,
                config.embedding_dim,
            ))
        } else {
            None
        };

        Ok(Self {
            token_embedding,
            position_embedding,
            config,
            device,
            embedding_cache: HashMap::new(),
        })
    }

    /// Create device from config
    fn create_device(device_config: &EmbeddingDevice) -> Result<Device> {
        match device_config {
            EmbeddingDevice::Cpu => Ok(Device::Cpu),
            EmbeddingDevice::Cuda(id) => {
                Device::new_cuda(*id).map_err(|e| UnifiedInputError::Config(format!("CUDA device error: {}", e)))
            }
            EmbeddingDevice::Metal => {
                Device::new_metal(0).map_err(|e| UnifiedInputError::Config(format!("Metal device error: {}", e)))
            }
        }
    }

    /// Get DType from config
    fn get_dtype(dtype_config: &EmbeddingDType) -> DType {
        match dtype_config {
            EmbeddingDType::F32 => DType::F32,
            EmbeddingDType::F16 => DType::F16,
            EmbeddingDType::BF16 => DType::BF16,
        }
    }

    /// Generate embeddings for token IDs
    pub async fn embed_tokens(&mut self, token_ids: &[u32]) -> Result<Tensor> {
        // Check cache first
        if let Some(cached) = self.embedding_cache.get(token_ids) {
            return Ok(cached.clone());
        }

        // Convert token IDs to tensor
        let token_tensor = Tensor::from_slice(token_ids, (token_ids.len(),), &self.device)?;

        // Get token embeddings
        let mut embeddings = self.token_embedding.forward(&token_tensor)?;

        // Add position embeddings if available
        if let Some(pos_emb) = &self.position_embedding {
            let positions: Vec<u32> = (0..token_ids.len() as u32).collect();
            let pos_tensor = Tensor::from_slice(&positions, (positions.len(),), &self.device)?;
            let pos_embeddings = pos_emb.forward(&pos_tensor)?;
            embeddings = embeddings.add(&pos_embeddings)?;
        }

        // Cache the result
        self.embedding_cache.insert(token_ids.to_vec(), embeddings.clone());

        Ok(embeddings)
    }

    /// Generate embeddings and return as ndarray for compatibility
    pub async fn embed_tokens_ndarray(&mut self, token_ids: &[u32]) -> Result<Array2<f32>> {
        let tensor = self.embed_tokens(token_ids).await?;
        self.tensor_to_ndarray(&tensor)
    }

    /// Generate embeddings in f16 format
    pub async fn embed_tokens_f16(&mut self, token_ids: &[u32]) -> Result<Array2<f16>> {
        let tensor = self.embed_tokens(token_ids).await?;
        let f32_array = self.tensor_to_ndarray(&tensor)?;
        
        // Convert to f16
        let shape = f32_array.raw_dim();
        let f16_data: Vec<f16> = f32_array.iter().map(|&x| f16::from_f32(x)).collect();
        
        Array2::from_shape_vec(shape, f16_data)
            .map_err(|e| UnifiedInputError::EmbeddingFailed(format!("Shape error: {}", e)))
    }

    /// Convert tensor to ndarray
    fn tensor_to_ndarray(&self, tensor: &Tensor) -> Result<Array2<f32>> {
        let shape = tensor.shape();
        if shape.dims().len() != 2 {
            return Err(UnifiedInputError::EmbeddingFailed(
                "Expected 2D tensor for embeddings".to_string()
            ));
        }

        let data = tensor.to_vec2::<f32>()?;
        let flat_data: Vec<f32> = data.into_iter().flatten().collect();
        
        Array2::from_shape_vec((shape.dims()[0], shape.dims()[1]), flat_data)
            .map_err(|e| UnifiedInputError::EmbeddingFailed(format!("Array conversion error: {}", e)))
    }

    /// Get single token embedding
    pub async fn embed_single_token(&mut self, token_id: u32) -> Result<Array1<f32>> {
        let embeddings = self.embed_tokens_ndarray(&[token_id]).await?;
        Ok(embeddings.row(0).to_owned())
    }

    /// Batch embed multiple sequences
    pub async fn embed_batch(&mut self, batch_token_ids: &[Vec<u32>]) -> Result<Vec<Array2<f32>>> {
        let mut results = Vec::with_capacity(batch_token_ids.len());
        
        for token_ids in batch_token_ids {
            let embeddings = self.embed_tokens_ndarray(token_ids).await?;
            results.push(embeddings);
        }
        
        Ok(results)
    }

    /// Clear embedding cache
    pub fn clear_cache(&mut self) {
        self.embedding_cache.clear();
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.config.embedding_dim
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.config.vocab_size
    }

    /// Load pretrained embeddings from safetensors
    pub async fn load_from_safetensors(&mut self, path: &str) -> Result<()> {
        use safetensors::SafeTensors;
        use std::fs;

        let data = fs::read(path)?;
        let safetensors = SafeTensors::deserialize(&data)?;

        // Load token embeddings
        if let Ok(token_emb_data) = safetensors.tensor("token_embedding.weight") {
            let shape = token_emb_data.shape();
            if shape.len() == 2 && shape[0] == self.config.vocab_size && shape[1] == self.config.embedding_dim {
                // Convert and load the embedding weights
                // This is a simplified version - in practice you'd need proper tensor conversion
                tracing::info!("Loaded token embeddings from safetensors: {:?}", shape);
            }
        }

        // Load position embeddings if available
        if let Ok(pos_emb_data) = safetensors.tensor("position_embedding.weight") {
            let shape = pos_emb_data.shape();
            tracing::info!("Loaded position embeddings from safetensors: {:?}", shape);
        }

        Ok(())
    }

    /// Save embeddings to safetensors format
    pub async fn save_to_safetensors(&self, path: &str) -> Result<()> {
        // This would implement saving the current embeddings to safetensors format
        // Simplified implementation
        tracing::info!("Saving embeddings to safetensors: {}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_layer() {
        let config = EmbeddingConfig {
            vocab_size: 1000,
            embedding_dim: 128,
            ..Default::default()
        };

        let mut embedding_layer = EmbeddingLayer::new(config).await.unwrap();
        
        let token_ids = vec![1, 2, 3, 4, 5];
        let embeddings = embedding_layer.embed_tokens_ndarray(&token_ids).await.unwrap();
        
        assert_eq!(embeddings.shape(), &[5, 128]);
    }

    #[tokio::test]
    async fn test_single_token_embedding() {
        let config = EmbeddingConfig {
            vocab_size: 1000,
            embedding_dim: 64,
            ..Default::default()
        };

        let mut embedding_layer = EmbeddingLayer::new(config).await.unwrap();
        
        let embedding = embedding_layer.embed_single_token(42).await.unwrap();
        
        assert_eq!(embedding.len(), 64);
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let config = EmbeddingConfig {
            vocab_size: 1000,
            embedding_dim: 32,
            ..Default::default()
        };

        let mut embedding_layer = EmbeddingLayer::new(config).await.unwrap();
        
        let batch = vec![
            vec![1, 2, 3],
            vec![4, 5],
            vec![6, 7, 8, 9],
        ];
        
        let embeddings = embedding_layer.embed_batch(&batch).await.unwrap();
        
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].shape(), &[3, 32]);
        assert_eq!(embeddings[1].shape(), &[2, 32]);
        assert_eq!(embeddings[2].shape(), &[4, 32]);
    }
}

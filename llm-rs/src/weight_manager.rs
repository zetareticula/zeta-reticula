//! Weight Manager for handling model weights with quantization and distributed storage

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use futures::future::join_all;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

use crate::metrics::MetricsRecorder;
use crate::petri_net::PetriNet;
use crate::quantizer::{QuantizationConfig, Quantizer, QuantizationError};
use crate::rl_optimizer::{RLOptimizer, RLOptimizerConfig};

#[derive(Error, Debug)]
pub enum WeightManagerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Quantization error: {0}")]
    QuantizationError(#[from] QuantizationError),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightManagerConfig {
    /// Base directory for storing weights
    pub base_dir: PathBuf,
    
    /// Enable weight quantization
    pub enable_quantization: bool,
    
    /// Quantization configuration
    pub quantization_config: QuantizationConfig,
    
    /// Enable RL-based bit-depth optimization
    pub enable_rl_optimization: bool,
    
    /// RL optimizer configuration
    pub rl_optimizer_config: RLOptimizerConfig,
    
    /// Number of shards for distributed storage
    pub num_shards: usize,
    
    /// Replication factor
    pub replication_factor: usize,
    
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Enable audit logging
    pub enable_audit_log: bool,
}

impl Default for WeightManagerConfig {
    fn default() -> Self {
        let mut base_dir = std::env::temp_dir();
        base_dir.push("zeta_reticula");
        base_dir.push("weights");
        
        Self {
            base_dir,
            enable_quantization: true,
            quantization_config: QuantizationConfig::default(),
            enable_rl_optimization: true,
            rl_optimizer_config: RLOptimizerConfig::default(),
            num_shards: 4,
            replication_factor: 2,
            enable_metrics: true,
            enable_audit_log: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightMetadata {
    pub model_id: String,
    pub version: String,
    pub total_size: u64,
    pub shard_size: u64,
    pub num_shards: usize,
    pub quantization_bits: Option<u8>,
    pub created_at: u64,
    pub updated_at: u64,
    pub checksum: String,
    pub tags: Vec<String>,
    pub custom_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WeightShard {
    pub shard_id: usize,
    pub data: Vec<u8>,
    pub metadata: WeightShardMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightShardMetadata {
    pub model_id: String,
    pub version: String,
    pub shard_id: usize,
    pub shard_size: u64,
    pub total_shards: usize,
    pub quantization_bits: Option<u8>,
    pub checksum: String,
    pub replica_nodes: Vec<String>,
}

/// Manages model weights with support for quantization and distributed storage
pub struct WeightManager {
    config: WeightManagerConfig,
    quantizer: Arc<Quantizer>,
    rl_optimizer: Option<Arc<RwLock<RLOptimizer>>>,
    metrics: Option<Arc<MetricsRecorder>>,
    audit_log: Option<Arc<PetriNet>>,
    shard_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl WeightManager {
    /// Create a new WeightManager with the given configuration
    pub fn new(config: WeightManagerConfig) -> Result<Self> {
        // Create base directory if it doesn't exist
        if !config.base_dir.exists() {
            std::fs::create_dir_all(&config.base_dir)
                .context("Failed to create base directory")?;
        }
        
        // Validate config
        if config.num_shards == 0 {
            return Err(WeightManagerError::InvalidConfig(
                "num_shards must be greater than 0".to_string()
            ).into());
        }
        
        // Validate replication factor
        if config.replication_factor == 0 {
            return Err(WeightManagerError::InvalidConfig(
                "replication_factor must be greater than 0".to_string()
            ).into());
        }
        
        // Initialize quantizer
        let quantizer = Arc::new(Quantizer::new(config.quantization_config.clone()));
        
        //if rl optimizer is enabled, initialize it
        let rl_optimizer = if config.enable_rl_optimization {
            Some(Arc::new(RwLock::new(
                RLOptimizer::new(config.rl_optimizer_config.clone())
            )))
        } else {
            None
        };
        
        let metrics = if config.enable_metrics {
            Some(Arc::new(MetricsRecorder::new("weight_manager")))
        } else {
            None
        };
        
        let audit_log = if config.enable_audit_log {
            Some(Arc::new(PetriNet::new()))
        } else {
            None
        };
        
        Ok(Self {
            config,
            quantizer,
            rl_optimizer,
            metrics,
            audit_log,
            shard_locks: DashMap::new(),
        })
    }
    
    /// Store model weights
    #[instrument(skip(self, weights))]
    pub async fn store_weights(
        &self,
        model_id: &str,
        version: &str,
        weights: &[u8],
        quantize: bool,
    ) -> Result<WeightMetadata> {
        let start_time = std::time::Instant::now();
        
        // Create model directory
        let model_dir = self.get_model_dir(model_id, version);
        if model_dir.exists() {
            return Err(WeightManagerError::AlreadyExists(
                format!("Weights for model {} version {} already exist", model_id, version)
            ).into());
        }
        
        fs::create_dir_all(&model_dir).await?;
        
        // Determine optimal bit depth if RL optimization is enabled
        let bit_depth = if quantize && self.config.enable_rl_optimization {
            if let Some(rl_optimizer) = &self.rl_optimizer {
                let state = self.compute_state(weights);
                let action = rl_optimizer.write().select_action(&state)?;
                // Map action to bit depth (4, 6, 8 bits)
                match action {
                    0 => 4,
                    1 => 6,
                    _ => 8,
                }
            } else {
                8
            }
        } else if quantize {
            8 // Default to 8-bit quantization
        } else {
            32 // No quantization
        };
        
        // Quantize weights if requested
        let (quantized_weights, scale, zero_point) = if quantize {
            self.quantize_weights(weights, bit_depth).await?
        } else {
            (weights.to_vec(), 1.0, 0.0)
        };
        
        // Split weights into shards
        let shard_size = (quantized_weights.len() + self.config.num_shards - 1) / self.config.num_shards;
        let shards = self.split_into_shards(&quantized_weights, shard_size);
        
        // Store shards in parallel
        let mut handles = Vec::new();
        for (shard_id, shard_data) in shards.into_iter().enumerate() {
            let model_id = model_id.to_string();
            let version = version.to_string();
            let shard_path = self.get_shard_path(&model_id, version.as_str(), shard_id);
            let shard_metadata = WeightShardMetadata {
                model_id: model_id.clone(),
                version: version.clone(),
                shard_id,
                shard_size: shard_data.len() as u64,
                total_shards: self.config.num_shards,
                quantization_bits: if quantize { Some(bit_depth) } else { None },
                checksum: self.compute_checksum(&shard_data),
                replica_nodes: Vec::new(),
            };
            
            handles.push(tokio::spawn(
                self.store_shard(shard_path, shard_data, shard_metadata)
            ));
        }
        
        // Wait for all shards to be stored
        let shard_metadatas: Result<Vec<WeightShardMetadata>> = join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| WeightManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to store shard: {}", e)
            )))?;
        
        // Create and store metadata
        let metadata = WeightMetadata {
            model_id: model_id.to_string(),
            version: version.to_string(),
            total_size: quantized_weights.len() as u64,
            shard_size: shard_size as u64,
            num_shards: self.config.num_shards,
            quantization_bits: if quantize { Some(bit_depth) } else { None },
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            updated_at: 0,
            checksum: self.compute_checksum(weights),
            tags: Vec::new(),
            custom_metadata: HashMap::new(),
        };
        
        self.store_metadata(&model_dir, &metadata).await?;
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_histogram(
                "store_weights_duration_seconds",
                start_time.elapsed().as_secs_f64(),
            );
            metrics.increment_counter("weights_stored");
        }
        
        Ok(metadata)
    }
    
    /// Load model weights
    #[instrument(skip(self))]
    pub async fn load_weights(
        &self,
        model_id: &str,
        version: &str,
    ) -> Result<Vec<u8>> {
        let start_time = std::time::Instant::now();
        let model_dir = self.get_model_dir(model_id, version);
        
        // Load metadata
        let metadata = self.load_metadata(&model_dir).await?;
        
        // Load shards in parallel
        let mut handles = Vec::new();
        for shard_id in 0..metadata.num_shards {
            let model_id = model_id.to_string();
            let version = version.to_string();
            let shard_path = self.get_shard_path(&model_id, &version, shard_id);
            
            handles.push(tokio::spawn(
                self.load_shard(shard_path)
            ));
        }
        
        // Wait for all shards to be loaded
        let shards: Result<Vec<WeightShard>> = join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| WeightManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load shard: {}", e)
            )))?;
        
        // Combine shards
        let mut combined = Vec::with_capacity(metadata.total_size as usize);
        for shard in shards {
            combined.extend_from_slice(&shard.data);
        }
        
        // Dequantize if needed
        let weights = if let Some(bit_depth) = metadata.quantization_bits {
            self.dequantize_weights(&combined, bit_depth).await?
        } else {
            combined
        };
        
        // Verify checksum
        let checksum = self.compute_checksum(&weights);
        if checksum != metadata.checksum {
            return Err(WeightManagerError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Checksum mismatch"
            )).into());
        }
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_histogram(
                "load_weights_duration_seconds",
                start_time.elapsed().as_secs_f64(),
            );
            metrics.increment_counter("weights_loaded");
        }
        
        Ok(weights)
    }
    
    /// Delete model weights
    #[instrument(skip(self))]
    pub async fn delete_weights(&self, model_id: &str, version: &str) -> Result<()> {
        let model_dir = self.get_model_dir(model_id, version);
        
        if !model_dir.exists() {
            return Err(WeightManagerError::NotFound(
                format!("Weights for model {} version {} not found", model_id, version)
            ).into());
        }
        
        // Delete all shards
        let metadata = self.load_metadata(&model_dir).await?;
        
        let mut handles = Vec::new();
        for shard_id in 0..metadata.num_shards {
            let shard_path = self.get_shard_path(model_id, version, shard_id);
            handles.push(tokio::spawn(async move {
                fs::remove_file(shard_path).await
            }));
        }
        
        // Wait for all shards to be deleted
        for handle in handles {
            handle.await??;
        }
        
        // Delete metadata
        fs::remove_file(model_dir.join("metadata.json")).await?;
        
        // Try to remove the directory (may fail if not empty)
        let _ = fs::remove_dir(model_dir).await;
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.increment_counter("weights_deleted");
        }
        
        Ok(())
    }
    
    /// List all versions of a model
    pub async fn list_versions(&self, model_id: &str) -> Result<Vec<String>> {
        let model_base_dir = self.config.base_dir.join(model_id);
        
        if !model_base_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut entries = fs::read_dir(&model_base_dir).await?;
        let mut versions = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(version) = entry.file_name().to_str() {
                    versions.push(version.to_string());
                }
            }
        }
        
        Ok(versions)
    }
    
    /// Get the path to a model's directory
    fn get_model_dir(&self, model_id: &str, version: &str) -> PathBuf {
        self.config.base_dir
            .join(sanitize_filename::sanitize(model_id))
            .join(sanitize_filename::sanitize(version))
    }
    
    /// Get the path to a shard file
    fn get_shard_path(&self, model_id: &str, version: &str, shard_id: usize) -> PathBuf {
        self.get_model_dir(model_id, version)
            .join(format!("shard_{:04}.bin", shard_id))
    }
    
    /// Split data into shards
    fn split_into_shards(&self, data: &[u8], shard_size: usize) -> Vec<Vec<u8>> {
        data.chunks(shard_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
    
    /// Compute a simple checksum for data validation
    fn compute_checksum(&self, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Store a shard to disk
    async fn store_shard(
        &self,
        path: PathBuf,
        data: Vec<u8>,
        metadata: WeightShardMetadata,
    ) -> Result<WeightShardMetadata> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Write shard data
        fs::write(&path, &data).await?;
        
        // Write shard metadata
        let metadata_path = path.with_extension("json");
        let metadata_json = serde_json::to_vec_pretty(&metadata)?;
        fs::write(metadata_path, metadata_json).await?;
        
        Ok(metadata)
    }
    
    /// Load a shard from disk
    async fn load_shard(&self, path: PathBuf) -> Result<WeightShard> {
        // Read shard data
        let data = fs::read(&path).await?;
        
        // Read shard metadata
        let metadata_path = path.with_extension("json");
        let metadata_json = fs::read_to_string(metadata_path).await?;
        let metadata: WeightShardMetadata = serde_json::from_str(&metadata_json)?;
        
        Ok(WeightShard {
            shard_id: metadata.shard_id,
            data,
            metadata,
        })
    }
    
    /// Store metadata to disk
    async fn store_metadata(&self, model_dir: &Path, metadata: &WeightMetadata) -> Result<()> {
        let metadata_path = model_dir.join("metadata.json");
        let metadata_json = serde_json::to_vec_pretty(metadata)?;
        fs::write(metadata_path, metadata_json).await?;
        Ok(())
    }
    
    /// Load metadata from disk
    async fn load_metadata(&self, model_dir: &Path) -> Result<WeightMetadata> {
        let metadata_path = model_dir.join("metadata.json");
        let metadata_json = fs::read_to_string(metadata_path).await?;
        let metadata: WeightMetadata = serde_json::from_str(&metadata_json)?;
        Ok(metadata)
    }
    
    /// Quantize weights to the specified bit depth
    async fn quantize_weights(
        &self,
        weights: &[u8],
        bit_depth: u8,
    ) -> Result<(Vec<u8>, f32, f32)> {
        if bit_depth >= 8 {
            return Ok((weights.to_vec(), 1.0, 0.0));
        }
        
        // Convert to f32 for quantization
        let weights_f32: Vec<f32> = weights
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        
        // Create a 2D array for quantization
        let weights_2d = ndarray::Array2::from_shape_vec(
            (weights_f32.len(), 1),
            weights_f32,
        )?;
        
        // Quantize
        let (quantized, scale, zero_point) = self.quantizer.quantize_2d(
            &weights_2d,
            bit_depth,
        )?;
        
        // Convert back to bytes
        let mut result = Vec::with_capacity(quantized.len());
        for &val in quantized.iter() {
            result.push(val);
        }
        
        Ok((result, scale, zero_point))
    }
    
    /// Dequantize weights from the specified bit depth
    async fn dequantize_weights(
        &self,
        quantized: &[u8],
        bit_depth: u8,
    ) -> Result<Vec<u8>> {
        if bit_depth >= 8 {
            return Ok(quantized.to_vec());
        }
        
        // Convert to 2D array for dequantization
        let quantized_2d = ndarray::Array2::from_shape_vec(
            (quantized.len(), 1),
            quantized.to_vec(),
        )?;
        
        // Dequantize
        let dequantized = self.quantizer.dequantize_2d(
            &quantized_2d,
            bit_depth,
            1.0,  // Scale (should be loaded from metadata)
            0.0,  // Zero point (should be loaded from metadata)
        )?;
        
        // Convert back to bytes
        let mut result = Vec::with_capacity(dequantized.len() * 4);
        for &val in dequantized.iter() {
            result.extend_from_slice(&val.to_le_bytes());
        }
        
        Ok(result)
    }
    
    /// Compute state for RL optimization
    fn compute_state(&self, weights: &[u8]) -> Vec<f32> {
        // Simple feature extraction for RL state
        // In a real implementation, this would be more sophisticated
        let mean = weights.iter().map(|&x| x as f32).sum::<f32>() / weights.len() as f32;
        let var = weights.iter()
            .map(|&x| (x as f32 - mean).powi(2))
            .sum::<f32>() / weights.len() as f32;
        let min = weights.iter().min().map(|&x| x as f32).unwrap_or(0.0);
        let max = weights.iter().max().map(|&x| x as f32).unwrap_or(0.0);
        
        vec![
            mean,
            var.sqrt(), // stddev
            min,
            max,
            (max - min) / 255.0, // normalized range
            weights.len() as f32, // size
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_weight_manager() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let config = WeightManagerConfig {
            base_dir: temp_dir.path().to_path_buf(),
            enable_quantization: true,
            quantization_config: QuantizationConfig::default(),
            enable_rl_optimization: false,
            rl_optimizer_config: RLOptimizerConfig::default(),
            num_shards: 2,
            replication_factor: 1,
            enable_metrics: false,
            enable_audit_log: false,
        };
        
        let weight_manager = WeightManager::new(config)?;
        
        // Generate some test weights (random floats)
        let num_weights = 1000;
        let mut weights = Vec::with_capacity(num_weights * 4);
        for _ in 0..num_weights {
            let val = rand::random::<f32>();
            weights.extend_from_slice(&val.to_le_bytes());
        }
        
        // Test storing weights
        let model_id = "test_model";
        let version = "1.0";
        
        let metadata = weight_manager.store_weights(
            model_id,
            version,
            &weights,
            true, // quantize
        ).await?;
        
        assert_eq!(metadata.model_id, model_id);
        assert_eq!(metadata.version, version);
        assert_eq!(metadata.num_shards, 2);
        assert!(metadata.quantization_bits.is_some());
        
        // Test loading weights
        let loaded_weights = weight_manager.load_weights(model_id, version).await?;
        assert_eq!(loaded_weights, weights);
        
        // Test listing versions
        let versions = weight_manager.list_versions(model_id).await?;
        assert_eq!(versions, vec![version.to_string()]);
        
        // Test deleting weights
        weight_manager.delete_weights(model_id, version).await?;
        
        // Verify deletion
        let versions = weight_manager.list_versions(model_id).await?;
        assert!(versions.is_empty());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_rl_optimization() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let config = WeightManagerConfig {
            base_dir: temp_dir.path().to_path_buf(),
            enable_quantization: true,
            quantization_config: QuantizationConfig::default(),
            enable_rl_optimization: true,
            rl_optimizer_config: RLOptimizerConfig {
                state_dim: 10,
                action_dim: 4,
                learning_rate: 1e-3,
                gamma: 0.99,
                epsilon: 1.0,
                epsilon_min: 0.01,
                epsilon_decay: 0.995,
                batch_size: 32,
                memory_capacity: 1000,
                ..Default::default()
            },
            num_shards: 2,
            replication_factor: 1,
            enable_metrics: true,
            enable_audit_log: false,
        };
        
        let weight_manager = WeightManager::new(config)?;
        
        // Generate some test weights
        let num_weights = 1000;
        let mut weights = Vec::with_capacity(num_weights * 4);
        for _ in 0..num_weights {
            let val = rand::random::<f32>();
            weights.extend_from_slice(&val.to_le_bytes());
        }
        
        // Test storing with RL optimization
        let model_id = "test_rl_model";
        let version = "1.0";
        
        let metadata = weight_manager.store_weights(
            model_id,
            version,
            &weights,
            true, // quantize
        ).await?;
        
        assert!(metadata.quantization_bits.is_some());
        
        // Clean up
        weight_manager.delete_weights(model_id, version).await?;
        
        Ok(())
    }
}

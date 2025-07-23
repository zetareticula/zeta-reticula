//! KV Cache Manager with RL-based optimization and lock-free operations

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use parking_lot::RwLock;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, warn};

use crate::kv_cache::{KVCache, KVCacheConfig, KVCacheError, KVCacheMetrics};
use crate::metrics::MetricsRecorder;
use crate::petri_net::PetriNet;
use crate::quantizer::{QuantizationConfig, Quantizer};
use crate::rl_optimizer::{RLOptimizer, RLOptimizerConfig};

#[derive(Error, Debug)]
pub enum KVCacheManagerError {
    #[error("Cache error: {0}")]
    CacheError(#[from] KVCacheError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation timed out")]
    Timeout,
    
    #[error("Operation canceled")]
    Canceled,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheManagerConfig {
    /// Maximum number of cache entries
    pub max_entries: usize,
    
    /// Number of shards for the cache
    pub num_shards: usize,
    
    /// Default time-to-live for cache entries in seconds
    pub default_ttl: u64,
    
    /// Enable RL-based bit-depth optimization
    pub enable_rl_optimization: bool,
    
    /// Configuration for the RL optimizer
    pub rl_optimizer_config: RLOptimizerConfig,
    
    /// Configuration for quantization
    pub quantization_config: QuantizationConfig,
    
    /// Maximum batch size for batch operations
    pub max_batch_size: usize,
    
    /// Maximum queue size for async operations
    pub max_queue_size: usize,
    
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Enable audit logging
    pub enable_audit_log: bool,
}

impl Default for KVCacheManagerConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            num_shards: 16,
            default_ttl: 3600, // 1 hour
            enable_rl_optimization: true,
            rl_optimizer_config: RLOptimizerConfig::default(),
            quantization_config: QuantizationConfig::default(),
            max_batch_size: 100,
            max_queue_size: 10_000,
            enable_metrics: true,
            enable_audit_log: true,
        }
    }
}

/// A batch of operations to be processed
#[derive(Debug)]
pub enum KVCacheBatchOp {
    Get {
        key: Vec<u8>,
        tx: oneshot::Sender<Result<Option<Vec<u8>>, KVCacheManagerError>>,
    },
    Set {
        key: Vec<u8>,
        value: Vec<u8>,
        ttl: Option<u64>,
        tx: oneshot::Sender<Result<(), KVCacheManagerError>>,
    },
    Delete {
        key: Vec<u8>,
        tx: oneshot::Sender<Result<bool, KVCacheManagerError>>,
    },
    Flush {
        tx: oneshot::Sender<Result<(), KVCacheManagerError>>,
    },
}

/// Manages multiple KV caches with RL-based optimization
pub struct KVCacheManager {
    /// Configuration
    config: KVCacheManagerConfig,
    
    /// Underlying KV caches
    caches: DashMap<String, Arc<KVCache>>,
    
    /// RL optimizers (one per cache)
    rl_optimizers: DashMap<String, Arc<RwLock<RLOptimizer>>>,
    
    /// Quantizer for compression
    quantizer: Arc<Quantizer>,
    
    /// Metrics recorder
    metrics: Option<Arc<MetricsRecorder>>,
    
    /// Audit log (Petri net)
    audit_log: Option<Arc<PetriNet>>,
    
    /// Background task handles
    bg_tasks: RwLock<Vec<tokio::task::JoinHandle<()>>>,
    
    /// Shutdown signal
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl KVCacheManager {
    /// Create a new KVCacheManager with the given configuration
    pub fn new(config: KVCacheManagerConfig) -> Result<Self> {
        if config.num_shards == 0 {
            return Err(KVCacheManagerError::InvalidConfig(
                "num_shards must be greater than 0".to_string()
            ).into());
        }
        
        if config.max_entries == 0 {
            return Err(KVCacheManagerError::InvalidConfig(
                "max_entries must be greater than 0".to_string()
            ).into());
        }
        
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        let metrics = if config.enable_metrics {
            Some(Arc::new(MetricsRecorder::new("kv_cache")))
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
            caches: DashMap::new(),
            rl_optimizers: DashMap::new(),
            quantizer: Arc::new(Quantizer::new(QuantizationConfig::default())),
            metrics,
            audit_log,
            bg_tasks: RwLock::new(Vec::new()),
            shutdown_tx,
            shutdown_rx,
        })
    }
    
    /// Start background tasks
    pub fn start(&self) -> Result<()> {
        // Start metrics collection task if enabled
        if let Some(metrics) = &self.metrics {
            let metrics_clone = metrics.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                let mut shutdown_rx = shutdown_rx;
                
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            // Collect and log metrics
                            let metrics = metrics_clone.get_metrics();
                            debug!("Collected metrics: {:?}", metrics);
                        }
                        _ = shutdown_rx.recv() => {
                            debug!("Metrics collection task shutting down");
                            break;
                        }
                    }
                }
            });
            
            self.bg_tasks.write().push(handle);
        }
        
        // Start RL optimizer training task if enabled
        if self.config.enable_rl_optimization {
            let rl_optimizers = self.rl_optimizers.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // Train every 5 minutes
                let mut shutdown_rx = shutdown_rx;
                
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            // Train all RL optimizers
                            for entry in rl_optimizers.iter() {
                                let mut optimizer = entry.value().write();
                                if let Err(e) = optimizer.train() {
                                    error!("Error training RL optimizer for cache {}: {}", entry.key(), e);
                                }
                            }
                        }
                        _ = shutdown_rx.recv() => {
                            debug!("RL training task shutting down");
                            break;
                        }
                    }
                }
            });
            
            self.bg_tasks.write().push(handle);
        }
        
        Ok(())
    }
    
    /// Shutdown the cache manager and all background tasks
    pub async fn shutdown(&self) -> Result<()> {
        // Send shutdown signal to all background tasks
        if let Err(e) = self.shutdown_tx.send(()).await {
            error!("Error sending shutdown signal: {}", e);
        }
        
        // Wait for all background tasks to complete
        let handles = std::mem::take(&mut *self.bg_tasks.write());
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Error joining background task: {}", e);
            }
        }
        
        // Flush all caches
        for mut entry in self.caches.iter_mut() {
            if let Err(e) = entry.value().flush().await {
                error!("Error flushing cache {}: {}", entry.key(), e);
            }
        }
        
        Ok(())
    }
    
    /// Get or create a cache with the given name
    pub fn get_or_create_cache(&self, name: &str) -> Result<Arc<KVCache>> {
        if let Some(cache) = self.caches.get(name) {
            return Ok(cache.value().clone());
        }
        
        // Create a new cache
        let cache_config = KVCacheConfig {
            max_entries: self.config.max_entries,
            num_shards: self.config.num_shards,
            default_ttl: self.config.default_ttl,
            enable_compression: true,
            enable_metrics: self.config.enable_metrics,
        };
        
        let cache = Arc::new(KVCache::new(cache_config)?);
        
        // Create RL optimizer if enabled
        if self.config.enable_rl_optimization {
            let rl_optimizer = Arc::new(RwLock::new(
                RLOptimizer::new(self.config.rl_optimizer_config.clone())
            ));
            self.rl_optimizers.insert(name.to_string(), rl_optimizer);
        }
        
        // Store the cache
        self.caches.insert(name.to_string(), cache.clone());
        
        Ok(cache)
    }
    
    /// Get a reference to the metrics recorder, if enabled
    pub fn metrics(&self) -> Option<&MetricsRecorder> {
        self.metrics.as_deref()
    }
    
    /// Get a reference to the audit log, if enabled
    pub fn audit_log(&self) -> Option<&PetriNet> {
        self.audit_log.as_deref()
    }
    
    /// Process a batch of operations
    pub async fn process_batch(
        &self,
        cache_name: &str,
        ops: Vec<KVCacheBatchOp>,
    ) -> Result<()> {
        let cache = self.get_or_create_cache(cache_name)?;
        
        // Split the batch into smaller chunks if necessary
        for chunk in ops.chunks(self.config.max_batch_size) {
            let mut futures = Vec::with_capacity(chunk.len());
            
            for op in chunk {
                match op {
                    KVCacheBatchOp::Get { key, tx } => {
                        let cache = cache.clone();
                        let key_clone = key.clone();
                        
                        let future = async move {
                            let result = cache.get(&key_clone).await;
                            let _ = tx.send(result);
                        };
                        
                        futures.push(future);
                    }
                    KVCacheBatchOp::Set { key, value, ttl, tx } => {
                        let cache = cache.clone();
                        let key_clone = key.clone();
                        let value_clone = value.clone();
                        
                        let future = async move {
                            let result = cache.set(&key_clone, &value_clone, ttl).await;
                            let _ = tx.send(result);
                        };
                        
                        futures.push(future);
                    }
                    KVCacheBatchOp::Delete { key, tx } => {
                        let cache = cache.clone();
                        let key_clone = key.clone();
                        
                        let future = async move {
                            let result = cache.delete(&key_clone).await;
                            let _ = tx.send(result);
                        };
                        
                        futures.push(future);
                    }
                    KVCacheBatchOp::Flush { tx } => {
                        let cache = cache.clone();
                        
                        let future = async move {
                            let result = cache.flush().await;
                            let _ = tx.send(result);
                        };
                        
                        futures.push(future);
                    }
                }
            }
            
            // Execute the batch in parallel
            futures::future::join_all(futures).await;
        }
        
        Ok(())
    }
    
    /// Get the RL optimizer for a cache, if enabled
    pub fn get_rl_optimizer(&self, cache_name: &str) -> Option<Arc<RwLock<RLOptimizer>>> {
        self.rl_optimizers.get(cache_name).map(|entry| entry.value().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_cache_manager() -> Result<()> {
        let config = KVCacheManagerConfig {
            max_entries: 100,
            num_shards: 4,
            default_ttl: 60,
            enable_rl_optimization: false,
            rl_optimizer_config: RLOptimizerConfig::default(),
            quantization_config: QuantizationConfig::default(),
            max_batch_size: 10,
            max_queue_size: 100,
            enable_metrics: true,
            enable_audit_log: true,
        };
        
        let manager = KVCacheManager::new(config)?;
        manager.start()?;
        
        // Get or create a cache
        let cache_name = "test_cache";
        let cache = manager.get_or_create_cache(cache_name)?;
        
        // Test basic operations
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();
        
        // Set a value
        cache.set(&key, &value, None).await?;
        
        // Get the value
        let retrieved = cache.get(&key).await?.unwrap();
        assert_eq!(retrieved, value);
        
        // Test batch operations
        let mut ops = Vec::new();
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        
        ops.push(KVCacheBatchOp::Get {
            key: key.clone(),
            tx: tx1,
        });
        
        ops.push(KVCacheBatchOp::Set {
            key: b"batch_key".to_vec(),
            value: b"batch_value".to_vec(),
            ttl: None,
            tx: tx2,
        });
        
        manager.process_batch(cache_name, ops).await?;
        
        // Check the results
        let result1 = rx1.await??;
        assert_eq!(result1, Some(value));
        
        let result2 = rx2.await?;
        assert!(result2.is_ok());
        
        // Clean up
        manager.shutdown().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_rl_optimization() -> Result<()> {
        let config = KVCacheManagerConfig {
            max_entries: 100,
            num_shards: 4,
            default_ttl: 60,
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
            quantization_config: QuantizationConfig::default(),
            max_batch_size: 10,
            max_queue_size: 100,
            enable_metrics: true,
            enable_audit_log: false,
        };
        
        let manager = KVCacheManager::new(config)?;
        manager.start()?;
        
        // Get or create a cache
        let cache_name = "test_rl_cache";
        let cache = manager.get_or_create_cache(cache_name)?;
        
        // Get the RL optimizer
        let rl_optimizer = manager.get_rl_optimizer(cache_name).unwrap();
        
        // Test that we can train the RL optimizer
        let mut rl_optimizer = rl_optimizer.write();
        
        // Add some random experience to the replay buffer
        for _ in 0..100 {
            let state = vec![rand::random::<f32>(); 10];
            let action = rand::random::<usize>() % 4;
            let reward = rand::random::<f32>();
            let next_state = vec![rand::random::<f32>(); 10];
            let done = false;
            
            rl_optimizer.remember(state, action, reward, next_state, done);
        }
        
        // Train the model
        let loss = rl_optimizer.train()?;
        assert!(loss >= 0.0);
        
        // Clean up
        manager.shutdown().await?;
        
        Ok(())
    }
}

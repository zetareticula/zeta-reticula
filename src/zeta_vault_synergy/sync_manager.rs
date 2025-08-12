// Copyright 2025 ZETA RETICULA
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    sync::Arc,
    time::Duration,
};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::stream::{self, StreamExt};
use thiserror::Error;
use tokio::{
    sync::{
        mpsc,
        broadcast,
        Semaphore,
    },
    time,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error};
use tracing::Instrument;
use crate::kv_cache_manager::KVCacheManager;
use crate::weight_manager::WeightManager;

/// Maximum number of concurrent replication tasks
const MAX_CONCURRENT_REPLICATIONS: usize = 10;
/// Maximum batch size for batched operations
const MAX_BATCH_SIZE: usize = 100;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Sync channel error: {0}")]
    ChannelError(#[from] broadcast::error::SendError<(String, Vec<u8>)>),
    
    #[error("Failed to replicate to node {node_id}")]
    ReplicationError {
        node_id: usize,
        #[source]
        source: anyhow::Error,
    },
    
    #[error("Batch operation failed: {0}")]
    BatchError(#[from] BatchError),
    
    #[error("Operation cancelled")]
    Cancelled,
}

#[derive(Error, Debug)]
pub enum BatchError {
    #[error("Batch size of {size} exceeds maximum of {max}")]
    BatchSizeExceeded { size: usize, max: usize },
    
    #[error("Batch operation timed out after {elapsed:?}")]
    Timeout { elapsed: Duration },
}

/// Trait for types that can be replicated
#[async_trait]
pub trait Replicable: Send + Sync + 'static {
    async fn replicate(&self, node_id: usize) -> Result<(), SyncError>;
}

/// Configuration for synchronization
#[derive(Clone, Debug)]
pub struct SyncConfig {
    pub sync_interval: Duration,
    pub replication_factor: usize,
    pub batch_size: usize,
    pub timeout: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(5),
            replication_factor: 3,
            batch_size: 50,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Manages synchronization between nodes
pub struct SyncManager {
    node_id: usize,
    config: SyncConfig,
    kv_cache_mgr: Arc<dyn KVCacheManager>,
    weight_mgr: Arc<dyn WeightManager>,
    sync_tx: mpsc::Sender<(String, Vec<u8>)>,
    cancel_token: CancellationToken,
    semaphore: Arc<Semaphore>,
    metrics: SyncMetrics,
}

//

/// Metrics for synchronization
#[derive(Clone, Debug, Default)]
struct SyncMetrics {
    sync_ops: Arc<DashMap<String, u64>>,
    errors: Arc<DashMap<String, u64>>,
}

impl SyncManager {
    /// Creates a new SyncManager with the given configuration
    pub async fn new(
        node_id: usize,
        config: SyncConfig,
        kv_cache_mgr: Arc<dyn KVCacheManager>,
        weight_mgr: Arc<dyn WeightManager>,
    ) -> Result<Self, SyncError> {
        let (sync_tx, sync_rx) = mpsc::channel(1000);
        let cancel_token = CancellationToken::new();
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REPLICATIONS));
        let metrics = SyncMetrics::default();
        
        let manager = Self {
            node_id,
            config,
            kv_cache_mgr,
            weight_mgr,
            sync_tx,
            cancel_token: cancel_token.clone(),
            semaphore: semaphore.clone(),
            metrics: metrics.clone(),
        };
        
        // Start background sync task
        let sync_handle = tokio::spawn({
            let mgr = manager.clone();
            async move { mgr.run_sync_loop(sync_rx).await }
        });
        
        // Store the handle for graceful shutdown
        tokio::spawn(async move {
            let _ = sync_handle.await;
        });
        
        Ok(manager)
    }
    
    /// Main sync loop that processes sync messages
    async fn run_sync_loop(
        self,
        mut rx: mpsc::Receiver<(String, Vec<u8>)>,
    ) -> Result<(), SyncError> {
        let mut interval = time::interval(self.config.sync_interval);
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.sync_batch().await {
                        error!(error = %e, "Failed to sync batch");
                    }
                }
                Some((key, data)) = rx.recv() => {
                    self.process_sync_message(&key, &data).await?;
                }
                _ = self.cancel_token.cancelled() => {
                    info!("Sync loop cancelled");
                    return Ok(());
                }
            }
        }
    }
    
    /// Processes a single sync message
    async fn process_sync_message(&self, key: &str, data: &[u8]) -> Result<(), SyncError> {
        // Process based on key prefix
        if key.starts_with("kv_") {
            self.process_kv_sync(key, data).await
        } else if key.starts_with("weight_") {
            self.process_weight_sync(key, data).await
        } else {
            warn!(key, "Unknown sync message type");
            Ok(())
        }
    }
    
    /// Processes a KV cache sync message
    async fn process_kv_sync(&self, _key: &str, _data: &[u8]) -> Result<(), SyncError> {
        // Deserialize and apply KV update
        // ... implementation ...
        Ok(())
    }
    
    /// Processes a weight sync message
    async fn process_weight_sync(&self, _key: &str, _data: &[u8]) -> Result<(), SyncError> {
        // Deserialize and apply weight update
        // ... implementation ...
        Ok(())
    }
    
    /// Syncs a batch of data to all replicas
    pub async fn sync_batch(&self) -> Result<(), SyncError> {
        let kv_data = self.kv_cache_mgr.get_all().await;
        let weight_data = self.weight_mgr.get_all().await;
        
        // Process in parallel with backpressure
        let semaphore = self.semaphore.clone();
        let permit = semaphore.acquire().await.map_err(|_| SyncError::Cancelled)?;
        
        let result = tokio::try_join!(
            self.replicate_data("kv", kv_data, move |node_id, data: Vec<u8>| {
                async move { Self::replicate_kv(node_id, &data).await }
            }),
            self.replicate_data("weight", weight_data, move |node_id, data: Vec<u8>| {
                async move { Self::replicate_weight(node_id, &data).await }
            }),
        );
        
        drop(permit);
        result?;
        
        Ok(())
    }
    
    /// Generic replication function with batching and backpressure
    async fn replicate_data<T, F, Fut>(
        &self,
        data_type: &str,
        data: Vec<T>,
        replicate_fn: F,
    ) -> Result<(), SyncError>
    where
        F: Fn(usize, T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), SyncError>> + Send,
        T: Send + Clone + 'static,
    {
        if data.is_empty() {
            return Ok(());
        }
        
        // Process data in batches (own the batches to ensure 'static futures)
        let batches: Vec<Vec<T>> = data
            .chunks(self.config.batch_size)
            .map(|c| c.to_vec())
            .collect();
        let batch_count = batches.len();
        
        info!(
            data_type,
            batch_count,
            total_items = data.len(),
            "Starting replication"
        );
        
        // Process batches in parallel with bounded concurrency
        let results = stream::iter(batches.into_iter().enumerate())
            .map(|(i, batch)| {
                let replicate_fn = &replicate_fn;
                let node_id = self.node_id;
                
                async move {
                    let start = std::time::Instant::now();
                    // Process items in this batch sequentially
                    for item in batch.into_iter() {
                        if let Err(e) = replicate_fn(node_id, item).await {
                            let elapsed = start.elapsed();
                            error!(
                                error = %e,
                                data_type,
                                batch = i + 1,
                                batch_count,
                                elapsed_ms = elapsed.as_millis(),
                                "Batch replication failed on item",
                            );
                            return Err(e);
                        }
                    }
                    let result: Result<(), SyncError> = Ok(());
                    let elapsed = start.elapsed();
                    
                    match &result {
                        Ok(_) => {
                            info!(
                                data_type,
                                batch = i + 1,
                                batch_count,
                                elapsed_ms = elapsed.as_millis(),
                                "Batch replicated successfully"
                            );
                        }
                        Err(e) => {
                            error!(
                                error = %e,
                                data_type,
                                batch = i + 1,
                                batch_count,
                                "Batch replication failed"
                            );
                        }
                    }
                    
                    result
                }
            })
            .buffer_unordered(MAX_CONCURRENT_REPLICATIONS)
            .collect::<Vec<_>>()
            .await;
        
        // Check for any errors
        for result in results {
            result?;
        }
        
        Ok(())
    }
    
    /// Replicates KV cache to a node
    async fn replicate_kv(node_id: usize, data: &[u8]) -> Result<(), SyncError> {
        // Implementation for KV replication
        // ...
        Ok(())
    }
    
    /// Replicates weights to a node
    async fn replicate_weight(node_id: usize, data: &[u8]) -> Result<(), SyncError> {
        // Implementation for weight replication
        // ...
        Ok(())
    }
    
    /// Gracefully shuts down the sync manager
    pub async fn shutdown(&self) -> Result<(), SyncError> {
        info!("Shutting down sync manager");
        self.cancel_token.cancel();
        Ok(())
    }
}

// Implement Clone for Arc-wrapped types
impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            config: self.config.clone(),
            kv_cache_mgr: self.kv_cache_mgr.clone(),
            weight_mgr: self.weight_mgr.clone(),
            sync_tx: self.sync_tx.clone(),
            cancel_token: self.cancel_token.clone(),
            semaphore: self.semaphore.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{predicate::*, mock};
    
    // Mock implementations for testing
    mock! {
        pub KVCacheManager {}
        #[async_trait]
        impl KVCacheManagerTrait for KVCacheManager {
            async fn get_all(&self) -> Result<Vec<u8>, SyncError>;
        }
    }
    
    mock! {
        pub WeightManager {}
        #[async_trait]
        impl WeightManagerTrait for WeightManager {
            async fn get_all(&self) -> Result<Vec<u8>, SyncError>;
        }
    }
    
    #[tokio::test]
    async fn test_sync_batch() {
        let kv_mock = MockKVCacheManager::new();
        let weight_mock = MockWeightManager::new();
        
        // Set up expectations
        kv_mock.expect_get_all()
            .returning(|| Ok(vec![1, 2, 3]));
            
        weight_mock.expect_get_all()
            .returning(|| Ok(vec![4, 5, 6]));
        
        let config = SyncConfig {
            batch_size: 10,
            ..Default::default()
        };
        
        let manager = SyncManager::new(
            1,
            config,
            Arc::new(kv_mock),
            Arc::new(weight_mock),
        ).await.unwrap();
        
        let result = manager.sync_batch().await;
        assert!(result.is_ok());
    }
}
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

pub mod kv_cache_manager;
pub mod weight_manager;
pub mod sync_manager;

use std::{
    sync::Arc,
    time::Duration,
};
use async_trait::async_trait;
use ndarray::Array2;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, error, instrument};
use half::f16;

use kv_cache_manager::{KVCache, KVCacheManager, KVCacheManagerImpl, KVCacheManagerError};
use weight_manager::{BinaryWeightSet, WeightManager, WeightManagerImpl, WeightManagerError};
use sync_manager::{SyncManager, SyncError, SyncConfig};

/// Main error type for ZetaVaultSynergy operations
#[derive(Error, Debug)]
pub enum ZetaVaultSynergyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),
    
    #[error("KV cache error: {0}")]
    KVCache(#[from] KVCacheManagerError),
    
    #[error("Weight manager error: {0}")]
    WeightManager(#[from] WeightManagerError),
    
    #[error("Validation error: {message}")]
    Validation { 
        message: String,
    },
    
    #[error("Operation timed out after {elapsed:?}")]
    Timeout { 
        elapsed: Duration,
    },
}

/// Configuration for ZetaVaultSynergy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Number of nodes in the cluster
    pub node_count: usize,
    
    /// Number of replicas for each piece of data
    pub replication_factor: usize,
    
    /// Interval between sync operations
    pub sync_interval: Duration,
    
    /// Batch size for sync operations
    pub batch_size: usize,
    
    /// Timeout for sync operations
    pub sync_timeout: Duration,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            node_count: 1,
            replication_factor: 3,
            sync_interval: Duration::from_secs(5),
            batch_size: 50,
            sync_timeout: Duration::from_secs(30),
        }
    }
}

/// Main orchestrator for ZetaVaultSynergy operations
pub struct ZetaVaultSynergy {
    kv_cache_mgr: Arc<dyn KVCacheManager>,
    weight_mgr: Arc<dyn WeightManager>,
    sync_mgr: Arc<SyncManager>,
    config: VaultConfig,
    node_id: usize,
}

impl ZetaVaultSynergy {
    /// Creates a new ZetaVaultSynergy instance
    #[instrument(skip_all)]
    pub async fn new(node_id: usize, config: VaultConfig) -> Result<Self, ZetaVaultSynergyError> {
        info!("Initializing ZetaVaultSynergy with config: {:?}", config);
        
        // Initialize managers
        let kv_cache_mgr = Arc::new(KVCacheManagerImpl::new(node_id, config.clone()).await?);
        let weight_mgr = Arc::new(WeightManagerImpl::new(node_id, config.clone()).await?);
        
        // Create sync config
        let sync_config = SyncConfig {
            sync_interval: config.sync_interval,
            replication_factor: config.replication_factor,
            batch_size: config.batch_size,
            timeout: config.sync_timeout,
        };
        
        // Initialize sync manager
        let sync_mgr = Arc::new(
            SyncManager::new(
                node_id,
                sync_config,
                Arc::clone(&kv_cache_mgr) as Arc<dyn KVCacheManager>,
                Arc::clone(&weight_mgr) as Arc<dyn WeightManager>,
            ).await?
        );
        
        Ok(Self {
            kv_cache_mgr,
            weight_mgr,
            sync_mgr,
            config,
            node_id,
        })
    }
    
    /// Stores KV cache for a model
    #[instrument(skip(self, keys, values))]
    pub async fn store_kv_cache(
        &self, 
        model_id: &str, 
        keys: Array2<f16>, 
        values: Array2<f16>
    ) -> Result<(), ZetaVaultSynergyError> {
        self.kv_cache_mgr.store_kv_cache(model_id, keys, values).await?;
        
        // Trigger async replication
        if let Err(e) = self.sync_mgr.sync_batch().await {
            error!(error = %e, "Failed to trigger replication after KV cache update");
        }
        
        Ok(())
    }
    
    /// Retrieves KV cache for a model
    #[instrument(skip(self))]
    pub async fn get_kv_cache(&self, model_id: &str) -> Result<Option<KVCache>, ZetaVaultSynergyError> {
        Ok(self.kv_cache_mgr.get_kv_cache(model_id).await)
    }
    
    /// Stores binary weights for a model
    #[instrument(skip(self, binary_set))]
    pub async fn store_binary_weights(
        &self, 
        model_id: &str, 
        binary_set: BinaryWeightSet
    ) -> Result<(), ZetaVaultSynergyError> {
        self.weight_mgr.store_binary_weights(model_id, binary_set).await?;
        
        // Trigger async replication
        if let Err(e) = self.sync_mgr.sync_batch().await {
            error!(error = %e, "Failed to trigger replication after weights update");
        }
        
        Ok(())
    }
    
    /// Retrieves binary weights for a model
    #[instrument(skip(self))]
    pub async fn get_binary_weights(&self, model_id: &str) -> Result<Option<BinaryWeightSet>, ZetaVaultSynergyError> {
        Ok(self.weight_mgr.get_binary_weights(model_id).await)
    }
    
    /// Triggers a manual sync of all data
    #[instrument(skip(self))]
    pub async fn sync_all(&self) -> Result<(), ZetaVaultSynergyError> {
        self.sync_mgr.sync_batch().await?;
        Ok(())
    }
    
    /// Gracefully shuts down the service
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), ZetaVaultSynergyError> {
        info!("Shutting down ZetaVaultSynergy");
        self.sync_mgr.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_zvs_initialization() {
        let config = VaultConfig {
            node_count: 3,
            replication_factor: 2,
            ..Default::default()
        };
        
        let zvs = ZetaVaultSynergy::new(1, config).await;
        assert!(zvs.is_ok());
    }
    
    // Add more tests for other methods
}
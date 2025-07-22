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

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use thiserror::Error;
use log;
use crate::zeta_vault_synergy::kv_cache_manager::KVCacheManager;
use crate::zeta_vault_synergy::weight_manager::WeightManager;
use crate::zeta_vault_synergy::sync_manager::SyncManager;

#[derive(Error, Debug)]
pub enum ZetaVaultSynergyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Sync error: {0}")]
    Sync(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct ZetaVaultSynergy {
    kv_cache_mgr: Arc<KVCacheManager>,
    weight_mgr: Arc<WeightManager>,
    sync_mgr: Arc<SyncManager>,
}

impl ZetaVaultSynergy {
    pub async fn new(node_id: usize, config: VaultConfig) -> Result<Self, ZetaVaultSynergyError> {
        let kv_cache_mgr = Arc::new(KVCacheManager::new(node_id, config.clone()).await?);
        let weight_mgr = Arc::new(WeightManager::new(node_id, config.clone()).await?);
        let sync_mgr = Arc::new(SyncManager::new(node_id, config, Arc::clone(&kv_cache_mgr), Arc::clone(&weight_mgr)).await?);
        Ok(ZetaVaultSynergy {
            kv_cache_mgr,
            weight_mgr,
            sync_mgr,
        })
    }

    pub async fn store_kv_cache(&self, model_id: &str, keys: Array2<f16>, values: Array2<f16>) -> Result<(), ZetaVaultSynergyError> {
        self.kv_cache_mgr.store_kv_cache(model_id, keys, values).await
    }

    pub async fn get_kv_cache(&self, model_id: &str) -> Option<KVCache> {
        self.kv_cache_mgr.get_kv_cache(model_id).await
    }

    pub async fn store_binary_weights(&self, model_id: &str, binary_set: BinaryWeightSet) -> Result<(), ZetaVaultSynergyError> {
        self.weight_mgr.store_binary_weights(model_id, binary_set).await
    }

    pub async fn get_binary_weights(&self, model_id: &str) -> Option<BinaryWeightSet> {
        self.weight_mgr.get_binary_weights(model_id).await
    }

    pub async fn sync_nodes(&self) -> Result<(), ZetaVaultSynergyError> {
        self.sync_mgr.sync_nodes().await
    }

    pub async fn replicate_data(&self) -> Result<(), ZetaVaultSynergyError> {
        self.sync_mgr.replicate_data().await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub node_count: usize,
    pub replication_factor: usize,
    pub sync_interval: std::time::Duration,
}
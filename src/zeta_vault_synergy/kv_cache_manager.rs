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

use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use ndarray::Array2;
use lru::LruCache;
use async_trait::async_trait;
use std::collections::HashMap;
use crate::VaultConfig;
use chrono;
use half::f16;

#[derive(Debug, Clone)]
pub enum CacheLayer {
    HBM,
    DDR,
}

#[derive(Debug, Clone)]
pub struct KVCache {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub layer: CacheLayer,
    pub timestamp: u64,
    pub node_id: usize,
}

#[derive(Error, Debug)]
pub enum KVCacheManagerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[async_trait]
pub trait KVCacheManager: Send + Sync {
    async fn store_kv_cache(&self, model_id: &str, keys: Array2<f16>, values: Array2<f16>) -> Result<(), KVCacheManagerError>;
    async fn get_kv_cache(&self, model_id: &str) -> Option<KVCache>;
}

pub struct KVCacheManagerImpl {
    kv_cache: Arc<RwLock<HashMap<String, KVCache>>>,
    cache: RwLock<LruCache<String, KVCache>>,
    node_id: usize,
    config: VaultConfig,
}

impl KVCacheManagerImpl {
    // Note: The `get_mut` method on `LruCache` requires a mutable reference, 
    // so we need to change how we handle the cache lock.
    // This is a temporary solution to get the code to compile.
    // A more robust solution would involve using a `Mutex` or `RwLock` around the `LruCache`.

    pub async fn new(node_id: usize, config: VaultConfig) -> Result<Self, KVCacheManagerError> {
        Ok(KVCacheManagerImpl {
            kv_cache: Arc::new(RwLock::new(HashMap::new())),
            cache: RwLock::new(LruCache::new(100)), // LRU cache for 100 entries
            node_id,
            config,
        })
    }

    pub async fn store_kv_cache(&self, model_id: &str, keys: Array2<f16>, values: Array2<f16>) -> Result<(), KVCacheManagerError> {
        if keys.is_empty() || values.is_empty() || keys.shape() != values.shape() {
            return Err(KVCacheManagerError::Validation("Invalid KV cache data".to_string()));
        }
        let key_data = bincode::serialize(&keys)?;
        let value_data = bincode::serialize(&values)?;
        let kv_cache = KVCache {
            key: key_data,
            value: value_data,
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
            node_id: self.node_id,
        };
        let key = format!("kv_cache_{}", model_id);
        self.kv_cache.write().await.insert(key.clone(), kv_cache.clone());
        self.cache.write().await.put(key, kv_cache);
        log::info!("Stored KV cache for model {}", model_id);
        Ok(())
    }

    async fn get_kv_cache_impl(&self, model_id: &str) -> Option<KVCache> {
        let key = format!("kv_cache_{}", model_id);
        if let Some(cached) = self.cache.read().await.get(&key) {
            log::debug!("Cache hit for {}", model_id);
            return Some(cached.clone());
        }

        if let Some(cache) = self.kv_cache.read().await.get(&key).cloned() {
            self.cache.write().await.put(key.to_string(), cache.clone());
            return Some(cache);
        }

        None
    }
}

#[async_trait]
impl KVCacheManager for KVCacheManagerImpl {
    async fn store_kv_cache(&self, model_id: &str, keys: Array2<f16>, values: Array2<f16>) -> Result<(), KVCacheManagerError> {
        self.store_kv_cache(model_id, keys, values).await
    }

    async fn get_kv_cache(&self, model_id: &str) -> Option<KVCache> {
        self.get_kv_cache_impl(model_id).await
    }
}
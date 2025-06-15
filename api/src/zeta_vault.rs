use std::sync::Arc;
use std::path::Path;
use tokio::sync::RwLock;
use dashmap::DashMap;
use sled::Db;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use log;
use crossbeam::queue::SegQueue;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum ZetaVaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KVCache {
    key: Vec<u8>,
    value: Vec<u8>,
    layer: CacheLayer,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum CacheLayer {
    HBM,
    HostMemory,
    Disk,
}

pub struct ZetaVault {
    hbm_cache: DashMap<String, KVCache>, // In-memory for GPU HBM
    host_cache: DashMap<String, KVCache>, // Host memory layer
    disk_cache: Db, // Disk layer using sled
    scheduler_queue: Arc<SegQueue<String>>, // Scheduler-aware hints
    config: VaultConfig,
}

#[derive(Clone)]
pub struct VaultConfig {
    hbm_size: usize,
    host_size: usize,
    disk_path: String,
    pre_load_batch: usize,
}

impl ZetaVault {
    pub async fn new(config: VaultConfig) -> Result<Self, ZetaVaultError> {
        let disk_cache = sled::open(&config.disk_path)?;
        let hbm_cache = DashMap::new();
        let host_cache = DashMap::new();
        let scheduler_queue = Arc::new(SegQueue::new());
        Ok(ZetaVault {
            hbm_cache,
            host_cache,
            disk_cache,
            scheduler_queue,
            config,
        })
    }

    pub async fn store(&self, key: String, value: Vec<u8>) -> Result<(), ZetaVaultError> {
        let kv_cache = KVCache {
            key: key.as_bytes().to_vec(),
            value,
            layer: CacheLayer::HBM,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        self.hbm_cache.insert(key.clone(), kv_cache.clone());
        if self.hbm_cache.len() > self.config.hbm_size {
            self.evict(CacheLayer::HBM).await?;
        }
        Ok(())
    }

    pub async fn fetch(&self, key: &str) -> Result<Vec<u8>, ZetaVaultError> {
        if let Some(cache) = self.hbm_cache.get(key) {
            return Ok(cache.value.clone());
        }
        if let Some(cache) = self.host_cache.get(key) {
            self.pre_load(cache.clone()).await?;
            return Ok(cache.value.clone());
        }
        if let Ok(Some(data)) = self.disk_cache.get(key) {
            let cache = KVCache {
                key: key.as_bytes().to_vec(),
                value: data.to_vec(),
                layer: CacheLayer::Disk,
                timestamp: chrono::Utc::now().timestamp() as u64,
            };
            self.pre_load(cache.clone()).await?;
            return Ok(cache.value);
        }
        Err(ZetaVaultError::KeyNotFound(key.to_string()))
    }

    async fn pre_load(&self, cache: KVCache) -> Result<(), ZetaVaultError> {
        if cache.layer == CacheLayer::Disk && self.host_cache.len() < self.config.host_size {
            self.host_cache.insert(String::from_utf8_lossy(&cache.key).to_string(), cache.clone());
            self.async_save(cache).await?;
        }
        Ok(())
    }

    async fn async_save(&self, cache: KVCache) -> Result<(), ZetaVaultError> {
        let disk_cache = self.disk_cache.clone();
        tokio::spawn(async move {
            if let Err(e) = disk_cache.insert(&cache.key, &cache.value) {
                log::error!("Async save failed: {}", e);
            }
        });
        Ok(())
    }

    async fn evict(&self, layer: CacheLayer) -> Result<(), ZetaVaultError> {
        match layer {
            CacheLayer::HBM => {
                if let Some((key, cache)) = self.hbm_cache.iter().min_by_key(|(_, c)| c.timestamp) {
                    self.host_cache.insert(key.clone(), cache.clone());
                    self.hbm_cache.remove(&key);
                }
            }
            CacheLayer::HostMemory => {
                if let Some((key, cache)) = self.host_cache.iter().min_by_key(|(_, c)| c.timestamp) {
                    self.disk_cache.insert(&key, &cache.value)?;
                    self.host_cache.remove(&key);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn schedule_fetch(&self, key: String) {
        self.scheduler_queue.push(key);
    }

    #[cfg(feature = "wasm")]
    #[wasm_bindgen]
    pub fn fetch_wasm(key: String) -> js_sys::Promise {
        use wasm_bindgen_futures::future_to_promise;
        future_to_promise(async move {
            let vault = ZetaVault::new(VaultConfig {
                hbm_size: 1000,
                host_size: 10000,
                disk_path: "wasm_db".to_string(),
                pre_load_batch: 10,
            }).await.unwrap();
            let result = vault.fetch(&key).await.map_err(|e| js_sys::Error::new(&e.to_string()))?;
            Ok(js_sys::Uint8Array::from(result.as_slice()).into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zeta_vault_store_fetch() {
        let config = VaultConfig {
            hbm_size: 2,
            host_size: 5,
            disk_path: "test_db".to_string(),
            pre_load_batch: 1,
        };
        let vault = ZetaVault::new(config).await.unwrap();
        vault.store("key1".to_string(), vec![1, 2, 3]).await.unwrap();
        let result = vault.fetch("key1").await.unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }
}
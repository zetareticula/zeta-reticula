use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use ndarray::Array2;
use lru::LruCache;
use crate::model_store::NeuronMatrix;

#[derive(Error, Debug)]
pub enum KVCacheManagerError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct KVCacheManager {
    kv_cache: Arc<RwLock<HashMap<String, KVCache>>>,
    cache: LruCache<String, KVCache>,
    node_id: usize,
    config: VaultConfig,
}

impl KVCacheManager {
    pub async fn new(node_id: usize, config: VaultConfig) -> Result<Self, KVCacheManagerError> {
        Ok(KVCacheManager {
            kv_cache: Arc::new(RwLock::new(HashMap::new())),
            cache: LruCache::new(100), // LRU cache for 100 entries
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
        self.cache.put(key, kv_cache);
        log::info!("Stored KV cache for model {}", model_id);
        Ok(())
    }

    pub async fn get_kv_cache(&self, model_id: &str) -> Option<KVCache> {
        let key = format!("kv_cache_{}", model_id);
        if let Some(cached) = self.cache.get(&key) {
            log::debug!("Cache hit for {}", model_id);
            Some(cached.clone())
        } else {
            self.kv_cache.read().await.get(&key).cloned().map(|cache| {
                self.cache.put(key, cache.clone());
                cache
            })
        }
    }
}
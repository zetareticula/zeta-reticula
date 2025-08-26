use std::sync::Arc;
use tokio::sync::Mutex;
use zeta_vault_synergy::{
    kv_cache_manager::{KVCacheManager, KVCacheManagerTrait},
    weight_manager::{WeightManager, WeightManagerTrait},
    sync_manager::{SyncManager, SyncConfig},
};

/// Test configuration for integration tests
pub struct TestConfig {
    pub kv_cache_mgr: Arc<dyn KVCacheManagerTrait>,
    pub weight_mgr: Arc<dyn WeightManagerTrait>,
    pub sync_manager: Arc<SyncManager>,
}

impl TestConfig {
    /// Create a new test configuration
    pub async fn new() -> Self {
        // Create mock KV cache manager
        let kv_cache_mgr = Arc::new(MockKVCacheManager::new());
        
        // Create mock weight manager
        let weight_mgr = Arc::new(MockWeightManager::new());
        
        // Create sync manager with test configuration
        let sync_config = SyncConfig {
            sync_interval: std::time::Duration::from_millis(100),
            replication_factor: 1,
            batch_size: 10,
            timeout: std::time::Duration::from_secs(5),
        };
        
        let sync_manager = Arc::new(
            SyncManager::new(0, sync_config, kv_cache_mgr.clone(), weight_mgr.clone())
                .await
                .expect("Failed to create SyncManager"),
        );

        Self {
            kv_cache_mgr,
            weight_mgr,
            sync_manager,
        }
    }
}

/// Mock implementation of KVCacheManager for testing
struct MockKVCacheManager {
    data: Arc<Mutex<Vec<u8>>>,
}

impl MockKVCacheManager {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl KVCacheManagerTrait for MockKVCacheManager {
    async fn get_all(&self) -> Result<Vec<u8>, zeta_vault_synergy::sync_manager::SyncError> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }

    async fn put(&self, _key: &str, value: Vec<u8>) -> Result<(), zeta_vault_synergy::sync_manager::SyncError> {
        let mut data = self.data.lock().await;
        *data = value;
        Ok(())
    }
}

/// Mock implementation of WeightManager for testing
struct MockWeightManager {
    weights: Arc<Mutex<Vec<u8>>>,
}

impl MockWeightManager {
    fn new() -> Self {
        Self {
            weights: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl WeightManagerTrait for MockWeightManager {
    async fn get_all(&self) -> Result<Vec<u8>, zeta_vault_synergy::sync_manager::SyncError> {
        let weights = self.weights.lock().await;
        Ok(weights.clone())
    }

    async fn update_weights(&self, weights: Vec<u8>) -> Result<(), zeta_vault_synergy::sync_manager::SyncError> {
        let mut w = self.weights.lock().await;
        *w = weights;
        Ok(())
    }
}

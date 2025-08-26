use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use zeta_vault_synergy::sync_manager::SyncError;

#[derive(Default)]
pub struct MockKVCacheManager {
    data: Arc<Mutex<Vec<u8>>>,
}

#[async_trait]
impl zeta_vault_synergy::kv_cache_manager::KVCacheManagerTrait for MockKVCacheManager {
    async fn get_all(&self) -> Result<Vec<u8>, SyncError> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }

    async fn put(&self, _key: &str, value: Vec<u8>) -> Result<(), SyncError> {
        let mut data = self.data.lock().await;
        *data = value;
        Ok(())
    }
}

#[derive(Default)]
pub struct MockWeightManager {
    data: Arc<Mutex<Vec<u8>>>,
}

#[async_trait]
impl zeta_vault_synergy::weight_manager::WeightManagerTrait for MockWeightManager {
    async fn get_all(&self) -> Result<Vec<u8>, SyncError> {
        let data = self.data.lock().await;
        Ok(data.clone())
    }

    async fn update_weights(&self, weights: Vec<u8>) -> Result<(), SyncError> {
        let mut data = self.data.lock().await;
        *data = weights;
        Ok(())
    }
}

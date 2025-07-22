use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use thiserror::Error;
use rayon::prelude::*;

#[derive(Error, Debug)]
pub enum SyncManagerError {
    #[error("Sync error: {0}")]
    Sync(String),
}

pub struct SyncManager {
    sync_channel: broadcast::Sender<(String, Vec<u8>)>,
    node_id: usize,
    config: VaultConfig,
    kv_cache_mgr: Arc<KVCacheManager>,
    weight_mgr: Arc<WeightManager>,
}

impl SyncManager {
    pub async fn new(node_id: usize, config: VaultConfig, kv_cache_mgr: Arc<KVCacheManager>, weight_mgr: Arc<WeightManager>) -> Result<Self, SyncManagerError> {
        let (tx, _rx) = broadcast::channel(100);
        Ok(SyncManager {
            sync_channel: tx,
            node_id,
            config,
            kv_cache_mgr,
            weight_mgr,
        })
    }

    pub async fn sync_nodes(&self) -> Result<(), SyncManagerError> {
        let mut rx = self.sync_channel.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.config.sync_interval).await;
                let data: Vec<(String, Vec<u8>)> = rx.try_iter().collect();
                data.par_iter().for_each(|(key, value)| {
                    log::info!("Syncing {} to node {}", key, self.node_id);
                    // Mock parallel sync (replace with gRPC in production)
                });
            }
        });
        Ok(())
    }

    pub async fn replicate_data(&self) -> Result<(), SyncManagerError> {
        let kv_caches = self.kv_cache_mgr.kv_cache.read().await.clone();
        let weights = self.weight_mgr.binary_weights.read().await.clone();
        rayon::scope(|s| {
            s.spawn(|_| {
                kv_caches.par_iter().for_each(|(key, cache)| {
                    for node in 0..self.config.replication_factor {
                        if node != self.node_id {
                            log::info!("Replicating KV cache {} to node {}", key, node);
                        }
                    }
                });
            });
            s.spawn(|_| {
                weights.par_iter().for_each(|(key, weight)| {
                    for node in 0..self.config.replication_factor {
                        if node != self.node_id {
                            log::info!("Replicating weight {} to node {}", key, node);
                        }
                    }
                });
            });
        });
        Ok(())
    }
}
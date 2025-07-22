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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use p2pstore::{KVCache, TransferEngine, AllocatedBufferDescriptor, TransferEngineError};
use zeta_vault_synergy::ZetaVaultSynergy;
use parking_lot::Mutex as ParkingMutex;
use crate::scheduler::Scheduler;

const MAX_GENERATION_STEPS: usize = 100;
const EOS_TOKEN: u32 = 2; // End-of-sequence token
const LAYER_COUNT: usize = 12; // Example number of transformer layers

#[derive(Error, Debug)]
pub enum AttentionStoreError {
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Transfer error: {0}")]
    Transfer(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionContext {
    session_id: String,
    kv_cache: Vec<KVCache>,
    last_active: Instant,
    truncated: bool,
}

pub struct AttentionStore {
    kv_caching_system: Arc<ZetaVaultSynergy>,
    transfer_engine: Arc<TransferEngine>,
    sessions: RwLock<std::collections::HashMap<String, SessionContext>>,
    scheduler: Arc<Scheduler>,
    hbm_buffer: Mutex<Vec<KVCache>>, // High Bandwidth Memory buffer
    host_memory: Mutex<Vec<SessionContext>>, // Host memory tier
    disk_storage: Mutex<Vec<SessionContext>>, // Disk tier
    host_memory_capacity: usize, // Simulated capacity in KB
    disk_capacity: usize,       // Simulated capacity in KB
}

impl AttentionStore {
    pub fn new(vault: Arc<ZetaVaultSynergy>, transfer_engine: Arc<TransferEngine>) -> Arc<Self> {
        let store = Arc::new(AttentionStore {
            kv_caching_system: Arc::clone(&vault),
            transfer_engine: Arc::clone(&transfer_engine),
            sessions: RwLock::new(std::collections::HashMap::new()),
            scheduler: Arc::new(Scheduler::new()),
            hbm_buffer: Mutex::new(Vec::with_capacity(LAYER_COUNT)),
            host_memory: Mutex::new(Vec::new()),
            disk_storage: Mutex::new(Vec::new()),
            host_memory_capacity: 1024 * 1024, // 1GB in KB
            disk_capacity: 10 * 1024 * 1024,  // 10GB in KB
        });

        tokio::spawn(AttentionStore::async_save_task(Arc::clone(&store)));
        store
    }

    pub async fn decode(&self, session_id: String, token: u32, kv_cache_prev: Vec<KVCache>) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert(SessionContext {
            session_id: session_id.clone(),
            kv_cache: kv_cache_prev,
            last_active: Instant::now(),
            truncated: false,
        });

        let mut hbm = self.hbm_buffer.lock();
        self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;

        let mut next_token = token;
        for step in 0..MAX_GENERATION_STEPS {
            let (new_token, new_kv_cache) = self.compute_attention(next_token, &ctx.kv_cache, step)?;
            next_token = new_token;
            ctx.kv_cache = new_kv_cache;

            if next_token == EOS_TOKEN {
                log::info!("Session {} reached EOS token at step {}", session_id, step);
                break;
            }
        }

        self.async_save(ctx.kv_cache.clone()).await?;
        Ok((next_token, ctx.kv_cache.clone()))
    }

    pub async fn prefill(&self, session_id: String, new_tokens: Vec<u32>) -> Result<Vec<KVCache>, AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert(SessionContext {
            session_id,
            kv_cache: Vec::new(),
            last_active: Instant::now(),
            truncated: false,
        });

        let mut hbm = self.hbm_buffer.lock();
        if !ctx.kv_cache.is_empty() {
            self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;
        }

        let new_kv_cache = self.compute_parallel_prefill(new_tokens)?;
        ctx.kv_cache.extend(new_kv_cache);
        self.async_save(ctx.kv_cache.clone()).await?;
        self.evict_if_needed().await?;
        Ok(ctx.kv_cache.clone())
    }

    async fn layer_wise_preload(&self, kv_cache: &[KVCache], hbm: &mut Vec<KVCache>) -> Result<(), AttentionStoreError> {
        hbm.clear();
        hbm.reserve(LAYER_COUNT);
        for (layer_idx, cache) in kv_cache.chunks(LAYER_COUNT).next().unwrap_or(&[]).iter().enumerate() {
            if layer_idx > 0 {
                let prev_cache = &kv_cache[(layer_idx - 1) * LAYER_COUNT..layer_idx * LAYER_COUNT];
                self.transfer_engine.async_load(&prev_cache[0], hbm).await?; // Load per layer
            }
            tokio::task::yield_now().await; // Overlap with computation
        }
        Ok(())
    }

    async fn async_save(&self, kv_cache: Vec<KVCache>) -> Result<(), AttentionStoreError> {
        let save_task = self.transfer_engine.async_save(kv_cache);
        tokio::spawn(async move {
            if let Err(e) = save_task.await {
                log::error!("Async save failed: {}", e);
            }
        });
        Ok(())
    }

    fn compute_attention(&self, token: u32, kv_cache: &[KVCache], step: usize) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        // Simplified attention computation: Update KV cache with new token context
        let mut new_kv_cache = kv_cache.to_vec();
        for layer_cache in new_kv_cache.iter_mut().take(LAYER_COUNT) {
            layer_cache.buffers[0].size_ += 1; // Simulate attention update
        }
        let new_token = if step % 5 == 0 { EOS_TOKEN } else { token + 1 }; // Simulated token generation
        Ok((new_token, new_kv_cache))
    }

    fn compute_parallel_prefill(&self, tokens: Vec<u32>) -> Result<Vec<KVCache>, AttentionStoreError> {
        // Parallel prefilling: Generate KV cache for each token
        let mut kv_caches = Vec::with_capacity(tokens.len() * LAYER_COUNT);
        for _ in tokens {
            for _ in 0..LAYER_COUNT {
                kv_caches.push(KVCache::new(vec![AllocatedBufferDescriptor {
                    buffer_address_: 0, // Simulated address
                    size_: tokens.len() as u64 * 1024, // Size based on token count
                }]));
            }
        }
        Ok(kv_caches)
    }

    pub async fn truncate_cache(&self, session_id: String, max_tokens: usize) -> Result<(), AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        if let Some(ctx) = sessions.get_mut(&session_id) {
            let token_count = ctx.kv_cache.len() / LAYER_COUNT;
            if token_count > max_tokens {
                ctx.kv_cache.truncate(max_tokens * LAYER_COUNT);
                ctx.truncated = true;
                // Re-embed positional encoding (simplified)
                for cache in &mut ctx.kv_cache {
                    cache.positional_encoding = Some((0..max_tokens as i32).collect());
                }
                self.async_save(ctx.kv_cache.clone()).await?;
            }
        }
        Ok(())
    }

    async fn evict_if_needed(&self) -> Result<(), AttentionStoreError> {
        let host_mem = self.host_memory.lock();
        let disk = self.disk_storage.lock();
        let total_size = host_mem.iter().map(|c| c.kv_cache.len() / LAYER_COUNT).sum::<usize>() +
                        disk.iter().map(|c| c.kv_cache.len() / LAYER_COUNT).sum::<usize>();
        if total_size * 1024 > self.host_memory_capacity + self.disk_capacity {
            self.evict().await;
        }
        Ok(())
    }

    pub async fn evict(&self) {
        let total_capacity = self.host_memory_capacity + self.disk_capacity;
        let look_ahead_window = total_capacity / 1024; // KB per KV cache
        self.scheduler.evict(look_ahead_window, &self.host_memory, &self.disk_storage).await;

        // Move excess from host to disk if needed
        let mut host_mem = self.host_memory.lock();
        while host_mem.len() * 1024 > self.host_memory_capacity {
            if let Some(ctx) = host_mem.pop() {
                self.disk_storage.lock().push(ctx);
            }
        }
    }
}

impl AttentionStore {
    async fn async_save_task(store: Arc<AttentionStore>) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let sessions = store.sessions.read().await;
            for ctx in sessions.values() {
                if ctx.last_active.elapsed() > Duration::from_secs(60) { // Inactive for 1 minute
                    store.async_save(ctx.kv_cache.clone()).await.ok();
                }
            }
        }
    }
}
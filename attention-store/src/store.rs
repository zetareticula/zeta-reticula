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
use llm_rs::LLMModel; // Assumed interface from llm-rs
use client::Client;    // For TransferEngine
use master_service::MasterService; // For segment management
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
    #[error("LLM error: {0}")]
    LLM(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionContext {
    session_id: String,
    kv_cache: Vec<KVCache>,
    last_active: Instant,
    truncated: bool,
    segment: Option<String>, // Associated segment from master-service
}

pub struct AttentionStore {
    kv_caching_system: Arc<ZetaVaultSynergy>,
    transfer_engine: Arc<TransferEngine>,
    client: Arc<Client>,    // For session and transfer management
    master_service: Arc<MasterService>, // For segment allocation
    sessions: RwLock<std::collections::HashMap<String, SessionContext>>,
    scheduler: Arc<Scheduler>,
    hbm_buffer: Mutex<Vec<KVCache>>, // High Bandwidth Memory buffer
    host_memory: Mutex<Vec<SessionContext>>, // Host memory tier
    disk_storage: Mutex<Vec<SessionContext>>, // Disk tier
    host_memory_capacity: usize, // Simulated capacity in KB
    disk_capacity: usize,       // Simulated capacity in KB
    model: Arc<LLMModel>,       // Transformer model from llm-rs
}

impl AttentionStore {
    pub fn new(
        vault: Arc<ZetaVaultSynergy>,
        transfer_engine: Arc<TransferEngine>,
        client: Arc<Client>,
        master_service: Arc<MasterService>,
    ) -> Result<Arc<Self>, AttentionStoreError> {
        let model = Arc::new(LLMModel::new(LAYER_COUNT)?); // Initialize transformer model
        let store = Arc::new(AttentionStore {
            kv_caching_system: Arc::clone(&vault),
            transfer_engine: Arc::clone(&transfer_engine),
            client: Arc::clone(&client),
            master_service: Arc::clone(&master_service),
            sessions: RwLock::new(std::collections::HashMap::new()),
            scheduler: Arc::new(Scheduler::new()),
            hbm_buffer: Mutex::new(Vec::with_capacity(LAYER_COUNT)),
            host_memory: Mutex::new(Vec::new()),
            disk_storage: Mutex::new(Vec::new()),
            host_memory_capacity: 1024 * 1024, // 1GB in KB
            disk_capacity: 10 * 1024 * 1024,  // 10GB in KB
            model,
        });

        tokio::spawn(AttentionStore::async_save_task(Arc::clone(&store)));
        Ok(store)
    }

    pub async fn decode(&self, session_id: String, token: u32, kv_cache_prev: Vec<KVCache>) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert_with(|| {
            let segment = self.allocate_segment(session_id.clone()).await?;
            SessionContext {
                session_id: session_id.clone(),
                kv_cache: kv_cache_prev,
                last_active: Instant::now(),
                truncated: false,
                segment: Some(segment),
            }
        });

        let mut hbm = self.hbm_buffer.lock();
        self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;

        let mut next_token = token;
        for step in 0..MAX_GENERATION_STEPS {
            let (new_token, new_kv_cache) = self.model.compute_attention(token, &ctx.kv_cache, step)?;
            next_token = new_token;
            ctx.kv_cache = new_kv_cache;

            if next_token == EOS_TOKEN {
                log::info!("Session {} reached EOS token at step {}", session_id, step);
                break;
            }
            ctx.last_active = Instant::now();
        }

        self.async_save(ctx.kv_cache.clone()).await?;
        Ok((next_token, ctx.kv_cache.clone()))
    }

    pub async fn prefill(&self, session_id: String, new_tokens: Vec<u32>) -> Result<Vec<KVCache>, AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert_with(|| {
            let segment = self.allocate_segment(session_id.clone()).await.unwrap_or_default();
            SessionContext {
                session_id,
                kv_cache: Vec::new(),
                last_active: Instant::now(),
                truncated: false,
                segment,
            }
        });

        let mut hbm = self.hbm_buffer.lock();
        if !ctx.kv_cache.is_empty() {
            self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;
        }

        let new_kv_cache = self.model.compute_prefill(new_tokens)?;
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
                self.transfer_engine.async_load(&prev_cache[0], hbm).await.map_err(|e| AttentionStoreError::Transfer(e.to_string()))?;
            }
            // Overlap with computation simulated by yielding
            tokio::task::yield_now().await;
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

    async fn allocate_segment(&self, session_id: String) -> Result<String, AttentionStoreError> {
        let segment = Segment {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("seg_{}", session_id),
            client_id: session_id,
        };
        self.master_service.mount_segment(segment, session_id).await?;
        Ok(segment.id)
    }

    pub async fn truncate_cache(&self, session_id: String, max_tokens: usize) -> Result<(), AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        if let Some(ctx) = sessions.get_mut(&session_id) {
            let token_count = ctx.kv_cache.len() / LAYER_COUNT;
            if token_count > max_tokens {
                ctx.kv_cache.truncate(max_tokens * LAYER_COUNT);
                ctx.truncated = true;
                // Re-embed positional encoding
                for (i, cache) in ctx.kv_cache.iter_mut().enumerate() {
                    let layer_idx = i % LAYER_COUNT;
                    cache.positional_encoding = Some(vec![layer_idx as i32 * max_tokens as i32; max_tokens]);
                }
                if let Some(segment_id) = &ctx.segment {
                    self.master_service.remount_segment(vec![Segment {
                        id: segment_id.clone(),
                        name: format!("seg_{}", session_id),
                        client_id: session_id,
                    }], session_id).await?;
                }
                self.async_save(ctx.kv_cache.clone()).await?;
            }
        }
        Ok(())
    }

    async fn evict_if_needed(&self) -> Result<(), AttentionStoreError> {
        let host_mem = self.host_memory.lock();
        let disk = self.disk_storage.lock();
        let total_size = host_mem.iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>() +
                        disk.iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>();
        if total_size > self.host_memory_capacity + self.disk_capacity {
            self.evict().await;
        }
        Ok(())
    }

    pub async fn evict(&self) {
        let total_capacity = self.host_memory_capacity + self.disk_capacity;
        let look_ahead_window = total_capacity / 1024; // KB per KV cache
        self.scheduler.evict(look_ahead_window, &self.host_memory, &self.disk_storage).await;

        let mut host_mem = self.host_memory.lock();
        while host_mem.iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>() > self.host_memory_capacity {
            if let Some(ctx) = host_mem.pop() {
                if self.disk_storage.lock().iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>() < self.disk_capacity {
                    self.disk_storage.lock().push(ctx);
                }
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
                    if let Err(e) = store.async_save(ctx.kv_cache.clone()).await {
                        log::error!("Save failed for session {}: {}", ctx.session_id, e);
                    }
                    if let Some(segment_id) = &ctx.segment {
                        if let Err(e) = store.master_service.unmount_segment(segment_id.clone(), ctx.session_id.clone()).await {
                            log::error!("Unmount failed for segment {}: {}", segment_id, e);
                        }
                    }
                }
            }
        }
    }
}
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
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use thiserror::Error;
use log;
use p2pstore::{KVCache, TransferEngine, Segment};
use zeta_vault_synergy::ZetaVaultSynergy;
use parking_lot::Mutex as ParkingMutex;
use client::Client;    // For TransferEngine
use master_service::MasterService; // For segment management (placeholder)
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

#[derive(Debug)]
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
    master_service: Arc<MasterService>, // Kept for compatibility (not directly used)
    segment_ops: Arc<dyn SegmentOps + Send + Sync>,
    sessions: RwLock<std::collections::HashMap<String, SessionContext>>,
    scheduler: Arc<Scheduler>,
    hbm_buffer: Mutex<Vec<KVCache>>, // High Bandwidth Memory buffer
    host_memory: Mutex<Vec<SessionContext>>, // Host memory tier
    disk_storage: Mutex<Vec<SessionContext>>, // Disk tier
    host_memory_capacity: usize, // Simulated capacity in KB
    disk_capacity: usize,       // Simulated capacity in KB
    // model removed for now; generation is stubbed to unblock build
}

<<<<<<< Updated upstream
impl AttentionStore {
=======
// Segment operations abstraction to decouple from master-service internals (sync to allow dyn)
pub trait SegmentOps: Send + Sync {
    fn mount_segment(&self, _segment: Segment, _client_id: String) -> Result<(), AttentionStoreError> { Ok(()) }
    fn remount_segment(&self, _segments: Vec<Segment>, _client_id: String) -> Result<(), AttentionStoreError> { Ok(()) }
    fn unmount_segment(&self, _segment_id: String, _client_id: String) -> Result<(), AttentionStoreError> { Ok(()) }
}

pub struct NoopSegmentOps;
impl SegmentOps for NoopSegmentOps {}

impl<T: TransferEngine + Send + Sync + 'static> AttentionStore<T> {
>>>>>>> Stashed changes
    pub fn new(
        vault: Arc<ZetaVaultSynergy>,
        transfer_engine: Arc<TransferEngine>,
        client: Arc<Client>,
        master_service: Arc<MasterService>,
    ) -> Result<Arc<Self>, AttentionStoreError> {
        let store = Arc::new(AttentionStore {
            kv_caching_system: Arc::clone(&vault),
            transfer_engine: Arc::clone(&transfer_engine),
            client: Arc::clone(&client),
            master_service: Arc::clone(&master_service),
            segment_ops: Arc::new(NoopSegmentOps),
            sessions: RwLock::new(std::collections::HashMap::new()),
            scheduler: Arc::new(Scheduler::new()),
            hbm_buffer: Mutex::new(Vec::with_capacity(LAYER_COUNT)),
            host_memory: Mutex::new(Vec::new()),
            disk_storage: Mutex::new(Vec::new()),
            host_memory_capacity: 1024 * 1024, // 1GB in KB
            disk_capacity: 10 * 1024 * 1024,  // 10GB in KB
        });

        Ok(store)
    }

    pub async fn decode(&self, session_id: String, token: u32, kv_cache_prev: Vec<KVCache>) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        // Check or create session without awaiting inside or_insert_with
        {
            let sessions = self.sessions.read().await;
            if sessions.get(&session_id).is_none() {
                drop(sessions);
                // Perform async work outside the lock
                let segment = self.allocate_segment(session_id.clone()).await?;
                let mut sessions_w = self.sessions.write().await;
                sessions_w.entry(session_id.clone()).or_insert(SessionContext {
                    session_id: session_id.clone(),
                    kv_cache: kv_cache_prev.clone(),
                    last_active: Instant::now(),
                    truncated: false,
                    segment: Some(segment),
                });
            }
        }
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.get_mut(&session_id).expect("session must exist");

        let mut hbm = self.hbm_buffer.lock().unwrap();
        self.layer_wise_preload(&ctx.kv_cache, &mut *hbm).await?;

        // Stub generation loop: just update last_active once
        let next_token = token;
        ctx.last_active = Instant::now();

        self.async_save(ctx.kv_cache.clone()).await?;
        Ok((next_token, ctx.kv_cache.clone()))
    }

    pub async fn prefill(&self, session_id: String, new_tokens: Vec<u32>) -> Result<Vec<KVCache>, AttentionStoreError> {
        // Ensure session exists (no await inside closure)
        {
            let sessions = self.sessions.read().await;
            if sessions.get(&session_id).is_none() {
                drop(sessions);
                let segment_opt = Some(self.allocate_segment(session_id.clone()).await.unwrap_or_default());
                let mut sessions_w = self.sessions.write().await;
                sessions_w.entry(session_id.clone()).or_insert(SessionContext {
                    session_id: session_id.clone(),
                    kv_cache: Vec::new(),
                    last_active: Instant::now(),
                    truncated: false,
                    segment: segment_opt,
                });
            }
        }
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.get_mut(&session_id).expect("session must exist");

        let mut hbm = self.hbm_buffer.lock().unwrap();
        if !ctx.kv_cache.is_empty() {
            self.layer_wise_preload(&ctx.kv_cache, &mut *hbm).await?;
        }

        // Stub prefill: no change to kv_cache for now
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
        let now_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let segment = Segment {
            id: format!("seg-{}", now_ns),
            name: format!("seg_{}", session_id),
            client_id: session_id.clone(),
        };
        let seg_id = segment.id.clone();
        self.segment_ops.mount_segment(segment, session_id)?;
        Ok(seg_id)
    }

    pub async fn truncate_cache(&self, session_id: String, max_tokens: usize) -> Result<(), AttentionStoreError> {
        let mut sessions = self.sessions.write().await;
        if let Some(ctx) = sessions.get_mut(&session_id) {
            let token_count = ctx.kv_cache.len() / LAYER_COUNT;
            if token_count > max_tokens {
                ctx.kv_cache.truncate(max_tokens * LAYER_COUNT);
                ctx.truncated = true;
                if let Some(segment_id) = &ctx.segment {
                    self.segment_ops.remount_segment(vec![Segment {
                        id: segment_id.clone(),
                        name: format!("seg_{}", session_id),
                        client_id: session_id.clone(),
                    }], session_id.clone())?;
                }
                self.async_save(ctx.kv_cache.clone()).await?;
            }
        }
        Ok(())
    }

    async fn evict_if_needed(&self) -> Result<(), AttentionStoreError> {
        let host_mem = self.host_memory.lock().unwrap();
        let disk = self.disk_storage.lock().unwrap();
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

        let mut host_mem = self.host_memory.lock().unwrap();
        while host_mem.iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>() > self.host_memory_capacity {
            if let Some(ctx) = host_mem.pop() {
                let mut disk = self.disk_storage.lock().unwrap();
                if disk.iter().map(|c| c.kv_cache.len() / LAYER_COUNT * 1024).sum::<usize>() < self.disk_capacity {
                    disk.push(ctx);
                }
            }
        }
    }
}

impl<T: TransferEngine + Send + Sync + 'static> AttentionStore<T> {
    async fn async_save_task(store: Arc<Self>) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let sessions = store.sessions.read().await;
            for ctx in sessions.values() {
                if ctx.last_active.elapsed() > Duration::from_secs(60) { // Inactive for 1 minute
                    if let Err(e) = store.async_save(ctx.kv_cache.clone()).await {
                        log::error!("Save failed for session {}: {}", ctx.session_id, e);
                    }
                    if let Some(segment_id) = &ctx.segment {
                        if let Err(e) = store.segment_ops.unmount_segment(segment_id.clone(), ctx.session_id.clone()) {
                            log::error!("Unmount failed for segment {}: {}", segment_id, e);
                        }
                    }
                }
            }
        }
    }
}
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

mod scheduler;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log;
use p2pstore::{KVCache, TransferEngine, AllocatedBufferDescriptor, TransferEngineError};
use zeta_vault_synergy::ZetaVaultSynergy;
use parking_lot::Mutex as ParkingMutex;
use llm_rs::LLMModel;
use client::Client;
use master_service::MasterService;
use quantize::Quantizer;
use agentflow::AgentFlow;

pub const MAX_GENERATION_STEPS: usize = 100;
pub const EOS_TOKEN: u32 = 2;
pub const LAYER_COUNT: usize = 12;

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
    #[error("Quantization error: {0}")]
    Quantization(String),
    #[error("AgentFlow error: {0}")]
    AgentFlow(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionContext {
    session_id: String,
    kv_cache: Vec<KVCache>,
    last_active: Instant,
    truncated: bool,
    segment: Option<String>,
    prefetch_priority: u32,
    quantized: bool,
}

pub struct InferenceHandler {
    model: Arc<LLMModel>,
    quantizer: Arc<Quantizer>,
}

impl InferenceHandler {
    pub fn new(model: Arc<LLMModel>, quantizer: Arc<Quantizer>) -> Result<Self, AttentionStoreError> {
        Ok(InferenceHandler { model, quantizer })
    }

    pub fn process(&self, token: u32, kv_cache: &mut Vec<KVCache>) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        if !kv_cache.is_empty() && !kv_cache[0].buffers.is_empty() {
            self.quantizer.dequantize_kv_cache(kv_cache).map_err(|e| AttentionStoreError::Quantization(e.to_string()))?;
        }
        let (new_token, new_kv_cache) = self.model.compute_attention(token, kv_cache, 0)?;
        self.quantizer.quantize_kv_cache(&mut new_kv_cache).map_err(|e| AttentionStoreError::Quantization(e.to_string()))?;
        Ok((new_token, new_kv_cache))
    }
}

pub struct AttentionStore {
    kv_caching_system: Arc<ZetaVaultSynergy>,
    transfer_engine: Arc<TransferEngine>,
    client: Arc<Client>,
    master_service: Arc<MasterService>,
    sessions: RwLock<std::collections::HashMap<String, SessionContext>>,
    scheduler: Arc<scheduler::Scheduler>,
    hbm_buffer: Mutex<Vec<KVCache>>,
    host_memory: Mutex<Vec<SessionContext>>,
    disk_storage: Mutex<Vec<SessionContext>>,
    host_memory_capacity: usize,
    disk_capacity: usize,
    model: Arc<LLMModel>,
    quantizer: Arc<Quantizer>,
    inference_handler: InferenceHandler,
    agent_flow: Option<Arc<AgentFlow>>, // Optional AgentFlow integration
    prefetch_queue: Mutex<Vec<String>>,
}

impl AttentionStore {
    pub fn new(
        vault: Arc<ZetaVaultSynergy>,
        transfer_engine: Arc<TransferEngine>,
        client: Arc<Client>,
        master_service: Arc<MasterService>,
    ) -> Result<Arc<Self>, AttentionStoreError> {
        let model = Arc::new(LLMModel::new(LAYER_COUNT)?);
        let quantizer = Arc::new(Quantizer::new(8)?);
        let inference_handler = InferenceHandler::new(Arc::clone(&model), Arc::clone(&quantizer))?;
        let store = Arc::new(AttentionStore {
            kv_caching_system: Arc::clone(&vault),
            transfer_engine: Arc::clone(&transfer_engine),
            client: Arc::clone(&client),
            master_service: Arc::clone(&master_service),
            sessions: RwLock::new(std::collections::HashMap::new()),
            scheduler: Arc::new(scheduler::Scheduler::new()),
            hbm_buffer: Mutex::new(Vec::with_capacity(LAYER_COUNT)),
            host_memory: Mutex::new(Vec::new()),
            disk_storage: Mutex::new(Vec::new()),
            host_memory_capacity: 1024 * 1024,
            disk_capacity: 10 * 1024 * 1024,
            model,
            quantizer,
            inference_handler,
            agent_flow: None,
            prefetch_queue: Mutex::new(Vec::new()),
        });

        let agent_flow = AgentFlow::new(Arc::clone(&store), Arc::clone(&vault), Arc::clone(&store.model), Arc::clone(&store.quantizer)).await?;
        store.agent_flow = Some(agent_flow.clone());
        tokio::spawn(agent_flow.process_tasks());
        tokio::spawn(agent_flow.monitor_slos());
        Ok(store)
    }

    pub async fn decode(&self, session_id: String, token: u32, kv_cache_prev: Vec<KVCache>) -> Result<(u32, Vec<KVCache>), AttentionStoreError> {
        if let Some(flow) = &self.agent_flow {
            flow.enqueue_task(AgentTask::Inference { session_id: session_id.clone(), token, kv_cache: kv_cache_prev.clone() }, 10).await?;
        }
        // Fallback or mock execution if agentflow is not used
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert_with(|| {
            let segment = self.allocate_segment(session_id.clone()).await.ok();
            SessionContext {
                session_id: session_id.clone(),
                kv_cache: kv_cache_prev,
                last_active: Instant::now(),
                truncated: false,
                segment,
                prefetch_priority: 0,
                quantized: false,
            }
        });

        let mut hbm = self.hbm_buffer.lock();
        self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;

        let mut next_token = token;
        for step in 0..MAX_GENERATION_STEPS {
            let mut kv_cache = ctx.kv_cache.clone();
            let (new_token, new_kv_cache) = self.inference_handler.process(next_token, &mut kv_cache)?;
            next_token = new_token;
            ctx.kv_cache = new_kv_cache;
            ctx.quantized = true;

            if next_token == EOS_TOKEN {
                log::info!("Session {} reached EOS token at step {}", session_id, step);
                break;
            }
            ctx.last_active = Instant::now();
            ctx.prefetch_priority += 1;
            self.update_prefetch_queue(&session_id).await;
        }

        self.async_save(ctx.kv_cache.clone()).await?;
        Ok((next_token, ctx.kv_cache.clone()))
    }

    pub async fn prefill(&self, session_id: String, new_tokens: Vec<u32>) -> Result<Vec<KVCache>, AttentionStoreError> {
        if let Some(flow) = &self.agent_flow {
            flow.enqueue_task(AgentTask::Quantization { model_id: session_id.clone(), bit_width: 8 }, 5).await?;
        }
        let mut sessions = self.sessions.write().await;
        let ctx = sessions.entry(session_id.clone()).or_insert_with(|| {
            let segment = self.allocate_segment(session_id.clone()).await.ok();
            SessionContext {
                session_id,
                kv_cache: Vec::new(),
                last_active: Instant::now(),
                truncated: false,
                segment,
                prefetch_priority: 0,
                quantized: false,
            }
        });

        let mut hbm = self.hbm_buffer.lock();
        if !ctx.kv_cache.is_empty() {
            self.layer_wise_preload(&ctx.kv_cache, &mut hbm).await?;
        }

        let mut new_kv_cache = self.model.compute_prefill(new_tokens)?;
        self.quantizer.quantize_kv_cache(&mut new_kv_cache).map_err(|e| AttentionStoreError::Quantization(e.to_string()))?;
        ctx.kv_cache.extend(new_kv_cache);
        ctx.quantized = true;
        self.async_save(ctx.kv_cache.clone()).await?;
        self.evict_if_needed().await?;
        ctx.prefetch_priority += new_tokens.len() as u32;
        self.update_prefetch_queue(&session_id).await;
        Ok(ctx.kv_cache.clone())
    }

    async fn layer_wise_preload(&self, kv_cache: &[KVCache], hbm: &mut Vec<KVCache>) -> Result<(), AttentionStoreError> {
        hbm.clear();
        hbm.reserve(LAYER_COUNT);
        for (layer_idx, cache) in kv_cache.chunks(LAYER_COUNT).next().unwrap_or(&[]).iter().enumerate() {
            if layer_idx > 0 {
                let mut temp_cache = cache.clone();
                if temp_cache.buffers[0].size_ > 0 {
                    self.quantizer.dequantize_kv_cache(&mut temp_cache).map_err(|e| AttentionStoreError::Quantization(e.to_string()))?;
                }
                self.transfer_engine.async_load(&temp_cache, hbm).await.map_err(|e| AttentionStoreError::Transfer(e.to_string()))?;
            }
            tokio::task::yield_now().await;
        }
        Ok(())
    }

    async fn async_save(&self, kv_cache: Vec<KVCache>) -> Result<(), AttentionStoreError> {
        let mut quantized_cache = kv_cache;
        self.quantizer.quantize_kv_cache(&mut quantized_cache).map_err(|e| AttentionStoreError::Quantization(e.to_string()))?;
        let save_task = self.transfer_engine.async_save(quantized_cache);
        tokio::spawn(async move {
            if let Err(e) = save_task.await {
                log::error!("Async save failed: {}", e);
            }
        });
        Ok(())
    }

    async fn allocate_segment(&self, session_id: String) -> Result<String, AttentionStoreError> {
        let segment = p2pstore::Segment {
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
                for (i, cache) in ctx.kv_cache.iter_mut().enumerate() {
                    let layer_idx = i % LAYER_COUNT;
                    cache.positional_encoding = Some(vec![layer_idx as i32 * max_tokens as i32; max_tokens]);
                }
                if let Some(segment_id) = &ctx.segment {
                    self.master_service.remount_segment(vec![p2pstore::Segment {
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
        let look_ahead_window = total_capacity / 1024;
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

    async fn update_prefetch_queue(&self, session_id: &str) {
        let mut queue = self.prefetch_queue.lock();
        if let Some(idx) = queue.iter().position(|id| id == session_id) {
            queue.remove(idx);
        }
        queue.push(session_id.to_string());
        queue.sort_by(|a, b| {
            let a_ctx = self.sessions.read().await.get(a).unwrap();
            let b_ctx = self.sessions.read().await.get(b).unwrap();
            b_ctx.prefetch_priority.cmp(&a_ctx.prefetch_priority)
        });
        if queue.len() > 10 {
            queue.truncate(10);
        }
    }
}

impl AttentionStore {
    async fn async_save_task(store: Arc<AttentionStore>) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let sessions = store.sessions.read().await;
            for ctx in sessions.values() {
                if ctx.last_active.elapsed() > Duration::from_secs(60) {
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

    async fn prefetch_task(store: Arc<AttentionStore>) {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let queue = store.prefetch_queue.lock();
            for session_id in queue.iter() {
                if let Some(ctx) = store.sessions.read().await.get(session_id) {
                    if !store.host_memory.lock().iter().any(|c| c.session_id == *session_id) &&
                       !store.hbm_buffer.lock().iter().any(|c| c.buffers[0].buffer_address_ != 0) {
                        if let Ok(mut hbm) = store.hbm_buffer.try_lock() {
                            store.layer_wise_preload(&ctx.kv_cache, &mut hbm).await.ok();
                        }
                    }
                }
            }
        }
    }
}
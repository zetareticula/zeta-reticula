// Copyright 2025 ZETA RETICULA INC
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
use tokio::sync::{RwLock, mpsc};
use std::collections::BinaryHeap;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use log;
use chrono::Utc;
use p2pstore::{KVCache, AllocatedBufferDescriptor};
use zeta_vault_synergy::ZetaVaultSynergy;
use attention_store::AttentionStore;
use quantize::Quantizer;
use llm_rs::LLMModel;

#[derive(Error, Debug)]
pub enum AgentFlowError {
    #[error("Task error: {0}")]
    Task(String),
    #[error("Queue error: {0}")]
    Queue(String),
    #[error("Vault error: {0}")]
    Vault(#[from] zeta_vault_synergy::ZetaVaultSynergyError),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct TaskPriority {
    priority: i32, // Higher value = higher priority
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
enum AgentTask {
    Inference { session_id: String, token: u32, kv_cache: Vec<KVCache> },
    Quantization { model_id: String, bit_width: usize },
    Compaction { segment_id: String },
}

struct Task {
    task: AgentTask,
    priority: TaskPriority,
    assigned_gpu: Option<u32>,
}

pub struct AgentFlow {
    attention_store: Arc<AttentionStore>,
    vault: Arc<ZetaVaultSynergy>,
    model: Arc<LLMModel>,
    quantizer: Arc<Quantizer>,
    task_queue: Arc<RwLock<BinaryHeap<Task>>>,
    task_sender: mpsc::Sender<Task>,
    task_receiver: Arc<RwLock<mpsc::Receiver<Task>>>,
}

impl AgentFlow {
    pub async fn new(
        attention_store: Arc<AttentionStore>,
        vault: Arc<ZetaVaultSynergy>,
        model: Arc<LLMModel>,
        quantizer: Arc<Quantizer>,
    ) -> Result<Arc<Self>, AgentFlowError> {
        let (tx, rx) = mpsc::channel(100);
        Ok(Arc::new(AgentFlow {
            attention_store,
            vault,
            model,
            quantizer,
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            task_sender: tx,
            task_receiver: Arc::new(RwLock::new(rx)),
        }))
    }

    pub async fn enqueue_task(&self, task: AgentTask, priority: i32) -> Result<(), AgentFlowError> {
        let timestamp = Utc::now().timestamp() as u64;
        let task_priority = TaskPriority { priority, timestamp };
        let mut queue = self.task_queue.write().await;
        let assigned_gpu = match &task {
            AgentTask::Inference { .. } => Some(0), // Mock GPU assignment
            _ => None,
        };
        let task_instance = Task { task, priority: task_priority, assigned_gpu };
        queue.push(task_instance.clone());
        self.task_sender.send(task_instance).await.map_err(|e| AgentFlowError::Queue(e.to_string()))?;
        Ok(())
    }

    pub async fn process_tasks(self: Arc<Self>) {
        let receiver = Arc::clone(&self.task_receiver);
        tokio::spawn(async move {
            while let Ok(task) = receiver.write().await.recv().await {
                match task.task {
                    AgentTask::Inference { session_id, token, kv_cache } => {
                        log::info!("Processing inference task for session {}", session_id);
                        let (next_token, new_kv_cache) = self.attention_store.decode(session_id, token, kv_cache).await
                            .map_err(|e| log::error!("Inference failed: {}", e)).unwrap_or((0, vec![]));
                        // Update KV cache in vault
                        self.vault.store_kv_cache(&session_id, new_kv_cache).await.ok();
                    }
                    AgentTask::Quantization { model_id, bit_width } => {
                        log::info!("Processing quantization task for model {}", model_id);
                        let mut kv_cache = vec![KVCache::new(vec![AllocatedBufferDescriptor { buffer_address_: 0, size_: 8192 }])];
                        self.quantizer.quantize_kv_cache(&mut kv_cache).map_err(|e| log::error!("Quantization failed: {}", e)).ok();
                        self.vault.store_kv_cache(&model_id, kv_cache).await.ok();
                    }
                    AgentTask::Compaction { segment_id } => {
                        log::info!("Processing compaction for segment {}", segment_id);
                        self.vault.compact_segment(&segment_id).await.ok();
                    }
                }
            }
        });
    }

    pub async fn monitor_slos(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let queue = self.task_queue.read().await;
                for task in queue.iter() {
                    if let AgentTask::Inference { session_id, .. } = &task.task {
                        // Mock SLO check (TTFT < 100ms, TBT < 50ms)
                        if rand::random::<f64>() * 100.0 > 100.0 || rand::random::<f64>() * 50.0 > 50.0 {
                            log::warn!("SLO violation for inference task on session {}", session_id);
                        }
                    }
                }
            }
        });
    }
}
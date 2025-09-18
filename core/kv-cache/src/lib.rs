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

//! Unified KV Cache Implementation for Zeta Reticula
//! 
//! This module consolidates all KV cache functionality from:
//! - kvquant_rs/src/block.rs (LogStructuredKVCache)
//! - llm-rs/src/kv_cache.rs
//! - llm-rs/src/kv_cache_manager.rs
//! - zeta-vault-synergy implementations

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KVCacheError {
    #[error("Cache capacity exceeded")]
    CapacityExceeded,
    #[error("Invalid cache key: {0}")]
    InvalidKey(String),
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub precision: PrecisionLevel,
    pub block_size: usize,
    pub spot_capacity: usize,
    pub max_cache_items: usize,
    pub salience_threshold: f32,
    pub enable_debug_logging: bool,
    pub eviction_policy: EvictionPolicy,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrecisionLevel {
    Int1,
    Int2,
    Int4,
    Int8,
    FP16,
    FP32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    SalienceBased,
    Adaptive,
}

impl Default for KVCacheConfig {
    fn default() -> Self {
        Self {
            precision: PrecisionLevel::Int4,
            block_size: 1024,
            spot_capacity: 10000,
            max_cache_items: 50000,
            salience_threshold: 0.7,
            enable_debug_logging: false,
            eviction_policy: EvictionPolicy::SalienceBased,
            compression_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    Free,
    Valid,
    Obsolete,
    Invalid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataBlock {
    pub id: usize,
    pub state: BlockState,
    pub data: HashMap<u32, f32>,
    pub pointers: Vec<usize>,
    pub biases: Vec<f32>,
    pub vector_ids: Vec<u32>,
    pub navigation_graph: HashMap<usize, Vec<usize>>,
    pub size: usize,
    pub capacity: usize,
    pub salience_scores: HashMap<u32, f32>,
    pub access_count: u64,
    pub last_accessed: u64,
}

impl DataBlock {
    pub fn new(id: usize, capacity: usize) -> Self {
        Self {
            id,
            state: BlockState::Free,
            data: HashMap::new(),
            pointers: Vec::with_capacity(capacity),
            biases: Vec::with_capacity(capacity),
            vector_ids: Vec::with_capacity(capacity),
            navigation_graph: HashMap::new(),
            salience_scores: HashMap::new(),
            size: 0,
            capacity,
            access_count: 0,
            last_accessed: 0,
        }
    }

    pub fn write(&mut self, token_id: u32, value: f32, pointer: usize, bias: f32, vector_id: u32, graph_entry: (usize, Vec<usize>)) {
        if self.state == BlockState::Free || self.state == BlockState::Valid {
            self.data.insert(token_id, value);
            self.pointers.push(pointer);
            self.biases.push(bias);
            self.vector_ids.push(vector_id);
            self.navigation_graph.insert(graph_entry.0, graph_entry.1);
            self.size += 1;
            self.state = BlockState::Valid;
            self.access_count += 1;
            self.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
    }

    pub fn update_salience(&mut self, token_id: u32, salience_score: f32) {
        self.salience_scores.insert(token_id, salience_score);
    }

    pub fn get_salience(&self, token_id: u32) -> Option<f32> {
        self.salience_scores.get(&token_id).copied()
    }

    pub fn invalidate(&mut self) {
        self.state = BlockState::Invalid;
    }

    pub fn erase(&mut self) {
        self.data.clear();
        self.pointers.clear();
        self.biases.clear();
        self.vector_ids.clear();
        self.navigation_graph.clear();
        self.salience_scores.clear();
        self.size = 0;
        self.state = BlockState::Free;
        self.access_count = 0;
    }
}

/// Unified KV Cache that consolidates all previous implementations
pub struct UnifiedKVCache {
    config: KVCacheConfig,
    blocks: DashMap<usize, DataBlock>,
    valid_bitmap: DashMap<(usize, usize), bool>,
    lock: Arc<Mutex<()>>,
    access_order: Arc<RwLock<Vec<usize>>>, // For LRU
    access_frequency: Arc<RwLock<HashMap<usize, u64>>>, // For LFU
}

impl UnifiedKVCache {
    pub fn new(config: KVCacheConfig) -> Self {
        Self {
            config,
            blocks: DashMap::new(),
            valid_bitmap: DashMap::new(),
            lock: Arc::new(Mutex::new(())),
            access_order: Arc::new(RwLock::new(Vec::new())),
            access_frequency: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store(&self, key: u32, value: f32, salience_score: f32) -> Result<(), KVCacheError> {
        if salience_score < self.config.salience_threshold {
            return Ok(()); // Skip low salience items
        }

        let block_id = (key as usize) % self.config.block_size;
        
        {
            let _guard = self.lock.lock().unwrap();
            let mut block = self.blocks.entry(block_id).or_insert_with(|| {
                DataBlock::new(block_id, self.config.block_size)
            });

            block.data.insert(key, value);
            block.update_salience(key, salience_score);
            block.access_count += 1;
            block.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        } // Guard is dropped here

        // Update access tracking for eviction policies
        self.update_access_tracking(block_id).await;

        // Check if eviction is needed
        if self.blocks.len() > self.config.max_cache_items {
            self.evict_blocks().await?;
        }

        Ok(())
    }

    pub async fn retrieve(&self, key: u32) -> Result<Option<f32>, KVCacheError> {
        let block_id = (key as usize) % self.config.block_size;
        
        if let Some(mut block) = self.blocks.get_mut(&block_id) {
            block.access_count += 1;
            block.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            self.update_access_tracking(block_id).await;
            Ok(block.data.get(&key).copied())
        } else {
            Ok(None)
        }
    }

    pub async fn get_salience(&self, key: u32) -> Option<f32> {
        let block_id = (key as usize) % self.config.block_size;
        self.blocks.get(&block_id)?.get_salience(key)
    }

    async fn update_access_tracking(&self, block_id: usize) {
        match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                let mut access_order = self.access_order.write().await;
                access_order.retain(|&id| id != block_id);
                access_order.push(block_id);
            }
            EvictionPolicy::LFU => {
                let mut access_frequency = self.access_frequency.write().await;
                *access_frequency.entry(block_id).or_insert(0) += 1;
            }
            _ => {} // Other policies handled elsewhere
        }
    }

    async fn evict_blocks(&self) -> Result<(), KVCacheError> {
        let blocks_to_evict = match self.config.eviction_policy {
            EvictionPolicy::LRU => self.select_lru_blocks().await,
            EvictionPolicy::LFU => self.select_lfu_blocks().await,
            EvictionPolicy::SalienceBased => self.select_low_salience_blocks().await,
            EvictionPolicy::Adaptive => self.select_adaptive_blocks().await,
        };

        for block_id in blocks_to_evict {
            if let Some(mut block) = self.blocks.get_mut(&block_id) {
                block.erase();
            }
            self.blocks.remove(&block_id);
        }

        Ok(())
    }

    async fn select_lru_blocks(&self) -> Vec<usize> {
        let access_order = self.access_order.read().await;
        let evict_count = (self.blocks.len() / 4).max(1); // Evict 25%
        access_order.iter().take(evict_count).copied().collect()
    }

    async fn select_lfu_blocks(&self) -> Vec<usize> {
        let access_frequency = self.access_frequency.read().await;
        let mut freq_blocks: Vec<_> = access_frequency.iter().collect();
        freq_blocks.sort_by_key(|(_, &freq)| freq);
        
        let evict_count = (self.blocks.len() / 4).max(1);
        freq_blocks.iter().take(evict_count).map(|(&id, _)| id).collect()
    }

    async fn select_low_salience_blocks(&self) -> Vec<usize> {
        let mut salience_blocks = Vec::new();
        
        for entry in self.blocks.iter() {
            let (block_id, block) = entry.pair();
            let avg_salience: f32 = block.salience_scores.values().sum::<f32>() / block.salience_scores.len().max(1) as f32;
            salience_blocks.push((*block_id, avg_salience));
        }

        salience_blocks.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let evict_count = (self.blocks.len() / 4).max(1);
        salience_blocks.iter().take(evict_count).map(|(id, _)| *id).collect()
    }

    async fn select_adaptive_blocks(&self) -> Vec<usize> {
        // Adaptive policy combines salience and access patterns
        let mut adaptive_scores = Vec::new();
        
        for entry in self.blocks.iter() {
            let (block_id, block) = entry.pair();
            let avg_salience: f32 = block.salience_scores.values().sum::<f32>() / block.salience_scores.len().max(1) as f32;
            let recency_score = 1.0 / (block.access_count as f32 + 1.0);
            let adaptive_score = avg_salience * 0.7 + recency_score * 0.3;
            adaptive_scores.push((*block_id, adaptive_score));
        }

        adaptive_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let evict_count = (self.blocks.len() / 4).max(1);
        adaptive_scores.iter().take(evict_count).map(|(id, _)| *id).collect()
    }

    pub fn get_stats(&self) -> KVCacheStats {
        let total_blocks = self.blocks.len();
        let valid_blocks = self.blocks.iter().filter(|entry| entry.value().state == BlockState::Valid).count();
        let total_items: usize = self.blocks.iter().map(|entry| entry.value().size).sum();
        let memory_usage = total_blocks * self.config.block_size * std::mem::size_of::<f32>();

        KVCacheStats {
            total_blocks,
            valid_blocks,
            total_items,
            memory_usage_bytes: memory_usage,
            hit_rate: 0.0, // Would need to track hits/misses
            eviction_count: 0, // Would need to track evictions
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KVCacheStats {
    pub total_blocks: usize,
    pub valid_blocks: usize,
    pub total_items: usize,
    pub memory_usage_bytes: usize,
    pub hit_rate: f32,
    pub eviction_count: u64,
}

/// Factory function to create KV cache instances
pub fn create_kv_cache(config: KVCacheConfig) -> UnifiedKVCache {
    UnifiedKVCache::new(config)
}

/// Async trait for KV cache operations (for compatibility with existing code)
#[async_trait::async_trait]
pub trait KVCacheManager: Send + Sync {
    async fn store(&self, key: String, value: Vec<u8>) -> Result<()>;
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn clear(&self) -> Result<()>;
}

/// Compatibility wrapper for existing KVCacheManager implementations
pub struct KVCacheManagerAdapter {
    cache: Arc<UnifiedKVCache>,
}

impl KVCacheManagerAdapter {
    pub fn new(cache: UnifiedKVCache) -> Self {
        Self {
            cache: Arc::new(cache),
        }
    }
}

#[async_trait::async_trait]
impl KVCacheManager for KVCacheManagerAdapter {
    async fn store(&self, key: String, value: Vec<u8>) -> Result<()> {
        // Convert string key to u32 hash
        let key_hash = key.chars().map(|c| c as u32).sum::<u32>();
        
        // Store as f32 (simplified for this trait implementation)
        let value_f32 = value.len() as f32;
        self.cache.store(key_hash, value_f32, 1.0).await
            .map_err(|e| anyhow::anyhow!("Store failed: {}", e))
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let key_hash = key.chars().map(|c| c as u32).sum::<u32>();
        match self.cache.retrieve(key_hash).await? {
            Some(value) => {
                // Convert f32 back to bytes (simplified)
                let bytes = (value as u32).to_le_bytes().to_vec();
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, _key: &str) -> Result<bool> {
        // Implementation would require adding delete method to UnifiedKVCache
        Ok(true) // Placeholder
    }

    async fn clear(&self) -> Result<()> {
        // Implementation would require adding clear method to UnifiedKVCache
        Ok(()) // Placeholder
    }
}

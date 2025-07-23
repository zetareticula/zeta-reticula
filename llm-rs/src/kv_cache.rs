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

use std::sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Instant, Duration};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use dashmap::{DashMap, DashSet};
use ndarray::{Array2, ArrayView2, ArrayViewMut2};
use parking_lot::RwLock as ParkingRwLock;
use rand::Rng;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::rl_optimizer::{RLOptimizer, RLOptimizerConfig};
use crate::quantizer::{QuantizationConfig, Quantizer};
use crate::metrics::{self, MetricsRecorder};
use crate::petri_net::{PetriNet, Transition};
use crate::utils::now_millis;

#[derive(Error, Debug)]
pub enum KVCacheError {
    #[error("Cache full")]
    CacheFull,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RL optimization error: {0}")]
    RLOptimization(#[from] crate::rl_optimizer::RLOptimizerError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVCacheConfig {
    pub max_size_bytes: usize,
    pub shard_count: usize,
    pub eviction_policy: EvictionPolicy,
    pub enable_rl_optimization: bool,
    pub initial_bit_depth: u8,
    pub min_bit_depth: u8,
    pub max_bit_depth: u8,
    pub compression_threshold: f32,
    pub enable_petri_net_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    Custom(fn(&KVCacheEntry) -> f32), // Custom scoring function
}

#[derive(Debug, Clone)]
pub struct KVCacheEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub bit_depth: u8,
    pub last_accessed: u64,
    pub access_count: AtomicUsize,
    pub size_bytes: usize,
    pub metadata: HashMap<String, String>,
}

impl KVCacheEntry {
    pub fn new(key: Vec<u8>, value: Vec<u8>, bit_depth: u8) -> Self {
        let size_bytes = key.len() + value.len();
        Self {
            key,
            value,
            bit_depth,
            last_accessed: now_millis(),
            access_count: AtomicUsize::new(1),
            size_bytes,
            metadata: HashMap::new(),
        }
    }

    pub fn update_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        // Use relaxed ordering since precise timing isn't critical
        unsafe { std::ptr::write_volatile(&self.last_accessed as *const _ as *mut _, now_millis()) };
    }
}

impl Hash for KVCacheEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for KVCacheEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for KVCacheEntry {}

#[derive(Debug)]
pub struct KVCacheShard {
    entries: DashMap<Vec<u8>, Arc<KVCacheEntry>>,
    lru: ParkingRwLock<VecDeque<Vec<u8>>>,
    size_bytes: AtomicUsize,
    max_size_bytes: usize,
    eviction_policy: EvictionPolicy,
}

impl KVCacheShard {
    pub fn new(max_size_bytes: usize, eviction_policy: EvictionPolicy) -> Self {
        Self {
            entries: DashMap::with_capacity(1024),
            lru: ParkingRwLock::new(VecDeque::with_capacity(1024)),
            size_bytes: AtomicUsize::new(0),
            max_size_bytes,
            eviction_policy,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Arc<KVCacheEntry>> {
        if let Some(entry) = self.entries.get(key) {
            let entry = entry.clone();
            entry.update_access();
            
            // Update LRU
            let mut lru = self.lru.write();
            if let Some(pos) = lru.iter().position(|k| k == key) {
                lru.remove(pos);
            }
            lru.push_back(key.to_vec());
            
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&self, key: Vec<u8>, value: Vec<u8>, bit_depth: u8) -> Result<(), KVCacheError> {
        let entry = Arc::new(KVCacheEntry::new(key.clone(), value, bit_depth));
        let entry_size = entry.size_bytes;
        
        // Check if we need to evict
        self.evict_if_needed(entry_size)?;
        
        // Insert the new entry
        self.entries.insert(key.clone(), entry);
        self.size_bytes.fetch_add(entry_size, Ordering::Relaxed);
        
        // Update LRU
        let mut lru = self.lru.write();
        lru.push_back(key);
        
        Ok(())
    }
    
    fn evict_if_needed(&self, new_entry_size: usize) -> Result<(), KVCacheError> {
        let current_size = self.size_bytes.load(Ordering::Relaxed);
        let max_size = self.max_size_bytes;
        
        if current_size + new_entry_size <= max_size {
            return Ok(());
        }
        
        // Need to evict some entries
        let mut lru = self.lru.write();
        let mut bytes_to_free = (current_size + new_entry_size).saturating_sub(max_size);
        let mut bytes_freed = 0;
        
        while bytes_freed < bytes_to_free {
            if let Some(key) = lru.pop_front() {
                if let Some((_, entry)) = self.entries.remove(&key) {
                    let entry_size = entry.size_bytes;
                    self.size_bytes.fetch_sub(entry_size, Ordering::Relaxed);
                    bytes_freed += entry_size;
                }
            } else {
                break;
            }
        }
        
        if bytes_freed < bytes_to_free {
            return Err(KVCacheError::CacheFull);
        }
        
        Ok(())
    }
}

pub struct KVCache {
    shards: Vec<Arc<KVCacheShard>>,
    rl_optimizer: Option<Arc<ParkingRwLock<RLOptimizer>>>,
    quantizer: Arc<Quantizer>,
    metrics: Arc<MetricsRecorder>,
    petri_net: Option<Arc<PetriNet>>,
    config: KVCacheConfig,
}

impl KVCache {
    pub fn new(config: KVCacheConfig) -> Result<Self, KVCacheError> {
        // Validate config
        if config.min_bit_depth > config.max_bit_depth {
            return Err(KVCacheError::InvalidConfig(
                "min_bit_depth cannot be greater than max_bit_depth".to_string(),
            ));
        }

        if config.initial_bit_depth < config.min_bit_depth || config.initial_bit_depth > config.max_bit_depth {
            return Err(KVCacheError::InvalidConfig(
                "initial_bit_depth must be between min_bit_depth and max_bit_depth".to_string(),
            ));
        }

        // Initialize shards
        let shard_size = config.max_size_bytes / config.shard_count;
        let mut shards = Vec::with_capacity(config.shard_count);
        
        for _ in 0..config.shard_count {
            shards.push(Arc::new(KVCacheShard::new(
                shard_size,
                config.eviction_policy.clone(),
            )));
        }

        // Initialize RL optimizer if enabled
        let rl_optimizer = if config.enable_rl_optimization {
            let rl_config = RLOptimizerConfig {
                state_dim: 128,  // Adjust based on your state representation
                action_dim: (config.max_bit_depth - config.min_bit_depth + 1) as usize,
                ..Default::default()
            };
            
            match RLOptimizer::new(rl_config) {
                Ok(optimizer) => Some(Arc::new(ParkingRwLock::new(optimizer))),
                Err(e) => {
                    error!("Failed to initialize RL optimizer: {}", e);
                    return Err(KVCacheError::RLOptimization(e));
                }
            }
        } else {
            None
        };

        // Initialize quantizer
        let quantizer = Arc::new(Quantizer::new(QuantizationConfig {
            min_bit_depth: config.min_bit_depth,
            max_bit_depth: config.max_bit_depth,
            initial_bit_depth: config.initial_bit_depth,
            compression_threshold: config.compression_threshold,
        }));

        // Initialize metrics
        let metrics = Arc::new(MetricsRecorder::new("kv_cache"));
        
        // Initialize Petri net if enabled
        let petri_net = if config.enable_petri_net_logging {
            Some(Arc::new(PetriNet::new()))
        } else {
            None
        };

        Ok(Self {
            shards,
            rl_optimizer,
            quantizer,
            metrics,
            petri_net,
            config,
        })
    }

    pub fn get_shard(&self, key: &[u8]) -> &Arc<KVCacheShard> {
        // Simple hash-based sharding
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let shard_idx = (hasher.finish() % self.shards.len() as u64) as usize;
        &self.shards[shard_idx]
    }

    pub async fn get(&self, key: &[u8]) -> Option<Arc<KVCacheEntry>> {
        let start_time = Instant::now();
        let shard = self.get_shard(key);
        
        let result = shard.get(key);
        
        // Record metrics
        let latency = start_time.elapsed();
        self.metrics.record_latency("get", latency);
        
        if result.is_some() {
            self.metrics.increment_counter("cache_hit");
        } else {
            self.metrics.increment_counter("cache_miss");
        }
        
        result
    }

    pub async fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), KVCacheError> {
        let start_time = Instant::now();
        let shard = self.get_shard(&key);
        
        // Determine optimal bit depth using RL if enabled
        let bit_depth = if let Some(optimizer) = &self.rl_optimizer {
            let state = self.compute_state(&key, &value);
            let action = {
                let optimizer = optimizer.read();
                optimizer.select_action(&state, 0.1).unwrap_or(self.config.initial_bit_depth as usize)
            };
            (action as u8).clamp(self.config.min_bit_depth, self.config.max_bit_depth)
        } else {
            self.config.initial_bit_depth
        };
        
        // Quantize the value
        let quantized_value = self.quantizer.quantize(&value, bit_depth)?;
        
        // Insert into cache
        shard.insert(key, quantized_value, bit_depth)?;
        
        // Record metrics
        let latency = start_time.elapsed();
        self.metrics.record_latency("insert", latency);
        self.metrics.record_histogram("bit_depth", bit_depth as f64);
        
        // Log to Petri net if enabled
        if let Some(petri_net) = &self.petri_net {
            let transition = Transition::new(
                "cache_insert".to_string(),
                vec![
                    ("key".to_string(), format!("{:?}", key)),
                    ("size".to_string(), value.len().to_string()),
                    ("bit_depth".to_string(), bit_depth.to_string()),
                ],
            );
            petri_net.fire_transition(transition).await;
        }
        
        Ok(())
    }
    
    fn compute_state(&self, key: &[u8], value: &[u8]) -> Vec<f32> {
        // Compute features for RL state
        let mut state = Vec::with_capacity(128);
        
        // Key features
        state.push(key.len() as f32 / 1024.0);  // Normalized key length
        
        // Value features
        state.push(value.len() as f32 / (1024.0 * 1024.0));  // Normalized value size (MB)
        
        // Add entropy estimate
        let entropy = self.estimate_entropy(value);
        state.push(entropy);
        
        // Pad with zeros if needed
        state.resize(128, 0.0);
        
        state
    }
    
    fn estimate_entropy(&self, data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mut counts = [0u32; 256];
        for &b in data {
            counts[b as usize] += 1;
        }
        
        let len = data.len() as f32;
        let mut entropy = 0.0;
        
        for &count in &counts {
            if count > 0 {
                let p = count as f32 / len;
                entropy -= p * p.log2();
            }
        }
        
        entropy / 8.0  // Normalize to [0, 1]
    }
    
    pub async fn train_rl_optimizer(&self, batch_size: usize) -> Result<f32, KVCacheError> {
        if let Some(optimizer) = &self.rl_optimizer {
            let mut optimizer = optimizer.write();
            optimizer.optimize()
                .map_err(KVCacheError::RLOptimization)
        } else {
            Ok(0.0)
        }
    }
    
    pub fn get_metrics(&self) -> HashMap<String, f64> {
        self.metrics.get_metrics()
    }
    
    pub async fn save_state(&self, path: &str) -> Result<(), KVCacheError> {
        if let Some(optimizer) = &self.rl_optimizer {
            let optimizer = optimizer.read();
            optimizer.save(&format!("{}/rl_optimizer.pt", path))?;
        }
        
        // Save other state as needed
        Ok(())
    }
    
    pub async fn load_state(&self, path: &str) -> Result<(), KVCacheError> {
        if let Some(optimizer) = &self.rl_optimizer {
            let mut optimizer = optimizer.write();
            optimizer.load(&format!("{}/rl_optimizer.pt", path))?;
        }
        
        // Load other state as needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    fn test_config() -> KVCacheConfig {
        KVCacheConfig {
            max_size_bytes: 1024 * 1024,  // 1MB
            shard_count: 4,
            eviction_policy: EvictionPolicy::LRU,
            enable_rl_optimization: false,
            initial_bit_depth: 8,
            min_bit_depth: 2,
            max_bit_depth: 8,
            compression_threshold: 0.5,
            enable_petri_net_logging: false,
        }
    }
    
    #[tokio::test]
    async fn test_kv_cache_basic() {
        let config = test_config();
        let cache = KVCache::new(config).unwrap();
        
        // Test insert and get
        let key = b"test_key".to_vec();
        let value = vec![1, 2, 3, 4, 5];
        
        cache.insert(key.clone(), value.clone()).await.unwrap();
        let entry = cache.get(&key).await.unwrap();
        
        assert_eq!(entry.value, value);
        assert_eq!(entry.bit_depth, 8);  // Should use initial bit depth
    }
    
    #[tokio::test]
    async fn test_eviction() {
        let mut config = test_config();
        config.max_size_bytes = 100;  // Very small cache for testing
        let cache = KVCache::new(config).unwrap();
        
        // Insert until cache is full
        for i in 0..100 {
            let key = format!("key_{}", i).into_bytes();
            let value = vec![0; 20];  // 20 bytes per value
            let _ = cache.insert(key, value).await;
        }
        
        // Verify some items were evicted
        let entry = cache.get(b"key_0").await;
        assert!(entry.is_none());
    }
    
    #[tokio::test]
    async fn test_rl_optimization() {
        let mut config = test_config();
        config.enable_rl_optimization = true;
        let cache = KVCache::new(config).unwrap();
        
        // Insert some data
        for i in 0..100 {
            let key = format!("key_{}", i).into_bytes();
            let value = vec![i; 100];  // 100 bytes per value
            cache.insert(key, value).await.unwrap();
        }
        
        // Train the RL optimizer
        let loss = cache.train_rl_optimizer(32).await.unwrap();
        assert!(loss >= 0.0);
    }
}

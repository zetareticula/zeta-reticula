use crate::spot::SpotManager;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use std::sync::Mutex;
use crate::block::KVQuantConfig;
use std::sync::Arc;
use std::collections::HashMap;
use rand::Rng;
use std::sync::RwLock;
use crate::block::{DataBlock, BlockState};
use dashmap::mapref::entry::Entry;
use rand_distr::{Distribution, Normal};

// KVQuantConfig defines the configuration for the KVQuant system
#[derive(Serialize, Deserialize, Clone)]
pub struct KVQuantConfig {
    pub spot_capacity: usize, // Maximum number of spots
    pub block_size: usize,    // Size of each block in the cache
    pub salience_threshold: f32, // Threshold for salience score to consider a token valid
}

// LogStructuredKVCache implements a log-structured key-value cache
#[derive(Serialize, Deserialize)]
pub struct LogStructuredKVCache {
    spots: SpotManager,
    block_size: usize,
    valid_bitmap: DashMap<(usize, usize), bool>,
    salience_threshold: f32,
    lock: Mutex<()>,
}

// SpotManager is a manager for handling spots in the cache
impl LogStructuredKVCache {
    // Creates a new LogStructuredKVCache with the given configuration
    pub fn new(config: KVQuantConfig) -> Self {
        LogStructuredKVCache {
            spots: SpotManager::new(config.spot_capacity),
            block_size: config.block_size,
            valid_bitmap: DashMap::new(),
            salience_threshold: 0.7,
            lock: Mutex::new(()),
        }
    }

    // update adds a new token to the cache if its salience score is above the threshold
    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        let _guard = self.lock.lock().unwrap();
        if salience_score < self.salience_threshold {
            return;
        }
        // Generate a random spot ID based on the token ID
        let (spot_id, block_id) = self.spots.append(token_id, value, pointer, bias);
        self.valid_bitmap.insert((spot_id, block_id), true);
    }

    // invalidate_low_salience invalidates tokens with low salience scores
    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        let _guard = self.lock.lock().unwrap();
    
        for &(token_id, salience) in salience_scores {
            if salience < self.salience_threshold {
                for entry in self.valid_bitmap.iter() {
                    let ((spot_id, block_id), _) = entry.pair();
                    if let Some(spot_lock) = self.spots.spots.get(spot_id) {
                        let mut spot = spot_lock.write().unwrap();
                        if let Some(block) = spot.blocks.get_mut(*block_id) {
                            if block.data.contains_key(&token_id) {
                                block.unmap();
                                block.invalidate();
                                self.valid_bitmap.insert((*spot_id, *block_id), false);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn erase_full_spots(&self) {
        let _guard = self.lock.lock().unwrap();
        for spot_entry in self.spots.spots.iter() {
            let spot = spot_entry.value().read().unwrap();
            if spot.is_full {
                drop(spot); // Drop the read lock before write
                self.spots.erase_spot(*spot_entry.key());
            }
        }
    }

    
}

pub fn initialize_kv_cache(config: KVQuantConfig) -> LogStructuredKVCache {
    LogStructuredKVCache::new(config)
}

#[derive(Serialize, Deserialize)]
pub struct KVCache {
    inner: LogStructuredKVCache,
}

impl KVCache {
    pub fn new(config: KVQuantConfig) -> Self {
        let inner = initialize_kv_cache(config);
        KVCache { inner }
    }

    pub fn update(&self, token_id: u32, value: f32, salience_score: f32, pointer: usize, bias: f32) {
        self.inner.update(token_id, value, salience_score, pointer, bias);
    }

    pub fn invalidate_low_salience(&self, salience_scores: &[(u32, f32)]) {
        self.inner.invalidate_low_salience(salience_scores);
    }

    pub fn erase_full_spots(&self) {
        self.inner.erase_full_spots();
    }
}



